use aurora_ui::prelude::*;

/// State for the composite toggle switch.
#[derive(Default)]
struct ToggleState {
    is_on: bool,
}

const TRACK_WIDTH: u32 = 50;
const TRACK_HEIGHT: u32 = 28;
const KNOB_SIZE: u32 = 22;
const KNOB_MARGIN: f32 = 3.0;

const TRACK_ON_COLOR: u64 = 0x4CAF50;
const TRACK_OFF_COLOR: u64 = 0x9E9E9E;
const KNOB_COLOR: u64 = 0xFFFFFF;

/// A toggle switch built by composing existing widgets.
///
/// Uses `Composite<S>` for state management and composes `TouchArea`,
/// `BoxWidget`, and layout primitives — no custom paint or event code needed.
pub fn toggle_switch() -> impl Widget {
    Composite::new(ToggleState::default(), move |state, set_state| {
        let setter = set_state.clone();
        let is_on = state.is_on;

        let track_color = if is_on {
            Color::from_hex(TRACK_ON_COLOR, false)
        } else {
            Color::from_hex(TRACK_OFF_COLOR, false)
        };

        // Position the knob: left when off, right when on
        let knob_x = if is_on {
            TRACK_WIDTH as f32 - KNOB_SIZE as f32 - KNOB_MARGIN
        } else {
            KNOB_MARGIN
        };

        Box::new(
            TouchArea::new()
                .width(TRACK_WIDTH as f32)
                .height(TRACK_HEIGHT as f32)
                .hover_cursor(CursorIcon::Pointer)
                .on_click(move |_| {
                    setter.set(|s| s.is_on = !s.is_on);
                })
                .child(
                    // Track (pill background)
                    BoxWidget::new()
                        .width(TRACK_WIDTH)
                        .height(TRACK_HEIGHT)
                        .corners(Corners::all(TRACK_HEIGHT as f32 / 2.0))
                        .background_color(track_color)
                        .child(
                            // Knob (circle)
                            Positioned::absolute((knob_x, KNOB_MARGIN)).child(
                                BoxWidget::new()
                                    .width(KNOB_SIZE)
                                    .height(KNOB_SIZE)
                                    .corners(Corners::all(KNOB_SIZE as f32 / 2.0))
                                    .background_color(Color::from_hex(KNOB_COLOR, false)),
                            ),
                        ),
                ),
        )
    })
}
