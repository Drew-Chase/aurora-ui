#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
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
                col!().child(
                    TouchArea::new()
                        .on_mouse_down(move |button| {
                            if button == MouseButton::Left {
                                let _ = handle.drag_window();
                            }
                        }),
                ),
            )
        })
        .expect("Failed to run app");
}
