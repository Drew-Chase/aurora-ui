use aurora_ui::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Opens the second window. Call from a button callback with a cloned `AppContext`.
///
/// The `open_flag` is set to `false` when the window closes (by any means —
/// the X button, `close_window()`, or the parent closing) so the primary
/// window can re-enable the "Open" button.
pub fn open(ctx: &AppContext, open_flag: &Arc<AtomicBool>) {
    let flag_for_close = open_flag.clone();

    ctx.open_window(
        WindowConfig::new()
            .title("Second Window")
            .size((400, 300))
            .use_system_fonts()
            .on_close(move || {
                flag_for_close.store(false, Ordering::Relaxed);
            }),
        move |window, _frame| {
            window.root(
                col!()
                    .spacing(12.0)
                    .padding(Edges::all(20.0))
                    .child(
                        Text::new("Second Window")
                            .font_size(20.0)
                            .align(Align::Center),
                    )
                    .child(
                        Text::new("This is an independent window with its own widget tree.")
                            .font_size(14.0),
                    )
                    .child({
                        let ctx = window.app_context().clone();
                        let id = window.id();
                        button!("Close This Window").on_click(move |_| {
                            ctx.close_window(id);
                        })
                    }),
            );
        },
    );
}
