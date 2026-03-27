use crate::box_widget::BoxWidget;
use crate::composite::{Composite, CompositeBuilder};
use crate::interactables::touch_area::{OnClickCallback, TouchArea};
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
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Instant;

type ClickHandler = Rc<RefCell<OnClickCallback>>;
type HoverHandler = Rc<RefCell<Box<dyn FnMut(bool)>>>;

const ANIM_DURATION: f32 = 0.12; // seconds

/// A styled button widget that wraps any child widget.
///
/// Background color transitions smoothly between normal and hover states.
///
/// # Examples
///
/// ```ignore
/// // Via macro:
/// button!("Click me").on_click(|e| println!("clicked"))
///
/// // With any child widget:
/// Button::new()
///     .child(my_icon_widget)
///     .on_click(|e| println!("clicked"))
///     .on_hover(|hovering| println!("hover: {hovering}"))
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
    /// Text/foreground color used by the `.text()` convenience method.
    pub foreground_color: Color,
    /// Corner radii for the button rectangle.
    pub border_radius: Corners,
    /// Cursor icon shown when hovering.
    pub hover_cursor: CursorIcon,
    /// Child widget displayed inside the button.
    #[composite(skip)]
    child: Option<Rc<RefCell<Box<dyn Widget>>>>,
    /// Label text set via `.text()`. Built into a Text widget lazily in `build()`.
    #[composite(skip)]
    label: Option<String>,
    /// Widget placed before the label/child (e.g. an icon).
    #[composite(skip)]
    start: Option<Rc<RefCell<Box<dyn Widget>>>>,
    /// Widget placed after the label/child (e.g. an icon).
    #[composite(skip)]
    end: Option<Rc<RefCell<Box<dyn Widget>>>>,
    /// Click callback. Use `.on_click()` to set.
    #[composite(skip)]
    on_click: ClickHandler,
    /// Hover callback. Receives `true` on enter, `false` on leave.
    #[composite(skip)]
    on_hover: Option<HoverHandler>,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            child: None,
            label: None,
            start: None,
            end: None,
            on_click: Rc::new(RefCell::new(Box::new(|_| {}))),
            on_hover: None,
            width: 100,
            height: 50,
            background_color: aurora_theme::color(aurora_theme::slots::PRIMARY),
            hover_background_color: aurora_theme::color(aurora_theme::slots::PRIMARY).opacity(0.8),
            foreground_color: aurora_theme::color(aurora_theme::slots::PRIMARY_FOREGROUND),
            border_radius: Corners::all(4.0),
            hover_cursor: CursorIcon::Pointer,
            __composite_inner: None,
        }
    }
}

impl Button {
    /// Sets any widget as the button's child content.
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Rc::new(RefCell::new(Box::new(child))));
        self
    }

    /// Sets the button label text with default centering.
    ///
    /// The text color is determined by [`foreground_color`](Self::foreground_color),
    /// so you can set it in any order:
    /// ```ignore
    /// button!("Save").foreground_color(colors::primary_foreground())
    /// ```
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }

    /// Sets a widget to display before the label (e.g. an icon).
    pub fn start(mut self, widget: impl Widget + 'static) -> Self {
        self.start = Some(Rc::new(RefCell::new(Box::new(widget))));
        self
    }

    /// Sets a widget to display after the label (e.g. an icon).
    pub fn end(mut self, widget: impl Widget + 'static) -> Self {
        self.end = Some(Rc::new(RefCell::new(Box::new(widget))));
        self
    }

    /// Sets the click callback.
    pub fn on_click(mut self, f: impl FnMut(&MouseClickEvent) + 'static) -> Self {
        self.on_click = Rc::new(RefCell::new(Box::new(f)));
        self
    }

    /// Sets the hover callback. Receives `true` when the cursor enters
    /// the button and `false` when it leaves.
    pub fn on_hover(mut self, f: impl FnMut(bool) + 'static) -> Self {
        self.on_hover = Some(Rc::new(RefCell::new(Box::new(f))));
        self
    }
}

#[derive(Clone, Copy)]
struct AnimData {
    from: f32,
    to: f32,
    start: Instant,
}

impl AnimData {
    fn current_t(&self) -> f32 {
        let elapsed = self.start.elapsed().as_secs_f32();
        let t = (elapsed / ANIM_DURATION).min(1.0);
        let eased = 1.0 - (1.0 - t).powi(3);
        self.from + (self.to - self.from) * eased
    }
}

#[derive(Default, Copy, Clone)]
struct ButtonState;

impl CompositeBuilder for Button {
    fn build(&self) -> Box<dyn Widget> {
        let on_click = self.on_click.clone();
        let on_hover = self.on_hover.clone();
        let background = self.background_color;
        let hover_background = self.hover_background_color;
        let border_radius = self.border_radius;
        let _has_slots = self.start.is_some() || self.end.is_some();
        // Build a Text child from the label if no explicit child was set
        let child = if self.child.is_some() {
            self.child.clone()
        } else {
            self.label.as_ref().and_then(|label| {
                #[cfg(feature = "text")]
                {
                    use crate::text_widget::Text;
                    let mut text_widget = Text::new(label.clone())
                        .color(self.foreground_color)
                        .justify(crate::layout::Justify::Center);
                    if !_has_slots {
                        // Single-child mode: fill button and center horizontally
                        text_widget = text_widget
                            .height(self.height as f32)
                            .align(crate::layout::Align::Center);
                    }
                    Some(Rc::new(RefCell::new(Box::new(text_widget) as Box<dyn Widget>)))
                }
                #[cfg(not(feature = "text"))]
                { let _ = label; None }
            })
        };
        let start = self.start.clone();
        let end = self.end.clone();
        let width = self.width;
        let height = self.height;
        let hover_cursor = self.hover_cursor;

        // Shared animation state that persists across Composite rebuilds
        let anim = Rc::new(Cell::new(AnimData {
            from: 0.0,
            to: 0.0,
            start: Instant::now(),
        }));

        let anim_for_closure = anim.clone();

        Box::new(Composite::new(
            ButtonState::default(),
            move |_state, _set_state| {
                let click_handler = on_click.clone();
                let on_hover = on_hover.clone();
                let child = child.clone();
                let start = start.clone();
                let end = end.clone();
                let anim = anim_for_closure.clone();

                let has_slots = start.is_some() || end.is_some();
                let mut box_widget = BoxWidget::new()
                    .corners(border_radius)
                    .background_color(Color::TRANSPARENT)
                    .width(width)
                    .height(height);

                if has_slots {
                    use crate::layout::{Align, Justify};
                    use crate::layout::row::Row;
                    use aurora_core::geometry::edges::Edges;
                    box_widget = box_widget.padding(Edges::new(0.0, 8.0, 0.0, 8.0));
                    let mut content_row = Row::new()
                        .spacing(6.0)
                        .height(height)
                        .align(Align::Center)
                        .justify(Justify::Center);
                    if let Some(s) = start {
                        content_row = content_row.child(FlexProxy { inner: s });
                    }
                    if let Some(c) = child {
                        content_row = content_row.child(FlexProxy { inner: c });
                    }
                    if let Some(e) = end {
                        content_row = content_row.child(FlexProxy { inner: e });
                    }
                    box_widget = box_widget.child(content_row);
                } else if let Some(child) = child {
                    box_widget = box_widget.child(ChildProxy {
                        inner: child,
                        width: width as f32,
                        height: height as f32,
                    });
                }

                Box::new(
                    TouchArea::new()
                        .hover_cursor(hover_cursor)
                        .child(AnimatedBg {
                            normal: background,
                            hover: hover_background,
                            corners: border_radius,
                            anim: anim.clone(),
                            inner: box_widget,
                        })
                        .on_hover(move |_position, hovering| {
                            let data = anim.get();
                            let new_data = AnimData {
                                from: data.current_t(),
                                to: if hovering { 1.0 } else { 0.0 },
                                start: Instant::now(),
                            };
                            anim.set(new_data);

                            if let Some(ref on_hover) = on_hover {
                                on_hover.borrow_mut()(hovering);
                            }
                        })
                        .on_click(move |event| {
                            click_handler.borrow_mut()(event);
                        }),
                )
            },
        ))
    }

    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        let mut info = aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Button);
        if let Some(ref label) = self.label {
            info = info.with_label(label.clone());
        }
        info
    }
}

/// Wraps a BoxWidget and paints an animated background color behind it.
struct AnimatedBg {
    normal: Color,
    hover: Color,
    corners: Corners,
    anim: Rc<Cell<AnimData>>,
    inner: BoxWidget,
}

impl Widget for AnimatedBg {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        self.inner.layout(available, ctx)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let t = self.anim.get().current_t();
        let bg = self.normal.lerp(&self.hover, t);
        canvas.fill_rounded_rect(rect, self.corners, bg);
        self.inner.paint(canvas, rect);
    }

    fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
        self.inner.paint_overlay(canvas, rect);
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        self.inner.children()
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        self.inner.event(event, rect)
    }

    fn needs_animation(&self) -> bool {
        self.anim.get().start.elapsed().as_secs_f32() < ANIM_DURATION
            || self.inner.needs_animation()
    }
}

/// Internal proxy that delegates to a shared child widget with fixed dimensions.
struct ChildProxy {
    inner: Rc<RefCell<Box<dyn Widget>>>,
    width: f32,
    height: f32,
}

impl Widget for ChildProxy {
    fn layout(&mut self, _available: Size, ctx: &mut LayoutCtx) -> Size {
        let size = Size::new(self.width, self.height);
        self.inner.borrow_mut().layout(size, ctx);
        size
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        self.inner.borrow().paint(canvas, rect);
    }

    fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
        self.inner.borrow().paint_overlay(canvas, rect);
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        self.inner.borrow_mut().event(event, rect)
    }
}

/// Proxy that delegates to a shared child widget, letting it size naturally.
struct FlexProxy {
    inner: Rc<RefCell<Box<dyn Widget>>>,
}

impl Widget for FlexProxy {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        self.inner.borrow_mut().layout(available, ctx)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        self.inner.borrow().paint(canvas, rect);
    }

    fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
        self.inner.borrow().paint_overlay(canvas, rect);
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        self.inner.borrow_mut().event(event, rect)
    }
}

/// Creates a [`Button`] with a child widget or text label.
///
/// String literals are passed to [`Button::text()`] (requires `text` feature).
/// All other expressions are passed to [`Button::child()`].
///
/// # Examples
///
/// ```ignore
/// button!("Click me")           // text button
/// button!(Image::new(my_image)) // image button
/// ```
#[macro_export]
macro_rules! button {
    ($text:literal) => {
        Button::new().text($text)
    };
    ($child:expr) => {
        Button::new().child($child)
    };
}
