use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;
use std::time::Instant;

use super::colors;

const ANIM_DURATION: f32 = 0.2; // seconds

/// A single section in an accordion.
struct AccordionSection {
    title: String,
    content: Box<dyn Widget>,
    expanded: bool,
    title_layout: Option<aurora_text::text_layout::TextLayout>,
    content_size: Size,
    /// 0.0 = collapsed, 1.0 = expanded
    anim_from: f32,
    anim_to: f32,
    anim_start: Instant,
}

impl AccordionSection {
    fn current_t(&self) -> f32 {
        let elapsed = self.anim_start.elapsed().as_secs_f32();
        let t = (elapsed / ANIM_DURATION).min(1.0);
        let eased = 1.0 - (1.0 - t).powi(3);
        self.anim_from + (self.anim_to - self.anim_from) * eased
    }

    fn is_animating(&self) -> bool {
        self.anim_start.elapsed().as_secs_f32() < ANIM_DURATION
    }

    fn visible_content_height(&self, padding: &Edges) -> f32 {
        let t = self.current_t();
        (padding.top + self.content_size.height + padding.bottom) * t
    }
}

/// Expandable sections with a title and content. Click a section title to toggle
/// its content visibility. Content animates open and closed.
///
/// # Example
/// ```ignore
/// Accordion::new()
///     .section("Section 1", Label::new("Content 1"))
///     .section("Section 2", Label::new("Content 2"))
///     .section("Section 3", Label::new("Content 3"))
/// ```
pub struct Accordion {
    sections: Vec<AccordionSection>,
    allow_multiple: bool,
    header_height: f32,
    header_padding: Edges,
    content_padding: Edges,
    border_color: Color,
    header_bg: Color,
    spacing: f32,
    width: Option<f32>,
    on_change: Option<Box<dyn FnMut(usize, bool)>>,
}

impl Accordion {
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
            allow_multiple: false,
            header_height: 44.0,
            header_padding: Edges::new(0.0, 16.0, 0.0, 16.0),
            content_padding: Edges::new(0.0, 16.0, 16.0, 16.0),
            border_color: colors::border(),
            header_bg: Color::TRANSPARENT,
            spacing: 0.0,
            width: None,
            on_change: None,
        }
    }

    pub fn section(mut self, title: impl Into<String>, content: impl Widget + 'static) -> Self {
        self.sections.push(AccordionSection {
            title: title.into(),
            content: Box::new(content),
            expanded: false,
            title_layout: None,
            content_size: Size::default(),
            anim_from: 0.0,
            anim_to: 0.0,
            anim_start: Instant::now(),
        });
        self
    }

    pub fn expanded(mut self, index: usize) -> Self {
        if let Some(section) = self.sections.get_mut(index) {
            section.expanded = true;
            section.anim_from = 1.0;
            section.anim_to = 1.0;
        }
        self
    }

    pub fn allow_multiple(mut self, allow: bool) -> Self {
        self.allow_multiple = allow;
        self
    }

    pub fn header_height(mut self, height: f32) -> Self {
        self.header_height = height;
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn on_change(mut self, cb: impl FnMut(usize, bool) + 'static) -> Self {
        self.on_change = Some(Box::new(cb));
        self
    }

    fn section_height(&self, i: usize) -> f32 {
        self.header_height + self.sections[i].visible_content_height(&self.content_padding)
    }
}

impl Default for Accordion {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Accordion {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);
        let inner_w = w - self.content_padding.left - self.content_padding.right;
        let mut total_h = 0.0;

        for (i, section) in self.sections.iter_mut().enumerate() {
            // Layout title
            let mut opts = ctx.font_options.clone();
            opts.size = Some(14.0);
            opts.weight = Some(aurora_text::font_options::FontWeight::Medium);
            let tl = aurora_text::text_layout::TextLayout::new(
                ctx.font_manager, &section.title, &opts, colors::foreground(), None,
            );
            section.title_layout = Some(tl);

            // Always layout content so we know its height for animation
            let content_available = Size::new(inner_w.max(0.0), f32::MAX);
            section.content_size = section.content.layout(content_available, ctx);

            total_h += self.header_height;
            total_h += section.visible_content_height(&self.content_padding);
            if i > 0 {
                total_h += self.spacing;
            }
        }

        Size::new(w, total_h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let mut y = rect.y1;

        for (i, section) in self.sections.iter().enumerate() {
            if i > 0 {
                y += self.spacing;
            }

            let sec_h = self.section_height(i);
            let t = section.current_t();

            // Header background
            if self.header_bg.alpha > 0 {
                let header_rect = Rect::new(rect.x1, y, rect.x2, y + self.header_height);
                canvas.fill_rect(header_rect, self.header_bg);
            }

            // Header text
            if let Some(ref tl) = section.title_layout {
                let s = tl.size();
                let tx = rect.x1 + self.header_padding.left;
                let ty = y + (self.header_height - s.height) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }

            // Animated chevron: rotates from right-pointing to down-pointing
            let chevron_x = rect.x2 - self.header_padding.right - 10.0;
            let chevron_cy = y + self.header_height / 2.0;
            // Interpolate rotation: t=0 → right chevron, t=1 → down chevron
            let angle = t * std::f32::consts::FRAC_PI_2; // 0 to 90 degrees
            let cos_a = angle.cos();
            let sin_a = angle.sin();
            // Right-pointing chevron points: (-2,-4)->(2,0) and (2,0)->(-2,4)
            let rotate = |dx: f32, dy: f32| -> (f32, f32) {
                (dx * cos_a - dy * sin_a, dx * sin_a + dy * cos_a)
            };
            let (dx1, dy1) = rotate(-2.0, -4.0);
            let (dx2, dy2) = rotate(2.0, 0.0);
            let (dx3, dy3) = rotate(-2.0, 4.0);
            canvas.draw_line(
                Point::new(chevron_x + dx1, chevron_cy + dy1),
                Point::new(chevron_x + dx2, chevron_cy + dy2),
                1, colors::muted_foreground(),
            );
            canvas.draw_line(
                Point::new(chevron_x + dx2, chevron_cy + dy2),
                Point::new(chevron_x + dx3, chevron_cy + dy3),
                1, colors::muted_foreground(),
            );

            // Bottom border
            let border_y = y + self.header_height;
            canvas.fill_rect(
                Rect::new(rect.x1, border_y - 1.0, rect.x2, border_y),
                self.border_color,
            );

            // Content (clipped for animation)
            if t > 0.0 {
                let content_area_top = y + self.header_height;
                let visible_h = section.visible_content_height(&self.content_padding);
                let clip = Rect::new(rect.x1, content_area_top, rect.x2, content_area_top + visible_h);
                canvas.push_clip(clip);

                let content_y = content_area_top + self.content_padding.top;
                let content_rect = Rect::new(
                    rect.x1 + self.content_padding.left,
                    content_y,
                    rect.x1 + self.content_padding.left + section.content_size.width,
                    content_y + section.content_size.height,
                );
                section.content.paint(canvas, content_rect);

                canvas.pop_clip();
            }

            y += sec_h;
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed && rect.contains(&e.position) =>
            {
                let mut y = rect.y1;
                for i in 0..self.sections.len() {
                    if i > 0 {
                        y += self.spacing;
                    }
                    let header_rect = Rect::new(rect.x1, y, rect.x2, y + self.header_height);
                    if header_rect.contains(&e.position) {
                        let new_state = !self.sections[i].expanded;
                        if !self.allow_multiple && new_state {
                            for j in 0..self.sections.len() {
                                if j != i {
                                    self.sections[j].expanded = false;
                                    self.sections[j].anim_from = self.sections[j].current_t();
                                    self.sections[j].anim_to = 0.0;
                                    self.sections[j].anim_start = Instant::now();
                                }
                            }
                        }
                        self.sections[i].expanded = new_state;
                        self.sections[i].anim_from = self.sections[i].current_t();
                        self.sections[i].anim_to = if new_state { 1.0 } else { 0.0 };
                        self.sections[i].anim_start = Instant::now();
                        if let Some(ref mut cb) = self.on_change {
                            cb(i, self.sections[i].expanded);
                        }
                        return EventResponse {
                            handled: true,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    y += self.section_height(i);
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) if rect.contains(pos) => {
                let mut y = rect.y1;
                for i in 0..self.sections.len() {
                    if i > 0 {
                        y += self.spacing;
                    }
                    let header_rect = Rect::new(rect.x1, y, rect.x2, y + self.header_height);
                    if header_rect.contains(pos) {
                        return EventResponse {
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    y += self.section_height(i);
                }
                // Forward to expanded content
                y = rect.y1;
                for (i, section) in self.sections.iter_mut().enumerate() {
                    if i > 0 {
                        y += self.spacing;
                    }
                    if section.expanded {
                        let content_y = y + self.header_height + self.content_padding.top;
                        let content_rect = Rect::new(
                            rect.x1 + self.content_padding.left,
                            content_y,
                            rect.x1 + self.content_padding.left + section.content_size.width,
                            content_y + section.content_size.height,
                        );
                        let resp = section.content.event(event, content_rect);
                        if resp.handled {
                            return resp;
                        }
                    }
                    let sec_h = self.header_height
                        + if section.expanded { self.content_padding.top + section.content_size.height + self.content_padding.bottom } else { 0.0 };
                    y += sec_h;
                }
                EventResponse::default()
            }
            _ => {
                let mut y = rect.y1;
                for (i, section) in self.sections.iter_mut().enumerate() {
                    if i > 0 {
                        y += self.spacing;
                    }
                    if section.expanded {
                        let content_y = y + self.header_height + self.content_padding.top;
                        let content_rect = Rect::new(
                            rect.x1 + self.content_padding.left,
                            content_y,
                            rect.x1 + self.content_padding.left + section.content_size.width,
                            content_y + section.content_size.height,
                        );
                        let resp = section.content.event(event, content_rect);
                        if resp.handled {
                            return resp;
                        }
                    }
                    let sec_h = self.header_height
                        + if section.expanded { self.content_padding.top + section.content_size.height + self.content_padding.bottom } else { 0.0 };
                    y += sec_h;
                }
                EventResponse::default()
            }
        }
    }

    fn needs_animation(&self) -> bool {
        self.sections.iter().any(|s| s.is_animating())
    }
}
