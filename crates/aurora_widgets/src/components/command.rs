use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::keyboard::{Key, KeyboardEvent};
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

/// A command palette with a search input and filterable list of commands.
///
/// # Example
/// ```ignore
/// Command::new()
///     .open(true)
///     .command("New File", || println!("new file"))
///     .command("Open File", || println!("open file"))
///     .command("Save", || println!("save"))
/// ```
pub struct Command {
    open: bool,
    search: String,
    commands: Vec<CommandEntry>,
    width: f32,
    item_height: f32,
    max_items: usize,
    background: Color,
    border_color: Color,
    hover_bg: Color,
    corners: Corners,
    padding: Edges,
    hover_index: Option<usize>,
    filtered_indices: Vec<usize>,
    input_layout: Option<aurora_text::text_layout::TextLayout>,
    item_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
}

struct CommandEntry {
    label: String,
    callback: Box<dyn FnMut()>,
}

impl Command {
    pub fn new() -> Self {
        Self {
            open: false,
            search: String::new(),
            commands: Vec::new(),
            width: 500.0,
            item_height: 40.0,
            max_items: 10,
            background: colors::popover(),
            border_color: colors::border(),
            hover_bg: colors::accent(),
            corners: Corners::all(12.0),
            padding: Edges::new(8.0, 8.0, 8.0, 8.0),
            hover_index: None,
            filtered_indices: Vec::new(),
            input_layout: None,
            item_layouts: Vec::new(),
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn command(mut self, label: impl Into<String>, cb: impl FnMut() + 'static) -> Self {
        self.commands.push(CommandEntry {
            label: label.into(),
            callback: Box::new(cb),
        });
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn set_open(&mut self, open: bool) {
        self.open = open;
        if open {
            self.search.clear();
            self.hover_index = None;
        }
    }

    fn update_filtered(&mut self) {
        self.filtered_indices.clear();
        let search_lower = self.search.to_lowercase();
        for (i, cmd) in self.commands.iter().enumerate() {
            if search_lower.is_empty() || cmd.label.to_lowercase().contains(&search_lower) {
                self.filtered_indices.push(i);
                if self.filtered_indices.len() >= self.max_items {
                    break;
                }
            }
        }
    }

    fn palette_rect(&self, viewport: &Rect) -> Rect {
        let input_h = 48.0;
        let items_h = self.item_height * self.filtered_indices.len().min(self.max_items) as f32;
        let separator_h = if self.filtered_indices.is_empty() { 0.0 } else { 1.0 };
        let total_h = self.padding.top + input_h + separator_h + items_h + self.padding.bottom;

        let cx = viewport.x1 + viewport.width() / 2.0;
        let top = viewport.y1 + viewport.height() * 0.2;

        Rect::new(
            cx - self.width / 2.0,
            top,
            cx + self.width / 2.0,
            top + total_h,
        )
    }
}

impl Default for Command {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Command {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        if !self.open {
            return Size::new(0.0, 0.0);
        }

        self.update_filtered();
        let inner_w = self.width - self.padding.left - self.padding.right - 16.0;

        // Input text
        let display = if self.search.is_empty() {
            "Type a command...".to_string()
        } else {
            self.search.clone()
        };
        let _text_color = if self.search.is_empty() {
            colors::muted_foreground()
        } else {
            colors::foreground()
        };

        let mut opts = ctx.font_options.clone();
        opts.size = Some(16.0);
        opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
        let mut tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, &display, &opts, colors::foreground(), None);
        tl.set_max_width(ctx.font_manager, inner_w.max(0.0));
        self.input_layout = Some(tl);

        // Item layouts
        self.item_layouts.clear();
        for &idx in &self.filtered_indices {
            let label = &self.commands[idx].label;
            let mut opts = ctx.font_options.clone();
            opts.size = Some(14.0);
            opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
            let mut tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, label, &opts, colors::foreground(), None);
            tl.set_max_width(ctx.font_manager, inner_w.max(0.0));
            self.item_layouts.push(Some(tl));
        }

        Size::new(available.width, available.height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        if !self.open {
            return;
        }

        // Backdrop
        canvas.fill_rect(rect, colors::overlay());

        let pr = self.palette_rect(&rect);
        canvas.fill_rounded_rect(pr, self.corners, self.background);
        canvas.stroke_rounded_rect(pr, self.corners, 1, self.border_color);

        let mut y = pr.y1 + self.padding.top;
        let x = pr.x1 + self.padding.left;

        // Search input area
        if let Some(ref tl) = self.input_layout {
            let th = tl.size().height;
            let tx = x + 8.0;
            let ty = y + (48.0 - th) / 2.0;
            canvas.draw_text(tl, tx as i32, ty as i32);
        }
        y += 48.0;

        // Separator
        if !self.filtered_indices.is_empty() {
            canvas.fill_rect(
                Rect::new(pr.x1, y, pr.x2, y + 1.0),
                colors::border(),
            );
            y += 1.0;
        }

        // Items
        for (i, _) in self.filtered_indices.iter().enumerate() {
            let item_rect = Rect::new(
                pr.x1 + self.padding.left,
                y,
                pr.x2 - self.padding.right,
                y + self.item_height,
            );

            if self.hover_index == Some(i) {
                let item_corners = Corners::all(6.0);
                canvas.fill_rounded_rect(item_rect, item_corners, self.hover_bg);
            }

            if let Some(Some(tl)) = self.item_layouts.get(i) {
                let th = tl.size().height;
                let tx = item_rect.x1 + 8.0;
                let ty = y + (self.item_height - th) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }

            y += self.item_height;
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        if !self.open {
            return EventResponse::default();
        }

        let pr = self.palette_rect(&rect);

        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed =>
            {
                if !pr.contains(&e.position) {
                    self.open = false;
                    return EventResponse {
                        handled: true,
                        ..Default::default()
                    };
                }
                // Check if clicked on an item
                let items_y = pr.y1 + self.padding.top + 48.0 + 1.0;
                if e.position.y >= items_y {
                    let relative_y = e.position.y - items_y;
                    let idx = (relative_y / self.item_height) as usize;
                    if idx < self.filtered_indices.len() {
                        let cmd_idx = self.filtered_indices[idx];
                        self.open = false;
                        self.commands[cmd_idx].callback.as_mut()();
                        return EventResponse {
                            handled: true,
                            ..Default::default()
                        };
                    }
                }
                EventResponse {
                    handled: true,
                    ..Default::default()
                }
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                if pr.contains(pos) {
                    let items_y = pr.y1 + self.padding.top + 48.0 + 1.0;
                    if pos.y >= items_y {
                        let relative_y = pos.y - items_y;
                        let idx = (relative_y / self.item_height) as usize;
                        self.hover_index = if idx < self.filtered_indices.len() { Some(idx) } else { None };
                    } else {
                        self.hover_index = None;
                    }
                    return EventResponse {
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                EventResponse {
                    handled: true,
                    ..Default::default()
                }
            }
            WidgetEvent::Keyboard(KeyboardEvent::CharTyped(ch)) => {
                if !ch.is_control() {
                    self.search.push(*ch);
                    self.hover_index = None;
                }
                EventResponse {
                    handled: true,
                    ..Default::default()
                }
            }
            WidgetEvent::Keyboard(KeyboardEvent::KeyPressed { key, .. }) => {
                match key {
                    Key::Backspace => {
                        self.search.pop();
                        self.hover_index = None;
                    }
                    Key::Escape => {
                        self.open = false;
                        self.search.clear();
                    }
                    Key::Enter => {
                        if let Some(hi) = self.hover_index
                            && hi < self.filtered_indices.len()
                        {
                            let cmd_idx = self.filtered_indices[hi];
                            self.open = false;
                            self.commands[cmd_idx].callback.as_mut()();
                        }
                    }
                    Key::Down => {
                        let max = self.filtered_indices.len();
                        if max > 0 {
                            self.hover_index = Some(
                                self.hover_index.map(|i| (i + 1).min(max - 1)).unwrap_or(0),
                            );
                        }
                    }
                    Key::Up => {
                        if let Some(i) = self.hover_index {
                            self.hover_index = Some(i.saturating_sub(1));
                        }
                    }
                    _ => {}
                }
                EventResponse {
                    handled: true,
                    ..Default::default()
                }
            }
            _ => EventResponse {
                handled: true,
                ..Default::default()
            },
        }
    }
}
