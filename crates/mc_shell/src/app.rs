//! Application state: frame loop, headless mode, configuration.
//!
//! SPEC-004 section 10: poll input -> commands, accumulate dt, tick fixed-timestep,
//! view = state_view(), draw(view, alpha).
//!
//! SPEC-004 section 11: MC_HEADLESS=1 suppresses window + audio, same simulation.

use crate::config::ValidatedConfig;
use crate::render::target::{ShellRenderTarget, INTERNAL_HEIGHT, INTERNAL_WIDTH};
use mc_core::command::{Command, StateView};
use mc_core::world::World;
use macroquad::prelude::*;
use tracing::info;

/// Fixed time step: 60 fps = 16.666... ms.
pub const FIXED_DT: f64 = 1.0 / 60.0;

/// Maximum accumulated time to prevent spiral of death.
pub const MAX_ACCUM: f64 = 0.25;

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
}

impl App {
    /// Create a new application from a seed and configuration.
    pub fn new(seed: u128, config: ValidatedConfig, headless: bool) -> Self {
        let world = World::new(seed);
        App {
            world,
            config,
            headless,
            accum: 0.0,
            alpha: 0.0,
            render_target: None,
        }
    }

    /// Advance the simulation by one frame's worth of fixed ticks.
    /// Called from headless mode.
    pub fn headless_update(&mut self) {
        let mut commands = Vec::new();
        // Headless mode reads no real input; just let the simulation tick
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

        // Take the render target out to avoid borrow conflicts
        let mut rt = self.render_target.take().expect("render target not initialised");
        rt.handle_resize();

        let view = StateView::from_world(&self.world, &[]);

        // Render to internal target
        rt.set_camera();
        clear_background(BLACK);

        // Draw tilemap pattern
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
        let mut commands = Vec::new();

        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::X) {
            commands.push(Command::CancelSelection);
        }
        if is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::Z)
            || is_key_pressed(KeyCode::Space)
        {
            commands.push(Command::Interact);
        }
        if is_key_pressed(KeyCode::C) {
            commands.push(Command::OpenMenu);
        }

        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            commands.push(Command::Move(mc_core::command::Dir::North));
        }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            commands.push(Command::Move(mc_core::command::Dir::South));
        }
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            commands.push(Command::Move(mc_core::command::Dir::West));
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            commands.push(Command::Move(mc_core::command::Dir::East));
        }

        commands
    }

    fn draw_tilemap(&self, _view: &StateView) {
        let tiles_x = INTERNAL_WIDTH / 16;
        let tiles_y = INTERNAL_HEIGHT / 16;
        for ty in 0..tiles_y {
            for tx in 0..tiles_x {
                let x = tx as f32 * 16.0;
                let y = ty as f32 * 16.0;
                let gray = ((tx + ty) % 2) as f32 * 0.1 + 0.15;
                draw_rectangle(
                    x,
                    y,
                    16.0,
                    16.0,
                    Color::from_rgba(
                        (gray * 255.0) as u8,
                        (gray * 255.0) as u8,
                        (gray * 255.0) as u8,
                        255,
                    ),
                );
                draw_rectangle_lines(x, y, 16.0, 16.0, 0.5, Color::from_rgba(40, 40, 50, 255));
            }
        }
    }

    fn draw_sprites(&self, _view: &StateView) {
        // M2 will fill in sprite rendering
    }

    fn draw_ui(&self, _view: &StateView) {
        // M4 will fill in UI rendering
    }
}
