//! Application state: frame loop, headless mode, configuration.
//!
//! SPEC-004 section 10: poll input -> commands, accumulate dt, tick fixed-timestep,
//! view = state_view(), draw(view, alpha).
//!
//! SPEC-004 section 11: MC_HEADLESS=1 suppresses window + audio, same simulation.
use crate::config::ValidatedConfig;
use crate::render::target::{ShellRenderTarget, INTERNAL_HEIGHT, INTERNAL_WIDTH};
use crate::render::tilemap::{Tilemap, TILES_X, TILES_Y};
use crate::ui::{battle::draw_battle_interface, menu::draw_menu_screen};
use mc_core::command::{Command, StateView};
use mc_core::world::World;
use macroquad::prelude::*;
use tracing::info;

/// Fixed time step: 60 fps = 16.666... ms.
pub const FIXED_DT: f64 = 1.0 / 60.0;

/// Maximum accumulated time to prevent spiral of death.
pub const MAX_ACCUM: f64 = 0.25;

/// Which screen overlay is currently active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenState {
    /// Field/exploration (default state).
    Field,
    /// Active battle.
    Battle,
    /// Menu screen open.
    Menu,
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
}

impl App {
    /// Create a new application from a seed and configuration.
    pub fn new(seed: u128, config: ValidatedConfig, headless: bool) -> Self {
        let world = World::new(seed);
        let audio_enabled = !headless;
        // Build an initial tilemap with a simple test pattern
        let mut tilemap = Tilemap::new();
        // Fill layer0 with a checkerboard pattern
        for y in 0..TILES_Y {
            for x in 0..TILES_X {
                let is_checker = ((x + y) % 2) as u16;
                tilemap.layer0.set_tile(x, y, is_checker);
                tilemap.layer1.set_tile(x, y, if is_checker == 0 { 2 } else { 3 });
            }
        }
        App {
            world,
            config,
            headless,
            accum: 0.0,
            alpha: 0.0,
            render_target: None,
            tilemap,
            audio: crate::audio::AudioState::new(audio_enabled),
            screen_state: ScreenState::Field,
        }
    }

    /// Advance the simulation by one frame's worth of fixed ticks.
    /// Called from headless mode.
    pub fn headless_update(&mut self) {
        let mut commands = Vec::new();
        self.accum += 1.0 / 60.0;
        if self.accum > MAX_ACCUM {
            self.accum = MAX_ACCUM;
        }
        while self.accum >= FIXED_DT {
            let _events = mc_core::command::apply_commands(&mut self.world, &commands);
            self.world.step();
            self.accum -= FIXED_DT;
        }
        self.alpha = (self.accum / FIXED_DT) as f32;
    }

    /// Process real input and advance the simulation.
    fn process_input_and_step(&mut self) -> Vec<Command> {
        let commands = self.poll_input();
        self.accum += get_frame_time() as f64;
        if self.accum > MAX_ACCUM {
            self.accum = MAX_ACCUM;
            info!("accumulation cap hit, dropping frames");
        }
        while self.accum >= FIXED_DT {
            let _events = mc_core::command::apply_commands(&mut self.world, &commands);
            self.world.step();
            self.accum -= FIXED_DT;
        }
        self.alpha = (self.accum / FIXED_DT) as f32;
        commands
    }

    /// Run one windowed frame. Called from macroquad's render loop.
    pub fn windowed_frame(&mut self) {
        // Process input and advance simulation
        self.process_input_and_step();

        // Update audio with current state
        self.audio.update(self.world.tick, self.world.act);

        // Take the render target out to avoid borrow conflicts
        let mut rt = self.render_target.take().expect("render target not initialised");
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
    }

    /// Poll input and translate to commands.
    fn poll_input(&self) -> Vec<Command> {
        // Use the remappable input system when available
        crate::input::poll_commands(&self.config.input_map)
    }

    /// Draw the tilemap layers (layer0, layer1, overlay).
    fn draw_tilemap(&self, _view: &StateView) {
        let tiles_x = INTERNAL_WIDTH / 16;
        let tiles_y = INTERNAL_HEIGHT / 16;

        // Draw layer 0 (background)
        for ty in 0..tiles_y {
            for tx in 0..tiles_x {
                let tile_id = self.tilemap.layer0.get_tile(tx, ty);
                let x = tx as f32 * 16.0;
                let y = ty as f32 * 16.0;

                // Map tile_id to a colour tint
                let (r, g, b) = match tile_id {
                    0 => (0.15, 0.15, 0.2),    // even checker
                    1 => (0.2, 0.25, 0.3),     // odd checker
                    2 => (0.25, 0.2, 0.15),    // alt
                    3 => (0.3, 0.25, 0.2),
                    _ => (0.1, 0.1, 0.1),
                };
                draw_rectangle(x, y, 16.0, 16.0, Color::from_rgba(
                    (r * 255.0) as u8,
                    (g * 255.0) as u8,
                    (b * 255.0) as u8,
                    255,
                ));
            }
        }

        // Draw layer 1 (midground) with scroll offset
        let scroll_x = self.tilemap.layer1.scroll_x;
        let scroll_y = self.tilemap.layer1.scroll_y;
        for ty in 0..tiles_y {
            for tx in 0..tiles_x {
                let tile_id = self.tilemap.layer1.get_tile(tx, ty);
                if tile_id == 0 {
                    continue;
                }
                let x = tx as f32 * 16.0 + scroll_x;
                let y = ty as f32 * 16.0 + scroll_y;
                let (r, g, b) = match tile_id {
                    2 => (0.35, 0.3, 0.25),
                    3 => (0.4, 0.35, 0.3),
                    _ => (0.2, 0.2, 0.25),
                };
                draw_rectangle(x, y, 16.0, 16.0, Color::from_rgba(
                    (r * 255.0) as u8,
                    (g * 255.0) as u8,
                    (b * 255.0) as u8,
                    255,
                ));
            }
        }

        // Grid overlay lines
        for ty in 0..=tiles_y {
            let y = ty as f32 * 16.0;
            draw_line(0.0, y, INTERNAL_WIDTH as f32, y, 0.5, Color::from_rgba(40, 40, 50, 255));
        }
        for tx in 0..=tiles_x {
            let x = tx as f32 * 16.0;
            draw_line(x, 0.0, x, INTERNAL_HEIGHT as f32, 0.5, Color::from_rgba(40, 40, 50, 255));
        }
    }

    fn draw_sprites(&self, _view: &StateView) {
        // M2: sprite rendering placeholder - uses Sprite struct from render::sprite
        // Full implementation in later milestones.
    }

    fn draw_ui(&self, _view: &StateView) {
        // Draw screen overlays conditionally based on current screen state
        match self.screen_state {
            ScreenState::Field => {
                // Field overlay: draw minimal HUD
            }
            ScreenState::Battle => {
                draw_battle_interface(self.world.tick);
            }
            ScreenState::Menu => {
                draw_menu_screen(self.world.tick);
            }
        }
    }
}
