use fast_image_resize::images::Image;
use iced::{
    Element, Rectangle, Task,
    widget::{Container, row},
    window::Screenshot,
};

mod canvas_as_canvas;

use canvas_as_canvas::{CanvasMessage, Number};

use image::DynamicImage;

use fast_image_resize::Resizer;

fn main() {
    let _ = iced::application(App::new, App::update, App::view)
        .title("Numbers")
        .run();
}

struct App {
    canvas: Number,
    canvas_bound: iced::Point,
    window_id: Option<iced::window::Id>,
    image_handle: Option<iced::advanced::image::Handle>,
}

#[derive(Debug, Clone)]
enum Message {
    //when received, the last point is the maximum point to determine the bounds
    CanvasMessage(CanvasMessage),
    UpdateWindowId(Option<iced::window::Id>),
    Screenshot(iced::window::Screenshot),
    FinishedDownsampling(iced::advanced::image::Handle),
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            App {
                canvas: Number::new(),
                canvas_bound: iced::Point::default(),
                window_id: None,
                image_handle: None,
            },
            iced::window::get_oldest().map(|f| Message::UpdateWindowId(f)),
        )
    }
    fn update(&mut self, _message: Message) -> Task<Message> {
        match _message {
            Message::CanvasMessage(CanvasMessage::LineComplete(vec)) => {
                self.canvas.add_line(vec[..vec.len() - 2].into());
                self.canvas_bound = vec[vec.len() - 1]; //last point is bounds
                self.canvas.redraw();

                if let Some(id) = self.window_id {
                    iced::window::screenshot(id).map(|f| Message::Screenshot(f))
                } else {
                    Task::none()
                }
            }
            Message::UpdateWindowId(Some(id)) => {
                self.window_id = Some(id);
                Task::none()
            }
            Message::UpdateWindowId(_) => {
                iced::window::get_oldest().map(|f| Message::UpdateWindowId(f))
            }
            Message::Screenshot(mut screenshot) => {
                screenshot = screenshot
                    .crop(Rectangle::with_size(iced::Size {
                        width: 500,
                        height: 500,
                    }))
                    .unwrap();
                Task::perform(downsample_screenshot(screenshot), |handle| {
                    Message::FinishedDownsampling(handle)
                })
            }
            Message::CanvasMessage(CanvasMessage::RedrawRequested) => {
                if let Some(id) = self.window_id {
                    iced::window::screenshot(id).map(|f| Message::Screenshot(f))
                } else {
                    Task::none()
                }
            }
            Message::FinishedDownsampling(handle) => {
                self.image_handle = Some(handle);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let mut content_row = row![];
        content_row = content_row.push(
            Container::new(self.canvas.view().map(Message::CanvasMessage))
                .width(iced::Length::Fill)
                .height(iced::Length::Fill),
        );
        if let Some(handle) = &self.image_handle {
            content_row = content_row.push(
                iced::widget::image(handle)
                    .width(iced::Length::Fill)
                    .height(iced::Length::Fill) //
                    .filter_method(iced::widget::image::FilterMethod::Nearest),
            );
        }
        Element::new(content_row)
    }
}

async fn downsample_screenshot(screenshot: Screenshot) -> iced::advanced::image::Handle {
    let thread_handle = tokio::task::spawn_blocking(move || {
        let src_image: DynamicImage =
            image::RgbaImage::from_raw(500, 500, screenshot.bytes.clone().into())
                .expect("rgba conversion")
                .into();

        let dst_width = 8;
        let dst_height = 8;
        let mut dst_image = Image::new(dst_width, dst_height, fast_image_resize::PixelType::U8x4);

        let mut resizer = Resizer::new();
        resizer.resize(&src_image, &mut dst_image, None).unwrap();

        iced::advanced::image::Handle::from_rgba(dst_width, dst_height, dst_image.into_vec())
    });
    thread_handle.await.unwrap()
}
