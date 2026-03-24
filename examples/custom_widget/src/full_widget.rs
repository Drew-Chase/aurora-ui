use aurora_ui::prelude::*;

const TRACK_WIDTH: f32 = 50.0;
const TRACK_HEIGHT: f32 = 28.0;
const KNOB_SIZE: f32 = 22.0;
const KNOB_MARGIN: f32 = 3.0;

const TRACK_ON_COLOR: u64 = 0x4CAF50;
const TRACK_OFF_COLOR: u64 = 0x9E9E9E;
const KNOB_COLOR: u64 = 0xFFFFFF;

/// A toggle switch built by implementing the `Widget` trait directly.
///
/// Handles its own layout, painting, and event processing — no composed
/// child widgets. This gives full control over rendering and behavior.
#[derive(Default)]
pub struct ToggleSwitch {
    is_on: bool,
}

impl ToggleSwitch {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget for ToggleSwitch {
    fn layout(&mut self, _available: Size, _ctx: &mut LayoutCtx) -> Size {
        Size::new(TRACK_WIDTH, TRACK_HEIGHT)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Draw the track
        let track_color = if self.is_on {
            Color::from_hex(TRACK_ON_COLOR, false)
        } else {
            Color::from_hex(TRACK_OFF_COLOR, false)
        };
        let corner_radius = TRACK_HEIGHT / 2.0;
        canvas.fill_rounded_rect(rect, Corners::all(corner_radius), track_color);

        // Draw the knob
        let knob_x = if self.is_on {
            rect.x1 + TRACK_WIDTH - KNOB_SIZE - KNOB_MARGIN
        } else {
            rect.x1 + KNOB_MARGIN
        };
        let knob_y = rect.y1 + KNOB_MARGIN;
        let knob_rect = Rect::new(knob_x, knob_y, knob_x + KNOB_SIZE, knob_y + KNOB_SIZE);
        canvas.fill_rounded_rect(
            knob_rect,
            Corners::all(KNOB_SIZE / 2.0),
            Color::from_hex(KNOB_COLOR, false),
        );
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[] // No children — everything is drawn manually
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(click)) => {
                if click.state == MouseState::Released && rect.contains(&click.position) {
                    self.is_on = !self.is_on;
                    return EventResponse {
                        handled: true,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                if rect.contains(pos) {
                    EventResponse {
                        handled: true,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    }
                } else {
                    EventResponse::default()
                }
            }
            _ => EventResponse::default(),
        }
    }
}
