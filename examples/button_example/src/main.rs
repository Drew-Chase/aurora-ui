#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aurora_ui::aurora_platform::app::App;
use aurora_ui::aurora_widgets::column;
use aurora_ui::aurora_widgets::interactables::button::{button, ButtonOptions};
use aurora_ui::aurora_widgets::layout::{Align, Justify};

fn main() {
    App::new()
        .title("Button Example")
        .min_size((310, 440))
        .font(include_bytes!("../Roboto-Regular.ttf"))
        .run(|window, _frame_info| {
            window.root(
                column!()
                    .spacing(10.0)
                    .align(Align::Center)
                    .justify(Justify::Center)
                    .child(button(
                        ButtonOptions{
                            text: "Click Me!".into(),
                            on_click: Box::new(|event| {
                                println!("Button clicked at position: {:?}", event.position);
                            }),
                            ..Default::default()
                        }
                    )),
            );
        })
        .expect("Failed to run app");
}
