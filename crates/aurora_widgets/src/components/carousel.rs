use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

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

    fn prev(&mut self) {
        if self.slides.is_empty() {
            return;
        }
        self.current = if self.current == 0 {
            self.slides.len() - 1
        } else {
            self.current - 1
        };
        if let Some(ref mut cb) = self.on_change {
            cb(self.current);
        }
    }

    fn next(&mut self) {
        if self.slides.is_empty() {
            return;
        }
        self.current = (self.current + 1) % self.slides.len();
        if let Some(ref mut cb) = self.on_change {
            cb(self.current);
        }
    }

    fn prev_rect(&self, rect: &Rect) -> Rect {
        let cy = rect.y1 + rect.height() / 2.0;
        let x = rect.x1 + 8.0;
        Rect::new(
            x,
            cy - self.arrow_size / 2.0,
            x + self.arrow_size,
            cy + self.arrow_size / 2.0,
        )
    }

    fn next_rect(&self, rect: &Rect) -> Rect {
        let cy = rect.y1 + rect.height() / 2.0;
        let x = rect.x2 - 8.0 - self.arrow_size;
        Rect::new(
            x,
            cy - self.arrow_size / 2.0,
            x + self.arrow_size,
            cy + self.arrow_size / 2.0,
        )
    }
}

impl Default for Carousel {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Carousel {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);
        let h = self.height.unwrap_or(available.height.min(300.0));
        let slide_available = Size::new(w, h);

        self.slide_sizes.clear();
        for slide in &mut self.slides {
            let ss = slide.layout(slide_available, ctx);
            self.slide_sizes.push(ss);
        }

        // Arrow text
        let mut opts = ctx.font_options.clone();
        opts.size = Some(18.0);
        opts.weight = Some(aurora_text::font_options::FontWeight::Bold);
        self.prev_layout = Some(aurora_text::text_layout::TextLayout::new(ctx.font_manager, "<", &opts, colors::foreground(), None));
        self.next_layout = Some(aurora_text::text_layout::TextLayout::new(ctx.font_manager, ">", &opts, colors::foreground(), None));

        if self.current >= self.slides.len() && !self.slides.is_empty() {
            self.current = 0;
        }

        Size::new(w, h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        canvas.push_clip(rect);

        // Paint current slide
        if let Some(slide) = self.slides.get(self.current) {
            slide.paint(canvas, rect);
        }

        canvas.pop_clip();

        // Arrows
        if self.show_arrows && self.slides.len() > 1 {
            let arrow_corners = Corners::all(self.arrow_size / 2.0);
            let arrow_bg = Color::new(255, 255, 255, 200);

            // Previous
            let pr = self.prev_rect(&rect);
            canvas.fill_rounded_rect(pr, arrow_corners, arrow_bg);
            canvas.stroke_rounded_rect(pr, arrow_corners, 1, colors::border());
            if let Some(ref tl) = self.prev_layout {
                let _s = tl.size(); let tw = _s.width; let th = _s.height;
                let tx = pr.x1 + (pr.width() - tw) / 2.0;
                let ty = pr.y1 + (pr.height() - th) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }

            // Next
            let nr = self.next_rect(&rect);
            canvas.fill_rounded_rect(nr, arrow_corners, arrow_bg);
            canvas.stroke_rounded_rect(nr, arrow_corners, 1, colors::border());
            if let Some(ref tl) = self.next_layout {
                let _s = tl.size(); let tw = _s.width; let th = _s.height;
                let tx = nr.x1 + (nr.width() - tw) / 2.0;
                let ty = nr.y1 + (nr.height() - th) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }
        }

        // Indicators
        if self.show_indicators && self.slides.len() > 1 {
            let total_w = self.indicator_size * self.slides.len() as f32
                + self.indicator_spacing * (self.slides.len() - 1) as f32;
            let start_x = rect.x1 + (rect.width() - total_w) / 2.0;
            let y = rect.y2 - self.indicator_bottom_offset;

            for i in 0..self.slides.len() {
                let ix = start_x + i as f32 * (self.indicator_size + self.indicator_spacing);
                let dot_rect = Rect::new(
                    ix,
                    y,
                    ix + self.indicator_size,
                    y + self.indicator_size,
                );
                let dot_corners = Corners::all(self.indicator_size / 2.0);
                let color = if i == self.current {
                    colors::primary()
                } else {
                    Color::new(255, 255, 255, 180)
                };
                canvas.fill_rounded_rect(dot_rect, dot_corners, color);
            }
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &self.slides
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed && rect.contains(&e.position) =>
            {
                if self.show_arrows && self.slides.len() > 1 {
                    if self.prev_rect(&rect).contains(&e.position) {
                        self.prev();
                        return EventResponse {
                            handled: true,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    if self.next_rect(&rect).contains(&e.position) {
                        self.next();
                        return EventResponse {
                            handled: true,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                }
                // Forward to current slide
                if let Some(slide) = self.slides.get_mut(self.current) {
                    return slide.event(event, rect);
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) if rect.contains(pos) => {
                if self.show_arrows && self.slides.len() > 1 {
                    if self.prev_rect(&rect).contains(pos) || self.next_rect(&rect).contains(pos) {
                        return EventResponse {
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
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
}
