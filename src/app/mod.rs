mod plot;

use iced::widget::{button, column, text};
use iced::{Alignment, Element, Length, Sandbox, Settings};
use crate::reader::PartModel;

#[derive(Default)]
pub struct LacoApp {
	model: Option<PartModel>,
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
        		self.model = Some(PartModel::load_dxf());
        		self.canvas_state.request_redraw();
        	},
        	Message::Clear => {
        		self.model = None;
        		self.canvas_state = plot::CanvasState::default();
        	},
        	Message::Plot(pm) => {
        		match pm {
        			plot::PlotMessage::Redraw => {
        				self.canvas_state.request_redraw();
        			},
        		}
        	},
        	Message::Nop => (),
        }
    }

    fn view(&self) -> Element<Self::Message> {
        column![
            self.canvas_state.view(self.model.as_ref()).map(Message::Plot),
            button("Load Model").padding(15).on_press(Message::LoadModel),
            button("Clear").padding(8).on_press(Message::Clear),
        ]
        .padding(20)
        .spacing(20)
        .align_items(Alignment::Center)
        .into()
    }
}