//! Engine events to held flags. See `specs/0004-camera-and-controls.md`.

use blitkit::keyboard::{KeyboardInput, KeyboardKey, KeyboardKeyState};
use blitkit::mouse::{MouseButton, MouseInput};
use glam::{Vec2, Vec3};

#[derive(Debug, Default)]
pub struct Input {
  pub forward: bool,
  pub back: bool,
  pub left: bool,
  pub right: bool,
  /// Pressed this frame, cleared once acted on.
  pub enter: bool,
  pub toggle_cursor: bool,
  pub quitting: bool,
  pub dragging: bool,
  /// Mouse movement gathered since the last frame.
  pub turn: f32,
  pub zoom: f32,
}

impl Input {
  pub fn new() -> Self {
    Default::default()
  }

  /// Which way the player is asking to roll, before the camera is considered.
  pub fn drive(&self) -> Vec2 {
    let x = (self.right as i32 - self.left as i32) as f32;
    let y = (self.forward as i32 - self.back as i32) as f32;

    Vec2::new(x, y)
  }

  pub fn is_driving(&self) -> bool {
    self.drive() != Vec2::ZERO
  }

  pub fn keyboard(&mut self, input: KeyboardInput) {
    let held = input.state == KeyboardKeyState::Pressed;

    match input.key {
      KeyboardKey::W | KeyboardKey::Up => self.forward = held,
      KeyboardKey::S | KeyboardKey::Down => self.back = held,
      KeyboardKey::A | KeyboardKey::Left => self.left = held,
      KeyboardKey::D | KeyboardKey::Right => self.right = held,
      KeyboardKey::Return if held && !input.repeat => self.enter = true,
      KeyboardKey::Space if held && !input.repeat => self.toggle_cursor = true,
      KeyboardKey::Escape => self.quitting = held,
      _ => (),
    }
  }

  pub fn mouse(&mut self, input: MouseInput) {
    if input.button == MouseButton::Left {
      self.dragging = input.is_pressed();
    }
  }

  /// Raw motion turns the camera while dragging, or always when the cursor is
  /// locked, per blitkit's spec 0013.
  pub fn mouse_motion(&mut self, delta: Vec2, cursor_locked: bool) {
    if self.dragging || cursor_locked {
      self.turn += delta.x * 0.005;
    }
  }

  pub fn mouse_wheel(&mut self, delta: Vec2) {
    self.zoom -= delta.y * 0.05;
  }

  /// Forgets what has been acted on. Held keys stay held.
  pub fn clear_frame(&mut self) {
    self.enter = false;
    self.toggle_cursor = false;
    self.turn = 0.0;
    self.zoom = 0.0;
  }
}

/// Where the marble should be pushed, given the camera.
pub fn drive_direction(input: &Input, camera_angle: f32) -> Vec3 {
  crate::marble::drive_direction(camera_angle, input.drive())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn key(key: KeyboardKey, state: KeyboardKeyState) -> KeyboardInput {
    KeyboardInput::new(key, state, false)
  }

  #[test]
  fn keys_drive_the_marble() {
    let mut input = Input::new();
    input.keyboard(key(KeyboardKey::W, KeyboardKeyState::Pressed));
    assert_eq!(input.drive(), Vec2::new(0.0, 1.0));

    input.keyboard(key(KeyboardKey::D, KeyboardKeyState::Pressed));
    assert_eq!(input.drive(), Vec2::new(1.0, 1.0));

    input.keyboard(key(KeyboardKey::W, KeyboardKeyState::Released));
    assert_eq!(input.drive(), Vec2::new(1.0, 0.0));
  }

  #[test]
  fn opposite_keys_cancel() {
    let mut input = Input::new();
    input.keyboard(key(KeyboardKey::A, KeyboardKeyState::Pressed));
    input.keyboard(key(KeyboardKey::D, KeyboardKeyState::Pressed));

    assert_eq!(input.drive().x, 0.0);
    assert!(!input.is_driving());
  }

  #[test]
  fn one_shot_presses_clear() {
    let mut input = Input::new();
    input.keyboard(key(KeyboardKey::Return, KeyboardKeyState::Pressed));
    input.keyboard(key(KeyboardKey::Space, KeyboardKeyState::Pressed));
    assert!(input.enter && input.toggle_cursor);

    input.clear_frame();

    assert!(!input.enter && !input.toggle_cursor);
  }

  #[test]
  fn turning_needs_a_drag_or_a_locked_cursor() {
    let mut input = Input::new();

    input.mouse_motion(Vec2::new(10.0, 0.0), false);
    assert_eq!(input.turn, 0.0, "a stray mouse should not turn the camera");

    input.mouse_motion(Vec2::new(10.0, 0.0), true);
    assert!(input.turn > 0.0);

    input.clear_frame();
    input.mouse(MouseInput::new(
      MouseButton::Left,
      blitkit::mouse::ButtonState::Pressed,
    ));
    input.mouse_motion(Vec2::new(10.0, 0.0), false);
    assert!(input.turn > 0.0, "dragging should turn it");
  }
}
