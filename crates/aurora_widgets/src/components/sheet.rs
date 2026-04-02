use crate::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::keyboard::{Key, KeyboardEvent};
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;
use std::time::Instant;

use super::colors;

/// Duration of the slide animation in seconds.
const ANIM_DURATION: f32 = 0.25;

/// Which edge of the screen the sheet slides in from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SheetSide {
    Left,
    #[default]
    Right,
    Top,
    Bottom,
}

/// A slide-in panel overlay. When open, paints a semi-transparent backdrop
/// over the content and slides a panel in from the specified edge.
///
/// # Example
/// ```ignore
/// Sheet::new()
///     .open(true)
///     .side(SheetSide::Right)
///     .child(
///         col!()
///             .child(Text::new("Sheet content"))
///             .child(button!("Close"))
///     )
///     .on_close(|| println!("sheet closed"))
/// ```
pub struct Sheet {
    child: Option<Box<dyn Widget>>,
    open: bool,
    side: SheetSide,
    panel_width: f32,
    panel_height: f32,
    overlay_color: Color,
    panel_bg: Color,
    on_close: Option<Box<dyn FnMut()>>,
    /// 0.0 = closed, 1.0 = open
    anim_from: f32,
    anim_to: f32,
    anim_start: Instant,
    child_size: Size,
    was_open: bool,
}

impl Sheet {
    pub fn new() -> Self {
        Self {
            child: None,
            open: false,
            side: SheetSide::Right,
            panel_width: 400.0,
            panel_height: 300.0,
            overlay_color: colors::overlay(),
            panel_bg: colors::background(),
            on_close: None,
            anim_from: 0.0,
            anim_to: 0.0,
            anim_start: Instant::now(),
            child_size: Size::default(),
            was_open: false,
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn side(mut self, side: SheetSide) -> Self {
        self.side = side;
        self
    }

    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(widget));
        self
    }

    pub fn panel_width(mut self, width: f32) -> Self {
        self.panel_width = width;
        self
    }

    pub fn panel_height(mut self, height: f32) -> Self {
        self.panel_height = height;
        self
    }

    pub fn overlay_color(mut self, color: Color) -> Self {
        self.overlay_color = color;
        self
    }

    pub fn panel_bg(mut self, color: Color) -> Self {
        self.panel_bg = color;
        self
    }

    pub fn on_close(mut self, cb: impl FnMut() + 'static) -> Self {
        self.on_close = Some(Box::new(cb));
        self
    }

    pub fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    /// Computes the current animation progress (0.0 = closed, 1.0 = open).
    /// Uses CubicOut easing: `1 - (1 - t)^3`.
    fn current_t(&self) -> f32 {
        let elapsed = self.anim_start.elapsed().as_secs_f32();
        let t = (elapsed / ANIM_DURATION).min(1.0);
        let eased = 1.0 - (1.0 - t).powi(3);
        self.anim_from + (self.anim_to - self.anim_from) * eased
    }

    /// Returns the panel rectangle for the given viewport and animation progress.
    fn panel_rect(&self, viewport: &Rect, anim: f32) -> Rect {
        match self.side {
            SheetSide::Right => {
                let w = self.panel_width;
                let h = viewport.height();
                // Slides from right edge: anim=0 means fully off-screen right,
                // anim=1 means flush against right edge.
                let x = viewport.x2 - w * anim;
                Rect::new(x, viewport.y1, x + w, viewport.y1 + h)
            }
            SheetSide::Left => {
                let w = self.panel_width;
                let h = viewport.height();
                // Slides from left edge: anim=0 means fully off-screen left,
                // anim=1 means flush against left edge.
                let x = viewport.x1 - w * (1.0 - anim);
                Rect::new(x, viewport.y1, x + w, viewport.y1 + h)
            }
            SheetSide::Top => {
                let w = viewport.width();
                let h = self.panel_height;
                // Slides from top: anim=0 means fully off-screen above,
                // anim=1 means flush against top edge.
                let y = viewport.y1 - h * (1.0 - anim);
                Rect::new(viewport.x1, y, viewport.x1 + w, y + h)
            }
            SheetSide::Bottom => {
                let w = viewport.width();
                let h = self.panel_height;
                // Slides from bottom: anim=0 means fully off-screen below,
                // anim=1 means flush against bottom edge.
                let y = viewport.y2 - h * anim;
                Rect::new(viewport.x1, y, viewport.x1 + w, y + h)
            }
        }
    }
}

impl Default for Sheet {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Sheet {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        // Detect open state changes and start animation
        if self.open != self.was_open {
            self.anim_from = self.current_t();
            self.anim_to = if self.open { 1.0 } else { 0.0 };
            self.anim_start = Instant::now();
            self.was_open = self.open;
        }

        // Layout child at panel dimensions
        if let Some(ref mut child) = self.child {
            let panel_w = match self.side {
                SheetSide::Left | SheetSide::Right => self.panel_width,
                SheetSide::Top | SheetSide::Bottom => available.width,
            };
            let panel_h = match self.side {
                SheetSide::Left | SheetSide::Right => available.height,
                SheetSide::Top | SheetSide::Bottom => self.panel_height,
            };
            let child_available = Size::new(panel_w.max(0.0), panel_h.max(0.0));
            self.child_size = child.layout(child_available, ctx);
        }

        // The sheet is an overlay -- it doesn't consume layout space in the parent
        Size::new(available.width, available.height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let anim = self.current_t();
        if anim <= 0.0 {
            return;
        }

        // Semi-transparent backdrop, alpha scaled by animation progress
        let backdrop = self
            .overlay_color
            .opacity((self.overlay_color.alpha as f32 / 255.0) * anim);
        canvas.fill_rect(rect, backdrop);

        // Panel sliding in from the specified side
        let panel = self.panel_rect(&rect, anim);

        // Clip to viewport so the panel doesn't paint outside during slide
        canvas.push_clip(rect);

        // Panel background
        canvas.fill_rect(panel, self.panel_bg);

        // Paint child inside panel
        if let Some(ref child) = self.child {
            let child_rect = Rect::new(
                panel.x1,
                panel.y1,
                panel.x1 + self.child_size.width,
                panel.y1 + self.child_size.height,
            );
            child.paint(canvas, child_rect);
        }

        canvas.pop_clip();
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, _event: &WidgetEvent, _rect: Rect) -> EventResponse {
        if self.open {
            // Block normal event phase when sheet is open — everything
            // is handled in event_overlay.
            EventResponse {
                status: EventStatus::Canceled,
                ..Default::default()
            }
        } else {
            EventResponse::default()
        }
    }

    fn event_overlay(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        if !self.open {
            return EventResponse::default();
        }

        let anim = self.current_t();
        let panel = self.panel_rect(&rect, anim);

        // Close on Escape key
        if let WidgetEvent::Keyboard(KeyboardEvent::KeyPressed {
            key: Key::Escape, ..
        }) = event
        {
            self.open = false;
            self.was_open = false;
            self.anim_from = self.current_t();
            self.anim_to = 0.0;
            self.anim_start = Instant::now();
            if let Some(ref mut cb) = self.on_close {
                cb();
            }
            return EventResponse {
                status: EventStatus::Canceled,
                ..Default::default()
            };
        }

        // Close on click outside the panel (on the backdrop)
        if let WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e)) = event
            && e.state == MouseState::Released
            && !panel.contains(&e.position)
        {
            self.open = false;
            self.was_open = false;
            self.anim_from = self.current_t();
            self.anim_to = 0.0;
            self.anim_start = Instant::now();
            if let Some(ref mut cb) = self.on_close {
                cb();
            }
            return EventResponse {
                status: EventStatus::Canceled,
                ..Default::default()
            };
        }

        // Forward events to child if inside panel
        if let Some(ref mut child) = self.child {
            let child_rect = Rect::new(
                panel.x1,
                panel.y1,
                panel.x1 + self.child_size.width,
                panel.y1 + self.child_size.height,
            );
            let resp = child.event(event, child_rect);
            if resp.status.is_handled() {
                return resp;
            }
        }

        // Block all events behind the sheet when open
        EventResponse {
            status: EventStatus::Canceled,
            ..Default::default()
        }
    }

    fn needs_animation(&self) -> bool {
        self.anim_start.elapsed().as_secs_f32() < ANIM_DURATION
    }

    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        let mut info =
            aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Dialog).with_modal();
        if !self.open {
            info = info.with_hidden();
        }
        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sheet() -> Sheet {
        Sheet::new()
    }

    #[test]
    fn open_close_state() {
        let sheet = make_sheet();
        assert!(!sheet.open, "sheet should default to closed");

        let sheet = Sheet::new().open(true);
        assert!(sheet.open, "sheet should be open after .open(true)");

        let sheet = Sheet::new().open(true).open(false);
        assert!(!sheet.open, "sheet should be closed after .open(false)");
    }

    #[test]
    fn side_positioning() {
        let viewport = Rect::new(0.0, 0.0, 800.0, 600.0);

        // Right side (default): panel flush against right edge when anim=1
        let sheet = Sheet::new().side(SheetSide::Right).panel_width(400.0);
        let panel = sheet.panel_rect(&viewport, 1.0);
        assert!((panel.x1 - 400.0).abs() < 0.01);
        assert!((panel.x2 - 800.0).abs() < 0.01);

        // Left side: panel flush against left edge when anim=1
        let sheet = Sheet::new().side(SheetSide::Left).panel_width(400.0);
        let panel = sheet.panel_rect(&viewport, 1.0);
        assert!((panel.x1 - 0.0).abs() < 0.01);
        assert!((panel.x2 - 400.0).abs() < 0.01);

        // Top side: panel flush against top edge when anim=1
        let sheet = Sheet::new().side(SheetSide::Top).panel_height(300.0);
        let panel = sheet.panel_rect(&viewport, 1.0);
        assert!((panel.y1 - 0.0).abs() < 0.01);
        assert!((panel.y2 - 300.0).abs() < 0.01);

        // Bottom side: panel flush against bottom edge when anim=1
        let sheet = Sheet::new().side(SheetSide::Bottom).panel_height(300.0);
        let panel = sheet.panel_rect(&viewport, 1.0);
        assert!((panel.y1 - 300.0).abs() < 0.01);
        assert!((panel.y2 - 600.0).abs() < 0.01);
    }

    #[test]
    fn default_values() {
        let sheet = make_sheet();
        assert!(!sheet.open);
        assert_eq!(sheet.side, SheetSide::Right);
        assert!((sheet.panel_width - 400.0).abs() < 0.01);
        assert!((sheet.panel_height - 300.0).abs() < 0.01);
        assert!(sheet.child.is_none());
        assert!(sheet.on_close.is_none());
    }

    #[test]
    fn animation_progress() {
        // Freshly created sheet: anim_from=0, anim_to=0, so current_t = 0
        let sheet = make_sheet();
        let t = sheet.current_t();
        assert!(
            (0.0..=1.0).contains(&t),
            "animation progress should be in [0, 1]"
        );

        // A sheet that just opened: anim_to=1 but elapsed is ~0
        let mut sheet = make_sheet();
        sheet.anim_from = 0.0;
        sheet.anim_to = 1.0;
        sheet.anim_start = Instant::now();
        let t = sheet.current_t();
        // At time ~0, the eased value should be very close to anim_from (0.0)
        assert!(
            t < 0.1,
            "animation should be near start immediately after opening"
        );
    }
}
