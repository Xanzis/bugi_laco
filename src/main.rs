mod app;
mod reader;

use iced::{Settings, Sandbox};

fn main() -> iced::Result {
    let model = reader::PartModel::load_dxf();

    println!("{:#?}", model);

    app::LacoApp::run(Settings::default())
}