#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aurora_ui::aurora_platform::winit;
use aurora_ui::prelude::*;

fn main() {
    App::new()
        .title("Custom Titlebar Example")
        .min_size((310, 440))
        .position(WindowPosition::Center)
        .custom_titlebar(true)
        .run(|window, _frame_info| {
            let handle = window.window_handle().clone();
            window.root(
                col!()
                    .child(
                        Positioned::fixed((50.0, 50.0)).child(
                            BoxWidget::new()
                                .width(50)
                                .height(50)
                                .background_color(Color::RED.opacity(0.5))
                                .corners(Corners::all(5.0)),
                        ),
                    )
                    .child(titlebar(handle)),
            )
        })
        .expect("Failed to run app");
}

fn titlebar(window: std::sync::Arc<winit::window::Window>) -> impl Widget {
    Composite::new((), move |_, _| {
        let handle = window.clone();

        Box::new(
            Positioned::fixed((0.0, 0.0)).height(80.0).child(
                TouchArea::new()
                    .on_mouse_down(move |button| {
                        if button == MouseButton::Left {
                            handle.drag_window().expect("Failed to drag window");
                        }
                    })
                    .child(BoxWidget::new().background_color(Color::BLUE)),
            ),
        )
    })
}
