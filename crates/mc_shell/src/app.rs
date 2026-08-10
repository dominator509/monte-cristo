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
use crate::ui::{battle::draw_battle_interface, menu::draw_field_hud, menu::draw_menu_screen};
use macroquad::prelude::*;
use mc_core::command::{Command, StateView};
use mc_core::world::World;
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
    /// Whether the first-run content advisory is currently blocking gameplay.
    pub advisory_pending: bool,
}

impl App {
    /// Create a new application from a seed and configuration.
    pub fn new(seed: u128, config: ValidatedConfig, headless: bool) -> Self {
        let world = World::new(seed);
        let audio_enabled = !headless;
        let advisory_pending = !config.advisory_acknowledged && !headless;
        let tilemap = Tilemap::for_scene(world.act, world.region);
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
            advisory_pending,
        }
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
        if self.advisory_pending {
            self.draw_advisory_frame();
            return;
        }

        // Process input and advance simulation
        let commands = self.process_input_and_step();
        self.apply_screen_commands(&commands);

        // Update audio with current state
        self.audio.update(self.world.tick, self.world.act);

        // Take the render target out to avoid borrow conflicts
        let mut rt = self
            .render_target
            .take()
            .expect("render target not initialised");
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

    /// Render and acknowledge the first-run advisory before any gameplay input
    /// can reach the authoritative world.
    fn draw_advisory_frame(&mut self) {
        let mut rt = self
            .render_target
            .take()
            .expect("render target not initialised");
        rt.handle_resize();
        rt.set_camera();
        let acknowledged = draw_advisory_screen(self.world.tick);
        set_default_camera();
        rt.blit();
        self.render_target = Some(rt);

        if acknowledged {
            self.config.advisory_acknowledged = true;
            self.config.save();
            self.advisory_pending = false;
        }
    }

    /// Apply presentation-only screen transitions from the same command batch
    /// that was sent to `mc_core`.
    fn apply_screen_commands(&mut self, commands: &[Command]) {
        self.screen_state = screen_state_after(self.screen_state, commands);
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
                (_, command) => command,
            })
            .collect()
    }

    /// Draw the tilemap layers (layer0, layer1, overlay).
    fn draw_tilemap(&self, view: &StateView) {
        let tiles_x = INTERNAL_WIDTH / 16;
        let tiles_y = INTERNAL_HEIGHT / 16;
        let palette = scene_palette(view.act);

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

        // Draw layer 0 as a tiled ground plane over the sky.
        for ty in 0..tiles_y {
            for tx in 0..tiles_x {
                let tile_id = self.tilemap.layer0.get_tile(tx, ty);
                let x = tx as f32 * 16.0;
                let y = ty as f32 * 16.0;
                let colour_index = match tile_id {
                    0 => 3,
                    1 => 4,
                    2 => 5,
                    3 => 4,
                    _ => 7,
                };
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
                let colour_index = if tile_id == 2 { 5 } else { 6 };
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

        // Grid overlay lines
        for ty in 0..=tiles_y {
            let y = ty as f32 * 16.0;
            draw_line(0.0, y, INTERNAL_WIDTH as f32, y, 0.5, palette[7].to_color());
        }
        for tx in 0..=tiles_x {
            let x = tx as f32 * 16.0;
            draw_line(
                x,
                0.0,
                x,
                INTERNAL_HEIGHT as f32,
                0.5,
                palette[7].to_color(),
            );
        }

        // A crisp horizon line and a few fixed stars keep the low-resolution scene legible.
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
                x,
                20.0 + (x as u32 % 3) as f32 * 5.0,
                1.0,
                1.0,
                palette[6].to_color(),
            );
        }
    }

    fn draw_sprites(&self, view: &StateView) {
        let palette = scene_palette(view.act);
        let bob = ((view.tick / 12) % 2) as f32;
        let x = INTERNAL_WIDTH as f32 / 2.0 - 12.0;
        let y = 116.0 + bob;
        // Edmond's silhouette is intentionally drawn from rectangles so it stays crisp at 1x.
        draw_rectangle(x + 5.0, y, 14.0, 9.0, palette[6].to_color());
        draw_rectangle(x + 3.0, y + 8.0, 18.0, 18.0, palette[2].to_color());
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

    fn draw_ui(&self, view: &StateView) {
        // Draw screen overlays conditionally based on current screen state
        match self.screen_state {
            ScreenState::Field => {
                draw_field_hud(view, self.config.high_contrast);
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

/// Pure screen transition function used by the window loop and unit tests.
pub fn screen_state_after(current: ScreenState, commands: &[Command]) -> ScreenState {
    let mut state = current;
    for command in commands {
        match command {
            Command::OpenMenu if state == ScreenState::Field => state = ScreenState::Menu,
            Command::CloseMenu | Command::CancelSelection if state == ScreenState::Menu => {
                state = ScreenState::Field
            }
            _ => {}
        }
    }
    state
}
