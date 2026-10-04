use blitzkit::start;

mod camera;
mod course;
mod input;
mod marble;
mod marble_game;
mod run;
mod skin;
mod thud;

use marble_game::MarbleGame;

/// Whether this run is only here to be photographed, for `refresh-screenshots`
/// in the project above.
pub fn staged() -> bool {
    std::env::args().any(|arg| arg == "--screenshot")
}

fn main() {
    start("marble", Box::new(MarbleGame::new()));
}
