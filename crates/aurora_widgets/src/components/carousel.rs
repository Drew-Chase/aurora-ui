use crate::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;
use std::time::Instant;

use super::colors;

const SLIDE_DURATION: f32 = 0.50;

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

/// A carousel/slider for cycling through content panels one at a time.
///
/// # Example
/// ```ignore
/// Carousel::new()
///     .slide(BoxWidget::new().background_color(Color::RED))
///     .slide(BoxWidget::new().background_color(Color::GREEN))
///     .slide(BoxWidget::new().background_color(Color::BLUE))
/// ```
pub struct Carousel {
    slides: Vec<Box<dyn Widget>>,
    current: usize,
    show_indicators: bool,
    show_arrows: bool,
    arrow_size: f32,
    indicator_size: f32,
    indicator_spacing: f32,
    indicator_bottom_offset: f32,
    width: Option<f32>,
    height: Option<f32>,
    on_change: Option<Box<dyn FnMut(usize)>>,
    slide_sizes: Vec<Size>,
    prev_layout: Option<aurora_text::text_layout::TextLayout>,
    next_layout: Option<aurora_text::text_layout::TextLayout>,
    // Slide animation
    slide_from: usize,
    slide_direction: f32,
    slide_start: Instant,
}

impl Carousel {
    pub fn new() -> Self {
        Self {
            slides: Vec::new(),
            current: 0,
            show_indicators: true,
            show_arrows: true,
            arrow_size: 36.0,
            indicator_size: 8.0,
            indicator_spacing: 6.0,
            indicator_bottom_offset: 16.0,
            width: None,
            height: None,
            on_change: None,
            slide_sizes: Vec::new(),
            prev_layout: None,
            next_layout: None,
            slide_from: 0,
            slide_direction: 0.0,
            slide_start: Instant::now(),
        }
    }

    pub fn slide(mut self, widget: impl Widget + 'static) -> Self {
        self.slides.push(Box::new(widget));
        self
    }

    pub fn current(mut self, index: usize) -> Self {
        self.current = index;
        self
    }

    pub fn show_indicators(mut self, show: bool) -> Self {
        self.show_indicators = show;
        self
    }

    pub fn show_arrows(mut self, show: bool) -> Self {
        self.show_arrows = show;
        self
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn height(mut self, height: impl Into<f32>) -> Self {
        self.height = Some(height.into());
        self
    }

    pub fn on_change(mut self, cb: impl FnMut(usize) + 'static) -> Self {
        self.on_change = Some(Box::new(cb));
        self
    }

    fn go_to(&mut self, index: usize) {
        if self.slides.is_empty() || index == self.current || index >= self.slides.len() {
            return;
        }
        self.slide_from = self.current;
        self.slide_direction = if index > self.current { 1.0 } else { -1.0 };
        self.slide_start = Instant::now();
        self.current = index;
        if let Some(ref mut cb) = self.on_change {
            cb(self.current);
        }
    }

    fn prev(&mut self) {
        if self.slides.is_empty() { return; }
        let new = if self.current == 0 { self.slides.len() - 1 } else { self.current - 1 };
        self.slide_from = self.current;
        self.slide_direction = -1.0; // new enters from left
        self.slide_start = Instant::now();
        self.current = new;
        if let Some(ref mut cb) = self.on_change { cb(self.current); }
    }

    fn next(&mut self) {
        if self.slides.is_empty() { return; }
        self.slide_from = self.current;
        self.slide_direction = 1.0; // new enters from right
        self.slide_start = Instant::now();
        self.current = (self.current + 1) % self.slides.len();
        if let Some(ref mut cb) = self.on_change { cb(self.current); }
    }

    fn is_sliding(&self) -> bool {
        self.slide_direction != 0.0 && self.slide_start.elapsed().as_secs_f32() < SLIDE_DURATION
    }

    fn slide_progress(&self) -> f32 {
        ease_out_cubic((self.slide_start.elapsed().as_secs_f32() / SLIDE_DURATION).min(1.0))
    }

    fn prev_rect(&self, rect: &Rect) -> Rect {
        let cy = rect.y1 + rect.height() / 2.0;
        let x = rect.x1 + 8.0;
        Rect::new(x, cy - self.arrow_size / 2.0, x + self.arrow_size, cy + self.arrow_size / 2.0)
    }

    fn next_rect(&self, rect: &Rect) -> Rect {
        let cy = rect.y1 + rect.height() / 2.0;
        let x = rect.x2 - 8.0 - self.arrow_size;
        Rect::new(x, cy - self.arrow_size / 2.0, x + self.arrow_size, cy + self.arrow_size / 2.0)
    }

    fn indicator_rect(&self, rect: &Rect, index: usize) -> Rect {
        let count = self.slides.len();
        let total_w = self.indicator_size * count as f32
            + self.indicator_spacing * count.saturating_sub(1) as f32;
        let start_x = rect.x1 + (rect.width() - total_w) / 2.0;
        let y = rect.y2 - self.indicator_bottom_offset;
        let ix = start_x + index as f32 * (self.indicator_size + self.indicator_spacing);
        Rect::new(ix, y, ix + self.indicator_size, y + self.indicator_size)
    }

    fn indicator_hit(&self, pos: &aurora_core::geometry::point::Point, rect: &Rect) -> Option<usize> {
        if !self.show_indicators || self.slides.len() <= 1 { return None; }
        let pad = 4.0;
        for i in 0..self.slides.len() {
            let ir = self.indicator_rect(rect, i);
            let hit = Rect::new(ir.x1 - pad, ir.y1 - pad, ir.x2 + pad, ir.y2 + pad);
            if hit.contains(pos) { return Some(i); }
        }
        None
    }
}

impl Default for Carousel {
    fn default() -> Self { Self::new() }
}

impl Widget for Carousel {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);
        let h = self.height.unwrap_or(available.height.min(300.0));
        let slide_available = Size::new(w, h);

        self.slide_sizes.clear();
        for slide in &mut self.slides {
            self.slide_sizes.push(slide.layout(slide_available, ctx));
        }

        let mut opts = ctx.font_options.clone();
        opts.size = Some(18.0);
        opts.weight = Some(aurora_text::font_options::FontWeight::Bold);
        self.prev_layout = Some(aurora_text::text_layout::TextLayout::new(
            ctx.font_manager, "<", &opts, colors::foreground(), None,
        ));
        self.next_layout = Some(aurora_text::text_layout::TextLayout::new(
            ctx.font_manager, ">", &opts, colors::foreground(), None,
        ));

        if self.current >= self.slides.len() && !self.slides.is_empty() {
            self.current = 0;
        }

        Size::new(w, h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        canvas.push_clip(rect);

        if self.is_sliding() {
            let t = self.slide_progress();
            let dir = self.slide_direction;
            let w = rect.width();

            // Old slide moving out
            let old_x = -t * w * dir;
            if let Some(slide) = self.slides.get(self.slide_from) {
                let r = Rect::new(rect.x1 + old_x, rect.y1, rect.x1 + old_x + w, rect.y2);
                slide.paint(canvas, r);
            }

            // New slide moving in
            let new_x = (1.0 - t) * w * dir;
            if let Some(slide) = self.slides.get(self.current) {
                let r = Rect::new(rect.x1 + new_x, rect.y1, rect.x1 + new_x + w, rect.y2);
                slide.paint(canvas, r);
            }
        } else if let Some(slide) = self.slides.get(self.current) {
            slide.paint(canvas, rect);
        }

        canvas.pop_clip();

        // Arrows
        if self.show_arrows && self.slides.len() > 1 {
            let ac = Corners::all(self.arrow_size / 2.0);
            let bg = colors::background().opacity(0.8);

            let pr = self.prev_rect(&rect);
            canvas.fill_rounded_rect(pr, ac, bg);
            canvas.stroke_rounded_rect(pr, ac, 1, colors::border());
            if let Some(ref tl) = self.prev_layout {
                let s = tl.size();
                canvas.draw_text(tl, (pr.x1 + (pr.width() - s.width) / 2.0) as i32, (pr.y1 + (pr.height() - s.height) / 2.0) as i32);
            }

            let nr = self.next_rect(&rect);
            canvas.fill_rounded_rect(nr, ac, bg);
            canvas.stroke_rounded_rect(nr, ac, 1, colors::border());
            if let Some(ref tl) = self.next_layout {
                let s = tl.size();
                canvas.draw_text(tl, (nr.x1 + (nr.width() - s.width) / 2.0) as i32, (nr.y1 + (nr.height() - s.height) / 2.0) as i32);
            }
        }

        // Indicators
        if self.show_indicators && self.slides.len() > 1 {
            for i in 0..self.slides.len() {
                let dr = self.indicator_rect(&rect, i);
                let dc = Corners::all(self.indicator_size / 2.0);
                let color = if i == self.current { colors::primary() } else { colors::muted_foreground() };
                canvas.fill_rounded_rect(dr, dc, color);
            }
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &self.slides
    }

    fn needs_animation(&self) -> bool {
        self.is_sliding() || self.slides.iter().any(|s| s.needs_animation())
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed && rect.contains(&e.position) =>
            {
                if self.show_arrows && self.slides.len() > 1 {
                    if self.prev_rect(&rect).contains(&e.position) {
                        self.prev();
                        return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Pointer), ..Default::default() };
                    }
                    if self.next_rect(&rect).contains(&e.position) {
                        self.next();
                        return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Pointer), ..Default::default() };
                    }
                }
                if let Some(i) = self.indicator_hit(&e.position, &rect) {
                    self.go_to(i);
                    return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Pointer), ..Default::default() };
                }
                if let Some(slide) = self.slides.get_mut(self.current) {
                    return slide.event(event, rect);
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) if rect.contains(pos) => {
                if self.show_arrows && self.slides.len() > 1
                    && (self.prev_rect(&rect).contains(pos) || self.next_rect(&rect).contains(pos))
                {
                    return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Pointer), ..Default::default() };
                }
                if self.indicator_hit(pos, &rect).is_some() {
                    return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Pointer), ..Default::default() };
                }
                if let Some(slide) = self.slides.get_mut(self.current) {
                    return slide.event(event, rect);
                }
                EventResponse::default()
            }
            _ => {
                if let Some(slide) = self.slides.get_mut(self.current) {
                    return slide.event(event, rect);
                }
                EventResponse::default()
            }
        }
    }

    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Group)
            .with_label("Carousel".to_string())
    }
}
