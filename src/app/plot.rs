use iced::widget::canvas::event::{self, Event};
use iced::widget::canvas::path::{Arc, Builder};
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke};
use iced::Color;
use iced::{keyboard, mouse};
use iced::{Element, Length, Rectangle, Renderer, Theme};

use spacemath::two::boundary::Edge;

use super::mark::{MarkedBound, MarkedModel};

#[derive(Default)]
pub struct CanvasState {
    cache: canvas::Cache,
}

impl CanvasState {
    pub fn view<'a>(
        &'a self,
        model: Option<&'a MarkedModel>,
        w: Length,
        h: Length,
    ) -> Element<PlotMessage> {
        Canvas::new(Plot {
            canvas_state: self,
            model,
        })
        .width(w)
        .height(h)
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
    Select(spacemath::two::Point),
    Deselect,
}

impl<'a> canvas::Program<PlotMessage> for Plot<'a> {
    type State = PlotState;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let content = self
            .canvas_state
            .cache
            .draw(renderer, bounds.size(), |frame: &mut Frame| {
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
        cursor: mouse::Cursor,
    ) -> (event::Status, Option<PlotMessage>) {
        let cursor_position = if let Some(position) = cursor.position_in(bounds) {
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
                        state.transform.zoom(1.25, bounds.center());
                        (event::Status::Captured, Some(PlotMessage::Redraw))
                    }
                    keyboard::Event::CharacterReceived('e') => {
                        state.transform.zoom(0.75, bounds.center());
                        (event::Status::Captured, Some(PlotMessage::Redraw))
                    }
                    keyboard::Event::CharacterReceived('d') => {
                        state.transform.x_shift(-0.2, bounds.center());
                        (event::Status::Captured, Some(PlotMessage::Redraw))
                    }
                    keyboard::Event::CharacterReceived('a') => {
                        state.transform.x_shift(0.2, bounds.center());
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
                    keyboard::Event::CharacterReceived('c') => {
                        if let Some(m) = self.model {
                            state
                                .transform
                                .center_model(m.bounding_box(), bounds.center());
                            (event::Status::Captured, Some(PlotMessage::Redraw))
                        } else {
                            (event::Status::Captured, None)
                        }
                    }
                    keyboard::Event::KeyPressed {
                        key_code: keyboard::KeyCode::Escape,
                        ..
                    } => (event::Status::Captured, Some(PlotMessage::Deselect)),
                    _ => (event::Status::Ignored, None),
                }
                // other key entries could send messages up to the top level app
            }
            _ => (event::Status::Ignored, None),
        }
    }
}

fn draw_model(model: &MarkedModel, transform: &Transform, frame: &mut Frame) {
    for b in model.bounds() {
        draw_bound(b, transform, frame);
    }
}

fn draw_bound(bound: &MarkedBound, transform: &Transform, frame: &mut Frame) {
    // dead simple to start - just plot polygon that shares the vertices of bound
    // arcs plot as lines for now

    let mut blank_builder = Builder::new();
    let mut clicked_builder = Builder::new();
    let mut force_builder = Builder::new();
    let mut constraint_builder = Builder::new();

    for (edge, mark) in bound.edges_and_marks() {
        if mark.is_clicked() {
            build_edge(&mut clicked_builder, edge, transform);
        } else if mark.is_force() {
            build_edge(&mut force_builder, edge, transform);
        } else if mark.is_constraint() {
            build_edge(&mut constraint_builder, edge, transform);
        } else {
            build_edge(&mut blank_builder, edge, transform);
        }
    }

    let blank_path = blank_builder.build();
    let clicked_path = clicked_builder.build();
    let force_path = force_builder.build();
    let constraint_path = constraint_builder.build();

    let clicked_stroke = Stroke::default()
        .with_width(2.0)
        .with_color(Color::from_rgb8(0, 150, 250));
    let force_stroke = Stroke::default()
        .with_width(2.0)
        .with_color(Color::from_rgb8(250, 150, 0));
    let constraint_stroke = Stroke::default()
        .with_width(2.0)
        .with_color(Color::from_rgb8(0, 250, 0));

    frame.stroke(&blank_path, Stroke::default().with_width(2.0));
    frame.stroke(&clicked_path, clicked_stroke);
    frame.stroke(&force_path, force_stroke);
    frame.stroke(&constraint_path, constraint_stroke);
}

fn build_edge(builder: &mut Builder, edge: &Edge, transform: &Transform) {
    match *edge {
        Edge::Arc(a) => {
            // use unbounded direction-corrected angles
            let (p_ang, q_ang) = a.pq_ang_unbounded();

            let arc = Arc {
                center: transform.forward(a.center()),
                radius: transform.apply_scale(a.radius()) as f32,
                start_angle: (-1.0 * p_ang) as f32,
                end_angle: (-1.0 * q_ang) as f32,
            };
            builder.arc(arc);
        }
        Edge::Segment(_) => {
            builder.move_to(transform.forward(edge.p()));
            builder.line_to(transform.forward(edge.q()));
        }
    }
}

#[derive(Clone, Debug)]
pub struct Transform {
    scale: f64,
    screen_offset: iced::Vector<f32>,
    model_offset: spacemath::two::Point,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            scale: 1.0,
            screen_offset: [0.0, 0.0].into(),
            model_offset: spacemath::two::Point::origin(),
        }
    }
}

impl Transform {
    fn forward(&self, r: spacemath::two::Point) -> iced::Point {
        let r = r - self.model_offset;
        let r = r * self.scale;
        let r = iced::Point {
            x: r.x as f32,
            y: -1.0 * r.y as f32,
        };
        r + self.screen_offset
    }

    fn reverse(&self, r: iced::Point) -> spacemath::two::Point {
        let r = r - self.screen_offset;
        let r = spacemath::two::Point {
            x: r.x as f64,
            y: -1.0 * r.y as f64,
        };
        let r = r / self.scale;
        r + self.model_offset
    }

    fn center_model(
        &mut self,
        model_bound: (spacemath::two::Point, spacemath::two::Point),
        screen_center: iced::Point,
    ) {
        // get the model centered in the screen in a way that should zoom well
        // changing model_offset after this operation shouldn't be necessary

        let model_x_span = model_bound.1.x - model_bound.0.x;
        let model_center = model_bound.0.mid(model_bound.1);

        self.model_offset = model_center;
        self.screen_offset = [screen_center.x, screen_center.y].into();

        self.scale = screen_center.x as f64 / model_x_span;
    }

    fn zoom(&mut self, zoom: f64, center: iced::Point) {
        // zoom while maintaining the current center
        let current_center = self.reverse(center);

        self.model_offset = current_center;
        self.screen_offset = [center.x, center.y].into();

        self.scale *= zoom;
    }

    fn apply_scale(&self, x: f64) -> f32 {
        (x * self.scale) as f32
    }

    fn x_shift(&mut self, shift: f32, center: iced::Point) {
        self.screen_offset.x = self.screen_offset.x + (center.x * shift);
    }

    fn y_shift(&mut self, shift: f32, center: iced::Point) {
        self.screen_offset.y = self.screen_offset.y + (center.y * shift);
    }
}
