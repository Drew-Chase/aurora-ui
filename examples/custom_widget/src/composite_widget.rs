use aurora_ui::prelude::*;

/// A toggle switch built by composing existing widgets.
///
/// Uses `#[derive(CompositeWidget)]` to auto-generate builder setters,
/// and lazily builds an internal `Composite` for state management.
/// The API mirrors full widgets: `ToggleSwitch::new().on_color(...)`.
#[derive(CompositeWidget)]
pub struct ToggleSwitch {
    pub track_width: u32,
    pub track_height: u32,
    pub knob_size: u32,
    pub knob_margin: f32,
    pub on_color: Color,
    pub off_color: Color,
    pub knob_color: Color,

    // Private — no setters generated, not visible to consumers.
    inner: Option<Box<dyn Widget>>,
}

impl Default for ToggleSwitch {
    fn default() -> Self {
        Self {
            track_width: 50,
            track_height: 28,
            knob_size: 22,
            knob_margin: 3.0,
            on_color: Color::from_hex(0x4CAF50, false),
            off_color: Color::from_hex(0x9E9E9E, false),
            knob_color: Color::from_hex(0xFFFFFF, false),
            inner: None,
        }
    }
}

impl ToggleSwitch {
    pub fn new() -> Self {
        Self::default()
    }

    fn build_inner(&self) -> Box<dyn Widget> {
        let track_width = self.track_width;
        let track_height = self.track_height;
        let knob_size = self.knob_size;
        let knob_margin = self.knob_margin;
        let on_color = self.on_color;
        let off_color = self.off_color;
        let knob_color = self.knob_color;

        Box::new(Composite::new(false, move |&is_on, set_state| {
            let setter = set_state.clone();

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
                        setter.set(|on| *on = !*on);
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
        }))
    }
}

impl Widget for ToggleSwitch {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        if self.inner.is_none() {
            self.inner = Some(self.build_inner());
        }
        self.inner.as_mut().unwrap().layout(available, ctx)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        if let Some(ref inner) = self.inner {
            inner.paint(canvas, rect);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        match &self.inner {
            Some(inner) => inner.children(),
            None => &[],
        }
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match &mut self.inner {
            Some(inner) => inner.event(event, rect),
            None => EventResponse::default(),
        }
    }
}
