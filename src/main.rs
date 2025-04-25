use iced::{
    Element, Subscription, Task,
    advanced::image::Bytes,
    futures::channel::mpsc::{self, Sender},
    widget::{Container, row},
};
use number_canvas::Number;

mod simulator;

use simulator::Simulator;
use sipper::{Never, Sipper, StreamExt, sipper};

fn main() {
    let _ = iced::application(App::new, App::update, App::view)
        .title("Numbers")
        .subscription(App::subscription)
        .run();
}

struct App {
    canvas: number_canvas::Number,
    canvas_bound: iced::Point,
    sender: Option<Sender<Command>>,
    render: Option<Bytes>,
}

#[derive(Debug)]
enum Message {
    //when received, the last point is the maximum point to determine the bounds
    CanvasMessage(number_canvas::LineComplete),
    Render(RenderMessage),
}

impl App {
    fn new() -> Self {
        App {
            canvas: Number::new(),
            canvas_bound: iced::Point::default(),
            sender: None,
            render: None,
        }
    }
    fn update(&mut self, _message: Message) -> Task<Message> {
        match _message {
            Message::CanvasMessage(vec) => {
                self.canvas.add_line(vec.0[..vec.0.len() - 2].into());
                self.canvas_bound = vec.0[vec.0.len() - 1]; //last point is bounds
                self.canvas.redraw();
                if let Some(sender) = &mut self.sender {
                    let _ = sender.try_send(Command::RenderSample {
                        canvas: self.canvas.clone(),
                    });
                }
                Task::none()
                // todo!("compute bitmap")
            }
            Message::Render(m) => match m {
                RenderMessage::Connected(sender) => {
                    self.sender = Some(sender);
                    Task::none()
                }
                RenderMessage::RenderCompleted(bytes) => {
                    self.render = Some(bytes);
                    Task::none()
                }
            },
        }
    }

    fn view(&self) -> Element<Message> {
        let canvas = Container::new(self.canvas.view().map(Message::CanvasMessage))
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into();
        if self.render.is_some() {
            println!("showing render");
            row![
                canvas,
                iced::widget::Image::new(iced::advanced::image::Handle::from_bytes(
                    self.render.clone().unwrap()
                ))
            ]
            .into()
        } else {
            canvas
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(stream).map(Message::Render)
    }
}

mod number_canvas {
    use iced::widget::Action;
    use iced::widget::canvas::{Cache, Frame, Path, Stroke};
    use iced::{Element, Point, Renderer, widget::canvas::Program};
    #[derive(Debug)]
    pub struct Number {
        lines: Vec<Vec<Point>>,
        cache: Cache,
    }
    impl Number {
        pub fn view(&self) -> Element<LineComplete> {
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
        fn with_lines(mut self, lines: Vec<Vec<Point>>) -> Number {
            self.lines = lines;
            self
        }
        pub fn redraw(&mut self) {
            self.cache.clear();
        }
        pub fn add_line(&mut self, line: Vec<Point>) {
            self.lines.push(line);
        }
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
    #[derive(Debug)]
    pub struct LineComplete(pub Vec<Point>);
    impl Program<LineComplete> for Number {
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
        ) -> Option<Action<LineComplete>> {
            let in_bounds = _cursor.position_in(_bounds);
            if in_bounds.is_none() {
                return None;
            }
            let position = in_bounds.unwrap();
            match _event {
                iced::Event::Mouse(iced::mouse::Event::ButtonPressed(
                    iced::mouse::Button::Left,
                )) => {
                    *_state = Some(InternalState {
                        incomplete_line: vec![_cursor.position().expect("In bounds")],
                    });
                    Some(Action::capture())
                }
                iced::Event::Mouse(iced::mouse::Event::ButtonReleased(
                    iced::mouse::Button::Left,
                )) => {
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
                        Some(Action::publish(LineComplete(new_line)))
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
                        Some(Action::request_redraw())
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
}

#[derive(Debug)]
enum Command {
    RenderSample { canvas: Number },
}

#[derive(Debug)]
enum RenderMessage {
    Connected(mpsc::Sender<Command>),
    RenderCompleted(iced::advanced::image::Bytes),
}

fn stream() -> impl Sipper<Never, RenderMessage> {
    use iced::futures::channel::mpsc;
    sipper(async move |mut event_sender| {
        let (command_sender, mut command_receiver) = mpsc::channel(100);

        let _ = event_sender
            .send(RenderMessage::Connected(command_sender))
            .await;

        let mut simulator: Simulator<iced::Renderer> = Simulator::new();

        loop {
            if let Some(command) = command_receiver.next().await {
                match command {
                    Command::RenderSample { mut canvas } => {
                        println!("Processing sample render request");
                        canvas.redraw();
                        let element = Element::new(iced::widget::Canvas::new(canvas));
                        let result = simulator
                            .screenshot(element, iced::Size::new(1000.0, 1000.0), 1.0)
                            .expect("should work because i have no clue what is happening")
                            .bytes;

                        println!("Render completed successfully");
                        let _ = event_sender
                            .send(RenderMessage::RenderCompleted(result))
                            .await;
                    }
                }
            }
        }
    })
}
