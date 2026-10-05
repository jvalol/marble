//! The `Game` impl: what happens each frame, and what gets drawn.

use blitzkit::camera::Camera;
use blitzkit::geometry::Geometry;
use blitzkit::keyboard::KeyboardInput;
use blitzkit::mesh::{MeshData, Transform};
use blitzkit::mouse::MouseInput;
use blitzkit::notice;
use blitzkit::renderer::render_text::{RenderText, TextRenderer, UNBOUNDED_F32};
use blitzkit::renderer::scene::{MeshId, Scene};
use blitzkit::renderer::Renderer;
use blitzkit::sound::SoundSystem;
use blitzkit::{Game, MAX_DELTA_TIME};
use glam::{vec2, vec4, Vec2, Vec3, Vec4};

use crate::camera::Follow;
use crate::course::{Course, PlatformKind};
use crate::input::{drive_direction, Input};
use crate::marble::{self, Marble};
use crate::run::{handle_fall, Phase, Run};
use crate::thud;

const MARBLE_COLOR: Vec4 = Vec4::new(0.95, 0.55, 0.15, 1.0);
/// The skin is already two colours, so the textured ball is tinted with
/// nothing. MARBLE_COLOR stays for the fallback, which is the ball as it was.
const SKIN_TINT: Vec4 = Vec4::ONE;

/// How wide to assume the window is until the engine says.
///
/// It says in `initialize`, before a frame is drawn, so this is only ever the
/// value a `MarbleGame` holds between being built and being handed a window.
const WIDE_AT_FIRST: f32 = 800.0;

/// The finish readout: how far down it sits, how big, and how far apart its
/// lines. Across is the middle of the window, which the game is told.
///
/// Apart is half again the size, which is a line of space between lines rather
/// than none. Three separate texts carry it, so the gap is this and nothing
/// else decides it.
const FINISH_TOP: f32 = 220.0;
const FINISH_SIZE: f32 = 22.0;
const FINISH_APART: f32 = FINISH_SIZE * 1.5;
const PLAIN_COLOR: Vec4 = Vec4::new(0.42, 0.45, 0.52, 1.0);
const CHECKPOINT_COLOR: Vec4 = Vec4::new(0.35, 0.55, 0.75, 1.0);
const GOAL_COLOR: Vec4 = Vec4::new(0.35, 0.75, 0.45, 1.0);
const GEM_COLOR: Vec4 = Vec4::new(0.95, 0.85, 0.35, 1.0);

pub struct MarbleGame {
    course: Course,
    marble: Marble,
    run: Run,
    follow: Follow,
    input: Input,
    cursor_locked: bool,
    want_cursor_locked: bool,
    /// Seconds since the course started, for bobbing the gems.
    time: f32,
    sphere: Option<MeshId>,
    /// What the marble is painted. Without it a turning ball and a still one
    /// look the same, and spec 0030's spin is wasted.
    skin: Option<blitzkit::renderer::scene::TextureId>,
    cube: Option<MeshId>,
    /// How wide the window is, so the finish readout can sit in the middle of
    /// it. The engine says so at startup and on every resize.
    width: f32,
}

impl MarbleGame {
    pub fn new() -> Self {
        let course = Course::first();
        let marble = Marble::new(course.start);
        let follow = Follow::new(course.start);

        Self {
            course,
            marble,
            run: Run::new(),
            follow,
            input: Input::new(),
            cursor_locked: false,
            want_cursor_locked: false,
            time: 0.0,
            sphere: None,
            skin: None,
            cube: None,
            width: WIDE_AT_FIRST,
        }
    }

    /// Where to put the thud so it is both heard and placed.
    ///
    /// The spatial output fades a sound by the square of its distance from the
    /// ears, measured in the units the ears are spaced in. The camera sits nine
    /// units back by default and twenty at full zoom, which would put a thud at
    /// the marble's own position somewhere between a hundredth and a four
    /// hundredth of its volume. That is silence.
    ///
    /// Distance carries nothing here anyway: the camera follows the marble, so
    /// it is always about the same distance away. Only the direction is worth
    /// hearing. So the sound goes one ear spacing from the listener, pointed at
    /// the marble, which keeps the side it came from and leaves the volume to
    /// `thud::volume` where it belongs.
    fn thud_position(&self) -> Vec3 {
        let ears = self.follow.position;
        let toward = (self.marble.position() - ears).normalize_or_zero();

        ears + toward * blitzkit::sound::EAR_DISTANCE
    }

    /// Back to the start of the course with a fresh clock.
    fn restart(&mut self) {
        self.course.reset();
        self.marble.reset_to(self.course.start);
        self.run.restart();
        self.follow = Follow::new(self.course.start);
        self.time = 0.0;
    }

    fn step(&mut self, dt: f32) {
        let colliders = self.course.colliders();

        if self.input.is_driving() {
            self.run.first_move();
        }

        let drive = drive_direction(&self.input, self.follow.angle);
        self.marble.update(drive, dt, &colliders);

        handle_fall(&mut self.marble, &self.course, &mut self.run);
        self.course.collect(&self.marble.sphere());
        self.course.update_checkpoint(self.marble.position());

        if self.course.on_goal(self.marble.position()) {
            self.run.finish(self.course.collected());
        }

        self.run.tick(dt);
    }
}

impl Default for MarbleGame {
    fn default() -> Self {
        Self::new()
    }
}

impl Game for MarbleGame {
    fn load(&mut self, renderer: &mut Renderer) {
        self.sphere = Some(renderer.add_mesh(&MeshData::sphere(24, 16)));
        self.skin = Some(renderer.add_texture(&crate::skin::checker()));
        self.cube = Some(renderer.add_mesh(&MeshData::cube()));

        // the shadow map covers the whole course rather than the default box
        let mut bounds = blitzkit::collision::Aabb::empty();
        for platform in self.course.platforms.iter() {
            bounds = bounds.union(&platform.bounds);
        }
        renderer.set_scene_bounds(bounds.expanded(Vec3::splat(6.0)));
    }

    fn initialize(
        &mut self,
        _geometry: &mut Geometry,
        _text_renderer: &mut TextRenderer,
        _sound_system: &SoundSystem,
        window_size: (f32, f32),
    ) {
        self.resized(window_size);
    }

    fn resized(&mut self, window_size: (f32, f32)) {
        self.width = window_size.0;
    }

    fn before_frame(&mut self, renderer: &mut Renderer) {
        if self.want_cursor_locked != renderer.cursor_locked() {
            // the platform may refuse, so believe what comes back
            self.cursor_locked = renderer.set_cursor_locked(self.want_cursor_locked);
            self.want_cursor_locked = self.cursor_locked;
        }
    }

    fn update(
        &mut self,
        dt: f32,
        geometry: &mut Geometry,
        text_renderer: &mut TextRenderer,
        sound_system: &SoundSystem,
    ) {
        let dt = dt.min(MAX_DELTA_TIME);

        if self.input.toggle_cursor {
            self.want_cursor_locked = !self.cursor_locked;
        }

        self.follow.turn(self.input.turn);
        self.follow.zoom(self.input.zoom);

        // held until the camera has caught up, because where the ears are when
        // a sound is queued is where it is heard from
        let landing = if self.run.is_playing() {
            self.time += dt;
            self.step(dt);
            self.marble.landing.take()
        } else {
            if self.run.phase == Phase::Finished && self.input.enter {
                self.restart();
            }
            None
        };

        self.follow
            .update(self.marble.position(), dt, &self.course.colliders());

        // the ears go where the camera is, looking where it looks, per the
        // engine's spec 0019. Without this they sit at the world origin, and
        // this course runs a hundred units out from it.
        sound_system.set_listener(
            self.follow.position,
            self.follow.target - self.follow.position,
            Vec3::Y,
        );

        // a landing the step found, loud in proportion to the drop and heard
        // from the side the marble is on
        if let Some(impact) = landing {
            if let Some(volume) = thud::volume(impact) {
                sound_system.queue_spatial(thud::thud(volume), self.thud_position().to_array());
            }
        }
        self.input.clear_frame();

        text_renderer.reset();
        self.draw_text(geometry, text_renderer);
    }

    fn draw(&mut self, scene: &mut Scene, camera: &mut Camera) {
        let (sphere, cube) = match (self.sphere, self.cube) {
            (Some(sphere), Some(cube)) => (sphere, cube),
            _ => return,
        };

        for platform in self.course.platforms.iter() {
            let color = match platform.kind {
                PlatformKind::Plain => PLAIN_COLOR,
                PlatformKind::Checkpoint => CHECKPOINT_COLOR,
                PlatformKind::Goal => GOAL_COLOR,
            };

            scene.push_colored(
                cube,
                &Transform::at(platform.bounds.center()).with_scale(platform.bounds.size()),
                color,
            );
        }

        // gems bob, which makes them read as collectable rather than as scenery
        for gem in self.course.gems.iter() {
            if gem.collected {
                continue;
            }
            let bob = (self.time * 2.0 + gem.position.z).sin() * 0.12;
            scene.push_material(
                sphere,
                &Transform::at(gem.position + Vec3::Y * bob)
                    .with_scale(Vec3::splat(crate::course::GEM_RADIUS * 2.0)),
                GEM_COLOR,
                96.0,
            );
        }

        // turned by how far it has rolled, and painted so that turning shows
        let rolled = Transform::at(self.marble.position())
            .with_rotation(self.marble.facing)
            .with_scale(Vec3::splat(marble::RADIUS * 2.0));

        match self.skin {
            Some(skin) => scene.push_textured(sphere, skin, &rolled, SKIN_TINT, 64.0),
            None => scene.push_material(sphere, &rolled, MARBLE_COLOR, 64.0),
        }

        camera.position = self.follow.position;
        camera.target = self.marble.position();
    }

    fn process_keyboard(&mut self, input: KeyboardInput) {
        self.input.keyboard(input);
    }

    fn process_mouse(&mut self, input: MouseInput) {
        self.input.mouse(input);
    }

    fn mouse_motion(&mut self, delta: Vec2) {
        self.input.mouse_motion(delta, self.cursor_locked);
    }

    fn mouse_wheel(&mut self, delta: Vec2) {
        self.input.mouse_wheel(delta);
    }

    fn is_quitting(&self) -> bool {
        self.input.quitting
    }

    fn focus_changed(&mut self, focus: bool) {
        // a staged run is photographed from behind the terminal and never has
        // focus, so pausing on losing it would photograph the pause screen
        if crate::staged() {
            return;
        }

        if focus {
            self.run.resume();
        } else {
            self.run.pause();
        }
    }
}

impl MarbleGame {
    fn draw_text(&self, geometry: &mut Geometry, text_renderer: &mut TextRenderer) {
        let line = |text: String, y: f32, size: f32| RenderText {
            position: vec2(20.0, y),
            color: vec4(1.0, 1.0, 1.0, 1.0),
            text,
            size,
            ..Default::default()
        };

        let lines: Vec<RenderText> = match self.run.phase {
            Phase::Finished => {
                // One text each rather than one with newlines in it. The lines
                // came out a size apart, and the blank line before the prompt
                // made that gap twice the first one.
                // vec!, not an array: edition 2018 hands out references.
                let done = vec![
                    format!("finished in {}", clock(self.run.time)),
                    format!(
                        "{} of {} gems, {} falls",
                        self.run.gems,
                        self.course.gem_count(),
                        self.run.falls
                    ),
                    String::from("enter to run it again"),
                ];

                done.into_iter()
                    .enumerate()
                    .map(|(n, text)| RenderText {
                        position: vec2(self.width * 0.5, FINISH_TOP + n as f32 * FINISH_APART),
                        bounds: (UNBOUNDED_F32, UNBOUNDED_F32).into(),
                        color: vec4(1.0, 1.0, 1.0, 1.0),
                        text,
                        size: FINISH_SIZE,
                        centered: true,
                        ..Default::default()
                    })
                    .collect()
            }
            _ => {
                let mut running = vec![
                    line(clock(self.run.time), 20.0, 24.0),
                    line(
                        format!(
                            "gems {} of {}   falls {}   speed {:.1}",
                            self.course.collected(),
                            self.course.gem_count(),
                            self.run.falls,
                            self.marble.speed()
                        ),
                        52.0,
                        14.0,
                    ),
                ];

                let hint = match self.run.phase {
                    Phase::Ready => "wasd rolls, drag turns, space locks the cursor",
                    Phase::Paused => "paused",
                    _ => "",
                };
                if !hint.is_empty() {
                    running.push(line(String::from(hint), 76.0, 14.0));
                }

                running
            }
        };

        // the readout goes on a panel, so it reads over the course rather than
        // into it. See blitzkit's spec 0038.
        geometry.reset();
        if let Some(frame) = notice::framing_all(&lines) {
            for quad in frame.iter() {
                geometry.push_quad(quad);
            }
        }

        for line in lines {
            text_renderer.push_render_text(line);
        }
    }
}

/// Seconds and hundredths, which is how a run is read.
pub fn clock(seconds: f32) -> String {
    let whole = seconds.max(0.0);
    let minutes = (whole / 60.0) as u32;
    let rest = whole - minutes as f32 * 60.0;

    if minutes > 0 {
        format!("{}:{:05.2}", minutes, rest)
    } else {
        format!("{:.2}", rest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clock_reads_as_seconds_then_minutes() {
        assert_eq!(clock(0.0), "0.00");
        assert_eq!(clock(9.5), "9.50");
        assert_eq!(clock(59.99), "59.99");
        assert_eq!(clock(61.25), "1:01.25");
    }

    /// Draws the finish readout into a renderer and hands back the lines.
    fn finish_lines(width: f32) -> Vec<RenderText> {
        let mut game = MarbleGame::new();
        game.resized((width, 600.0));
        game.run.phase = Phase::Finished;

        let mut text = TextRenderer::new();
        game.draw_text(&mut Geometry::new(), &mut text);

        text.render_texts
    }

    #[test]
    fn the_finish_readout_sits_in_the_middle_of_the_window() {
        // it was pinned at 400, which is the middle of a window 800 wide and
        // nowhere near it in any other
        for width in [800.0, 1440.0] {
            for line in finish_lines(width) {
                assert!(line.centered);
                assert_eq!(line.position.x, width * 0.5, "at {}", width);
            }
        }
    }

    #[test]
    fn the_finish_readout_is_evenly_spaced() {
        // one text with newlines in it put the lines a size apart and the
        // blank line before the prompt made that gap twice the first
        let lines = finish_lines(800.0);
        assert_eq!(lines.len(), 3);

        let gaps: Vec<f32> = lines
            .windows(2)
            .map(|pair| pair[1].position.y - pair[0].position.y)
            .collect();

        assert_eq!(gaps[0], gaps[1], "{:?}", gaps);
        assert!(gaps[0] > lines[0].size, "{:?}", gaps);
        assert!(lines.iter().all(|line| !line.text.contains('\n')));
    }

    #[test]
    fn a_new_game_starts_on_the_course() {
        let game = MarbleGame::new();

        assert_eq!(game.run.phase, Phase::Ready);
        assert!((game.marble.position() - game.course.start).length() < 1e-5);
        assert_eq!(game.course.collected(), 0);
    }

    #[test]
    fn restarting_puts_everything_back() {
        let mut game = MarbleGame::new();
        game.run.first_move();
        game.run.tick(5.0);
        game.run.record_fall();
        game.course.collect(&blitzkit::collision::Sphere::new(
            game.course.gems[0].position,
            marble::RADIUS,
        ));
        game.marble.body.position = glam::vec3(3.0, 9.0, -20.0);

        game.restart();

        assert_eq!(game.run.phase, Phase::Ready);
        assert_eq!(game.run.time, 0.0);
        assert_eq!(game.run.falls, 0);
        assert_eq!(game.course.collected(), 0);
        assert!((game.marble.position() - game.course.start).length() < 1e-5);
    }

    #[test]
    fn losing_focus_pauses_and_regaining_it_resumes() {
        let mut game = MarbleGame::new();
        game.run.first_move();

        game.focus_changed(false);
        assert_eq!(game.run.phase, Phase::Paused);

        game.focus_changed(true);
        assert_eq!(game.run.phase, Phase::Running);
    }

    #[test]
    fn rolling_starts_the_clock_and_moves_the_marble() {
        let mut game = MarbleGame::new();
        game.input.forward = true;

        let before = game.marble.position();
        for _ in 0..30 {
            game.step(1.0 / 60.0);
        }

        assert_eq!(game.run.phase, Phase::Running);
        assert!(game.run.time > 0.0);
        assert!(
            (game.marble.position() - before).length() > 0.5,
            "it did not roll"
        );
    }

    #[test]
    fn reaching_the_goal_finishes_the_run() {
        let mut game = MarbleGame::new();
        let goal = game
            .course
            .platforms
            .iter()
            .find(|platform| platform.kind == PlatformKind::Goal)
            .expect("a course has a goal")
            .resting_point();

        game.run.first_move();
        game.marble.reset_to(goal);
        game.step(1.0 / 60.0);

        assert_eq!(game.run.phase, Phase::Finished);
    }
}
