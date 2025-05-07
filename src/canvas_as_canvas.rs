use iced::widget::Action;
use iced::widget::canvas::{Cache, Frame, Path, Stroke};
use iced::{Element, Point, Renderer, widget::canvas::Program};
#[derive(Debug)]
pub struct Number {
    lines: Vec<Vec<Point>>,
    cache: Cache,
}
impl Default for Number {
    fn default() -> Self {
        Number {
            lines: vec![],
            cache: Cache::new(),
        }
    }
}
impl Number {
    pub fn view(&self) -> Element<CanvasMessage> {
        iced::widget::Canvas::new(self)
            .width(500.0)
            .height(500.0)
            .into()
    }
    pub fn new() -> Self {
        Number {
            lines: vec![],
            cache: Cache::new(),
        }
    }
    pub fn with_lines(mut self, lines: Vec<Vec<Point>>) -> Number {
        self.lines = lines;
        self
    }
    pub fn redraw(&mut self) {
        self.cache.clear();
    }
    pub fn add_line(&mut self, line: Vec<Point>) {
        self.lines.push(line);
    }
    // pub fn lines(&self) -> Vec<Vec<Point>> {
    //     self.lines.clone()
    // }
}
impl Clone for Number {
    fn clone(&self) -> Self {
        Number::new().with_lines(self.lines.clone())
    }
}
#[derive(Default)]
pub struct InternalState {
    incomplete_line: Vec<Point>,
}
#[derive(Debug, Clone)]
pub enum CanvasMessage {
    LineComplete(Vec<Point>),
    RedrawRequested,
}
impl Program<CanvasMessage> for Number {
    type State = Option<InternalState>;
    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry<Renderer>> {
        let cache_geom = self.cache.draw(renderer, bounds.size(), |frame| {
            let lines = Path::new(|build| {
                for line in &self.lines {
                    build.move_to(line[0]);
                    for point in &line[1..] {
                        build.line_to(*point);
                        build.rounded_rectangle(
                            iced::Point::new(point.x - 1.0, point.y - 1.0),
                            iced::Size::new(1.0, 1.0),
                            iced::border::radius(20),
                        );
                    }
                }
            });
            frame.stroke(
                &lines,
                Stroke::default()
                    .with_color(iced::Color::BLACK)
                    .with_width(40.0),
            );
        });
        if state.is_some() {
            let mut frame = Frame::new(renderer, bounds.size());
            frame.stroke(
                &Path::new(|build| {
                    build.move_to(state.as_ref().unwrap().incomplete_line[0]);
                    for point in &state.as_ref().unwrap().incomplete_line[1..] {
                        build.line_to(*point);
                        build.rounded_rectangle(
                            iced::Point::new(point.x - 1.0, point.y - 1.0),
                            iced::Size::new(1.0, 1.0),
                            iced::border::radius(20),
                        );
                    }
                }),
                Stroke::default()
                    .with_color(iced::Color::BLACK)
                    .with_width(40.0),
            );
            vec![cache_geom, frame.into_geometry()]
        } else {
            vec![cache_geom]
        }
    }
    fn update(
        &self,
        _state: &mut Self::State,
        _event: &iced::Event,
        _bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Option<Action<CanvasMessage>> {
        let in_bounds = _cursor.position_in(_bounds);
        if in_bounds.is_none() {
            return None;
        }
        let position = in_bounds.unwrap();
        match _event {
            iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) => {
                *_state = Some(InternalState {
                    incomplete_line: vec![_cursor.position().expect("In bounds")],
                });
                Some(Action::capture())
            }
            iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                if let Some(InternalState {
                    incomplete_line: vec,
                }) = _state.as_mut()
                {
                    if vec.len() == 1 {
                        vec.push(position);
                    }
                    let mut new_line = vec.clone();
                    new_line.push(Point {
                        x: _bounds.width,
                        y: _bounds.height,
                    });
                    *_state = None;
                    Some(Action::publish(CanvasMessage::LineComplete(new_line)))
                } else {
                    None
                }
            }
            iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                if let Some(InternalState {
                    incomplete_line: vec,
                }) = _state.as_mut()
                {
                    vec.push(*position);
                    Some(Action::publish(CanvasMessage::RedrawRequested))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> iced::advanced::mouse::Interaction {
        if _cursor.position_in(_bounds).is_some() {
            iced::advanced::mouse::Interaction::Crosshair
        } else {
            iced::advanced::mouse::Interaction::Idle
        }
    }
}
