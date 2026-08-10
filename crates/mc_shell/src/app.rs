//! Application state: frame loop, headless mode, configuration.
//!
//! SPEC-004 section 10: poll input -> commands, accumulate dt, tick fixed-timestep,
//! view = state_view(), draw(view, alpha).
//!
//! SPEC-004 section 11: MC_HEADLESS=1 suppresses window + audio, same simulation.
use crate::config::ValidatedConfig;
use crate::render::palette::{scene_palette, sky_gradient};
use crate::render::target::{ShellRenderTarget, INTERNAL_HEIGHT, INTERNAL_WIDTH};
use crate::render::tilemap::Tilemap;
use crate::ui::advisory::draw_advisory_screen;
use crate::ui::{
    battle::draw_battle_interface,
    confidence::draw_confidence_scene,
    menu::{
        draw_field_hud, draw_file_select_screen, draw_menu_detail_screen, draw_menu_screen,
        MenuDetail, MENU_ENTRIES, MENU_LOAD_INDEX, MENU_SAVE_INDEX, SAVE_SLOT_COUNT,
    },
    title::draw_title_screen,
};
use macroquad::prelude::*;
use mc_core::command::{
    apply_commands_with_catalogs, Action, ActorId, ChoiceIdx, Command, CoreEvent, Dir, SaveSlot,
    StateView, TargetId,
};
use mc_core::item::AuthoredItemCatalog;
use mc_core::scene::AuthoredSceneCatalog;
use mc_core::world::World;
use std::time::Instant;
use tracing::info;

/// Fixed time step: 60 fps = 16.666... ms.
pub const FIXED_DT: f64 = 1.0 / 60.0;

/// Maximum accumulated time to prevent spiral of death.
pub const MAX_ACCUM: f64 = 0.25;

/// Which screen overlay is currently active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenState {
    /// Title screen shown before a windowed new game or continue.
    Title,
    /// Field/exploration (default state).
    Field,
    /// Active battle.
    Battle,
    /// Menu screen open.
    Menu,
    /// Read-only detail view opened from the main menu.
    MenuDetail(MenuDetail),
    /// Save/load slot picker open.
    FileSelect(FileSelectMode),
}

/// Which file operation the slot picker performs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSelectMode {
    /// Write the current world to the selected slot.
    Save,
    /// Load the selected slot into the current world.
    Load,
}

/// The main application state.
pub struct App {
    /// The game world (the authoritative state).
    pub world: World,
    /// The configuration (validated once at startup).
    pub config: ValidatedConfig,
    /// Whether we are running in headless mode.
    pub headless: bool,
    /// Accumulated time since last fixed tick.
    pub accum: f64,
    /// Current interpolation alpha for rendering.
    pub alpha: f32,
    /// The render target for the 256x224 internal buffer.
    pub render_target: Option<ShellRenderTarget>,
    /// The tilemap for current location.
    pub tilemap: Tilemap,
    /// The audio state.
    pub audio: crate::audio::AudioState,
    /// The current screen overlay state.
    pub screen_state: ScreenState,
    /// Whether the first-run content advisory is currently blocking gameplay.
    pub advisory_pending: bool,
    /// The loaded authored scene catalog used by the deterministic command bus.
    pub scene_catalog: AuthoredSceneCatalog,
    /// The loaded authored item catalog used by deterministic battle actions.
    pub item_catalog: AuthoredItemCatalog,
    /// Confined save-slot persistence for the loaded content pack.
    pub slot_store: Option<crate::persistence::SlotStore>,
    /// Currently highlighted authored scene choice.
    pub scene_choice_index: usize,
    /// Currently highlighted main-menu entry.
    pub menu_choice_index: usize,
    /// Currently highlighted title entry (New Game or Continue).
    pub title_choice_index: usize,
    /// Currently highlighted save slot.
    pub file_slot_index: u8,
    /// Latest typed save/load failure shown by the file picker.
    pub file_error: Option<String>,
    /// Cached slot occupancy for the file picker.
    pub slot_occupied: [bool; SAVE_SLOT_COUNT],
    /// Commands captured between fixed ticks. Input is still applied only at
    /// the tick boundary, but a short render frame can no longer drop a press.
    pending_commands: Vec<Command>,
    /// Tick at which the current authoritative battle became active, used for
    /// encounter metrics and transition-safe presentation state.
    battle_started_tick: Option<u64>,
}

impl App {
    /// Create a new application from a seed and configuration.
    pub fn new(seed: u128, config: ValidatedConfig, headless: bool) -> Self {
        Self::new_base(
            seed,
            config,
            headless,
            AuthoredSceneCatalog::default(),
            AuthoredItemCatalog::default(),
            None,
        )
    }

    /// Create an application with a verified authored scene catalog.
    pub fn new_with_catalog(
        seed: u128,
        config: ValidatedConfig,
        headless: bool,
        scene_catalog: AuthoredSceneCatalog,
    ) -> Result<Self, String> {
        Self::new_with_catalog_and_items(
            seed,
            config,
            headless,
            scene_catalog,
            AuthoredItemCatalog::default(),
        )
    }

    /// Create an application with verified authored scene and item catalogs.
    pub fn new_with_catalog_and_items(
        seed: u128,
        config: ValidatedConfig,
        headless: bool,
        scene_catalog: AuthoredSceneCatalog,
        item_catalog: AuthoredItemCatalog,
    ) -> Result<Self, String> {
        Self::new_with_catalog_and_items_and_store(
            seed,
            config,
            headless,
            scene_catalog,
            item_catalog,
            None,
        )
    }

    /// Create an application with authored catalogs and a confined save store.
    pub fn new_with_catalog_and_items_and_store(
        seed: u128,
        config: ValidatedConfig,
        headless: bool,
        scene_catalog: AuthoredSceneCatalog,
        item_catalog: AuthoredItemCatalog,
        slot_store: Option<crate::persistence::SlotStore>,
    ) -> Result<Self, String> {
        let mut app = Self::new_base(
            seed,
            config,
            headless,
            scene_catalog,
            item_catalog,
            slot_store,
        );
        if app.scene_catalog.scene("SCN_ARREST").is_some() {
            app.scene_catalog
                .begin(&mut app.world, "SCN_ARREST")
                .map_err(|error| format!("failed to start SCN_ARREST: {error}"))?;
        }
        Ok(app)
    }

    fn new_base(
        seed: u128,
        config: ValidatedConfig,
        headless: bool,
        scene_catalog: AuthoredSceneCatalog,
        item_catalog: AuthoredItemCatalog,
        slot_store: Option<crate::persistence::SlotStore>,
    ) -> Self {
        let world = World::new(seed);
        crate::obs::record_state_hash(*world.state_hash().as_bytes());
        let audio_enabled = !headless;
        let advisory_pending = !config.advisory_acknowledged && !headless;
        let tilemap = Tilemap::for_scene(world.act, world.region);
        let mut app = App {
            world,
            config,
            headless,
            accum: 0.0,
            alpha: 0.0,
            render_target: None,
            tilemap,
            audio: crate::audio::AudioState::new(audio_enabled),
            screen_state: if headless || advisory_pending {
                ScreenState::Field
            } else {
                ScreenState::Title
            },
            advisory_pending,
            scene_catalog,
            item_catalog,
            slot_store,
            scene_choice_index: 0,
            menu_choice_index: 0,
            title_choice_index: 0,
            file_slot_index: 0,
            file_error: None,
            slot_occupied: [false; SAVE_SLOT_COUNT],
            pending_commands: Vec::new(),
            battle_started_tick: None,
        };
        app.refresh_slot_presence();
        app
    }

    /// Advance the simulation by one frame's worth of fixed ticks.
    /// Called from headless mode.
    pub fn headless_update(&mut self) {
        let commands = Vec::new();
        self.accum += 1.0 / 60.0;
        if self.accum > MAX_ACCUM {
            self.accum = MAX_ACCUM;
        }
        while self.accum >= FIXED_DT {
            let _events = self.apply_commands(&commands);
            let step_started = Instant::now();
            self.world.step();
            self.sync_battle_screen_state();
            crate::obs::record_core_step(step_started.elapsed());
            crate::obs::CURRENT_TICK.store(self.world.tick, std::sync::atomic::Ordering::Relaxed);
            crate::obs::record_state_hash(*self.world.state_hash().as_bytes());
            crate::obs::record_tick();
            self.accum -= FIXED_DT;
        }
        self.alpha = (self.accum / FIXED_DT) as f32;
    }

    /// Process real input and advance the simulation.
    fn process_input_and_step(&mut self) {
        let raw_commands = self.poll_input();
        let translated_commands = self.translate_scene_input(raw_commands);
        self.pending_commands.extend(translated_commands);
        self.accum += get_frame_time() as f64;
        if self.accum > MAX_ACCUM {
            self.accum = MAX_ACCUM;
            info!("accumulation cap hit, dropping frames");
        }
        while self.accum >= FIXED_DT {
            let commands = std::mem::take(&mut self.pending_commands);
            let events = self.apply_commands(&commands);
            self.apply_screen_commands(&commands, &events);
            for _ in &commands {
                crate::obs::record_command();
            }
            let step_started = Instant::now();
            self.world.step();
            self.sync_battle_screen_state();
            crate::obs::record_core_step(step_started.elapsed());
            crate::obs::CURRENT_TICK.store(self.world.tick, std::sync::atomic::Ordering::Relaxed);
            crate::obs::record_state_hash(*self.world.state_hash().as_bytes());
            crate::obs::record_tick();
            self.accum -= FIXED_DT;
        }
        self.alpha = (self.accum / FIXED_DT) as f32;
    }

    /// Apply one command batch and perform shell-owned save/load side effects.
    ///
    /// Core remains the authority for command validation. Persistence is
    /// handled only after the core event is produced, and a failed slot
    /// operation is surfaced as a rejected event without mutating the world.
    pub fn apply_commands(&mut self, commands: &[Command]) -> Vec<CoreEvent> {
        let previous_scene = (self.world.act, self.world.region);
        let mut events = apply_commands_with_catalogs(
            &mut self.world,
            commands,
            Some(&self.scene_catalog),
            Some(&self.item_catalog),
        );
        for (index, command) in commands.iter().enumerate() {
            let result = match command {
                Command::Save(slot) => self.save_slot(*slot),
                Command::Load(slot) => self.load_slot(*slot),
                _ => continue,
            };
            if let Err(error) = result {
                events[index] = CoreEvent::Rejected {
                    command: command.clone(),
                    reason: error.to_string(),
                };
                tracing::error!(command = ?command, error = %error, "save-slot command failed");
            }
        }
        if commands
            .iter()
            .any(|command| matches!(command, Command::Save(_) | Command::Load(_)))
        {
            self.refresh_slot_presence();
        }
        if previous_scene != (self.world.act, self.world.region) {
            self.tilemap = Tilemap::for_scene(self.world.act, self.world.region);
        }
        events
    }

    fn save_slot(&self, slot: SaveSlot) -> Result<(), crate::persistence::SlotError> {
        let store = self.slot_store.as_ref().ok_or_else(|| {
            crate::persistence::SlotError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "save store is not configured",
            ))
        })?;
        store.save(slot, &self.world)
    }

    fn load_slot(&mut self, slot: SaveSlot) -> Result<(), crate::persistence::SlotError> {
        let store = self.slot_store.as_ref().ok_or_else(|| {
            crate::persistence::SlotError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "save store is not configured",
            ))
        })?;
        let save = store.load(slot)?;
        self.world = save.world;
        self.tilemap = Tilemap::for_scene(self.world.act, self.world.region);
        self.scene_choice_index = 0;
        Ok(())
    }

    fn refresh_slot_presence(&mut self) {
        let Some(store) = self.slot_store.as_ref() else {
            self.slot_occupied = [false; SAVE_SLOT_COUNT];
            return;
        };
        for index in 0..SAVE_SLOT_COUNT {
            self.slot_occupied[index] = match store.is_occupied(SaveSlot(index as u8)) {
                Ok(occupied) => occupied,
                Err(_) => {
                    // An unreadable or unconfined slot is not safe to present
                    // as empty: doing so hides recoverable data and prevents
                    // the load path from surfacing its typed error.
                    tracing::error!(
                        slot = index,
                        "save-slot occupancy check failed; treating slot as occupied"
                    );
                    true
                }
            };
        }
    }

    /// Run one windowed frame. Called from macroquad's render loop.
    pub fn windowed_frame(&mut self) {
        let frame_started = Instant::now();
        if self.advisory_pending {
            self.draw_advisory_frame();
            crate::obs::record_frame(frame_started.elapsed());
            return;
        }

        let Some(mut rt) = self.render_target.take() else {
            tracing::error!("windowed frame skipped because render target is not initialised");
            crate::obs::record_frame(frame_started.elapsed());
            return;
        };

        // Process input and advance simulation
        self.process_input_and_step();

        // Update audio with current state
        self.audio.update(self.world.tick, self.world.act);

        // The render target was taken before simulation to avoid borrow conflicts.
        rt.handle_resize();

        let view = StateView::from_world(&self.world, &[]);

        // Render to internal target
        rt.set_camera();
        clear_background(BLACK);

        // Draw tilemap layers
        self.draw_tilemap(&view);
        self.draw_sprites(&view);
        self.draw_ui(&view);

        // Blit to screen
        set_default_camera();
        clear_background(BLACK);
        rt.blit();

        // Put the render target back
        self.render_target = Some(rt);

        // Draw HUD on screen space
        draw_text(
            &format!("MC v{} t:{}", env!("CARGO_PKG_VERSION"), view.tick),
            10.0,
            20.0,
            14.0,
            WHITE,
        );
        crate::obs::record_frame(frame_started.elapsed());
    }

    /// Render and acknowledge the first-run advisory before any gameplay input
    /// can reach the authoritative world.
    fn draw_advisory_frame(&mut self) {
        let Some(mut rt) = self.render_target.take() else {
            tracing::error!("advisory frame skipped because render target is not initialised");
            return;
        };
        rt.handle_resize();
        rt.set_camera();
        let acknowledged = draw_advisory_screen(self.world.tick);
        set_default_camera();
        rt.blit();
        self.render_target = Some(rt);

        if acknowledged {
            self.config.advisory_acknowledged = true;
            if let Err(error) = self.config.save() {
                tracing::error!(error = %error, "failed to persist settings acknowledgement");
            }
            self.advisory_pending = false;
            self.screen_state = ScreenState::Title;
        }
    }

    /// Poll input and translate to commands.
    fn poll_input(&self) -> Vec<Command> {
        // Use the remappable input system when available
        let commands = crate::input::poll_commands(&self.config.input_map);
        commands
            .into_iter()
            .map(|command| match (self.screen_state, command) {
                (ScreenState::Menu, Command::CancelSelection) => Command::CloseMenu,
                (ScreenState::Menu, Command::OpenMenu) => Command::CloseMenu,
                (ScreenState::MenuDetail(_), Command::CancelSelection) => Command::CloseMenu,
                (ScreenState::MenuDetail(_), Command::OpenMenu) => Command::CloseMenu,
                (ScreenState::FileSelect(_), Command::CancelSelection) => Command::CloseMenu,
                (ScreenState::FileSelect(_), Command::OpenMenu) => Command::CloseMenu,
                (_, command) => command,
            })
            .collect()
    }

    /// Translate the existing remappable field controls into authored scene
    /// traversal without introducing a second input channel.
    fn translate_scene_input(&mut self, commands: Vec<Command>) -> Vec<Command> {
        match self.screen_state {
            ScreenState::Title => return self.translate_title_input(commands),
            ScreenState::Menu => return self.translate_menu_input(commands),
            ScreenState::MenuDetail(_) => return self.translate_menu_detail_input(commands),
            ScreenState::FileSelect(mode) => return self.translate_file_input(mode, commands),
            ScreenState::Battle => return self.translate_battle_input(commands),
            ScreenState::Field => {}
        }
        let Some(state) = self.world.scene.as_ref() else {
            return commands;
        };
        let Some(node) = self.scene_catalog.node(state.current) else {
            return commands;
        };
        if node.choices.is_empty() {
            return commands
                .into_iter()
                .map(|command| {
                    if matches!(command, Command::Interact) {
                        Command::SceneAdvance
                    } else {
                        command
                    }
                })
                .collect();
        }

        self.scene_choice_index = self
            .scene_choice_index
            .min(node.choices.len().saturating_sub(1));
        let mut translated = Vec::with_capacity(commands.len());
        for command in commands {
            match command {
                Command::Move(Dir::North) => {
                    self.scene_choice_index = self.scene_choice_index.saturating_sub(1);
                }
                Command::Move(Dir::South) => {
                    self.scene_choice_index = (self.scene_choice_index + 1) % node.choices.len();
                }
                Command::Interact => {
                    translated.push(Command::SceneChoose(ChoiceIdx(
                        self.scene_choice_index as u32,
                    )));
                }
                other => translated.push(other),
            }
        }
        translated
    }

    /// Translate the shared interact control into the default authored battle action.
    /// Targeting remains deterministic: the core exposes the first living enemy.
    fn translate_battle_input(&self, commands: Vec<Command>) -> Vec<Command> {
        let Some((actor, target)) = self.world.battle.as_ref().and_then(|battle| {
            if battle.state != mc_core::battle::BattleState::Active {
                return None;
            }
            Some((battle.first_party_index()?, battle.first_enemy_index()?))
        }) else {
            return commands;
        };
        commands
            .into_iter()
            .map(|command| match command {
                Command::Interact => Command::SelectAction(
                    ActorId(actor),
                    Action::Attack {
                        target: TargetId(target),
                    },
                ),
                other => other,
            })
            .collect()
    }

    /// Keep the shell overlay aligned with the authoritative battle state.
    fn sync_battle_screen_state(&mut self) {
        let active = self
            .world
            .battle
            .as_ref()
            .is_some_and(|battle| battle.state == mc_core::battle::BattleState::Active);
        match (self.battle_started_tick, active) {
            (None, true) => {
                crate::obs::record_battle();
                self.battle_started_tick = Some(self.world.tick);
            }
            (Some(start_tick), false) => {
                crate::obs::record_encounter();
                crate::obs::record_encounter_ticks(self.world.tick.saturating_sub(start_tick));
                self.battle_started_tick = None;
            }
            _ => {}
        }
        match (self.screen_state, active) {
            (ScreenState::Field, true) => self.screen_state = ScreenState::Battle,
            (ScreenState::Battle, false) => self.screen_state = ScreenState::Field,
            _ => {}
        }
    }

    fn translate_title_input(&mut self, commands: Vec<Command>) -> Vec<Command> {
        for command in commands {
            match command {
                Command::Move(Dir::North) => {
                    self.title_choice_index = self.title_choice_index.saturating_sub(1);
                }
                Command::Move(Dir::South) => {
                    self.title_choice_index = (self.title_choice_index + 1) % 2;
                }
                Command::Interact if self.title_choice_index == 0 => {
                    self.screen_state = ScreenState::Field;
                }
                Command::Interact => {
                    self.file_slot_index = 0;
                    self.file_error = None;
                    self.screen_state = ScreenState::FileSelect(FileSelectMode::Load);
                }
                _ => {}
            }
        }
        Vec::new()
    }

    fn translate_menu_input(&mut self, commands: Vec<Command>) -> Vec<Command> {
        let mut translated = Vec::with_capacity(commands.len());
        for command in commands {
            match command {
                Command::Move(Dir::North) => {
                    self.menu_choice_index = self.menu_choice_index.saturating_sub(1);
                }
                Command::Move(Dir::South) => {
                    self.menu_choice_index =
                        (self.menu_choice_index + 1) % MENU_ENTRIES.len().max(1);
                }
                Command::Interact if self.menu_choice_index == MENU_SAVE_INDEX => {
                    self.file_slot_index = 0;
                    self.file_error = None;
                    self.screen_state = ScreenState::FileSelect(FileSelectMode::Save);
                }
                Command::Interact if self.menu_choice_index == MENU_LOAD_INDEX => {
                    self.file_slot_index = 0;
                    self.file_error = None;
                    self.screen_state = ScreenState::FileSelect(FileSelectMode::Load);
                }
                Command::Interact => {
                    if let Some(detail) = menu_detail_for(self.menu_choice_index) {
                        self.screen_state = ScreenState::MenuDetail(detail);
                    }
                }
                other => translated.push(other),
            }
        }
        translated
    }

    fn translate_menu_detail_input(&mut self, commands: Vec<Command>) -> Vec<Command> {
        commands
            .into_iter()
            .filter_map(|command| match command {
                Command::CancelSelection | Command::CloseMenu | Command::OpenMenu => {
                    Some(Command::CloseMenu)
                }
                _ => None,
            })
            .collect()
    }

    fn translate_file_input(
        &mut self,
        mode: FileSelectMode,
        commands: Vec<Command>,
    ) -> Vec<Command> {
        let mut translated = Vec::with_capacity(commands.len());
        for command in commands {
            match command {
                Command::Move(Dir::North) => {
                    self.file_slot_index = self.file_slot_index.saturating_sub(1);
                    self.file_error = None;
                }
                Command::Move(Dir::South) => {
                    self.file_slot_index = (self.file_slot_index + 1) % SAVE_SLOT_COUNT as u8;
                    self.file_error = None;
                }
                Command::Interact => translated.push(match mode {
                    FileSelectMode::Save => Command::Save(SaveSlot(self.file_slot_index)),
                    FileSelectMode::Load => Command::Load(SaveSlot(self.file_slot_index)),
                }),
                other => translated.push(other),
            }
        }
        translated
    }

    /// Apply presentation transitions and surface save/load failures.
    fn apply_screen_commands(&mut self, commands: &[Command], events: &[CoreEvent]) {
        let prior = self.screen_state;
        self.screen_state = screen_state_after(self.screen_state, commands);
        for (command, event) in commands.iter().zip(events) {
            let Some(mode) = (match command {
                Command::Save(_) => Some(FileSelectMode::Save),
                Command::Load(_) => Some(FileSelectMode::Load),
                _ => None,
            }) else {
                continue;
            };
            match event {
                CoreEvent::Applied { .. } if matches!(prior, ScreenState::FileSelect(_)) => {
                    self.screen_state = ScreenState::Field;
                    self.file_error = None;
                    self.menu_choice_index = 0;
                }
                CoreEvent::Rejected { reason, .. } => {
                    self.screen_state = ScreenState::FileSelect(mode);
                    self.file_error = Some(reason.clone());
                }
                _ => {}
            }
        }
    }

    /// Draw the tilemap layers (layer0, layer1, overlay).
    fn draw_tilemap(&self, view: &StateView) {
        let tiles_x = INTERNAL_WIDTH / 16;
        let tiles_y = INTERNAL_HEIGHT / 16;
        let palette = scene_palette(view.act);
        let (far_scroll_x, near_scroll_x) = self.tilemap.scroll_offsets(view.tick);

        // Act-locked sky bands give every location a distinct 16-bit identity.
        for (offset, colour_index, height) in sky_gradient(view.act) {
            draw_rectangle(
                0.0,
                offset as f32,
                INTERNAL_WIDTH as f32,
                height as f32,
                palette[colour_index as usize].to_color(),
            );
        }

        // Draw layer 0 as a slowly moving far plane over the sky. Wrapping the
        // finite authored layer keeps the internal target covered at every
        // integer tick while preserving the 16-bit tile grid.
        let far_width = self.tilemap.layer0.width.max(1) as f32 * 16.0;
        for ty in 0..tiles_y {
            for tx in 0..tiles_x {
                let tile_id = self.tilemap.layer0.get_tile(tx, ty);
                let y = ty as f32 * 16.0;
                let colour_index = match tile_id {
                    0 => 3,
                    1 => 4,
                    2 => 5,
                    3 => 4,
                    _ => 7,
                };
                for repeat in -1..=1 {
                    let x = tx as f32 * 16.0 + far_scroll_x + repeat as f32 * far_width;
                    if x <= -16.0 || x >= INTERNAL_WIDTH as f32 {
                        continue;
                    }
                    draw_rectangle(x, y, 16.0, 16.0, palette[colour_index].to_color());
                    if tile_id % 2 == 1 {
                        draw_line(
                            x + 3.0,
                            y + 12.0,
                            x + 13.0,
                            y + 4.0,
                            1.0,
                            palette[7].to_color(),
                        );
                    }
                }
            }
        }

        // Draw layer 1 (midground) with an independent, faster parallax
        // offset. The authored layer's explicit offset remains available for
        // scene-specific composition, while replay time supplies the motion.
        let scroll_x = self.tilemap.layer1.scroll_x + near_scroll_x;
        let scroll_y = self.tilemap.layer1.scroll_y;
        let near_width = self.tilemap.layer1.width.max(1) as f32 * 16.0;
        for ty in 0..tiles_y {
            for tx in 0..tiles_x {
                let tile_id = self.tilemap.layer1.get_tile(tx, ty);
                if tile_id == 0 {
                    continue;
                }
                let y = ty as f32 * 16.0 + scroll_y;
                let colour_index = if tile_id == 2 { 5 } else { 6 };
                for repeat in -1..=1 {
                    let x = tx as f32 * 16.0 + scroll_x + repeat as f32 * near_width;
                    if x <= -16.0 || x >= INTERNAL_WIDTH as f32 {
                        continue;
                    }
                    draw_rectangle(x, y, 16.0, 16.0, palette[colour_index].to_color());
                    draw_line(
                        x + 2.0,
                        y + 2.0,
                        x + 14.0,
                        y + 2.0,
                        1.0,
                        palette[6].to_color(),
                    );
                }
            }
        }

        // Draw the fixed foreground overlay after the scrolling layers. The
        // overlay is deliberately sparse so the scene remains legible while
        // landmarks retain crisp 16-bit silhouettes at the internal scale.
        for ty in 0..tiles_y {
            for tx in 0..tiles_x {
                let tile_id = self.tilemap.overlay.get_tile(tx, ty);
                if tile_id == 0 {
                    continue;
                }
                let x = tx as f32 * 16.0;
                let y = ty as f32 * 16.0;
                match tile_id {
                    1 => {
                        let accent = palette[4].to_color();
                        draw_rectangle(x + 2.0, y + 8.0, 12.0, 3.0, accent);
                        draw_rectangle(x + 3.0, y + 4.0, 2.0, 10.0, accent);
                        draw_rectangle(x + 11.0, y + 4.0, 2.0, 10.0, accent);
                    }
                    2 => {
                        let shadow = palette[7].to_color();
                        let leaf = palette[2].to_color();
                        draw_rectangle(x + 7.0, y + 9.0, 2.0, 6.0, shadow);
                        draw_circle(x + 6.0, y + 8.0, 4.0, leaf);
                        draw_circle(x + 11.0, y + 7.0, 5.0, leaf);
                    }
                    3 => {
                        let glow = palette[6].to_color();
                        draw_rectangle(x + 7.0, y + 5.0, 2.0, 2.0, glow);
                        draw_rectangle(x + 5.0, y + 7.0, 6.0, 2.0, glow);
                        draw_rectangle(x + 7.0, y + 9.0, 2.0, 2.0, glow);
                    }
                    _ => {}
                }
            }
        }

        // A crisp horizon line and a few deterministic, parallaxed landmarks
        // keep the low-resolution scene legible without a static checkerboard.
        draw_line(
            0.0,
            96.0,
            INTERNAL_WIDTH as f32,
            96.0,
            1.0,
            palette[6].to_color(),
        );
        for x in [18.0, 76.0, 156.0, 224.0] {
            draw_rectangle(
                (x + far_scroll_x * 0.5).rem_euclid(INTERNAL_WIDTH as f32),
                20.0 + (x as u32 % 3) as f32 * 5.0,
                1.0,
                1.0,
                palette[6].to_color(),
            );
        }
        for index in 0..6 {
            let x = ((index * 53) as f32 + far_scroll_x * 0.25).rem_euclid(INTERNAL_WIDTH as f32);
            let height = 6.0 + (index % 3) as f32 * 3.0;
            draw_rectangle(x, 96.0 - height, 10.0, height, palette[7].to_color());
        }
    }

    fn draw_sprites(&self, view: &StateView) {
        let palette = scene_palette(view.act);
        let bob = ((view.tick / 12) % 2) as f32;
        let count = view.party.active.len().min(3);
        for (index, member) in view.party.active.iter().take(count).enumerate() {
            let x = INTERNAL_WIDTH as f32 / 2.0 - 12.0
                + (index as f32 - (count.saturating_sub(1) as f32 / 2.0)) * 30.0;
            let y = 116.0 + bob + (member.char_id.raw() % 2) as f32;
            let coat = palette[2 + (member.char_id.raw() as usize % 3)].to_color();
            // Party silhouettes stay authored-free and crisp at 1x, while
            // identity-specific coat tones make a full party readable.
            draw_rectangle(x + 5.0, y, 14.0, 9.0, palette[6].to_color());
            draw_rectangle(x + 3.0, y + 8.0, 18.0, 18.0, coat);
            draw_rectangle(x, y + 26.0, 9.0, 6.0, palette[7].to_color());
            draw_rectangle(x + 15.0, y + 26.0, 9.0, 6.0, palette[7].to_color());
            draw_line(
                x + 8.0,
                y + 3.0,
                x + 12.0,
                y + 3.0,
                1.0,
                palette[7].to_color(),
            );
            draw_line(
                x + 6.0,
                y + 14.0,
                x + 18.0,
                y + 14.0,
                1.0,
                palette[4].to_color(),
            );
        }
    }

    fn draw_ui(&self, view: &StateView) {
        // Draw screen overlays conditionally based on current screen state
        match self.screen_state {
            ScreenState::Title => {
                draw_title_screen(self.title_choice_index);
            }
            ScreenState::Field => {
                if let Some(state) = view.scene {
                    if let Some(node) = self.scene_catalog.node(state.current) {
                        draw_confidence_scene(
                            view.tick,
                            &node.text_key,
                            &node.choices,
                            self.scene_choice_index,
                        );
                    } else {
                        draw_field_hud(view, self.config.high_contrast);
                    }
                } else {
                    draw_field_hud(view, self.config.high_contrast);
                }
            }
            ScreenState::Battle => {
                draw_battle_interface(view);
            }
            ScreenState::Menu => {
                draw_menu_screen(self.world.tick, self.menu_choice_index);
            }
            ScreenState::MenuDetail(detail) => {
                draw_menu_detail_screen(detail, view, &self.config);
            }
            ScreenState::FileSelect(mode) => {
                draw_file_select_screen(
                    mode,
                    self.file_slot_index,
                    &self.slot_occupied,
                    self.file_error.as_deref(),
                );
            }
        }
    }
}

fn menu_detail_for(index: usize) -> Option<MenuDetail> {
    match index {
        0 => Some(MenuDetail::Party),
        1 => Some(MenuDetail::Curriculum),
        2 => Some(MenuDetail::Inventory),
        3 => Some(MenuDetail::WebOfDebt),
        4 => Some(MenuDetail::Ledger),
        5 => Some(MenuDetail::Settings),
        _ => None,
    }
}

/// Pure screen transition function used by the window loop and unit tests.
pub fn screen_state_after(current: ScreenState, commands: &[Command]) -> ScreenState {
    let mut state = current;
    for command in commands {
        match command {
            Command::OpenMenu if state == ScreenState::Field => state = ScreenState::Menu,
            Command::CloseMenu | Command::CancelSelection if state == ScreenState::Menu => {
                state = ScreenState::Field
            }
            Command::CloseMenu | Command::CancelSelection
                if matches!(state, ScreenState::MenuDetail(_)) =>
            {
                state = ScreenState::Menu
            }
            Command::CloseMenu | Command::CancelSelection
                if matches!(state, ScreenState::FileSelect(_)) =>
            {
                state = ScreenState::Menu
            }
            _ => {}
        }
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ShellConfig, ValidatedConfig};
    use mc_core::battle::atb::AtbGauge;
    use mc_core::battle::status::StatusList;
    use mc_core::battle::{Affiliation, Battle, BattleState, Combatant, CombatantKind};
    use mc_core::fx::Fx;
    use mc_core::ids::{CharId, EnemyId};
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("mc-file-select-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("temporary file-select directory");
        path
    }

    fn app(name: &str) -> App {
        App::new(
            42,
            ValidatedConfig::from_config(ShellConfig::default(), temp_dir(name)),
            true,
        )
    }

    fn active_battle() -> Battle {
        Battle::new(
            vec![Combatant {
                kind: CombatantKind::PartyMember(CharId::CHR_EDMOND),
                affiliation: Affiliation::Party,
                name: "Edmond".into(),
                atb: AtbGauge::new(Fx::from_int(12)),
                hp: Fx::from_int(100),
                max_hp: Fx::from_int(100),
                attack: Fx::from_int(10),
                defense: Fx::from_int(8),
                speed: Fx::from_int(12),
                level: 1,
                statuses: StatusList::new(),
            }],
            vec![Combatant {
                kind: CombatantKind::Enemy(EnemyId::ENM_BANDIT),
                affiliation: Affiliation::Enemy,
                name: "Bandit".into(),
                atb: AtbGauge::new(Fx::from_int(8)),
                hp: Fx::from_int(30),
                max_hp: Fx::from_int(30),
                attack: Fx::from_int(6),
                defense: Fx::from_int(4),
                speed: Fx::from_int(8),
                level: 1,
                statuses: StatusList::new(),
            }],
        )
    }

    #[test]
    fn battle_input_translates_interact_to_default_attack() {
        let mut app = app("battle-input");
        let mut battle = active_battle();
        battle.combatants[0].atb.force_full();
        app.world.battle = Some(battle);
        app.screen_state = ScreenState::Battle;

        assert_eq!(
            app.translate_scene_input(vec![Command::Interact]),
            vec![Command::SelectAction(
                ActorId(0),
                Action::Attack {
                    target: TargetId(1)
                }
            )]
        );
    }

    #[test]
    fn battle_screen_sync_follows_authoritative_state() {
        let mut app = app("battle-screen-sync");
        app.world.battle = Some(active_battle());
        app.sync_battle_screen_state();
        assert_eq!(app.screen_state, ScreenState::Battle);
        assert_eq!(app.battle_started_tick, Some(app.world.tick));

        app.world.battle.as_mut().unwrap().state = BattleState::Victory;
        app.sync_battle_screen_state();
        assert_eq!(app.screen_state, ScreenState::Field);
        assert_eq!(app.battle_started_tick, None);
    }

    #[test]
    fn menu_selection_opens_save_and_load_pickers() {
        let mut save_app = app("save-navigation");
        save_app.screen_state = ScreenState::Menu;
        let mut save_commands = vec![Command::Move(Dir::South); MENU_SAVE_INDEX];
        save_commands.push(Command::Interact);
        assert!(save_app.translate_scene_input(save_commands).is_empty());
        assert_eq!(
            save_app.screen_state,
            ScreenState::FileSelect(FileSelectMode::Save)
        );

        let mut load_app = app("load-navigation");
        load_app.screen_state = ScreenState::Menu;
        let mut load_commands = vec![Command::Move(Dir::South); MENU_LOAD_INDEX];
        load_commands.push(Command::Interact);
        assert!(load_app.translate_scene_input(load_commands).is_empty());
        assert_eq!(
            load_app.screen_state,
            ScreenState::FileSelect(FileSelectMode::Load)
        );
    }

    #[test]
    fn menu_detail_entries_open_read_only_views_and_back_out() {
        for index in 0..=5 {
            let mut app = app(&format!("menu-detail-{index}"));
            app.screen_state = ScreenState::Menu;
            app.menu_choice_index = index;
            assert!(app
                .translate_scene_input(vec![Command::Interact])
                .is_empty());
            assert!(matches!(app.screen_state, ScreenState::MenuDetail(_)));
            assert_eq!(
                screen_state_after(app.screen_state, &[Command::CloseMenu]),
                ScreenState::Menu
            );
        }
    }

    #[test]
    fn title_selection_starts_new_game_or_opens_continue_picker() {
        let acknowledged = App::new(
            42,
            ValidatedConfig::from_config(
                ShellConfig {
                    advisory_acknowledged: true,
                    ..ShellConfig::default()
                },
                temp_dir("title-windowed"),
            ),
            false,
        );
        assert_eq!(acknowledged.screen_state, ScreenState::Title);

        let mut new_game = app("title-new-game");
        new_game.screen_state = ScreenState::Title;
        assert!(new_game
            .translate_scene_input(vec![Command::Interact])
            .is_empty());
        assert_eq!(new_game.screen_state, ScreenState::Field);

        let mut continue_game = app("title-continue");
        continue_game.screen_state = ScreenState::Title;
        assert!(continue_game
            .translate_scene_input(vec![Command::Move(Dir::South), Command::Interact])
            .is_empty());
        assert_eq!(
            continue_game.screen_state,
            ScreenState::FileSelect(FileSelectMode::Load)
        );
    }

    #[test]
    fn empty_load_stays_in_picker_and_surfaces_typed_error() {
        let data_dir = temp_dir("empty-load");
        let mut app = App::new_with_catalog_and_items_and_store(
            42,
            ValidatedConfig::from_config(ShellConfig::default(), temp_dir("empty-load-config")),
            true,
            AuthoredSceneCatalog::default(),
            AuthoredItemCatalog::default(),
            Some(crate::persistence::SlotStore::new(data_dir, [7; 32])),
        )
        .expect("app should construct with an empty confined save store");
        app.screen_state = ScreenState::FileSelect(FileSelectMode::Load);
        let commands = [Command::Load(SaveSlot(0))];
        let events = app.apply_commands(&commands);
        app.apply_screen_commands(&commands, &events);

        assert!(matches!(
            events.as_slice(),
            [CoreEvent::Rejected { reason, .. }] if reason.contains("empty")
        ));
        assert_eq!(
            app.screen_state,
            ScreenState::FileSelect(FileSelectMode::Load)
        );
        assert!(app
            .file_error
            .as_deref()
            .unwrap_or_default()
            .contains("empty"));
    }

    #[test]
    fn successful_save_closes_picker_and_marks_slot_used() {
        let data_dir = temp_dir("successful-save");
        let store = crate::persistence::SlotStore::new(data_dir, [7; 32]);
        let mut app = App::new_with_catalog_and_items_and_store(
            42,
            ValidatedConfig::from_config(ShellConfig::default(), temp_dir("successful-config")),
            true,
            AuthoredSceneCatalog::default(),
            AuthoredItemCatalog::default(),
            Some(store),
        )
        .expect("app should construct with a confined save store");
        app.screen_state = ScreenState::FileSelect(FileSelectMode::Save);
        let commands = [Command::Save(SaveSlot(0))];
        let events = app.apply_commands(&commands);
        app.apply_screen_commands(&commands, &events);

        assert!(matches!(events.as_slice(), [CoreEvent::Applied { .. }]));
        assert_eq!(app.screen_state, ScreenState::Field);
        assert!(app.slot_occupied[0]);
        assert!(app.file_error.is_none());
    }
}
