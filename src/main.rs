mod app;
mod reader;
mod writer;

use iced::{Sandbox, Settings};

fn main() -> iced::Result {
    app::LacoApp::run(Settings::default())
}
