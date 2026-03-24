#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use aurora_ui::prelude::*;

/// Generates a checkerboard pattern as RGBA pixels.
fn checkerboard(width: u32, height: u32, cell: u32) -> ImageData {
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let dark = ((x / cell) + (y / cell)).is_multiple_of(2);
            let idx = ((y * width + x) * 4) as usize;
            if dark {
                pixels[idx] = 60;
                pixels[idx + 1] = 60;
                pixels[idx + 2] = 60;
            } else {
                pixels[idx] = 200;
                pixels[idx + 1] = 200;
                pixels[idx + 2] = 200;
            }
            pixels[idx + 3] = 255;
        }
    }
    ImageData::from_raw(pixels, width, height)
}

/// Generates a gradient with transparency as RGBA pixels.
fn gradient(width: u32, height: u32) -> ImageData {
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            pixels[idx] = (x * 255 / width) as u8; // red
            pixels[idx + 1] = 50; // green
            pixels[idx + 2] = (y * 255 / height) as u8; // blue
            pixels[idx + 3] = ((x + y) * 255 / (width + height)) as u8; // alpha
        }
    }
    ImageData::from_raw(pixels, width, height)
}

fn main() {
    let checker = Arc::new(checkerboard(128, 128, 16));
    let grad = Arc::new(gradient(200, 200));

    // Load a PNG from an embedded file
    let github_png = Arc::new(
        ImageData::from_bytes(include_bytes!("../github.png")).expect("Failed to decode github.png"),
    );

    // Load an SVG from an embedded file
    let github_svg = Arc::new(
        SvgData::from_bytes(include_bytes!("../github.svg")).expect("Failed to parse github.svg"),
    );

    App::new()
        .title("Image Example")
        .size((600, 700))
        .min_size((400, 300))
        .position(WindowPosition::Center)
        .background_color(Color::from_rgb(30, 30, 30))
        .run(move |window, _frame_info| {
            window.root(
                col!()
                    .spacing(10.0)
                    .padding(Edges::all(10.0))
                    .child(
                        // Row of different fit modes on the checkerboard
                        row!()
                            .spacing(10.0)
                            .height(150)
                            .child(
                                BoxWidget::new()
                                    .background_color(Color::from_rgb(50, 50, 50))
                                    .corners(Corners::all(6.0))
                                    .child(
                                        Image::new(checker.clone())
                                            .fit(ImageFit::Contain),
                                    ),
                            )
                            .child(
                                BoxWidget::new()
                                    .background_color(Color::from_rgb(50, 50, 50))
                                    .corners(Corners::all(6.0))
                                    .child(
                                        Image::new(checker.clone())
                                            .fit(ImageFit::Cover),
                                    ),
                            )
                            .child(
                                BoxWidget::new()
                                    .background_color(Color::from_rgb(50, 50, 50))
                                    .corners(Corners::all(6.0))
                                    .child(
                                        Image::new(checker.clone())
                                            .fit(ImageFit::Fill),
                                    ),
                            ),
                    )
                    .child(
                        // Gradient with alpha blending over white background
                        BoxWidget::new()
                            .height(150)
                            .background_color(Color::WHITE)
                            .corners(Corners::all(6.0))
                            .child(
                                Image::new(grad.clone())
                                    .fit(ImageFit::Contain),
                            ),
                    )
                    .child(
                        // PNG loaded from file via include_bytes!
                        row!()
                            .spacing(10.0)
                            .height(120)
                            .child(
                                BoxWidget::new()
                                    .background_color(Color::from_rgb(50, 50, 50))
                                    .corners(Corners::all(6.0))
                                    .child(
                                        Image::new(github_png.clone())
                                            .fit(ImageFit::Contain),
                                    ),
                            )
                            .child(
                                BoxWidget::new()
                                    .background_color(Color::WHITE)
                                    .corners(Corners::all(6.0))
                                    .child(
                                        Image::new(github_png.clone())
                                            .fit(ImageFit::Contain),
                                    ),
                            ),
                    )
                    .child(
                        // SVG loaded from file via include_bytes!
                        row!()
                            .spacing(10.0)
                            .height(320)
                            .child(
                                BoxWidget::new()
                                    .background_color(Color::from_rgb(50, 50, 50))
                                    .corners(Corners::all(6.0))
                                    .child(
                                        Svg::new(github_svg.clone())
                                            .width(300.0)
                                            .height(300.0),
                                    ),
                            )
                            .child(
                                BoxWidget::new()
                                    .background_color(Color::WHITE)
                                    .corners(Corners::all(6.0))
                                    .child(
                                        Svg::new(github_svg.clone())
                                            .width(300.0)
                                            .height(300.0),
                                    ),
                            ),
                    ),
            );
        })
        .expect("Failed to run app");
}
