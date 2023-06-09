pub mod mark;
mod plot;

use iced::widget::{button, column, row, scrollable, text, text_input, pick_list};
use iced::{Alignment, Element, Length, Sandbox};

use crate::reader::PartModel;
use crate::writer::Writer;
use mark::{Annotation, MarkedModel};

#[derive(Default)]
pub struct LacoApp {
    model: Option<MarkedModel>,
    canvas_state: plot::CanvasState,
    log: String,

    selected_unit: Option<Unit>,

    // contents of text input boxes
    source_text: String,
    constraint_text: String,
    force_text: String,
    size_text: String,
    material_text: String,
    thickness_text: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    SourceChanged(String),
    LoadModel,
    UnitSelected(Unit),
    Clear,
    Plot(plot::PlotMessage),
    ConstraintChanged(String),
    ForceChanged(String),
    SetConstraint,
    SetForce,
    Write,
    SizeChanged(String),
    Segmentify,
    MaterialChanged(String),
    ThicknessChanged(String),
}

impl Sandbox for LacoApp {
    type Message = Message;

    fn new() -> LacoApp {
        LacoApp::default()
    }

    fn title(&self) -> String {
        String::from("bugi_laco: a boundary definition tool for bugi")
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::SourceChanged(s) => {
                self.source_text = s;
            }
            Message::LoadModel => {
                self.model = Some(PartModel::load_dxf(&self.source_text).into());
                self.canvas_state.request_redraw();

                self.source_text.clear();

                self.log.push_str("loaded model\n"); // capture stderr from load?
            }
            Message::UnitSelected(u) => {
                self.selected_unit = Some(u);
            }
            Message::Clear => {
                self.model = None;
                self.canvas_state = plot::CanvasState::default();
            }
            Message::Plot(pm) => match pm {
                plot::PlotMessage::Redraw => {
                    self.canvas_state.request_redraw();
                }
                plot::PlotMessage::Select(p) => {
                    self.model.as_mut().map(|m| {
                        m.clear_interactions();

                        let (edge, mark) = m.click_at_pos(p);
                        self.log
                            .push_str(&format!("selected {:?} with annotation {:?}\n", edge, mark));
                    });
                    self.canvas_state.request_redraw();
                }
            },
            Message::ConstraintChanged(c) => {
                self.constraint_text = c;
            }
            Message::ForceChanged(f) => {
                self.force_text = f;
            }
            Message::SetConstraint => {
                self.model.as_mut().map(|m| {
                    if let Some(annot) = Annotation::parse_constraint(&self.constraint_text) {
                        let marked = m.annotate_clicked(annot);

                        m.clear_interactions();

                        for (edge, mark) in marked {
                            self.log
                                .push_str(&format!("annotated {:?} with {:?}\n", edge, mark));
                        }
                    } else {
                        self.log.push_str("ill-formed constraint text\n");
                    }
                });

                self.constraint_text.clear();

                self.canvas_state.request_redraw();
            }
            Message::SetForce => {
                self.model.as_mut().map(|m| {
                    if let Some(annot) = Annotation::parse_force(&self.force_text) {
                        let marked = m.annotate_clicked(annot);

                        m.clear_interactions();

                        for (edge, mark) in marked {
                            self.log
                                .push_str(&format!("annotated {:?} with {:?}\n", edge, mark));
                        }
                    } else {
                        self.log.push_str("ill-formed force text\n");
                    }
                });

                self.force_text.clear();

                self.canvas_state.request_redraw();
            }
            Message::Write => {
                // TODO: set scale with a units radio button
                let scale = self.selected_unit.map(|u| u.scale()).unwrap_or(1.0);

                let mut writer = Writer::new().scale(scale);

                self.model.as_ref().map(|m| {
                    for b in m.bounds().cloned() {
                        writer.add_boundary(b);
                    }
                });

                if let Ok(thickness) = self.thickness_text.parse::<f64>() {
                    let thickness = thickness * scale;
                    writer.write("out.bbnd", &self.material_text, thickness);
                }

                self.material_text.clear();
                self.thickness_text.clear();
            }
            Message::SizeChanged(s) => {
                self.size_text = s;
            }
            Message::Segmentify => {
                if let Ok(s) = self.size_text.parse() {
                    self.model.as_mut().map(|m| {
                        // TODO user-specified precision
                        m.segmentify(s);
                    });

                    self.log.push_str(&format!("segmented with size {}\n", s));
                } else {
                    self.log.push_str("ill-formed segmentation size\n");
                }

                self.size_text.clear();

                self.canvas_state.request_redraw();
            }
            Message::MaterialChanged(m) => {
                self.material_text = m;
            }
            Message::ThicknessChanged(t) => {
                self.thickness_text = t;
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let view_pane = column![
            self.canvas_state
                .view(
                    self.model.as_ref(),
                    Length::FillPortion(1),
                    Length::FillPortion(4)
                )
                .map(Message::Plot),
            scrollable(text(&self.log)).height(Length::FillPortion(1))
        ]
        .padding(20)
        .spacing(20)
        .align_items(Alignment::Center)
        .width(Length::FillPortion(3));

        let load_field = row![
            text_input("source file path", &self.source_text)
                .on_input(Message::SourceChanged)
                .padding(8),
            button("Load").padding(8).on_press(Message::LoadModel)
        ]
        .spacing(10);

        let constraint_field = row![
            text_input("constraint value", &self.constraint_text)
                .on_input(Message::ConstraintChanged)
                .padding(8),
            button("Set").padding(8).on_press(Message::SetConstraint)
        ]
        .spacing(10);

        let force_field = row![
            text_input("force value", &self.force_text)
                .on_input(Message::ForceChanged)
                .padding(8),
            button("Set").padding(8).on_press(Message::SetForce)
        ]
        .spacing(10);

        let segment_field = row![
            text_input("arc segmentation length", &self.size_text)
                .on_input(Message::SizeChanged)
                .padding(8),
            button("Segmentify")
                .padding(8)
                .on_press(Message::Segmentify)
        ]
        .spacing(10);

        let write_field = row![
            text_input("material", &self.material_text)
                .on_input(Message::MaterialChanged)
                .padding(8),
            text_input("thickness", &self.thickness_text)
                .on_input(Message::ThicknessChanged)
                .padding(8),
            button("Write").padding(8).on_press(Message::Write)
        ]
        .spacing(10);

        let misc_field = row![
            button("Clear").padding(8).on_press(Message::Clear),
            pick_list(&Unit::ALL[..], self.selected_unit, Message::UnitSelected).placeholder("unit")
        ]
        .spacing(10);

        let control_pane = column![
            load_field,
            misc_field,
            constraint_field,
            force_field,
            segment_field,
            write_field,
        ]
        .padding(20)
        .spacing(20)
        .align_items(Alignment::Start)
        .width(Length::FillPortion(2));

        row![view_pane, control_pane]
            .padding(20)
            .spacing(20)
            .align_items(Alignment::Start)
            .into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Unit {
    #[default]
    Meter,
    Millimeter,
    Inch,
}

impl Unit {
    const ALL: [Unit; 3] = [
        Unit::Meter,
        Unit::Millimeter,
        Unit::Inch,
    ];

    fn scale(self) -> f64 {
        // the scale by which to multiply a value in these units to obtain a value in meters
        match self {
            Unit::Meter => 1.0,
            Unit::Inch => 2.54 / 100.0,
            Unit::Millimeter => 1.0 / 1000.0,
        }
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Unit::Meter => "m",
                Unit::Millimeter => "mm",
                Unit::Inch => "in",
            }
        )
    }
}