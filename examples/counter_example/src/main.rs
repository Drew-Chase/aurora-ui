#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use aurora_ui::aurora_platform::app::WindowPosition;
use aurora_ui::prelude::*;

fn main() {
    App::new()
        .title("Counter Example")
        .size((300, 150))
        .resizable(false)
        .position(WindowPosition::Center)
        .use_system_fonts()
        .font_options(FontOptions::new().family("Roboto"))
        .run(|window, _frame_info| {
            window.root(
                col!()
                    .spacing(0.0)
                    .align(Align::Center)
                    .justify(Justify::Center)
                    .child(
                        Text::new("Counter: ")
                            .font_weight(FontWeight::Black)
                            .font_size(24.0)
                            .height(24.0)
                            .align(Align::Center),
                    )
                    .child(counter()),
            );
        })
        .expect("Failed to run app");
}

#[derive(Default)]
struct CounterState {
    value: i32,
}

pub fn counter() -> impl Widget {
    Composite::new(CounterState::default(), move |state, set_state| {
        let decrement_setter = set_state.clone();
        let increment_setter = set_state.clone();

        Box::new(
            row!()
                .height(40)
                .width(150)
                .spacing(10.0)
                .justify(Justify::Center)
                .align(Align::Center)
                .child(button(ButtonOptions {
                    text_options: Text::new("-").font_size(20.0).align(Align::Center).justify(Justify::Center),
                    width: 40,
                    height: 40,
                    on_click: Box::new(move |_| decrement_setter.set(|prev| prev.value -= 1)),
                    ..ButtonOptions::default()
                }))
                .child(
                    Text::new(state.value.to_string())
                        .font_size(20.0)
                        .align(Align::Center)
                        .padding(Edges::new(20.0, 0.0, 0.0, 0.0)),
                )
                .child(button(ButtonOptions {
                    text_options: Text::new("+").font_size(20.0).height(40.0).width(40.0).align(Align::Center).justify(Justify::Center),
                    width: 40,
                    height: 40,
                    on_click: Box::new(move |_| increment_setter.set(|prev| prev.value += 1)),
                    ..ButtonOptions::default()
                })),
        )
    })
}
