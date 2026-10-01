//! The rolling, falling ball. See `specs/0001-rolling.md` and
//! `specs/0006-it-rolls.md`.

use blitzkit::collision::{Aabb, Sphere};
use blitzkit::physics::{self, Body};
use glam::{vec3, Quat, Vec2, Vec3};

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

/// How heavy the marble is. Nothing else has mass yet, so this only decides
/// how hard a bounce is, not who wins a shove.
pub const MASS: f32 = 1.0;

/// How much of a drop comes back. Low: a marble that bounces far is a marble
/// you are not driving.
pub const BOUNCE: f32 = 0.18;

/// How much the course grips. High enough that the ball rolls rather than
/// slides, which is what spec 0030's friction is for.
pub const GRIP: f32 = 0.9;

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
    /// Position, velocity and spin together, per blitzkit spec 0030. The ball
    /// used to be a point that was moved and stopped; it has weight now, and
    /// the course bounces it and turns it.
    pub body: Body,
    /// Which way round it has turned, which is the spin added up. Nothing but
    /// drawing reads it.
    pub facing: Quat,
    pub on_ground: bool,
    /// The downward speed of a landing that happened this update, if one did.
    /// See `specs/0005-landing-sound.md`. Read it after `update` and take it;
    /// the next `update` overwrites it either way.
    pub landing: Option<f32>,
}

impl Marble {
    pub fn new(position: Vec3) -> Self {
        Self {
            body: Body::new(position, RADIUS, MASS)
                .with_restitution(BOUNCE)
                .with_friction(GRIP),
            facing: Quat::IDENTITY,
            on_ground: false,
            landing: None,
        }
    }

    pub fn position(&self) -> Vec3 {
        self.body.position
    }

    /// Puts the marble somewhere and takes all its speed away, which is what a
    /// fall does, per spec 0003.
    pub fn reset_to(&mut self, position: Vec3) {
        self.body.position = position;
        self.body.velocity = Vec3::ZERO;
        self.body.spin = Vec3::ZERO;
        self.on_ground = false;
        self.landing = None;
    }

    pub fn sphere(&self) -> Sphere {
        Sphere::new(self.body.position, RADIUS)
    }

    pub fn speed(&self) -> f32 {
        let moving = self.body.velocity;

        vec3(moving.x, 0.0, moving.z).length()
    }

    /// One step: push, coast, fall, then move through the course.
    /// Drives it, and lets spec 0030 do the falling, hitting and turning.
    ///
    /// The handling stays the game's: how hard a held direction pushes, what
    /// the top speed is, how little of it works in the air, and how a coast
    /// settles. Physics owns what the course does back, which is the part that
    /// used to be faked.
    pub fn update(&mut self, drive: Vec3, dt: f32, colliders: &[Aabb]) {
        let was_on_ground = self.on_ground;
        let control = if self.on_ground { 1.0 } else { AIR_CONTROL };
        let mut flat = vec3(self.body.velocity.x, 0.0, self.body.velocity.z);

        if drive.length_squared() > 0.0 {
            flat += drive.normalize_or_zero() * ACCELERATION * control * dt;
            if flat.length() > MAX_SPEED {
                flat = flat.normalize_or_zero() * MAX_SPEED;
            }
        } else if self.on_ground {
            // coasting: take a fixed amount off rather than scaling, so it
            // settles. Rolling costs nothing under spec 0030, so without this
            // a ball let go of would roll until it fell off something.
            let slowed = (flat.length() - FRICTION * dt).max(0.0);
            flat = flat.normalize_or_zero() * slowed;
        }

        self.body.velocity = vec3(flat.x, self.body.velocity.y, flat.z);

        // how fast it is going down before anything stops it. Taken here
        // because the step is what takes the speed away, and afterwards there
        // is nothing left to measure.
        let impact = (-self.body.velocity.y).max(0.0);

        let mut one = [self.body];
        physics::step(&mut one, colliders, Vec3::NEG_Y * GRAVITY, dt);
        self.body = one[0];

        // a long drop stays readable rather than becoming a blur
        self.body.velocity.y = self.body.velocity.y.max(-MAX_FALL);

        self.turn(dt);
        self.on_ground = is_on_ground(self.body.position, colliders);
        self.landing = (!was_on_ground && self.on_ground).then_some(impact);
    }

    /// Adds this step's spin to which way round it is.
    ///
    /// Spin is radians a second about the axis it points along, so the turn is
    /// that axis by that many radians times the time.
    fn turn(&mut self, dt: f32) {
        let spin = self.body.spin;
        let rate = spin.length();

        if rate > 1e-6 {
            self.facing = Quat::from_axis_angle(spin / rate, rate * dt) * self.facing;
            self.facing = self.facing.normalize();
        }
    }
}

/// True when a platform is within reach below the marble.
pub fn is_on_ground(position: Vec3, colliders: &[Aabb]) -> bool {
    let feet = Sphere::new(position - Vec3::Y * GROUND_REACH, RADIUS);

    colliders
        .iter()
        .any(|platform| feet.intersects_aabb(platform) && position.y > platform.max.y - RADIUS)
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
    fn a_rolling_marble_turns() {
        // spec 0030 spins it through friction, and nothing else does
        let mut marble = resting();
        assert_eq!(marble.facing, Quat::IDENTITY);

        for _ in 0..120 {
            marble.update(vec3(1.0, 0.0, 0.0), 1.0 / 60.0, &[floor()]);
        }

        assert!(marble.body.spin.length() > 1.0, "it never spun");
        assert!(
            marble.facing.angle_between(Quat::IDENTITY) > 0.5,
            "it spun but never turned"
        );
    }

    #[test]
    fn it_turns_the_way_it_rolls() {
        // rolling towards positive x turns it about negative z, or it looks
        // like a ball being dragged backwards
        let mut marble = resting();

        for _ in 0..120 {
            marble.update(vec3(1.0, 0.0, 0.0), 1.0 / 60.0, &[floor()]);
        }

        assert!(
            marble.body.spin.z < 0.0,
            "it is rolling the wrong way round"
        );
    }

    #[test]
    fn a_still_marble_does_not_turn() {
        let mut marble = resting();

        for _ in 0..120 {
            marble.update(Vec3::ZERO, 1.0 / 60.0, &[floor()]);
        }

        assert!(
            marble.facing.angle_between(Quat::IDENTITY) < 0.2,
            "it turned while standing still"
        );
    }

    #[test]
    fn a_reset_forgets_the_spin() {
        let mut marble = resting();
        for _ in 0..60 {
            marble.update(vec3(1.0, 0.0, 0.0), 1.0 / 60.0, &[floor()]);
        }

        marble.reset_to(vec3(0.0, 5.0, 0.0));

        assert_eq!(marble.body.spin, Vec3::ZERO);
        assert_eq!(marble.body.velocity, Vec3::ZERO);
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
        assert!(
            marble.speed() <= MAX_SPEED + 1e-3,
            "speed {}",
            marble.speed()
        );
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
        assert!(
            marble.speed() > rolling * 0.8,
            "it braked instead of coasting"
        );

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

        assert!(marble.body.velocity.y < 0.0);
        assert!(marble.position().y < 10.0);
        assert!(!marble.on_ground);
    }

    #[test]
    fn falling_has_a_limit() {
        let mut marble = Marble::new(vec3(0.0, 1000.0, 0.0));

        for _ in 0..600 {
            marble.update(Vec3::ZERO, 1.0 / 60.0, &[]);
        }

        assert!(
            marble.body.velocity.y >= -MAX_FALL - 1e-3,
            "{}",
            marble.body.velocity.y
        );
    }

    /// Drops a marble from `height` and returns the landing it reported, per
    /// `specs/0005-landing-sound.md`.
    fn drop_from(height: f32) -> Option<f32> {
        let mut marble = Marble::new(vec3(0.0, height, 0.0));

        for _ in 0..600 {
            marble.update(Vec3::ZERO, 1.0 / 60.0, &[floor()]);
            if let Some(impact) = marble.landing {
                return Some(impact);
            }
        }

        None
    }

    #[test]
    fn landing_is_a_transition() {
        let impact = drop_from(4.0).expect("falling onto the floor is a landing");
        assert!(impact > 0.0, "it landed at {}", impact);
    }

    #[test]
    fn resting_does_not_land_again() {
        let mut marble = resting();

        for _ in 0..120 {
            marble.update(Vec3::X, 1.0 / 60.0, &[floor()]);
            assert_eq!(
                marble.landing, None,
                "rolling along the floor is not a landing"
            );
        }
    }

    #[test]
    fn impact_speed_survives_the_landing() {
        let impact = drop_from(6.0).expect("it lands");

        // the platform takes the vertical speed, so anything reading velocity
        // after the landing sees nothing
        assert!(impact > 1.0, "impact was {}", impact);
    }

    #[test]
    fn a_harder_drop_lands_harder() {
        let short = drop_from(1.0).expect("it lands");
        let long = drop_from(20.0).expect("it lands");

        assert!(long > short, "{} should beat {}", long, short);
    }

    #[test]
    fn a_checkpoint_drop_is_silent() {
        // a fall puts the marble back on its checkpoint, a platform's height
        // above the floor, and that short drop should not thud
        let mut marble = resting();
        marble.reset_to(vec3(0.0, RADIUS, 0.0));
        assert_eq!(marble.landing, None, "a reset is not a landing");

        let impact = drop_from(RADIUS + 0.05).expect("it settles onto the floor");
        assert_eq!(
            crate::thud::volume(impact),
            None,
            "a settle of {} should be silent",
            impact
        );
    }

    #[test]
    fn landing_stops_the_fall() {
        let mut marble = Marble::new(vec3(0.0, 4.0, 0.0));

        for _ in 0..180 {
            marble.update(Vec3::ZERO, 1.0 / 60.0, &[floor()]);
        }

        assert!(marble.on_ground, "it should have landed");
        assert!(
            marble.body.velocity.y.abs() < 1.0,
            "still falling at {}",
            marble.body.velocity.y
        );
        // resting on top of the floor, not inside it
        assert!(
            (marble.position().y - RADIUS).abs() < 0.05,
            "y {}",
            marble.position().y
        );
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
        assert!(
            airborne.speed() > 0.0,
            "the air should still steer a little"
        );
    }

    #[test]
    fn a_graze_keeps_most_of_the_speed() {
        let wall = Aabb::from_center_size(vec3(2.0, 1.0, 0.0), vec3(1.0, 2.0, 20.0));
        let mut marble = resting();
        marble.body.velocity = vec3(1.0, 0.0, 8.0);

        marble.update(vec3(0.0, 0.0, 1.0), 1.0 / 60.0, &[floor(), wall]);

        // the push into the wall is gone, the pace along it is not
        assert!(
            marble.body.velocity.z > 7.0,
            "along the wall: {}",
            marble.body.velocity.z
        );
    }
}
