//! Platforms, gems, checkpoints and the goal. See `specs/0002-course.md`.

use crate::marble;
use blitzkit::collision::{Aabb, Sphere};
use glam::{vec3, Vec3};

pub const GEM_RADIUS: f32 = 0.3;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlatformKind {
  Plain,
  Checkpoint,
  Goal,
}

#[derive(Debug, Copy, Clone)]
pub struct Platform {
  pub bounds: Aabb,
  pub kind: PlatformKind,
}

impl Platform {
  fn new(center: Vec3, size: Vec3, kind: PlatformKind) -> Self {
    Self {
      bounds: Aabb::from_center_size(center, size),
      kind,
    }
  }

  pub fn top(&self) -> f32 {
    self.bounds.max.y
  }

  /// Where a marble sits when it is resting on this platform.
  pub fn resting_point(&self) -> Vec3 {
    let center = self.bounds.center();
    vec3(center.x, self.top() + marble::RADIUS, center.z)
  }

  /// True when the marble is on top of this platform rather than beside it.
  pub fn is_standing_on(&self, position: Vec3) -> bool {
    let feet = Sphere::new(
      position - Vec3::Y * marble::GROUND_REACH,
      marble::RADIUS,
    );

    feet.intersects_aabb(&self.bounds) && position.y > self.top() - marble::RADIUS
  }
}

#[derive(Debug, Copy, Clone)]
pub struct Gem {
  pub position: Vec3,
  pub collected: bool,
}

#[derive(Debug, Clone)]
pub struct Course {
  pub platforms: Vec<Platform>,
  pub gems: Vec<Gem>,
  pub start: Vec3,
  /// A marble below this has fallen off the course.
  pub floor_level: f32,
  /// Where a fall puts the marble back, which starts at the start.
  pub checkpoint: Vec3,
}

impl Course {
  /// The one course, built to teach its own moves: a wide start, a gap that
  /// needs speed, a narrow ledge that needs restraint, then a climb down to
  /// the goal.
  pub fn first() -> Self {
    let mut platforms = vec![
      // a wide pad to get the feel of it
      Platform::new(vec3(0.0, -0.5, 4.0), vec3(10.0, 1.0, 10.0), PlatformKind::Plain),
      // a straight run to build up speed
      Platform::new(vec3(0.0, -0.5, -6.5), vec3(5.0, 1.0, 11.0), PlatformKind::Checkpoint),
      // the gap: a drop, so only speed carries you across
      Platform::new(vec3(0.0, -2.5, -19.0), vec3(9.0, 1.0, 8.0), PlatformKind::Checkpoint),
      // a narrow ledge where speed is the enemy
      Platform::new(vec3(0.0, -3.5, -28.0), vec3(2.4, 1.0, 10.0), PlatformKind::Plain),
      // stepping down to the end
      Platform::new(vec3(-4.0, -4.5, -34.0), vec3(7.0, 1.0, 4.0), PlatformKind::Checkpoint),
      Platform::new(vec3(-10.0, -5.5, -34.0), vec3(6.0, 1.0, 6.0), PlatformKind::Goal),
    ];
    platforms.shrink_to_fit();

    // each one floats a little above the platform it belongs to
    let gems = vec![
      gem(vec3(0.0, 0.6, 0.0)),
      gem(vec3(0.0, 0.6, -9.0)),
      // near the far edge of the landing, for someone taking the gap fast
      gem(vec3(0.0, -1.4, -17.0)),
      // on the narrow ledge, which is the hard one
      gem(vec3(0.0, -2.4, -30.0)),
      gem(vec3(-10.0, -4.4, -34.0)),
    ];

    let start = platforms[0].resting_point();

    Self {
      platforms,
      gems,
      start,
      floor_level: -20.0,
      checkpoint: start,
    }
  }

  /// Every platform, as something to collide with.
  pub fn colliders(&self) -> Vec<Aabb> {
    self.platforms.iter().map(|platform| platform.bounds).collect()
  }

  /// Back to the beginning: gems uncollected, checkpoint at the start.
  pub fn reset(&mut self) {
    for gem in self.gems.iter_mut() {
      gem.collected = false;
    }
    self.checkpoint = self.start;
  }

  pub fn gem_count(&self) -> usize {
    self.gems.len()
  }

  pub fn collected(&self) -> usize {
    self.gems.iter().filter(|gem| gem.collected).count()
  }

  /// Collects whatever the marble is touching, and says how many that was.
  pub fn collect(&mut self, marble: &Sphere) -> usize {
    let mut taken = 0;

    for gem in self.gems.iter_mut() {
      if gem.collected {
        continue;
      }
      if marble.intersects(&Sphere::new(gem.position, GEM_RADIUS)) {
        gem.collected = true;
        taken += 1;
      }
    }

    taken
  }

  /// Remembers a checkpoint the marble is standing on, and says whether this
  /// is a new one.
  pub fn update_checkpoint(&mut self, position: Vec3) -> bool {
    for platform in self.platforms.iter() {
      if platform.kind != PlatformKind::Checkpoint || !platform.is_standing_on(position) {
        continue;
      }

      let point = vec3(position.x, platform.top() + marble::RADIUS, position.z);
      if (point - self.checkpoint).length() > 0.5 {
        self.checkpoint = point;
        return true;
      }
    }

    false
  }

  pub fn on_goal(&self, position: Vec3) -> bool {
    self
      .platforms
      .iter()
      .any(|platform| platform.kind == PlatformKind::Goal && platform.is_standing_on(position))
  }

  pub fn has_fallen(&self, position: Vec3) -> bool {
    position.y < self.floor_level
  }
}

fn gem(position: Vec3) -> Gem {
  Gem {
    position,
    collected: false,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn platforms_are_colliders() {
    let course = Course::first();
    let colliders = course.colliders();

    assert_eq!(colliders.len(), course.platforms.len());
    assert_eq!(colliders[0], course.platforms[0].bounds);
  }

  #[test]
  fn the_start_and_goal_rest_on_platforms() {
    let course = Course::first();

    assert!(
      course.platforms[0].is_standing_on(course.start),
      "the start is in the air"
    );

    let goal = course
      .platforms
      .iter()
      .find(|platform| platform.kind == PlatformKind::Goal)
      .expect("a course has a goal");
    assert!(goal.is_standing_on(goal.resting_point()));
  }

  #[test]
  fn a_gem_is_collected_once() {
    let mut course = Course::first();
    let gem = course.gems[0].position;
    let marble = Sphere::new(gem, marble::RADIUS);

    assert_eq!(course.collect(&marble), 1);
    assert_eq!(course.collected(), 1);
    // rolling over it again takes nothing
    assert_eq!(course.collect(&marble), 0);
    assert_eq!(course.collected(), 1);
  }

  #[test]
  fn a_distant_gem_is_left_alone() {
    let mut course = Course::first();
    let far_away = Sphere::new(vec3(100.0, 0.0, 100.0), marble::RADIUS);

    assert_eq!(course.collect(&far_away), 0);
    assert_eq!(course.collected(), 0);
  }

  #[test]
  fn the_goal_finishes_the_course() {
    let course = Course::first();
    let goal = course
      .platforms
      .iter()
      .find(|platform| platform.kind == PlatformKind::Goal)
      .expect("a course has a goal");

    assert!(course.on_goal(goal.resting_point()));
    assert!(!course.on_goal(course.start));
  }

  #[test]
  fn a_checkpoint_is_remembered() {
    let mut course = Course::first();
    let checkpoint = course
      .platforms
      .iter()
      .find(|platform| platform.kind == PlatformKind::Checkpoint)
      .expect("a course has checkpoints")
      .resting_point();

    assert!(course.update_checkpoint(checkpoint));
    assert!((course.checkpoint - checkpoint).length() < 1e-3);
    // standing there still does not make it new again
    assert!(!course.update_checkpoint(checkpoint));
  }

  #[test]
  fn the_latest_checkpoint_wins() {
    let mut course = Course::first();
    let checkpoints: Vec<Vec3> = course
      .platforms
      .iter()
      .filter(|platform| platform.kind == PlatformKind::Checkpoint)
      .map(|platform| platform.resting_point())
      .collect();

    course.update_checkpoint(checkpoints[0]);
    course.update_checkpoint(checkpoints[1]);

    assert!((course.checkpoint - checkpoints[1]).length() < 1e-3);
  }

  #[test]
  fn falling_below_the_course_counts_as_a_fall() {
    let course = Course::first();

    assert!(course.has_fallen(vec3(0.0, course.floor_level - 1.0, 0.0)));
    assert!(!course.has_fallen(course.start));
  }

  #[test]
  fn every_gem_is_reachable() {
    let course = Course::first();

    for gem in course.gems.iter() {
      let above = course.platforms.iter().any(|platform| {
        let bounds = platform.bounds;
        gem.position.x >= bounds.min.x - 0.5
          && gem.position.x <= bounds.max.x + 0.5
          && gem.position.z >= bounds.min.z - 0.5
          && gem.position.z <= bounds.max.z + 0.5
          && gem.position.y > platform.top()
          && gem.position.y < platform.top() + 3.0
      });

      assert!(above, "no platform under the gem at {:?}", gem.position);
    }
  }

  #[test]
  fn platforms_do_not_interpenetrate() {
    let course = Course::first();

    for (i, from) in course.platforms.iter().enumerate() {
      for (j, to) in course.platforms.iter().enumerate().skip(i + 1) {
        let shared = shared_volume(&from.bounds, &to.bounds);

        assert!(
          shared <= 0.0,
          "platforms {} and {} share {:.2} cubic units, so their faces land in \
           the same plane and fight over which one the camera sees",
          i,
          j,
          shared
        );
      }
    }
  }

  #[test]
  fn the_course_is_connected() {
    let course = Course::first();

    for pair in course.platforms.windows(2) {
      let (from, to) = (pair[0], pair[1]);
      let gap = horizontal_gap(&from.bounds, &to.bounds);
      let drop = from.top() - to.top();

      // rolling off at full speed, how far you get before you are level with
      // the next platform
      let fall_time = if drop > 0.0 {
        (2.0 * drop / marble::GRAVITY).sqrt()
      } else {
        0.0
      };
      let reach = marble::MAX_SPEED * fall_time;

      assert!(
        gap <= reach + 1e-3,
        "a {:.1} unit gap with only a {:.1} unit drop is not crossable (reach {:.1})",
        gap,
        drop,
        reach
      );
    }
  }

  /// How much space two boxes have in common. Platforms laid end to end touch
  /// without sharing any, which is what keeps their seams clean.
  fn shared_volume(a: &Aabb, b: &Aabb) -> f32 {
    let min = a.min.max(b.min);
    let max = a.max.min(b.max);
    let size = (max - min).max(Vec3::ZERO);

    size.x * size.y * size.z
  }

  /// How far apart two boxes are on the ground, zero when they overlap.
  fn horizontal_gap(from: &Aabb, to: &Aabb) -> f32 {
    let x = (to.min.x - from.max.x).max(from.min.x - to.max.x).max(0.0);
    let z = (to.min.z - from.max.z).max(from.min.z - to.max.z).max(0.0);

    (x * x + z * z).sqrt()
  }
}
