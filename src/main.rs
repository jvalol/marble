use blitzkit::start;

mod camera;
mod course;
mod input;
mod marble;
mod marble_game;
mod run;
mod thud;

use marble_game::MarbleGame;

fn main() {
    start("marble", Box::new(MarbleGame::new()));
}
