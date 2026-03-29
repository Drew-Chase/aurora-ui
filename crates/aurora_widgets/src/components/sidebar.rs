use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

/// Side of the layout where the sidebar is placed.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SidebarSide {
    #[default]
    Left,
    Right,
}

/// A sidebar layout container with a fixed sidebar and flexible main content.
///
/// # Example
/// ```ignore
/// Sidebar::new()
///     .sidebar_content(nav_menu)
///     .main_content(page_content)
///     .sidebar_width(250.0)
///     .side(SidebarSide::Left)
/// ```
pub struct Sidebar {
    sidebar: Option<Box<dyn Widget>>,
    main: Option<Box<dyn Widget>>,
    side: SidebarSide,
    sidebar_width: f32,
    border_color: Color,
    sidebar_bg: Color,
    main_bg: Color,
    sidebar_size: Size,
    main_size: Size,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            sidebar: None,
            main: None,
            side: SidebarSide::Left,
            sidebar_width: 250.0,
            border_color: colors::border(),
            sidebar_bg: colors::background(),
            main_bg: colors::background(),
            sidebar_size: Size::default(),
            main_size: Size::default(),
        }
    }

    pub fn sidebar_content(mut self, widget: impl Widget + 'static) -> Self {
        self.sidebar = Some(Box::new(widget));
        self
    }

    pub fn main_content(mut self, widget: impl Widget + 'static) -> Self {
        self.main = Some(Box::new(widget));
        self
    }

    pub fn side(mut self, side: SidebarSide) -> Self {
        self.side = side;
        self
    }

    pub fn sidebar_width(mut self, width: f32) -> Self {
        self.sidebar_width = width;
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    pub fn sidebar_bg(mut self, color: Color) -> Self {
        self.sidebar_bg = color;
        self
    }

    pub fn main_bg(mut self, color: Color) -> Self {
        self.main_bg = color;
        self
    }

    fn sidebar_rect(&self, rect: &Rect) -> Rect {
        match self.side {
            SidebarSide::Left => Rect::new(rect.x1, rect.y1, rect.x1 + self.sidebar_width, rect.y2),
            SidebarSide::Right => {
                Rect::new(rect.x2 - self.sidebar_width, rect.y1, rect.x2, rect.y2)
            }
        }
    }

    fn main_rect(&self, rect: &Rect) -> Rect {
        match self.side {
            SidebarSide::Left => Rect::new(rect.x1 + self.sidebar_width, rect.y1, rect.x2, rect.y2),
            SidebarSide::Right => {
                Rect::new(rect.x1, rect.y1, rect.x2 - self.sidebar_width, rect.y2)
            }
        }
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Sidebar {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let main_w = (available.width - self.sidebar_width).max(0.0);

        if let Some(ref mut sidebar) = self.sidebar {
            let sidebar_available = Size::new(self.sidebar_width, available.height);
            self.sidebar_size = sidebar.layout(sidebar_available, ctx);
        }

        if let Some(ref mut main) = self.main {
            let main_available = Size::new(main_w, available.height);
            self.main_size = main.layout(main_available, ctx);
        }

        Size::new(available.width, available.height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let sr = self.sidebar_rect(&rect);
        let mr = self.main_rect(&rect);

        // Sidebar background
        canvas.fill_rect(sr, self.sidebar_bg);

        // Main background
        canvas.fill_rect(mr, self.main_bg);

        // Paint sidebar content
        if let Some(ref sidebar) = self.sidebar {
            canvas.push_clip(sr);
            sidebar.paint(canvas, sr);
            canvas.pop_clip();
        }

        // Paint main content
        if let Some(ref main) = self.main {
            canvas.push_clip(mr);
            main.paint(canvas, mr);
            canvas.pop_clip();
        }

        // Border between sidebar and main
        let border_x = match self.side {
            SidebarSide::Left => sr.x2,
            SidebarSide::Right => sr.x1,
        };
        canvas.fill_rect(
            Rect::new(border_x - 0.5, rect.y1, border_x + 0.5, rect.y2),
            self.border_color,
        );
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        let sr = self.sidebar_rect(&rect);
        let mr = self.main_rect(&rect);

        // Try sidebar first
        if let Some(ref mut sidebar) = self.sidebar {
            let resp = sidebar.event(event, sr);
            if resp.handled {
                return resp;
            }
        }

        // Then main
        if let Some(ref mut main) = self.main {
            let resp = main.event(event, mr);
            if resp.handled {
                return resp;
            }
        }

        EventResponse::default()
    }
    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Complementary)
    }
}
