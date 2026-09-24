//! The clock, falls and finishing. See `specs/0003-the-run.md`.

use crate::course::Course;
use crate::marble::Marble;

/// What a fall costs, in seconds.
pub const FALL_PENALTY: f32 = 3.0;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Phase {
    /// On the course, clock not started: the player can look around first.
    Ready,
    Running,
    /// Window focus lost. Nothing moves and the clock stops.
    Paused,
    Finished,
}

#[derive(Debug, Clone)]
pub struct Run {
    pub phase: Phase,
    /// Seconds, including any penalties.
    pub time: f32,
    pub falls: u32,
    pub gems: usize,
}

impl Run {
    pub fn new() -> Self {
        Self {
            phase: Phase::Ready,
            time: 0.0,
            falls: 0,
            gems: 0,
        }
    }

    /// The first move starts the clock, and nothing else does.
    pub fn first_move(&mut self) {
        if self.phase == Phase::Ready {
            self.phase = Phase::Running;
        }
    }

    pub fn tick(&mut self, dt: f32) {
        if self.phase == Phase::Running {
            self.time += dt;
        }
    }

    pub fn finish(&mut self, gems: usize) {
        if self.phase == Phase::Running || self.phase == Phase::Ready {
            self.gems = gems;
            self.phase = Phase::Finished;
        }
    }

    /// A fall costs time and is counted, but never ends the attempt.
    pub fn record_fall(&mut self) {
        self.falls += 1;
        self.time += FALL_PENALTY;
    }

    pub fn pause(&mut self) {
        if self.phase == Phase::Running {
            self.phase = Phase::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.phase == Phase::Paused {
            self.phase = Phase::Running;
        }
    }

    pub fn restart(&mut self) {
        *self = Run::new();
    }

    pub fn is_playing(&self) -> bool {
        self.phase == Phase::Running || self.phase == Phase::Ready
    }
}

impl Default for Run {
    fn default() -> Self {
        Self::new()
    }
}

/// Puts a fallen marble back at its checkpoint and charges it. Says whether
/// the marble had in fact fallen.
pub fn handle_fall(marble: &mut Marble, course: &Course, run: &mut Run) -> bool {
    if !course.has_fallen(marble.position) {
        return false;
    }

    marble.reset_to(course.checkpoint);
    run.record_fall();
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::vec3;

    #[test]
    fn the_clock_waits_for_the_first_move() {
        let mut run = Run::new();

        run.tick(1.0);
        run.tick(1.0);

        assert_eq!(run.time, 0.0, "the clock should not have started");
    }

    #[test]
    fn the_clock_runs() {
        let mut run = Run::new();
        run.first_move();

        run.tick(0.5);
        run.tick(0.25);

        assert!((run.time - 0.75).abs() < 1e-6, "time {}", run.time);
    }

    #[test]
    fn the_goal_stops_the_clock() {
        let mut run = Run::new();
        run.first_move();
        run.tick(2.0);

        run.finish(3);
        run.tick(5.0);

        assert!((run.time - 2.0).abs() < 1e-6, "time {}", run.time);
        assert_eq!(run.phase, Phase::Finished);
        assert_eq!(run.gems, 3);
    }

    #[test]
    fn a_fall_costs_time() {
        let mut run = Run::new();
        run.first_move();
        run.tick(1.0);

        run.record_fall();

        assert!(
            (run.time - (1.0 + FALL_PENALTY)).abs() < 1e-6,
            "time {}",
            run.time
        );
    }

    #[test]
    fn a_fall_returns_to_the_checkpoint() {
        let mut course = Course::first();
        let checkpoint = course
            .platforms
            .iter()
            .find(|platform| platform.kind == crate::course::PlatformKind::Checkpoint)
            .expect("a course has checkpoints")
            .resting_point();
        course.update_checkpoint(checkpoint);

        let mut marble = Marble::new(vec3(0.0, course.floor_level - 5.0, 0.0));
        marble.velocity = vec3(3.0, -20.0, 1.0);
        let mut run = Run::new();
        run.first_move();

        assert!(handle_fall(&mut marble, &course, &mut run));

        assert!(
            (marble.position - checkpoint).length() < 1e-3,
            "{:?}",
            marble.position
        );
        assert_eq!(
            marble.velocity,
            glam::Vec3::ZERO,
            "it should arrive at rest"
        );
    }

    #[test]
    fn falls_are_counted() {
        let course = Course::first();
        let mut run = Run::new();
        let mut marble = Marble::new(vec3(0.0, course.floor_level - 1.0, 0.0));

        handle_fall(&mut marble, &course, &mut run);
        marble.position.y = course.floor_level - 1.0;
        handle_fall(&mut marble, &course, &mut run);
        // back on the course, so this one is not a fall
        marble.position = course.start;
        handle_fall(&mut marble, &course, &mut run);

        assert_eq!(run.falls, 2);
    }

    #[test]
    fn finishing_keeps_the_result() {
        let mut run = Run::new();
        run.first_move();
        run.tick(12.5);
        run.record_fall();
        run.finish(4);

        assert_eq!(run.phase, Phase::Finished);
        assert_eq!(run.falls, 1);
        assert_eq!(run.gems, 4);
        assert!((run.time - 15.5).abs() < 1e-6, "time {}", run.time);
    }

    #[test]
    fn a_new_run_starts_clean() {
        let mut run = Run::new();
        run.first_move();
        run.tick(9.0);
        run.record_fall();
        run.finish(2);

        run.restart();

        assert_eq!(run.phase, Phase::Ready);
        assert_eq!(run.time, 0.0);
        assert_eq!(run.falls, 0);
        assert_eq!(run.gems, 0);
    }

    #[test]
    fn pausing_stops_the_clock() {
        let mut run = Run::new();
        run.first_move();
        run.tick(1.0);

        run.pause();
        run.tick(10.0);
        assert!((run.time - 1.0).abs() < 1e-6, "the clock ran while paused");

        run.resume();
        run.tick(0.5);
        assert!((run.time - 1.5).abs() < 1e-6, "time {}", run.time);
    }
}
