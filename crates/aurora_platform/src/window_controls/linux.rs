use super::ControlButton;
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_render::canvas::Canvas;

// GNOME Adwaita-inspired colors
const BUTTON_BG: Color = Color::new(222, 222, 222, 255); // #dedede
const HOVER_BG: Color = Color::new(200, 200, 200, 255); // #c8c8c8
const PRESSED_BG: Color = Color::new(178, 178, 178, 255); // #b2b2b2
const CLOSE_HOVER_BG: Color = Color::new(224, 72, 58, 255); // #e0483a
const CLOSE_PRESSED_BG: Color = Color::new(196, 56, 44, 255); // #c4382c
const DISABLED_BG: Color = Color::new(230, 230, 230, 128); // faded

const ICON_COLOR: Color = Color::new(48, 48, 48, 255); // #303030
const ICON_ON_CLOSE_HOVER: Color = Color::new(255, 255, 255, 255);
const ICON_DISABLED: Color = Color::new(48, 48, 48, 76);

const ICON_SIZE: f32 = 8.0;
const ICON_THICKNESS: u32 = 1;

pub(super) fn paint_button(
    canvas: &mut Canvas,
    rect: Rect,
    button: ControlButton,
    is_hovered: bool,
    is_pressed: bool,
    is_enabled: bool,
    is_maximized: bool,
) {
    let diameter = rect.x2 - rect.x1;
    let radius = diameter / 2.0;

    // Background circle
    let bg = if !is_enabled {
        DISABLED_BG
    } else if is_pressed {
        match button {
            ControlButton::Close => CLOSE_PRESSED_BG,
            _ => PRESSED_BG,
        }
    } else if is_hovered {
        match button {
            ControlButton::Close => CLOSE_HOVER_BG,
            _ => HOVER_BG,
        }
    } else {
        BUTTON_BG
    };
    canvas.fill_rounded_rect(rect, Corners::all(radius), bg);

    // Icon
    let icon_color = if !is_enabled {
        ICON_DISABLED
    } else if button == ControlButton::Close && (is_hovered || is_pressed) {
        ICON_ON_CLOSE_HOVER
    } else {
        ICON_COLOR
    };

    let cx = (rect.x1 + rect.x2) / 2.0;
    let cy = (rect.y1 + rect.y2) / 2.0;
    let half = ICON_SIZE / 2.0;

    match button {
        ControlButton::Minimize => {
            // Horizontal line
            canvas.fill_rect(
                Rect::new(cx - half, cy, cx + half, cy + 1.0),
                icon_color,
            );
        }
        ControlButton::Maximize => {
            if is_maximized {
                // Restore: two overlapping rectangles
                let small = ICON_SIZE - 2.0;
                let offset = 2.0;
                canvas.stroke_rect(
                    Rect::new(
                        cx - half + offset,
                        cy - half,
                        cx - half + offset + small,
                        cy - half + small,
                    ),
                    ICON_THICKNESS,
                    icon_color,
                );
                // Fill behind front rect
                canvas.fill_rounded_rect(
                    Rect::new(
                        cx - half,
                        cy - half + offset,
                        cx - half + small,
                        cy - half + offset + small,
                    ),
                    Corners::zero(),
                    bg,
                );
                canvas.stroke_rect(
                    Rect::new(
                        cx - half,
                        cy - half + offset,
                        cx - half + small,
                        cy - half + offset + small,
                    ),
                    ICON_THICKNESS,
                    icon_color,
                );
            } else {
                // Maximize: rectangle outline
                canvas.stroke_rect(
                    Rect::new(cx - half, cy - half, cx + half, cy + half),
                    ICON_THICKNESS,
                    icon_color,
                );
            }
        }
        ControlButton::Close => {
            // X icon
            canvas.draw_line(
                Point::new(cx - half, cy - half),
                Point::new(cx + half, cy + half),
                ICON_THICKNESS,
                icon_color,
            );
            canvas.draw_line(
                Point::new(cx + half, cy - half),
                Point::new(cx - half, cy + half),
                ICON_THICKNESS,
                icon_color,
            );
        }
    }
}
