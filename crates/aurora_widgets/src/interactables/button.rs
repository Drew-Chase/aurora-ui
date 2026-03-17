use crate::box_widget::BoxWidget;
use crate::composite::Composite;
use crate::interactables::touch_area::{OnClickCallback, TouchArea};
use crate::layout::{Align, Justify};
use crate::text_widget::Text;
use crate::widgets::Widget;
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use std::cell::RefCell;
use std::rc::Rc;
use aurora_core::kmi::cursor_icon::CursorIcon;

/// Configuration for the [`button`] function.
///
/// All fields have sensible defaults via [`Default`], so you can override
/// only what you need with `..ButtonOptions::default()`.
pub struct ButtonOptions {
    /// Text widget used as the button label.
    pub text_options: Text,
    /// Callback invoked on click.
    pub on_click: OnClickCallback,
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
    pub hover_cursor: CursorIcon,
}

impl Default for ButtonOptions {
    fn default() -> ButtonOptions {
        ButtonOptions {
            text_options: Text::default()
                .align(Align::Center)
                .justify(Justify::Center),
            on_click: Box::new(|_| {}),
            width: 100,
            height: 50,
            background_color: Color::from_hex(0xcccccc, false),
            hover_background_color: Color::from_hex(0xbbbbbb, false),
            text_color: Color::BLACK,
            text_hover_color: Color::BLACK,
            border_radius: Corners::all(4.0),
            hover_cursor: CursorIcon::Pointer,
        }
    }
}

#[derive(Default, Copy, Clone)]
struct ButtonState {
    is_hovering: bool,
}

/// Creates a styled button widget from the given [`ButtonOptions`].
///
/// Internally uses a [`Composite`] to manage hover state and rebuild
/// the visual tree when the cursor enters or leaves the button.
pub fn button(options: ButtonOptions) -> impl Widget {
    let on_click = Rc::new(RefCell::new(options.on_click));
    let background = options.background_color;
    let hover_background = options.hover_background_color;
    let text_color = options.text_color;
    let text_hover_color = options.text_hover_color;
    let border_radius = options.border_radius;
    let text_options = options.text_options;
    let width = options.width;
    let height = options.height;
    let hover_cursor = options.hover_cursor;

    Composite::new(ButtonState::default(), move |state, set_state| {
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
    })
}
