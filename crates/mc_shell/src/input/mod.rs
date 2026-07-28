//! Input handling: polling, remapping, translation to Command.
//!
//! SPEC-004 section 7. Fully remappable for keyboard and gamepad.
//! Defaults: arrows/WASD move, Z/Enter confirm, X/Esc cancel, C menu,
//! Shift run, Tab wait mode.

pub mod remap;

use crate::config::{InputAction, InputMap};
use mc_core::command::{Command, Dir};
use macroquad::prelude::*;

/// Poll input and translate to commands using the given input map.
pub fn poll_commands(input_map: &InputMap) -> Vec<Command> {
    let mut commands = Vec::new();

    // Check each input action
    if action_pressed(input_map, InputAction::Confirm) {
        commands.push(Command::Interact);
    }
    if action_pressed(input_map, InputAction::Cancel) {
        commands.push(Command::CancelSelection);
    }
    if action_pressed(input_map, InputAction::Menu) {
        commands.push(Command::OpenMenu);
    }

    // Directional movement (held keys)
    if action_down(input_map, InputAction::MoveUp) {
        commands.push(Command::Move(Dir::North));
    }
    if action_down(input_map, InputAction::MoveDown) {
        commands.push(Command::Move(Dir::South));
    }
    if action_down(input_map, InputAction::MoveLeft) {
        commands.push(Command::Move(Dir::West));
    }
    if action_down(input_map, InputAction::MoveRight) {
        commands.push(Command::Move(Dir::East));
    }

    // Shift = run modifier (not a command by itself)
    // Tab = wait mode toggle
    if action_pressed(input_map, InputAction::WaitMode) {
        commands.push(Command::SetWaitMode(true));
    }

    commands
}

/// Check if any key/gamepad binding for an action was just pressed.
fn action_pressed(map: &InputMap, action: InputAction) -> bool {
    if let Some(bindings) = map.get(&action) {
        for key_name in bindings {
            if let Some(kc) = key_name_to_keycode(key_name) {
                if is_key_pressed(kc) {
                    return true;
                }
            }
        }
    }
    false
}

/// Check if any key/gamepad binding for an action is currently held.
fn action_down(map: &InputMap, action: InputAction) -> bool {
    if let Some(bindings) = map.get(&action) {
        for key_name in bindings {
            if let Some(kc) = key_name_to_keycode(key_name) {
                if is_key_down(kc) {
                    return true;
                }
            }
        }
    }
    false
}

/// Convert a string key name to a macroquad KeyCode.
pub fn key_name_to_keycode(name: &str) -> Option<KeyCode> {
    match name {
        "Up" => Some(KeyCode::Up),
        "Down" => Some(KeyCode::Down),
        "Left" => Some(KeyCode::Left),
        "Right" => Some(KeyCode::Right),
        "W" => Some(KeyCode::W),
        "A" => Some(KeyCode::A),
        "S" => Some(KeyCode::S),
        "D" => Some(KeyCode::D),
        "Z" => Some(KeyCode::Z),
        "X" => Some(KeyCode::X),
        "C" => Some(KeyCode::C),
        "Enter" => Some(KeyCode::Enter),
        "Escape" => Some(KeyCode::Escape),
        "Space" => Some(KeyCode::Space),
        "Tab" => Some(KeyCode::Tab),
        "LeftShift" => Some(KeyCode::LeftShift),
        "RightShift" => Some(KeyCode::RightShift),
        "LShift" => Some(KeyCode::LeftShift),
        "RShift" => Some(KeyCode::RightShift),
        _ => None,
    }
}

/// Convert a KeyCode to a string key name.
pub fn keycode_to_key_name(kc: KeyCode) -> &'static str {
    match kc {
        KeyCode::Up => "Up",
        KeyCode::Down => "Down",
        KeyCode::Left => "Left",
        KeyCode::Right => "Right",
        KeyCode::W => "W",
        KeyCode::A => "A",
        KeyCode::S => "S",
        KeyCode::D => "D",
        KeyCode::Z => "Z",
        KeyCode::X => "X",
        KeyCode::C => "C",
        KeyCode::Enter => "Enter",
        KeyCode::Escape => "Escape",
        KeyCode::Space => "Space",
        KeyCode::Tab => "Tab",
        KeyCode::LeftShift => "LeftShift",
        KeyCode::RightShift => "RightShift",
        _ => "Unknown",
    }
}
