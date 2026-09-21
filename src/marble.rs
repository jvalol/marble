//! The rolling, falling ball. See `specs/0001-rolling.md`.

use blitkit::collision::{move_and_slide, Aabb, Sphere};
use glam::{vec3, Vec2, Vec3};

pub const RADIUS: f32 = 0.4;
/// How hard a held direction pushes, in units per second squared.
pub const ACCELERATION: f32 = 24.0;
pub const MAX_SPEED: f32 = 9.0;
/// How much speed a second of coasting takes off, on the ground.
pub const FRICTION: f32 = 6.0;
pub const GRAVITY: f32 = 30.0;
/// A long drop stays readable rather than becoming a blur.
pub const MAX_FALL: f32 = 40.0;
/// Enough control in the air to save a bad jump, not enough to fly.
pub const AIR_CONTROL: f32 = 0.25;
/// How close a platform has to be below to count as standing on it.
pub const GROUND_REACH: f32 = 0.08;

/// Where a held direction points in the world, given where the camera is.
///
/// `input.y` is forward and `input.x` is right. Forward is away from the
/// camera, and right is `cross(forward, up)`: get that backwards and the
/// controls mirror.
pub fn drive_direction(camera_angle: f32, input: Vec2) -> Vec3 {
  let forward = vec3(-camera_angle.sin(), 0.0, -camera_angle.cos());
  let right = vec3(-forward.z, 0.0, forward.x);
  let drive = forward * input.y + right * input.x;

  drive.normalize_or_zero()
}

#[derive(Debug, Clone)]
pub struct Marble {
  pub position: Vec3,
  pub velocity: Vec3,
  pub on_ground: bool,
}

impl Marble {
  pub fn new(position: Vec3) -> Self {
    Self {
      position,
      velocity: Vec3::ZERO,
      on_ground: false,
    }
  }

  /// Puts the marble somewhere and takes all its speed away, which is what a
  /// fall does, per spec 0003.
  pub fn reset_to(&mut self, position: Vec3) {
    self.position = position;
    self.velocity = Vec3::ZERO;
    self.on_ground = false;
  }

  pub fn sphere(&self) -> Sphere {
    Sphere::new(self.position, RADIUS)
  }

  pub fn speed(&self) -> f32 {
    vec3(self.velocity.x, 0.0, self.velocity.z).length()
  }

  /// One step: push, coast, fall, then move through the course.
  pub fn update(&mut self, drive: Vec3, dt: f32, colliders: &[Aabb]) {
    let control = if self.on_ground { 1.0 } else { AIR_CONTROL };
    let mut flat = vec3(self.velocity.x, 0.0, self.velocity.z);

    if drive.length_squared() > 0.0 {
      flat += drive.normalize_or_zero() * ACCELERATION * control * dt;
      if flat.length() > MAX_SPEED {
        flat = flat.normalize_or_zero() * MAX_SPEED;
      }
    } else if self.on_ground {
      // coasting: take a fixed amount off rather than scaling, so it settles
      let slowed = (flat.length() - FRICTION * dt).max(0.0);
      flat = flat.normalize_or_zero() * slowed;
    }

    let falling = (self.velocity.y - GRAVITY * dt).max(-MAX_FALL);
    self.velocity = vec3(flat.x, falling, flat.z);

    let wanted = self.velocity * dt;
    let before = self.position;
    self.position = move_and_slide(self.sphere(), self.velocity, dt, colliders);
    let moved = self.position - before;

    // whatever the course took, it took: an axis that was blocked loses its
    // speed, and the others keep theirs, which is what sliding along a wall is
    for axis in 0..3 {
      if wanted[axis].abs() > 1e-5 && moved[axis].abs() < wanted[axis].abs() * 0.5 {
        self.velocity[axis] = 0.0;
      }
    }

    self.on_ground = is_on_ground(self.position, colliders);
  }
}

/// True when a platform is within reach below the marble.
pub fn is_on_ground(position: Vec3, colliders: &[Aabb]) -> bool {
  let feet = Sphere::new(position - Vec3::Y * GROUND_REACH, RADIUS);

  colliders.iter().any(|platform| {
    feet.intersects_aabb(platform) && position.y > platform.max.y - RADIUS
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  fn floor() -> Aabb {
    Aabb::from_center_size(vec3(0.0, -0.5, 0.0), vec3(40.0, 1.0, 40.0))
  }

  /// A marble resting on the floor, which is where most of these start.
  fn resting() -> Marble {
    let mut marble = Marble::new(vec3(0.0, RADIUS, 0.0));
    marble.on_ground = true;
    marble
  }

  #[test]
  fn rolling_builds_speed() {
    let mut marble = resting();
    let mut last = 0.0;

    for _ in 0..10 {
      marble.update(Vec3::X, 1.0 / 60.0, &[floor()]);
      assert!(marble.speed() > last, "speed should be climbing");
      last = marble.speed();
    }

    // and it stops climbing at the limit
    for _ in 0..600 {
      marble.update(Vec3::X, 1.0 / 60.0, &[floor()]);
    }
    assert!(marble.speed() <= MAX_SPEED + 1e-3, "speed {}", marble.speed());
    assert!(marble.speed() > MAX_SPEED - 0.5, "speed {}", marble.speed());
  }

  #[test]
  fn friction_settles_the_marble() {
    let mut marble = resting();
    for _ in 0..60 {
      marble.update(Vec3::X, 1.0 / 60.0, &[floor()]);
    }
    let rolling = marble.speed();

    // let go: it should coast a while, not stop dead
    marble.update(Vec3::ZERO, 1.0 / 60.0, &[floor()]);
    assert!(marble.speed() > rolling * 0.8, "it braked instead of coasting");

    for _ in 0..600 {
      marble.update(Vec3::ZERO, 1.0 / 60.0, &[floor()]);
    }
    assert_eq!(marble.speed(), 0.0, "it should settle");
  }

  #[test]
  fn rolling_follows_the_camera() {
    let forward = Vec2::new(0.0, 1.0);

    // camera behind, looking down -z: forward rolls away from it
    let away = drive_direction(0.0, forward);
    assert!(away.z < -0.99, "{:?}", away);

    // swing the camera a quarter turn and forward turns with it
    let turned = drive_direction(std::f32::consts::FRAC_PI_2, forward);
    assert!(turned.x < -0.99, "{:?}", turned);

    // right is a quarter turn clockwise from forward, never its opposite
    let right = drive_direction(0.0, Vec2::new(1.0, 0.0));
    assert!(right.x > 0.99, "{:?}", right);
    assert!(right.dot(away).abs() < 1e-5);
  }

  #[test]
  fn gravity_pulls_it_down() {
    let mut marble = Marble::new(vec3(0.0, 10.0, 0.0));

    marble.update(Vec3::ZERO, 1.0 / 60.0, &[floor()]);

    assert!(marble.velocity.y < 0.0);
    assert!(marble.position.y < 10.0);
    assert!(!marble.on_ground);
  }

  #[test]
  fn falling_has_a_limit() {
    let mut marble = Marble::new(vec3(0.0, 1000.0, 0.0));

    for _ in 0..600 {
      marble.update(Vec3::ZERO, 1.0 / 60.0, &[]);
    }

    assert!(marble.velocity.y >= -MAX_FALL - 1e-3, "{}", marble.velocity.y);
  }

  #[test]
  fn landing_stops_the_fall() {
    let mut marble = Marble::new(vec3(0.0, 4.0, 0.0));

    for _ in 0..180 {
      marble.update(Vec3::ZERO, 1.0 / 60.0, &[floor()]);
    }

    assert!(marble.on_ground, "it should have landed");
    assert!(marble.velocity.y.abs() < 1.0, "still falling at {}", marble.velocity.y);
    // resting on top of the floor, not inside it
    assert!((marble.position.y - RADIUS).abs() < 0.05, "y {}", marble.position.y);
  }

  #[test]
  fn the_air_gives_less_control() {
    let mut grounded = resting();
    let mut airborne = Marble::new(vec3(0.0, 20.0, 0.0));

    for _ in 0..10 {
      grounded.update(Vec3::X, 1.0 / 60.0, &[floor()]);
      airborne.update(Vec3::X, 1.0 / 60.0, &[floor()]);
    }

    assert!(
      airborne.speed() < grounded.speed() * 0.5,
      "air {} ground {}",
      airborne.speed(),
      grounded.speed()
    );
    assert!(airborne.speed() > 0.0, "the air should still steer a little");
  }

  #[test]
  fn a_graze_keeps_most_of_the_speed() {
    let wall = Aabb::from_center_size(vec3(2.0, 1.0, 0.0), vec3(1.0, 2.0, 20.0));
    let mut marble = resting();
    marble.velocity = vec3(1.0, 0.0, 8.0);

    marble.update(vec3(0.0, 0.0, 1.0), 1.0 / 60.0, &[floor(), wall]);

    // the push into the wall is gone, the pace along it is not
    assert!(marble.velocity.z > 7.0, "along the wall: {}", marble.velocity.z);
  }
}

#[cfg(test)]
mod debug {
  use super::*;

  #[test]
  #[ignore]
  fn trace_a_long_roll() {
    let floor = Aabb::from_center_size(vec3(0.0, -0.5, 0.0), vec3(40.0, 1.0, 40.0));
    let mut marble = Marble::new(vec3(0.0, RADIUS, 0.0));
    marble.on_ground = true;

    for frame in 0..400 {
      marble.update(Vec3::X, 1.0 / 60.0, &[floor]);
      if frame % 40 == 0 {
        println!(
          "frame {:3} pos {:7.2},{:6.2},{:5.2} vel {:6.2},{:7.2} speed {:5.2} ground {}",
          frame,
          marble.position.x,
          marble.position.y,
          marble.position.z,
          marble.velocity.x,
          marble.velocity.y,
          marble.speed(),
          marble.on_ground
        );
      }
    }
  }
}
