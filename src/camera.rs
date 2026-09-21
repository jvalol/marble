//! The camera that follows the marble. See `specs/0004-camera-and-controls.md`.

use blitkit::collision::{Aabb, Ray};
use glam::{vec3, Vec3};

pub const DEFAULT_DISTANCE: f32 = 9.0;
pub const MIN_DISTANCE: f32 = 5.0;
pub const MAX_DISTANCE: f32 = 20.0;
/// How far above the marble the camera sits, before any pulling in.
pub const HEIGHT: f32 = 5.0;
/// How quickly it closes the gap to where it should be. Higher is snappier.
pub const LAG: f32 = 6.0;
/// How far short of a wall the camera stops, so it never sits inside one.
pub const WALL_MARGIN: f32 = 0.4;

#[derive(Debug, Clone)]
pub struct Follow {
  pub angle: f32,
  pub distance: f32,
  /// Where the camera actually is, which trails where it wants to be.
  pub position: Vec3,
  pub target: Vec3,
}

impl Follow {
  pub fn new(target: Vec3) -> Self {
    let mut camera = Self {
      angle: 0.0,
      distance: DEFAULT_DISTANCE,
      position: Vec3::ZERO,
      target,
    };
    camera.position = camera.wanted_position(target);
    camera
  }

  pub fn turn(&mut self, amount: f32) {
    self.angle += amount;
  }

  pub fn zoom(&mut self, amount: f32) {
    self.distance = (self.distance + amount).clamp(MIN_DISTANCE, MAX_DISTANCE);
  }

  /// Behind and above the marble, at the current angle.
  pub fn wanted_position(&self, target: Vec3) -> Vec3 {
    target
      + vec3(
        self.angle.sin() * self.distance,
        HEIGHT,
        self.angle.cos() * self.distance,
      )
  }

  /// Moves toward where it should be, then pulls in if a platform is in the
  /// way, so the marble never disappears behind a wall.
  pub fn update(&mut self, target: Vec3, dt: f32, colliders: &[Aabb]) {
    self.target = target;

    let wanted = self.wanted_position(target);
    // an exponential approach: frame rate independent, and it never overshoots
    let caught_up = 1.0 - (-LAG * dt).exp();
    self.position += (wanted - self.position) * caught_up;

    self.position = clear_of_walls(target, self.position, colliders);
  }
}

/// Pulls `camera` toward `target` until nothing is between them.
pub fn clear_of_walls(target: Vec3, camera: Vec3, colliders: &[Aabb]) -> Vec3 {
  let to_camera = camera - target;
  let distance = to_camera.length();
  if distance < 1e-4 {
    return camera;
  }

  let ray = Ray::new(target, to_camera);
  let blocked = colliders
    .iter()
    .filter_map(|platform| ray.hit_aabb(platform))
    .map(|hit| hit.distance)
    .filter(|hit| *hit < distance)
    .min_by(|a, b| a.total_cmp(b));

  match blocked {
    Some(hit) => target + ray.direction * (hit - WALL_MARGIN).max(1.0),
    None => camera,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn marble_at(position: Vec3) -> Follow {
    Follow::new(position)
  }

  #[test]
  fn the_camera_trails_the_marble() {
    let marble = vec3(0.0, 0.0, 0.0);
    let camera = marble_at(marble);

    // above it, and behind it along +z at the starting angle
    assert!(camera.position.y > marble.y, "{:?}", camera.position);
    assert!(camera.position.z > marble.z, "{:?}", camera.position);
    assert!((camera.position - marble).length() > MIN_DISTANCE);
  }

  #[test]
  fn turning_orbits_the_marble() {
    let marble = Vec3::ZERO;
    let mut camera = marble_at(marble);
    let before = camera.position;

    camera.turn(std::f32::consts::FRAC_PI_2);
    camera.update(marble, 1.0, &[]);

    let after = camera.position;
    assert!((after - before).length() > 1.0, "it did not move");
    // the same distance out, just somewhere else
    let flat = |p: Vec3| vec3(p.x, 0.0, p.z).length();
    assert!((flat(after) - flat(before)).abs() < 0.5, "{} {}", flat(after), flat(before));
  }

  #[test]
  fn the_camera_lags() {
    let mut camera = marble_at(Vec3::ZERO);
    let moved = vec3(0.0, 0.0, -10.0);

    camera.update(moved, 1.0 / 60.0, &[]);

    let wanted = camera.wanted_position(moved);
    // it has set off, but has not arrived
    assert!((camera.position - wanted).length() > 0.5, "it snapped");
    assert!(camera.position.z < DEFAULT_DISTANCE, "it did not move at all");
  }

  #[test]
  fn the_wheel_changes_the_distance() {
    let mut camera = marble_at(Vec3::ZERO);

    camera.zoom(-2.0);
    assert!((camera.distance - (DEFAULT_DISTANCE - 2.0)).abs() < 1e-6);

    // and it stops at the limits
    camera.zoom(-100.0);
    assert_eq!(camera.distance, MIN_DISTANCE);
    camera.zoom(100.0);
    assert_eq!(camera.distance, MAX_DISTANCE);
  }

  #[test]
  fn a_wall_pulls_the_camera_in() {
    let marble = Vec3::ZERO;
    let camera = vec3(0.0, 0.0, 10.0);
    // a wall standing between the two
    let wall = Aabb::from_center_size(vec3(0.0, 0.0, 5.0), vec3(10.0, 10.0, 1.0));

    let pulled = clear_of_walls(marble, camera, &[wall]);

    assert!(pulled.z < 5.0, "the camera is still behind the wall: {:?}", pulled);
    assert!(pulled.z > 0.0, "it went past the marble: {:?}", pulled);
  }

  #[test]
  fn a_clear_view_keeps_the_distance() {
    let marble = Vec3::ZERO;
    let camera = vec3(0.0, 5.0, 10.0);
    let somewhere_else = Aabb::from_center_size(vec3(40.0, 0.0, 0.0), Vec3::splat(4.0));

    let kept = clear_of_walls(marble, camera, &[somewhere_else]);

    assert!((kept - camera).length() < 1e-5, "{:?}", kept);
  }
}
