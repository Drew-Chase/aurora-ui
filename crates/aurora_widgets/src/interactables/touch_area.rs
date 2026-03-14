use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseClickEvent, MouseEvent, MouseState};
use aurora_render::canvas::Canvas;

pub type OnClickCallback = Box<dyn FnMut(&MouseClickEvent)>;
pub type OnHoverCallback = Box<dyn FnMut(Rect, bool)>;

#[derive(Default)]
pub struct TouchArea {
    on_click: Option<OnClickCallback>,
    on_hover: Option<OnHoverCallback>,
    width: Option<f32>,
    height: Option<f32>,
    child: Option<Box<dyn Widget>>,
    child_rect: Rect,
    hovered: bool,
}

impl TouchArea {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn on_click(mut self, f: impl FnMut(&MouseClickEvent) + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
    pub fn on_hover(mut self, f: impl FnMut(Rect, bool) + 'static) -> Self {
        self.on_hover = Some(Box::new(f));
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }
}

impl Widget for TouchArea {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let child_size = self
            .child
            .as_mut()
            .map(|child| child.layout(available, ctx));
        let width = self
            .width
            .unwrap_or(child_size.unwrap_or(available).width)
            .min(available.width);
        let height = self
            .height
            .unwrap_or(child_size.unwrap_or(available).height)
            .min(available.height);

        self.child_rect = Rect::new(0f32, 0f32, width, height);

        Size::new(width, height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        if let Some(child) = self.child.as_ref() {
            let translated = self.child_rect.translate(&rect.origin());
            child.paint(canvas, translated);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        match &self.child {
            Some(child) => std::slice::from_ref(child),
            None => &[],
        }
    }

    fn event(&mut self, event: &MouseEvent, rect: Rect) -> EventResponse {
        match event {
            MouseEvent::MouseMoveEvent(pos) => {
                self.hovered = rect.contains(pos);
                if let Some(ref mut on_hover) = self.on_hover {
                    on_hover(rect, self.hovered);
                }
                if self.hovered {
                    EventResponse {
                        handled: true,
                        cursor: Some(CursorIcon::Pointer),
                    }
                } else {
                    EventResponse {
                        handled: true,
                        cursor: Some(CursorIcon::Default),
                    }
                }
            }
            MouseEvent::MouseClickEvent(click) => {
                if click.state == MouseState::Released && rect.contains(&click.position) {
                    if let Some(ref mut on_click) = self.on_click {
                        on_click(click);
                    }
                    return EventResponse {
                        handled: true,
                        ..EventResponse::default()
                    };
                }

                EventResponse {
                    handled: false,
                    ..EventResponse::default()
                }
            }
            _ => EventResponse {
                handled: false,
                ..EventResponse::default()
            },
        }
    }
}
