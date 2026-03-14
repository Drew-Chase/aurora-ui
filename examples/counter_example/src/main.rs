use aurora_ui::prelude::*;

fn main() {
    App::new()
        .title("Button Example")
        .size((300, 300))
        .resizable(false)
        .font(include_bytes!("../../Roboto-Regular.ttf"))
        .run(|window, _frame_info| {
            window.root(
                col!()
                    .spacing(10.0)
                    .align(Align::Center)
                    .justify(Justify::Center)
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
                .height(80)
                .width(150)
                .spacing(10.0)
                .justify(Justify::Center)
                .align(Align::Center)
                .child(button(ButtonOptions {
                    text_options: Text {
                        text: "-".into(),
                        font_size: 20.0,
                        align: Align::Center,
                        ..Text::default()
                    },
                    width: 40,
                    height: 40,
                    on_click: Box::new(move |_| decrement_setter.set(|prev| prev.value -= 1)),
                    ..ButtonOptions::default()
                }))
                .child(
                    Text::new(state.value.to_string())
                        .font_size(20.0)
                        .align(Align::Center)
                        .padding(Edges::new(30.0, 0.0, 0.0, 0.0)),
                )
                .child(button(ButtonOptions {
                    text_options: Text {
                        text: "+".into(),
                        align: Align::Center,
                        font_size: 20.0,
                        ..Text::default()
                    },
                    width: 40,
                    height: 40,
                    on_click: Box::new(move |_| increment_setter.set(|prev| prev.value += 1)),
                    ..ButtonOptions::default()
                })),
        )
    })
}
