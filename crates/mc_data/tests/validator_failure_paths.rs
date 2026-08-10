use mc_data::error::{ContentError, SaveError};
use mc_data::pack::{verify_references, Pack};
use mc_data::validate::{
    orphan_detect, reference_resolve, region_affinity_check, reserved_identifier_reject,
    schema_check, vocabulary_check,
};
use std::path::{Path, PathBuf};

const REPO_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("mc-data-validator-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("create validator fixture root");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, relative: &str, body: &str) {
        let path = self.0.join(relative);
        std::fs::create_dir_all(path.parent().expect("fixture path has parent"))
            .expect("create fixture parent");
        std::fs::write(path, body).expect("write validator fixture");
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn authored(relative: &str) -> String {
    std::fs::read_to_string(Path::new(REPO_ROOT).join(relative))
        .unwrap_or_else(|error| panic!("read authored fixture {relative}: {error}"))
}

#[test]
fn schema_vocabulary_and_reserved_identifier_failures_are_reported() {
    let fixture = TempDir::new("syntax");
    for directory in ["bestiary", "regions", "scenes", "spawn_tables", "items"] {
        fixture.write(&format!("{directory}/invalid.ron"), "this is not ron");
    }
    let schema_errors = schema_check(fixture.path());
    assert_eq!(schema_errors.len(), 5);
    assert!(schema_errors
        .iter()
        .all(|error| error.message.contains("failed to deserialise")));

    let missing_flags = vocabulary_check(fixture.path());
    assert_eq!(missing_flags.len(), 1);
    assert!(missing_flags[0].message.contains("cannot read flags.ron"));

    fixture.write("flags.ron", "this is not a string list");
    let invalid_flags = vocabulary_check(fixture.path());
    assert_eq!(invalid_flags.len(), 1);
    assert!(invalid_flags[0].message.contains("cannot parse flags.ron"));

    fixture.write(
        "raw/reserved.ron",
        "\"MERCEDES_ROUTE FERNAND_FORGIVEN POWER_OF_FRIENDSHIP DEUS_EX_MACHINA\"",
    );
    let reserved = reserved_identifier_reject(fixture.path());
    assert_eq!(reserved.len(), 4);
    assert!(reserved
        .iter()
        .all(|error| error.message.contains("reserved identifier")));
}

#[test]
fn references_orphans_and_affinity_are_checked_on_parsed_content() {
    let fixture = TempDir::new("references");
    let forest_bandit = authored("content/bestiary/enm_forest_bandit.ron");
    fixture.write("bestiary/enm_forest_bandit.ron", &forest_bandit);
    fixture.write(
        "bestiary/enm_orphan.ron",
        &forest_bandit.replace("ENM_FOREST_BANDIT", "ENM_ORPHAN"),
    );
    fixture.write("regions/R01.ron", &authored("content/regions/R01.ron"));
    fixture.write(
        "spawn_tables/R01-test.ron",
        r#"SpawnTable(
    region: "R01",
    chapter_stage: 1,
    pool: 2,
    entries: [
        (enemy: "ENM_FOREST_BANDIT", weight: 1, gate: Always),
        (enemy: "ENM_MISSING", weight: 1, gate: Always),
    ],
)"#,
    );

    let references = reference_resolve(fixture.path());
    assert!(references
        .iter()
        .any(|error| error.message.contains("ENM_MISSING")));
    assert!(references
        .iter()
        .any(|error| error.message.contains("R02")));

    let orphans = orphan_detect(fixture.path());
    assert!(orphans
        .iter()
        .any(|error| error.message.contains("ENM_ORPHAN")));
    assert!(orphans
        .iter()
        .any(|error| error.message.contains("R01")));

    let affinity = region_affinity_check(fixture.path());
    assert_eq!(affinity.len(), 1);
    assert!(affinity[0].message.contains("region_affinity"));
    assert_eq!(affinity[0].field.as_deref(), Some("ENM_FOREST_BANDIT"));

    let pack = Pack::from_content(fixture.path()).expect("parsed validator fixture should pack");
    let pack_references = verify_references(&pack);
    assert!(pack_references
        .iter()
        .any(|error| error.contains("ENM_MISSING not in bestiary")));
    assert!(pack_references
        .iter()
        .any(|error| error.contains("R02 not in regions")));
}

#[test]
fn content_and_save_errors_preserve_actionable_context() {
    let plain = ContentError::new("plain");
    assert_eq!(plain.to_string(), "plain");
    let file = ContentError::in_file("broken", "enemy.ron");
    assert_eq!(file.to_string(), "broken [file: enemy.ron]");
    let field = ContentError::in_field("invalid", "scene.ron", "gate");
    assert_eq!(field.to_string(), "invalid [file: scene.ron] [field: gate]");

    let io = SaveError::from(std::io::Error::new(std::io::ErrorKind::NotFound, "gone"));
    assert!(io.to_string().contains("I/O error"));
    assert!(std::error::Error::source(&io).is_some());
    let digest = SaveError::Digest("bad hex".into());
    assert_eq!(digest.to_string(), "digest error: bad hex");
    assert!(std::error::Error::source(&digest).is_none());
    let deserialize = SaveError::Deserialize("bad pack".into());
    assert_eq!(deserialize.to_string(), "deserialize error: bad pack");
    let mismatch = SaveError::DigestMismatch {
        expected: "aa".into(),
        actual: "bb".into(),
    };
    assert_eq!(
        mismatch.to_string(),
        "digest mismatch: expected aa, computed bb"
    );
}

#[test]
fn empty_content_pack_round_trips_through_every_public_loader() {
    let fixture = TempDir::new("pack");
    let pack =
        Pack::from_content(fixture.path()).expect("empty content root is a valid empty pack");
    let counts = pack.counts();
    assert_eq!(
        (
            counts.enemies,
            counts.regions,
            counts.scenes,
            counts.spawn_tables,
            counts.items,
            counts.flags,
        ),
        (0, 0, 0, 0, 0, 0)
    );
    assert!(verify_references(&pack).is_empty());

    let bytes = pack.to_bytes().expect("empty pack serializes");
    assert_eq!(Pack::load_from_bytes(&bytes).expect("raw pack loads"), pack);
    assert_eq!(
        pack.digest().expect("empty pack hashes"),
        blake3::hash(&bytes)
    );

    let pack_path = fixture.path().join("content.pack");
    pack.save(&pack_path).expect("empty pack saves with digest");
    assert_eq!(
        Pack::load_from_dir(fixture.path()).expect("saved pack loads from conventional paths"),
        pack
    );
    assert!(Pack::load_from_bytes(b"not postcard").is_err());

    fixture.write("flags.ron", "not a string list");
    let error = Pack::from_content(fixture.path()).expect_err("invalid flags must fail");
    assert!(error.message.contains("cannot parse"));

    std::fs::remove_file(fixture.path().join("flags.ron")).expect("remove invalid flags fixture");
    fixture.write("strings/en/invalid.ron", "not a localized string tuple");
    let error = Pack::from_content(fixture.path()).expect_err("invalid strings must fail");
    assert!(error.message.contains("cannot parse"));
}
