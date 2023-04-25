use iced::widget::canvas::event::{self, Event};
use iced::widget::canvas::path::Builder;
use iced::widget::canvas::{self, Canvas, Cursor, Frame, Geometry, Path, Stroke};
use iced::Color;
use iced::{keyboard, mouse};
use iced::{Element, Length, Rectangle, Theme};

use spacemath::two::boundary::Boundary;

use super::mark::{MarkedBound, MarkedModel};
use crate::reader::PartModel;

#[derive(Default)]
pub struct CanvasState {
    cache: canvas::Cache,
}

impl CanvasState {
    pub fn view<'a>(&'a self, model: Option<&'a MarkedModel>) -> Element<'a, PlotMessage> {
        Canvas::new(Plot {
            canvas_state: self,
            model,
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    pub fn request_redraw(&mut self) {
        self.cache.clear()
    }
}

struct Plot<'a> {
    canvas_state: &'a CanvasState,
    model: Option<&'a MarkedModel>,
}

#[derive(Default, Clone, Debug)]
struct PlotState {
    transform: Transform,
}

#[derive(Clone, Debug)]
pub enum PlotMessage {
    Redraw,
    Hover(spacemath::two::Point),
    Select(spacemath::two::Point),
}

impl<'a> canvas::Program<PlotMessage> for Plot<'a> {
    type State = PlotState;

    fn draw(
        &self,
        state: &Self::State,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let content = self
            .canvas_state
            .cache
            .draw(bounds.size(), |frame: &mut Frame| {
                self.model.map(|m| draw_model(m, &state.transform, frame));

                frame.stroke(
                    &Path::rectangle(iced::Point::ORIGIN, frame.size()),
                    Stroke::default().with_width(2.0),
                );
            });

        vec![content]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (event::Status, Option<PlotMessage>) {
        let cursor_position = if let Some(position) = cursor.position_in(&bounds) {
            position
        } else {
            return (event::Status::Ignored, None);
        };

        match event {
            Event::Mouse(mouse_event) => {
                let message = match mouse_event {
                    mouse::Event::ButtonPressed(mouse::Button::Left) => Some(PlotMessage::Select(
                        state.transform.reverse(cursor_position),
                    )),
                    _ => None,
                };

                (event::Status::Captured, message)
            }
            Event::Keyboard(keyboard_event) => {
                match keyboard_event {
                    keyboard::Event::CharacterReceived('q') => {
                        state.transform.centered_zoom(1.25, bounds.center());
                        (event::Status::Captured, Some(PlotMessage::Redraw))
                    }
                    keyboard::Event::CharacterReceived('e') => {
                        state.transform.centered_zoom(0.75, bounds.center());
                        (event::Status::Captured, Some(PlotMessage::Redraw))
                    }
                    keyboard::Event::CharacterReceived('d') => {
                        state.transform.x_shift(0.2, bounds.center());
                        (event::Status::Captured, Some(PlotMessage::Redraw))
                    }
                    keyboard::Event::CharacterReceived('a') => {
                        state.transform.x_shift(-0.2, bounds.center());
                        (event::Status::Captured, Some(PlotMessage::Redraw))
                    }
                    keyboard::Event::CharacterReceived('w') => {
                        state.transform.y_shift(0.2, bounds.center());
                        (event::Status::Captured, Some(PlotMessage::Redraw))
                    }
                    keyboard::Event::CharacterReceived('s') => {
                        state.transform.y_shift(-0.2, bounds.center());
                        (event::Status::Captured, Some(PlotMessage::Redraw))
                    }
                    _ => (event::Status::Ignored, None),
                }
                // other key entries could send messages up to the top level app
            }
            _ => (event::Status::Ignored, None),
        }
    }
}

fn draw_bound(bound: &MarkedBound, transform: &Transform, frame: &mut Frame) {
    // dead simple to start - just plot polygon that shares the vertices of bound
    // arcs plot as lines for now

    let mut blank_builder = Builder::new();
    let mut marked_builder = Builder::new(); // replace soon with various marking types

    for (edge, mark) in bound.edges_and_marks() {
        if mark.is_empty() {
            // replace soon with proper draw handling (separate function to stay succint)
            blank_builder.move_to(transform.forward(edge.p()));
            blank_builder.line_to(transform.forward(edge.q()));
        } else {
            marked_builder.move_to(transform.forward(edge.p()));
            marked_builder.line_to(transform.forward(edge.q()));
        }
    }

    let blank_path = blank_builder.build();
    let marked_path = marked_builder.build();

    let marked_stroke = Stroke::default()
        .with_width(2.0)
        .with_color(Color::from_rgb8(0, 150, 250));

    frame.stroke(&blank_path, Stroke::default().with_width(2.0));
    frame.stroke(&marked_path, marked_stroke);
}

fn draw_model(model: &MarkedModel, transform: &Transform, frame: &mut Frame) {
    for b in model.bounds() {
        draw_bound(b, transform, frame);
    }
}

#[derive(Clone, Debug)]
pub struct Transform {
    pub scale: f64,
    pub offset: spacemath::two::Point,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            scale: 1.0,
            offset: spacemath::two::Point::origin(),
        }
    }
}

impl Transform {
    fn forward(&self, r: spacemath::two::Point) -> iced::Point {
        let r = r * self.scale;
        let r = r + self.offset;
        iced::Point {
            x: r.x as f32,
            y: -1.0 * r.y as f32,
        }
    }

    fn reverse(&self, r: iced::Point) -> spacemath::two::Point {
        let r = spacemath::two::Point {
            x: r.x as f64,
            y: -1.0 * r.y as f64,
        };
        let r = r - self.offset;
        r / self.scale
    }

    fn centered_zoom(&mut self, zoom: f64, center: iced::Point) {
        // proportional zoom in a way that preserves the center location
        let model_center = self.reverse(center);
        let delta = (model_center * zoom) - model_center;
        self.scale *= zoom;
        self.offset = self.offset - delta;
    }

    fn x_shift(&mut self, shift: f64, center: iced::Point) {
        let shift = ((center.x as f64) * 1.0) * shift;
        self.offset.x = self.offset.x + shift;
    }

    fn y_shift(&mut self, shift: f64, center: iced::Point) {
        let shift = ((center.y as f64) * 1.0) * shift;
        self.offset.y = self.offset.y + shift;
    }
}
