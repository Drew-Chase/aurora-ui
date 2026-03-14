use crate::box_widget::BoxWidget;
use crate::composite::Composite;
use crate::interactables::touch_area::{OnClickCallback, TouchArea};
use crate::layout::Align;
use crate::layout::column::Column;
use crate::text_widget::Text;
use crate::widgets::Widget;
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use std::cell::RefCell;
use std::rc::Rc;

pub struct ButtonOptions {
    pub text_options: Text,
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
            text_options: Text::default(),
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
    let text_options = options.text_options;
    let width = options.width;
    let height = options.height;

    Composite::new(ButtonState::default(), move |state, set_state| {
        let setter = set_state.clone();
        let click_handler = on_click.clone();
        let text_options = text_options.clone();

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
                            Column::new().child(text_options.width(width as f32).height(height as f32).color(if state.is_hovering {
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
