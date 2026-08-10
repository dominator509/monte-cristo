use mc_core::command::{Command, CoreEvent, SaveSlot};
use mc_core::world::World;
use mc_shell::app::App;
use mc_shell::config::{ShellConfig, ValidatedConfig};
use mc_shell::persistence::{SlotError, SlotStore};
use std::path::PathBuf;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mc-shell-save-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temporary save directory should be creatable");
    dir
}

fn digest(label: &[u8]) -> [u8; 32] {
    *blake3::hash(label).as_bytes()
}

#[test]
fn slot_store_round_trips_and_rejects_a_different_content_pack() {
    let root = temp_dir("store");
    let content_digest = digest(b"content-v1");
    let store = SlotStore::new(root.clone(), content_digest);
    let mut world = World::new(42);
    world.tick = 17;

    store
        .save(SaveSlot(0), &world)
        .expect("save slot should be written");
    let loaded = store.load(SaveSlot(0)).expect("save slot should load");
    assert_eq!(loaded.world, world);
    assert_eq!(loaded.content_digest, content_digest);

    let other_store = SlotStore::new(root, digest(b"content-v2"));
    assert!(matches!(
        other_store.load(SaveSlot(0)),
        Err(SlotError::ContentMismatch { .. })
    ));
}

#[test]
fn app_save_and_load_commands_restore_the_authoritative_world() {
    let root = temp_dir("app");
    let content_digest = digest(b"content-v1");
    let config = ValidatedConfig::from_config(ShellConfig::default(), root.clone());
    let store = SlotStore::new(root, content_digest);
    let mut app = App::new_with_catalog_and_items_and_store(
        42,
        config,
        true,
        Default::default(),
        Default::default(),
        Some(store),
    )
    .expect("app should start");

    app.world.tick = 9;
    let saved_world = app.world.clone();
    let events = app.apply_commands(&[Command::Save(SaveSlot(1))]);
    assert!(matches!(events.as_slice(), [CoreEvent::Applied { .. }]));

    app.world.tick = 99;
    let events = app.apply_commands(&[Command::Load(SaveSlot(1))]);
    assert!(matches!(events.as_slice(), [CoreEvent::Applied { .. }]));
    assert_eq!(app.world, saved_world);
}

#[test]
fn missing_slot_is_reported_as_a_rejected_command() {
    let root = temp_dir("empty");
    let config = ValidatedConfig::from_config(ShellConfig::default(), root.clone());
    let store = SlotStore::new(root, digest(b"content-v1"));
    let mut app = App::new_with_catalog_and_items_and_store(
        42,
        config,
        true,
        Default::default(),
        Default::default(),
        Some(store),
    )
    .expect("app should start");

    let events = app.apply_commands(&[Command::Load(SaveSlot(2))]);
    assert!(matches!(events.as_slice(), [CoreEvent::Rejected { .. }]));
}
