use super::ControlButton;
use aurora_core::color::Color;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_render::canvas::Canvas;

// Windows 11 titlebar button colors
const HOVER_BG: Color = Color::new(229, 229, 229, 255); // #e5e5e5
const PRESSED_BG: Color = Color::new(204, 204, 204, 255); // #cccccc
const CLOSE_HOVER_BG: Color = Color::new(196, 43, 28, 255); // #c42b1c
const CLOSE_PRESSED_BG: Color = Color::new(181, 37, 23, 255); // #b52517
const ICON_COLOR: Color = Color::new(0, 0, 0, 255);
const ICON_ON_CLOSE_HOVER: Color = Color::new(255, 255, 255, 255);
const ICON_DISABLED: Color = Color::new(0, 0, 0, 76); // ~30% opacity

// Icon dimensions
const ICON_SIZE: f32 = 10.0;
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
    // Background
    if is_hovered || is_pressed {
        let bg = match button {
            ControlButton::Close => {
                if is_pressed {
                    CLOSE_PRESSED_BG
                } else {
                    CLOSE_HOVER_BG
                }
            }
            _ => {
                if is_pressed {
                    PRESSED_BG
                } else {
                    HOVER_BG
                }
            }
        };
        canvas.fill_rect(rect, bg);
    }

    // Icon color
    let icon_color = if !is_enabled {
        ICON_DISABLED
    } else if button == ControlButton::Close && (is_hovered || is_pressed) {
        ICON_ON_CLOSE_HOVER
    } else {
        ICON_COLOR
    };

    // Center the icon in the button
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
                // Restore icon: two overlapping rectangles
                let small = ICON_SIZE - 2.0;
                let offset = 2.0;

                // Back rectangle (top-right, partially hidden)
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

                // Front rectangle (bottom-left, fully visible)
                // First fill the area behind the front rect to cover the back rect's lines
                canvas.fill_rect(
                    Rect::new(
                        cx - half,
                        cy - half + offset,
                        cx - half + small,
                        cy - half + offset + small,
                    ),
                    if is_pressed {
                        if button == ControlButton::Close {
                            CLOSE_PRESSED_BG
                        } else {
                            PRESSED_BG
                        }
                    } else if is_hovered {
                        if button == ControlButton::Close {
                            CLOSE_HOVER_BG
                        } else {
                            HOVER_BG
                        }
                    } else {
                        Color::TRANSPARENT
                    },
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
                // Maximize icon: single rectangle outline
                canvas.stroke_rect(
                    Rect::new(cx - half, cy - half, cx + half, cy + half),
                    ICON_THICKNESS,
                    icon_color,
                );
            }
        }
        ControlButton::Close => {
            // X icon: two diagonal lines
            let x0 = cx - half;
            let y0 = cy - half;
            let x1 = cx + half;
            let y1 = cy + half;

            canvas.draw_line(
                Point::new(x0, y0),
                Point::new(x1, y1),
                ICON_THICKNESS,
                icon_color,
            );
            canvas.draw_line(
                Point::new(x1, y0),
                Point::new(x0, y1),
                ICON_THICKNESS,
                icon_color,
            );
        }
    }
}
