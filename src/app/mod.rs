mod mark;
mod plot;

use iced::widget::{button, column, text};
use iced::{Alignment, Element, Length, Sandbox, Settings};

use crate::reader::PartModel;
use mark::{Mark, MarkedModel};

#[derive(Default)]
pub struct LacoApp {
    model: Option<MarkedModel>,
    canvas_state: plot::CanvasState,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadModel,
    Clear,
    Plot(plot::PlotMessage),
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
                        m.mark_at_pos(p, Mark::clicked());
                    });
                    self.canvas_state.request_redraw();
                }
                plot::PlotMessage::Hover(_) => unimplemented!(),
            },
            Message::Nop => (),
        }
    }

    fn view(&self) -> Element<Self::Message> {
        column![
            self.canvas_state
                .view(self.model.as_ref())
                .map(Message::Plot),
            button("Load Model")
                .padding(15)
                .on_press(Message::LoadModel),
            button("Clear").padding(8).on_press(Message::Clear),
        ]
        .padding(20)
        .spacing(20)
        .align_items(Alignment::Center)
        .into()
    }
}
