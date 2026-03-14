#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use aurora_ui::prelude::*;

fn main() {
    App::new()
        .title("Button Example")
        .min_size((310, 440))
        .font(include_bytes!("../../Roboto-Regular.ttf"))
        .run(|window, _frame_info| {
            window.root(
                col!()
                    .spacing(10.0)
                    .align(Align::Center)
                    .justify(Justify::Center)
                    .child(button(
                        ButtonOptions{
                            text_options: Text{
                                text: "Test".to_string(),
                                align: Align::Center,
                                font_size: 24.0,
                              ..Text::default()
                            },
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
