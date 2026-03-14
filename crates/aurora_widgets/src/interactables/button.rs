//use crate::text_widget::Text;
//use crate::widgets::{EventResponse, LayoutCtx, Widget};
//use aurora_core::color::Color;
//use aurora_core::geometry::corners::Corners;
//use aurora_core::geometry::edges::Edges;
//use aurora_core::geometry::rect::Rect;
//use aurora_core::geometry::size::Size;
//use aurora_core::kmi::cursor_icon::CursorIcon;
//use aurora_core::kmi::mouse::{MouseClickEvent, MouseEvent, MouseState};
//use aurora_render::canvas::Canvas;
//
//type OnClickCallback = Box<dyn FnMut(&MouseClickEvent)>;
//
//pub struct Button {
//    child: Box<dyn Widget>,
//    background: Color,
//    hover_background: Option<Color>,
//    corners: Corners,
//    padding: Edges,
//    width: Option<f32>,
//    height: Option<f32>,
//    on_click: Option<OnClickCallback>,
//    hovered: bool,
//    child_rect: Rect,
//}
//
//impl Button {
//    pub fn new(child: impl Widget + 'static) -> Self {
//        Self {
//            child: Box::new(child),
//            ..Self::default()
//        }
//    }
//
//    pub fn background(mut self, color: Color) -> Self {
//        self.background = color;
//        self
//    }
//
//    pub fn hover_background(mut self, color: Color) -> Self {
//        self.hover_background = Some(color);
//        self
//    }
//
//    pub fn corners(mut self, corners: impl Into<Corners>) -> Self {
//        self.corners = corners.into();
//        self
//    }
//
//    pub fn padding(mut self, padding: impl Into<Edges>) -> Self {
//        self.padding = padding.into();
//        self
//    }
//
//    pub fn on_click(mut self, f: impl FnMut(&MouseClickEvent) + 'static) -> Self {
//        self.on_click = Some(Box::new(f));
//        self
//    }
//    pub fn width(mut self, width: f32) -> Self {
//        self.width = Some(width);
//        self
//    }
//    pub fn height(mut self, height: f32) -> Self {
//        self.height = Some(height);
//        self
//    }
//}
//impl Default for Button {
//    fn default() -> Self {
//        Self {
//            child: Box::new(Text::default()),
//            background: Color::TRANSPARENT,
//            hover_background: None,
//            corners: Corners::zero(),
//            padding: Edges::zero(),
//            width: None,
//            height: None,
//            on_click: None,
//            hovered: false,
//            child_rect: Rect::zero(),
//        }
//    }
//}
//impl Widget for Button {
//    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
//        let content_width = available.width - self.padding.horizontal();
//        let content_height = available.height - self.padding.vertical();
//        let content_area = Size::new(content_width.max(0.0), content_height.max(0.0));
//
//        let child_size = self.child.layout(content_area, ctx);
//
//        let width = self
//            .width
//            .unwrap_or(child_size.width + self.padding.horizontal());
//        let height = self
//            .height
//            .unwrap_or(child_size.height + self.padding.vertical());
//
//        self.child_rect = Rect::new(
//            self.padding.left,
//            self.padding.top,
//            self.padding.left + child_size.width,
//            self.padding.top + child_size.height,
//        );
//
//        Size::new(width, height)
//    }
//
//    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
//        let bg = if self.hovered {
//            self.hover_background.unwrap_or(self.background)
//        } else {
//            self.background
//        };
//        canvas.fill_rounded_rect(rect, self.corners, bg);
//
//        let translated = self.child_rect.translate(&rect.origin());
//        self.child.paint(canvas, translated);
//    }
//
//    fn children(&self) -> &[Box<dyn Widget>] {
//        std::slice::from_ref(&self.child)
//    }
//
//    fn event(&mut self, event: &MouseEvent, rect: Rect) -> EventResponse {
//        match event {
//            MouseEvent::MouseMoveEvent(pos) => {
//                let is_hovered = rect.contains(pos);
//                self.hovered = is_hovered;
//                if is_hovered {
//                    EventResponse {
//                        handled: true,
//                        cursor: Some(CursorIcon::Pointer),
//                    }
//                } else {
//                    EventResponse {
//                        handled: true,
//                        cursor: Some(CursorIcon::Default),
//                    }
//                }
//            }
//            MouseEvent::MouseClickEvent(click) => {
//                if click.state == MouseState::Released && rect.contains(&click.position) {
//                    if let Some(ref mut on_click) = self.on_click {
//                        on_click(click);
//                    }
//                    return EventResponse {
//                        handled: true,
//                        ..EventResponse::default()
//                    };
//                }
//
//                EventResponse {
//                    handled: false,
//                    ..EventResponse::default()
//                }
//            }
//            _ => EventResponse {
//                handled: false,
//                ..EventResponse::default()
//            },
//        }
//    }
//}

use crate::box_widget::BoxWidget;
use crate::composite::Composite;
use crate::interactables::touch_area::{OnClickCallback, TouchArea};
use crate::layout::column::Column;
use crate::text_widget::Text;
use crate::widgets::Widget;
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use std::cell::RefCell;
use std::rc::Rc;
pub struct ButtonOptions {
    pub text: String,
    pub on_click: OnClickCallback,
    pub width: u32,
    pub height: u32,
    pub background_color: Color,
    pub hover_background_color: Color,
    pub text_color: Color,
    pub text_hover_color: Color,
    pub border_radius: Corners,
}

impl Default for ButtonOptions {
    fn default() -> ButtonOptions {
        ButtonOptions {
            text: String::from("Button"),
            on_click: Box::new(|_| {}),
            width: 100,
            height: 50,
            background_color: Color::from_hex(0xcccccc, false),
            hover_background_color: Color::from_hex(0xbbbbbb, false),
            text_color: Color::BLACK,
            text_hover_color: Color::BLACK,
            border_radius: Corners::all(4.0),
        }
    }
}

#[derive(Default, Copy, Clone)]
pub struct ButtonState {
    is_hovering: bool,
}

pub fn button(options: ButtonOptions) -> impl Widget {
    let on_click = Rc::new(RefCell::new(options.on_click));
    let background = options.background_color;
    let hover_background = options.hover_background_color;
    let text_color = options.text_color;
    let text_hover_color = options.text_hover_color;
    let border_radius = options.border_radius;
    let text = options.text;
    let width = options.width;
    let height = options.height;

    Composite::new(ButtonState::default(), move |state, set_state| {
        let setter = set_state.clone();
        let click_handler = on_click.clone();

        Box::new(
            TouchArea::new()
                .child(
                    BoxWidget::new()
                        .corners(border_radius)
                        .background_color(if state.is_hovering {
                            hover_background
                        } else {
                            background
                        })
                        .width(width)
                        .height(height)
                        .child(
                            Column::new().child(Text::new(&text).color(if state.is_hovering {
                                text_hover_color
                            } else {
                                text_color
                            })),
                        ),
                )
                .on_hover(move |_position, hovering| {
                    setter.set(|prev| {
                        prev.is_hovering = hovering;
                    });
                })
                .on_click(move |event| {
                    click_handler.borrow_mut()(event);
                }),
        )
    })
}
