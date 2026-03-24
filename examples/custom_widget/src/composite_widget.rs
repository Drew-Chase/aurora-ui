use aurora_ui::prelude::*;

/// Configuration for the composite toggle switch.
///
/// Uses `#[derive(CompositeWidget)]` to auto-generate builder-pattern
/// setters for all `pub` fields — no boilerplate needed.
#[derive(CompositeWidget)]
pub struct ToggleOptions {
    pub track_width: u32,
    pub track_height: u32,
    pub knob_size: u32,
    pub knob_margin: f32,
    pub on_color: Color,
    pub off_color: Color,
    pub knob_color: Color,
}

impl Default for ToggleOptions {
    fn default() -> Self {
        Self {
            track_width: 50,
            track_height: 28,
            knob_size: 22,
            knob_margin: 3.0,
            on_color: Color::from_hex(0x4CAF50, false),
            off_color: Color::from_hex(0x9E9E9E, false),
            knob_color: Color::from_hex(0xFFFFFF, false),
        }
    }
}

/// Internal state for the toggle — not exposed to consumers.
#[derive(Default)]
struct ToggleState {
    is_on: bool,
}

/// Creates a toggle switch from the given options.
///
/// Uses `Composite<S>` for state and composes `TouchArea`, `BoxWidget`,
/// and `Positioned` — no custom paint or event code needed.
pub fn toggle_switch(options: ToggleOptions) -> impl Widget {
    let track_width = options.track_width;
    let track_height = options.track_height;
    let knob_size = options.knob_size;
    let knob_margin = options.knob_margin;
    let on_color = options.on_color;
    let off_color = options.off_color;
    let knob_color = options.knob_color;

    Composite::new(ToggleState::default(), move |state, set_state| {
        let setter = set_state.clone();
        let is_on = state.is_on;

        let track_color = if is_on { on_color } else { off_color };

        let knob_x = if is_on {
            track_width as f32 - knob_size as f32 - knob_margin
        } else {
            knob_margin
        };

        Box::new(
            TouchArea::new()
                .width(track_width as f32)
                .height(track_height as f32)
                .hover_cursor(CursorIcon::Pointer)
                .on_click(move |_| {
                    setter.set(|s| s.is_on = !s.is_on);
                })
                .child(
                    BoxWidget::new()
                        .width(track_width)
                        .height(track_height)
                        .corners(Corners::all(track_height as f32 / 2.0))
                        .background_color(track_color)
                        .child(
                            Positioned::absolute((knob_x, knob_margin)).child(
                                BoxWidget::new()
                                    .width(knob_size)
                                    .height(knob_size)
                                    .corners(Corners::all(knob_size as f32 / 2.0))
                                    .background_color(knob_color),
                            ),
                        ),
                ),
        )
    })
}
