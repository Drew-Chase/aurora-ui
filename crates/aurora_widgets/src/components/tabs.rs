use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;
use std::time::Instant;

use super::colors;

const ANIM_DURATION: f32 = 0.2; // seconds

/// A tab bar with switchable content panels.
///
/// # Example
/// ```ignore
/// Tabs::new()
///     .tab("Overview", Label::new("Overview content"))
///     .tab("Settings", Label::new("Settings content"))
///     .tab("About", Label::new("About content"))
///     .selected(0)
/// ```
pub struct Tabs {
    tabs: Vec<TabEntry>,
    selected: usize,
    tab_height: f32,
    tab_padding: f32,
    indicator_height: f32,
    indicator_color: Color,
    width: Option<f32>,
    on_change: Option<Box<dyn FnMut(usize)>>,
    tab_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    tab_widths: Vec<f32>,
    content_sizes: Vec<Size>,
    /// Animated indicator x-position (left edge)
    indicator_from_x: f32,
    indicator_from_w: f32,
    indicator_to_x: f32,
    indicator_to_w: f32,
    indicator_anim_start: Instant,
}

struct TabEntry {
    label: String,
    content: Box<dyn Widget>,
}

impl Tabs {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            selected: 0,
            tab_height: 40.0,
            tab_padding: 16.0,
            indicator_height: 2.0,
            indicator_color: colors::primary(),
            width: None,
            on_change: None,
            tab_layouts: Vec::new(),
            tab_widths: Vec::new(),
            content_sizes: Vec::new(),
            indicator_from_x: 0.0,
            indicator_from_w: 0.0,
            indicator_to_x: 0.0,
            indicator_to_w: 0.0,
            indicator_anim_start: Instant::now(),
        }
    }

    pub fn tab(mut self, label: impl Into<String>, content: impl Widget + 'static) -> Self {
        self.tabs.push(TabEntry {
            label: label.into(),
            content: Box::new(content),
        });
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    pub fn tab_height(mut self, height: f32) -> Self {
        self.tab_height = height;
        self
    }

    pub fn indicator_color(mut self, color: Color) -> Self {
        self.indicator_color = color;
        self
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn on_change(mut self, cb: impl FnMut(usize) + 'static) -> Self {
        self.on_change = Some(Box::new(cb));
        self
    }

    /// Returns the (x, width) of tab at `index`, offset from `base_x`.
    fn tab_x_and_width(&self, index: usize, base_x: f32) -> (f32, f32) {
        let mut x = base_x;
        for (i, tw) in self.tab_widths.iter().enumerate() {
            if i == index {
                return (x, *tw);
            }
            x += tw;
        }
        (base_x, 0.0)
    }

    fn current_indicator(&self) -> (f32, f32) {
        let elapsed = self.indicator_anim_start.elapsed().as_secs_f32();
        let t = (elapsed / ANIM_DURATION).min(1.0);
        let eased = 1.0 - (1.0 - t).powi(3);
        let ix = self.indicator_from_x + (self.indicator_to_x - self.indicator_from_x) * eased;
        let iw = self.indicator_from_w + (self.indicator_to_w - self.indicator_from_w) * eased;
        (ix, iw)
    }
}

impl Default for Tabs {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Tabs {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);
        self.tab_layouts.clear();
        self.tab_widths.clear();
        self.content_sizes.clear();

        // Layout tab labels
        for tab in &self.tabs {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(14.0);
            opts.weight = Some(aurora_text::font_options::FontWeight::Medium);
            let mut tl = aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                &tab.label,
                &opts,
                colors::foreground(),
                None,
            );
            tl.set_max_width(ctx.font_manager, f32::MAX);
            let tw = tl.size().width;
            self.tab_widths.push(tw + self.tab_padding * 2.0);
            self.tab_layouts.push(Some(tl));
        }

        // Layout content of selected tab
        let mut max_content_h = 0.0f32;
        for (i, tab) in self.tabs.iter_mut().enumerate() {
            if i == self.selected {
                let content_available = Size::new(w, f32::MAX);
                let cs = tab.content.layout(content_available, ctx);
                max_content_h = max_content_h.max(cs.height);
                self.content_sizes.push(cs);
            } else {
                self.content_sizes.push(Size::default());
            }
        }

        // Initialize indicator position if not yet animated
        if self.indicator_anim_start.elapsed().as_secs_f32() >= ANIM_DURATION {
            let (tx, tw) = self.tab_x_and_width(self.selected, 0.0);
            self.indicator_to_x = tx;
            self.indicator_to_w = tw;
            self.indicator_from_x = tx;
            self.indicator_from_w = tw;
        }

        let h = self.tab_height + max_content_h;
        Size::new(w, h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Tab bar background border
        let tab_bar_bottom = rect.y1 + self.tab_height;
        canvas.fill_rect(
            Rect::new(rect.x1, tab_bar_bottom - 1.0, rect.x2, tab_bar_bottom),
            colors::border(),
        );

        // Paint tabs
        let mut x = rect.x1;
        for (i, tab_w) in self.tab_widths.iter().enumerate() {
            let tab_rect = Rect::new(x, rect.y1, x + tab_w, tab_bar_bottom);

            // Tab label
            if let Some(Some(tl)) = self.tab_layouts.get(i) {
                let _s = tl.size();
                let tw = _s.width;
                let th = _s.height;
                let tx = tab_rect.x1 + (tab_rect.width() - tw) / 2.0;
                let ty = tab_rect.y1 + (tab_rect.height() - th) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }

            x += tab_w;
        }

        // Animated indicator
        if !self.tab_widths.is_empty() {
            let elapsed = self.indicator_anim_start.elapsed().as_secs_f32();
            let t = (elapsed / ANIM_DURATION).min(1.0);
            let eased = 1.0 - (1.0 - t).powi(3);
            let ix = self.indicator_from_x + (self.indicator_to_x - self.indicator_from_x) * eased;
            let iw = self.indicator_from_w + (self.indicator_to_w - self.indicator_from_w) * eased;
            canvas.fill_rect(
                Rect::new(
                    rect.x1 + ix,
                    tab_bar_bottom - self.indicator_height,
                    rect.x1 + ix + iw,
                    tab_bar_bottom,
                ),
                self.indicator_color,
            );
        }

        // Paint selected content
        if let Some(tab) = self.tabs.get(self.selected) {
            let cs = self
                .content_sizes
                .get(self.selected)
                .copied()
                .unwrap_or_default();
            let content_rect = Rect::new(
                rect.x1,
                tab_bar_bottom,
                rect.x1 + cs.width,
                tab_bar_bottom + cs.height,
            );
            tab.content.paint(canvas, content_rect);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        let tab_bar_bottom = rect.y1 + self.tab_height;

        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed && rect.contains(&e.position) =>
            {
                // Check if click is in tab bar
                if e.position.y < tab_bar_bottom {
                    let mut x = rect.x1;
                    for (i, tab_w) in self.tab_widths.iter().enumerate() {
                        let tab_rect = Rect::new(x, rect.y1, x + tab_w, tab_bar_bottom);
                        if tab_rect.contains(&e.position) {
                            // Start indicator slide animation (0-based positions)
                            let (cur_x, cur_w) = self.current_indicator();
                            self.indicator_from_x = cur_x;
                            self.indicator_from_w = cur_w;
                            let (to_x, to_w) = self.tab_x_and_width(i, 0.0);
                            self.indicator_to_x = to_x;
                            self.indicator_to_w = to_w;
                            self.indicator_anim_start = Instant::now();

                            self.selected = i;
                            if let Some(ref mut cb) = self.on_change {
                                cb(i);
                            }
                            return EventResponse {
                                handled: true,
                                cursor: Some(CursorIcon::Pointer),
                                ..Default::default()
                            };
                        }
                        x += tab_w;
                    }
                }
                // Forward to selected content
                if let Some(tab) = self.tabs.get_mut(self.selected) {
                    let cs = self
                        .content_sizes
                        .get(self.selected)
                        .copied()
                        .unwrap_or_default();
                    let content_rect = Rect::new(
                        rect.x1,
                        tab_bar_bottom,
                        rect.x1 + cs.width,
                        tab_bar_bottom + cs.height,
                    );
                    return tab.content.event(event, content_rect);
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) if rect.contains(pos) => {
                if pos.y < tab_bar_bottom {
                    let mut x = rect.x1;
                    for tab_w in self.tab_widths.iter() {
                        let tab_rect = Rect::new(x, rect.y1, x + tab_w, tab_bar_bottom);
                        if tab_rect.contains(pos) {
                            return EventResponse {
                                cursor: Some(CursorIcon::Pointer),
                                ..Default::default()
                            };
                        }
                        x += tab_w;
                    }
                }
                // Forward to content
                if let Some(tab) = self.tabs.get_mut(self.selected) {
                    let cs = self
                        .content_sizes
                        .get(self.selected)
                        .copied()
                        .unwrap_or_default();
                    let content_rect = Rect::new(
                        rect.x1,
                        tab_bar_bottom,
                        rect.x1 + cs.width,
                        tab_bar_bottom + cs.height,
                    );
                    return tab.content.event(event, content_rect);
                }
                EventResponse::default()
            }
            _ => {
                if let Some(tab) = self.tabs.get_mut(self.selected) {
                    let cs = self
                        .content_sizes
                        .get(self.selected)
                        .copied()
                        .unwrap_or_default();
                    let content_rect = Rect::new(
                        rect.x1,
                        tab_bar_bottom,
                        rect.x1 + cs.width,
                        tab_bar_bottom + cs.height,
                    );
                    return tab.content.event(event, content_rect);
                }
                EventResponse::default()
            }
        }
    }

    fn needs_animation(&self) -> bool {
        self.indicator_anim_start.elapsed().as_secs_f32() < ANIM_DURATION
    }
    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::TabList)
    }
}
