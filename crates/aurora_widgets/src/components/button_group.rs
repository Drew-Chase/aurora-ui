use crate::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;

use super::colors;

/// A group of buttons arranged in a row with shared borders.
///
/// By default the group behaves like regular buttons — each has a hover
/// effect and fires the `on_click` callback, but none stay selected.
/// Use `.selected(index)` to opt into persistent selection (tab-like).
///
/// # Example
/// ```ignore
/// ButtonGroup::new()
///     .button("Cut")
///     .button("Copy")
///     .button("Paste")
///     .on_click(|idx| println!("clicked: {idx}"))
/// ```
pub struct ButtonGroup {
    buttons: Vec<String>,
    selected: Option<usize>,
    hovered_index: Option<usize>,
    height: f32,
    padding: f32,
    border_color: Color,
    selected_bg: Color,
    selected_fg: Color,
    normal_fg: Color,
    hover_bg: Color,
    corners: Corners,
    on_click: Option<Box<dyn FnMut(usize)>>,
    button_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    button_widths: Vec<f32>,
}

impl ButtonGroup {
    pub fn new() -> Self {
        Self {
            buttons: Vec::new(),
            selected: None,
            hovered_index: None,
            height: 36.0,
            padding: 16.0,
            border_color: colors::border(),
            selected_bg: colors::primary(),
            selected_fg: colors::primary_foreground(),
            normal_fg: colors::foreground(),
            hover_bg: colors::secondary(),
            corners: Corners::all(6.0),
            on_click: None,
            button_layouts: Vec::new(),
            button_widths: Vec::new(),
        }
    }

    pub fn button(mut self, label: impl Into<String>) -> Self {
        self.buttons.push(label.into());
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = Some(index);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn corners(mut self, corners: Corners) -> Self {
        self.corners = corners;
        self
    }

    pub fn selected_bg(mut self, color: Color) -> Self {
        self.selected_bg = color;
        self
    }

    pub fn selected_fg(mut self, color: Color) -> Self {
        self.selected_fg = color;
        self
    }

    pub fn hover_bg(mut self, color: Color) -> Self {
        self.hover_bg = color;
        self
    }

    pub fn on_click(mut self, cb: impl FnMut(usize) + 'static) -> Self {
        self.on_click = Some(Box::new(cb));
        self
    }

    /// Computes the per-button corner radii for the button at index `i`.
    fn button_corners(&self, i: usize) -> Corners {
        if self.buttons.len() == 1 {
            self.corners
        } else if i == 0 {
            Corners::new(self.corners.top_left, 0.0, 0.0, self.corners.bottom_left)
        } else if i == self.buttons.len() - 1 {
            Corners::new(0.0, self.corners.top_right, self.corners.bottom_right, 0.0)
        } else {
            Corners::zero()
        }
    }
}

impl Default for ButtonGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ButtonGroup {
    fn layout(&mut self, _available: Size, ctx: &mut LayoutCtx) -> Size {
        self.button_layouts.clear();
        self.button_widths.clear();
        let mut total_w = 0.0;

        for (i, label) in self.buttons.iter().enumerate() {
            let is_selected = self.selected == Some(i);
            let fg = if is_selected {
                self.selected_fg
            } else {
                self.normal_fg
            };
            let mut opts = ctx.font_options.clone();
            opts.size = Some(14.0);
            opts.weight = Some(aurora_text::font_options::FontWeight::Medium);
            let mut tl =
                aurora_text::text_layout::TextLayout::new(ctx.font_manager, label, &opts, fg, None);
            tl.set_max_width(ctx.font_manager, f32::MAX);
            let tw = tl.size().width;
            let btn_w = tw + self.padding * 2.0;
            self.button_widths.push(btn_w);
            total_w += btn_w;
            self.button_layouts.push(Some(tl));
        }

        Size::new(total_w, self.height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Outer border
        canvas.stroke_rounded_rect(rect, self.corners, 1, self.border_color);

        let mut x = rect.x1;
        for (i, btn_w) in self.button_widths.iter().enumerate() {
            let btn_rect = Rect::new(x, rect.y1, x + btn_w, rect.y2);
            let is_selected = self.selected == Some(i);
            let is_hovered = self.hovered_index == Some(i);

            if is_selected {
                canvas.fill_rounded_rect(btn_rect, self.button_corners(i), self.selected_bg);
            } else if is_hovered {
                canvas.fill_rounded_rect(btn_rect, self.button_corners(i), self.hover_bg);
            }

            // Label
            if let Some(Some(tl)) = self.button_layouts.get(i) {
                let s = tl.size();
                let tx = btn_rect.x1 + (btn_rect.width() - s.width) / 2.0;
                let ty = btn_rect.y1 + (btn_rect.height() - s.height) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }

            // Vertical separator between buttons (except last)
            if i < self.buttons.len() - 1 {
                let sep_x = x + btn_w;
                canvas.fill_rect(
                    Rect::new(sep_x - 0.5, rect.y1 + 2.0, sep_x + 0.5, rect.y2 - 2.0),
                    self.border_color,
                );
            }

            x += btn_w;
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
                let mut x = rect.x1;
                for (i, btn_w) in self.button_widths.iter().enumerate() {
                    let btn_rect = Rect::new(x, rect.y1, x + btn_w, rect.y2);
                    if btn_rect.contains(&e.position) {
                        if let Some(ref mut cb) = self.on_click {
                            cb(i);
                        }
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    x += btn_w;
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                if rect.contains(pos) {
                    // Hit-test which button is hovered
                    let mut x = rect.x1;
                    let mut found = None;
                    for (i, btn_w) in self.button_widths.iter().enumerate() {
                        let btn_rect = Rect::new(x, rect.y1, x + btn_w, rect.y2);
                        if btn_rect.contains(pos) {
                            found = Some(i);
                            break;
                        }
                        x += btn_w;
                    }
                    self.hovered_index = found;
                    EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    }
                } else {
                    self.hovered_index = None;
                    EventResponse::default()
                }
            }
            _ => EventResponse::default(),
        }
    }
    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Group)
    }
}
