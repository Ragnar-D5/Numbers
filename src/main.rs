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
    screenshot: Option<Screenshot>,
}

#[derive(Debug, Clone)]
enum Message {
    //when received, the last point is the maximum point to determine the bounds
    CanvasMessage(CanvasMessage),
    UpdateWindowId(Option<iced::window::Id>),
    Screenshot(iced::window::Screenshot),
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            App {
                canvas: Number::new(),
                canvas_bound: iced::Point::default(),
                window_id: None,
                screenshot: None,
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
                // todo!("compute bitmap")
            }
            Message::UpdateWindowId(Some(id)) => {
                self.window_id = Some(id);
                Task::none()
            }
            Message::UpdateWindowId(_) => {
                iced::window::get_oldest().map(|f| Message::UpdateWindowId(f))
            }
            Message::Screenshot(screenshot) => {
                self.screenshot = Some(
                    screenshot
                        .crop(Rectangle::with_size(iced::Size {
                            width: 500,
                            height: 500,
                        }))
                        .unwrap(),
                );
                Task::none()
            }
            Message::CanvasMessage(CanvasMessage::RedrawRequested) => {
                //slows everything down too much
                //
                // if let Some(id) = self.window_id {
                //     iced::window::screenshot(id).map(|f| Message::Screenshot(f))
                // } else {
                //     Task::none()
                // }
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
        if let Some(screenshot) = &self.screenshot {
            // let image = ImageReader::new(std::io::Cursor::new(screenshot.clone()))
            let src_image: DynamicImage =
                image::RgbaImage::from_raw(500, 500, screenshot.bytes.clone().into())
                    .expect("rgba conversion")
                    .into();
            // let mut dyn_image = DynamicImage::new_rgba8(500, 500);
            // dyn_image

            let dst_width = 8;
            let dst_height = 8;
            let mut dst_image =
                Image::new(dst_width, dst_height, fast_image_resize::PixelType::U8x4);

            // Create Resizer instance and resize source image
            // into buffer of destination image
            let mut resizer = Resizer::new();
            resizer.resize(&src_image, &mut dst_image, None).unwrap();

            // OpenOptions::new()
            //     .write(true)
            //     .truncate(true)
            //     .create(true)
            //     .open("/home/derivat/picture")
            //     .unwrap()
            //     .write_all(vec.clone().as_ref());

            // content_row = content_row.push(iced::widget::image(
            //     iced::advanced::image::Handle::from_bytes(vec.clone()),
            // ));

            content_row = content_row.push(
                iced::widget::image(iced::advanced::image::Handle::from_rgba(
                    dst_width,
                    dst_height,
                    dst_image.into_vec(),
                ))
                .width(iced::Length::Fill)
                .height(iced::Length::Fill) //
                .filter_method(iced::widget::image::FilterMethod::Nearest),
            );

            // content_row = content_row.push(iced::widget::image(
            //     iced::advanced::image::Handle::from_rgba(
            //         500,
            //         500,
            //         screenshot
            //             .crop(Rectangle::with_size(iced::Size {
            //                 width: 500,
            //                 height: 500,
            //             }))
            //             .expect("failed to crop"),
            //     ),
            // ))
        }
        Element::new(content_row) //.explain(iced::Color::from_rgb(1.0, 0.0, 0.0))
    }
}
