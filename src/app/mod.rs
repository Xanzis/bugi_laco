pub mod mark;
mod plot;

use iced::widget::{button, column, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length, Sandbox, Settings};

use crate::reader::PartModel;
use crate::writer::Writer;
use mark::{Annotation, Mark, MarkedModel};

#[derive(Default)]
pub struct LacoApp {
    model: Option<MarkedModel>,
    canvas_state: plot::CanvasState,
    log: String,

    // contents of text input boxes
    constraint_text: String,
    force_text: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadModel,
    Clear,
    Plot(plot::PlotMessage),
    ConstraintChanged(String),
    ForceChanged(String),
    SetConstraint,
    SetForce,
    Write,
    Segmentify,
    Nop,
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
            Message::LoadModel => {
                self.model = Some(PartModel::load_dxf().into());
                self.canvas_state.request_redraw();

                self.log.push_str("loaded model\n"); // capture stderr from load?
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

                self.canvas_state.request_redraw();
            }
            Message::Write => {
                let mut writer = Writer::new();

                self.model.as_ref().map(|m| {
                    for b in m.bounds().cloned() {
                        writer.add_boundary(b);
                    }
                });

                writer.write("out.bbnd");
            }
            Message::Segmentify => {
                self.model.as_mut().map(|m| {
                    // TODO user-specified precision
                    m.segmentify(2.0);
                });

                self.canvas_state.request_redraw();
            }
            Message::Nop => (),
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

        let control_pane = column![
            button("Load Model")
                .padding(15)
                .on_press(Message::LoadModel),
            button("Clear").padding(8).on_press(Message::Clear),
            text_input(
                "constraint value",
                &self.constraint_text,
                Message::ConstraintChanged
            )
            .padding(8),
            button("Set Constraint")
                .padding(15)
                .on_press(Message::SetConstraint),
            text_input("force value", &self.force_text, Message::ForceChanged).padding(8),
            button("Set Force").padding(15).on_press(Message::SetForce),
            button("Write").padding(15).on_press(Message::Write),
            button("Segmentify")
                .padding(15)
                .on_press(Message::Segmentify),
        ]
        .padding(20)
        .spacing(20)
        .align_items(Alignment::Center)
        .width(Length::FillPortion(2));

        row![view_pane, control_pane]
            .padding(20)
            .spacing(20)
            .align_items(Alignment::Center)
            .into()
    }
}
