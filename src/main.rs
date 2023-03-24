use eframe::egui;

mod reader;

fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    //tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };

    reader::PartModel::load_dxf();

    /* eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )*/
}

struct MyApp {
    name: String,
    age: u32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Click each year").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));

            let to_draw = egui::Shape::line(vec![(15.0, 10.0).into(), (20.0, 25.0).into()], egui::Stroke{width: 5.0, color: egui::Color32::RED});

            let plot_center = (100.0, 100.0).into();
            let plot_size = (100.0, 100.0).into();
            let plot_bounds = egui::Rect::from_center_size(plot_center, plot_size);

            let mut plot_ui = ui.child_ui(plot_bounds, Default::default());

            plot_ui.visuals_mut().panel_fill = egui::Color32::WHITE;

            plot_ui.heading("AAA");

            plot_ui.painter_at(plot_ui.clip_rect()).add(to_draw);
        });
    }
}