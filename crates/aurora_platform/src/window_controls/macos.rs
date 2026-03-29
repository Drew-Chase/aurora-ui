use super::ControlButton;
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_render::canvas::Canvas;

// macOS stoplight colors
const CLOSE_COLOR: Color = Color::new(255, 95, 87, 255); // #FF5F57
const MINIMIZE_COLOR: Color = Color::new(254, 188, 46, 255); // #FEBC2E
const MAXIMIZE_COLOR: Color = Color::new(40, 200, 64, 255); // #28C840
const DISABLED_COLOR: Color = Color::new(204, 204, 204, 255); // #cccccc
const PRESSED_DARKEN: Color = Color::new(0, 0, 0, 40); // overlay

// Icon colors (drawn inside circles on hover)
const ICON_COLOR: Color = Color::new(76, 0, 0, 180); // dark, slightly transparent

const ICON_SIZE: f32 = 6.0;
const ICON_THICKNESS: f32 = 1.0;

fn button_color(button: ControlButton, is_enabled: bool) -> Color {
    if !is_enabled {
        return DISABLED_COLOR;
    }
    match button {
        ControlButton::Close => CLOSE_COLOR,
        ControlButton::Minimize => MINIMIZE_COLOR,
        ControlButton::Maximize => MAXIMIZE_COLOR,
    }
}

pub(super) fn paint_button(
    canvas: &mut Canvas,
    rect: Rect,
    button: ControlButton,
    is_hovered: bool,
    is_pressed: bool,
    is_enabled: bool,
) {
    let color = button_color(button, is_enabled);
    let diameter = rect.x2 - rect.x1;
    let radius = diameter / 2.0;

    // Draw filled circle
    canvas.fill_rounded_rect(rect, Corners::all(radius), color);

    // Darken on press
    if is_pressed && is_enabled {
        canvas.fill_rounded_rect(rect, Corners::all(radius), PRESSED_DARKEN);
    }

    // Draw icon inside circle on hover
    if is_hovered && is_enabled {
        let cx = (rect.x1 + rect.x2) / 2.0;
        let cy = (rect.y1 + rect.y2) / 2.0;
        let half = ICON_SIZE / 2.0;

        match button {
            ControlButton::Close => {
                // X icon
                canvas.draw_line(
                    Point::new(cx - half, cy - half),
                    Point::new(cx + half, cy + half),
                    ICON_THICKNESS,
                    ICON_COLOR,
                );
                canvas.draw_line(
                    Point::new(cx + half, cy - half),
                    Point::new(cx - half, cy + half),
                    ICON_THICKNESS,
                    ICON_COLOR,
                );
            }
            ControlButton::Minimize => {
                // Horizontal line (minus)
                canvas.fill_rect(Rect::new(cx - half, cy, cx + half, cy + 1.0), ICON_COLOR);
            }
            ControlButton::Maximize => {
                // Expand arrows: two diagonal arrows pointing to corners
                let arrow = half * 0.7;
                // Top-left to bottom-right diagonal
                canvas.draw_line(
                    Point::new(cx - arrow, cy + arrow),
                    Point::new(cx + arrow, cy - arrow),
                    ICON_THICKNESS,
                    ICON_COLOR,
                );
                // Small horizontal and vertical ticks at each end
                // Top-right tick
                canvas.draw_line(
                    Point::new(cx + arrow, cy - arrow),
                    Point::new(cx, cy - arrow),
                    ICON_THICKNESS,
                    ICON_COLOR,
                );
                canvas.draw_line(
                    Point::new(cx + arrow, cy - arrow),
                    Point::new(cx + arrow, cy),
                    ICON_THICKNESS,
                    ICON_COLOR,
                );
                // Bottom-left tick
                canvas.draw_line(
                    Point::new(cx - arrow, cy + arrow),
                    Point::new(cx, cy + arrow),
                    ICON_THICKNESS,
                    ICON_COLOR,
                );
                canvas.draw_line(
                    Point::new(cx - arrow, cy + arrow),
                    Point::new(cx - arrow, cy),
                    ICON_THICKNESS,
                    ICON_COLOR,
                );
            }
        }
    }
}
