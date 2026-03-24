#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
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
                    .spacing(10.0)
                    .align(Align::Center)
                    .justify(Justify::Center)
                    .child(
                        Text::new("Counter: ")
                            .font_weight(FontWeight::Black)
                            .height(24.0)
                            .font_size(24.0)
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
                .child(
                    button!("-")
                        .width(40)
                        .height(40)
                        .on_click(move |_| decrement_setter.set(|prev| prev.value -= 1)),
                )
                .child(
                    Text::new(state.value.to_string())
                        .font_size(24.0)
                        .font_weight(FontWeight::Medium)
                        .justify(Justify::Center)
                        .align(Align::Center),
                )
                .child(
                    button!("+")
                        .width(40)
                        .height(40)
                        .on_click(move |_| increment_setter.set(|prev| prev.value += 1)),
                ),
        )
    })
}
