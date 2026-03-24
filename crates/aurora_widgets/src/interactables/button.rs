use crate::box_widget::BoxWidget;
use crate::composite::{Composite, CompositeBuilder};
use crate::interactables::touch_area::{OnClickCallback, TouchArea};
use crate::layout::{Align, Justify};
use crate::text_widget::Text;
use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::MouseClickEvent;
use aurora_core::kmi::WidgetEvent;
use aurora_macros::composite_widget;
use aurora_render::canvas::Canvas;
use std::cell::RefCell;
use std::rc::Rc;

type ClickHandler = Rc<RefCell<OnClickCallback>>;

/// A styled button widget.
///
/// Uses `#[composite_widget]` for builder-pattern setters and automatic
/// `Widget` implementation. Internally manages hover state via a
/// [`Composite`].
///
/// # Example
///
/// ```ignore
/// Button::new()
///     .text("Click me")
///     .on_click(|event| println!("clicked at {:?}", event.position))
/// ```
#[composite_widget]
pub struct Button {
    /// Button width in pixels.
    pub width: u32,
    /// Button height in pixels.
    pub height: u32,
    /// Background color in the normal state.
    pub background_color: Color,
    /// Background color when hovered.
    pub hover_background_color: Color,
    /// Text color in the normal state.
    pub text_color: Color,
    /// Text color when hovered.
    pub text_hover_color: Color,
    /// Corner radii for the button rectangle.
    pub border_radius: Corners,
    /// Cursor icon shown when hovering.
    pub hover_cursor: CursorIcon,
    /// Text widget used as the button label. Use `.text()` for simple
    /// labels or `.text_options()` for full control.
    #[composite(skip)]
    pub text_options: Text,
    /// Click callback. Use `.on_click()` to set.
    #[composite(skip)]
    on_click: ClickHandler,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            text_options: Text::default().align(Align::Center).justify(Justify::Center),
            on_click: Rc::new(RefCell::new(Box::new(|_| {}))),
            width: 100,
            height: 50,
            background_color: Color::from_hex(0xcccccc, false),
            hover_background_color: Color::from_hex(0xbbbbbb, false),
            text_color: Color::BLACK,
            text_hover_color: Color::BLACK,
            border_radius: Corners::all(4.0),
            hover_cursor: CursorIcon::Pointer,
            __composite_inner: None,
        }
    }
}

impl Button {
    /// Sets the button label text with default centering.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text_options = Text::new(text.into())
            .align(Align::Center)
            .justify(Justify::Center);
        self
    }

    /// Sets the full [`Text`] widget for the label. Use this when you
    /// need custom font options, alignment, or styling beyond `.text()`.
    pub fn text_options(mut self, text: Text) -> Self {
        self.text_options = text;
        self
    }

    /// Sets the click callback.
    pub fn on_click(mut self, f: impl FnMut(&MouseClickEvent) + 'static) -> Self {
        self.on_click = Rc::new(RefCell::new(Box::new(f)));
        self
    }
}

#[derive(Default, Copy, Clone)]
struct ButtonState {
    is_hovering: bool,
}

impl CompositeBuilder for Button {
    fn build(&self) -> Box<dyn Widget> {
        let on_click = self.on_click.clone();
        let background = self.background_color;
        let hover_background = self.hover_background_color;
        let text_color = self.text_color;
        let text_hover_color = self.text_hover_color;
        let border_radius = self.border_radius;
        let text_options = self.text_options.clone();
        let width = self.width;
        let height = self.height;
        let hover_cursor = self.hover_cursor;

        Box::new(Composite::new(
            ButtonState::default(),
            move |state, set_state| {
                let setter = set_state.clone();
                let click_handler = on_click.clone();
                let text_options = text_options.clone();

                Box::new(
                    TouchArea::new()
                        .hover_cursor(hover_cursor)
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
                                    text_options
                                        .width(width as f32)
                                        .height(height as f32)
                                        .color(if state.is_hovering {
                                            text_hover_color
                                        } else {
                                            text_color
                                        }),
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
            },
        ))
    }
}
