#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aurora_ui::aurora_platform::winit;
use aurora_ui::prelude::*;

const TITLEBAR_HEIGHT: f32 = 32.0;
const TITLEBAR_BG: Color = Color::new(32, 32, 32, 255);

fn main() {
    App::new()
        .title("Custom Titlebar Example")
        .icon(include_bytes!("../../../logo.png"))
        .min_size((310, 440))
        .position(WindowPosition::Center)
        .background_color(hsl!(0, 0.0, 0.1))
        .custom_titlebar(true)
        .use_system_fonts()
        .run(|window, _frame_info| {
            let handle = window.window_handle().clone();
            window.root(col!().child(titlebar(handle)));
        })
        .expect("Failed to run app");
}

fn titlebar(window: std::sync::Arc<winit::window::Window>) -> impl Widget {
    Composite::new((), move |_, _| {
        let drag_handle = window.clone();
        let controls_handle = window.clone();

        Box::new(
            Stack::new()
                .child(
                    Positioned::fixed((0.0, 0.0))
                        .height(TITLEBAR_HEIGHT)
                        .child(BoxWidget::new().background_color(TITLEBAR_BG)),
                )
                .child(
                    Positioned::fixed((0.0, 0.0))
                        .height(TITLEBAR_HEIGHT)
                        .child(WindowControls::new(controls_handle).dark(true)),
                )
                .child(
                    Positioned::fixed((0.0, 0.0)).height(TITLEBAR_HEIGHT).child(
                        row!().height(TITLEBAR_HEIGHT as u32).child(
                            TouchArea::new()
                                .child(
                                    Text::new(window.title())
                                        .color(Color::WHITE)
                                        .justify(Justify::Center)
                                        .padding(Edges::symmetric(13.0, 0.0))
                                        .height(TITLEBAR_HEIGHT)
                                        .font_size(13.0),
                                )
                                .on_mouse_down(move |button| {
                                    if button == MouseButton::Left {
                                        drag_handle.drag_window().expect("Failed to drag window");
                                    }
                                }),
                        ),
                    ),
                ),
        )
    })
}
