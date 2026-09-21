# marble

Roll a marble across a course of platforms to a goal, against a clock. The first
3D game built on `blitzkit`, which lives at `../blitzkit` and owns the window,
rendering, input, collision and sound.

## Build and test

Requires Rust 1.87 or newer, the engine's MSRV.

```
cargo build
cargo test
cargo run
cargo clippy
cargo fmt
```

## How work happens here

Behavior changes are spec driven:

1. **Write the spec first.** Copy `specs/TEMPLATE.md` to `specs/NNNN-short-name.md`
   and fill it in. Say what the game does, not how the code does it.
2. **Make the acceptance criteria testable.** Each one names the test that proves
   it, or goes under "Verified by hand" when it needs a window.
3. **Write the tests, then the code.** `cargo test` passes before a commit.
4. **Update the spec when behavior changes.** A spec that disagrees with the game
   is a bug in the spec.

## Layout

- `src/main.rs` — hands a `MarbleGame` to `blitzkit::start`.
- `src/marble_game.rs` — the `Game` impl and the state machine.
- `src/marble.rs` — the rolling, falling ball.
- `src/course.rs` — platforms, gems, checkpoints, the goal.
- `src/run.rs` — the clock, falls, finishing.
- `src/camera.rs` — the camera that follows and does not clip through walls.
- `src/input.rs` — engine events to held flags.

## Conventions

- **The engine does the hard parts.** Collision, rays and rendering come from
  blitzkit. This repo is game rules.
- **Units are world units and seconds.** Speeds are per second, multiplied by
  the frame's delta time.
- **Game logic is pure.** It takes input and state and touches no GPU, window or
  audio, which is what makes it testable.
- Tests live next to the code in `#[cfg(test)] mod tests`.
- Two space indentation, matching pong, snake and tetris.
