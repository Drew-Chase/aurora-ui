use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_render::canvas::Canvas;
use criterion::{Criterion, black_box, criterion_group, criterion_main};

const W: u32 = 1920;
const H: u32 = 1080;

#[cfg(not(feature = "text"))]
fn make_canvas(w: u32, h: u32, buf: &mut [u32]) -> Canvas<'_> {
    Canvas::new(w, h, buf)
}

#[cfg(feature = "text")]
fn make_canvas<'a>(
    w: u32,
    h: u32,
    buf: &'a mut [u32],
    fm: &'a mut aurora_text::font_manager::FontManager,
    sc: &'a mut aurora_text::cosmic_text::SwashCache,
) -> Canvas<'a> {
    Canvas::new(w, h, buf, fm, sc)
}

#[cfg(not(feature = "text"))]
macro_rules! with_canvas {
    ($buf:expr, |$canvas:ident| $body:expr) => {{
        let mut $canvas = make_canvas(W, H, $buf);
        $body
    }};
}

#[cfg(feature = "text")]
macro_rules! with_canvas {
    ($buf:expr, |$canvas:ident| $body:expr) => {{
        let mut fm = aurora_text::font_manager::FontManager::new();
        let mut sc = aurora_text::cosmic_text::SwashCache::new();
        let mut $canvas = make_canvas(W, H, $buf, &mut fm, &mut sc);
        $body
    }};
}

fn bench_fill_rect_opaque(c: &mut Criterion) {
    let mut buf = vec![0u32; (W * H) as usize];
    let rect = Rect::new(100.0, 100.0, 500.0, 400.0);
    c.bench_function("fill_rect_opaque_400x300", |bench| {
        bench.iter(|| {
            with_canvas!(&mut buf, |canvas| {
                canvas.fill_rect(black_box(rect), Color::RED);
            })
        })
    });
}

fn bench_fill_rect_transparent(c: &mut Criterion) {
    let mut buf = vec![0x00FFFFFFu32; (W * H) as usize];
    let rect = Rect::new(100.0, 100.0, 500.0, 400.0);
    let color = Color::new(0, 0, 0, 128);
    c.bench_function("fill_rect_semi_400x300", |bench| {
        bench.iter(|| {
            with_canvas!(&mut buf, |canvas| {
                canvas.fill_rect(black_box(rect), color);
            })
        })
    });
}

fn bench_fill_rounded_rect(c: &mut Criterion) {
    let mut buf = vec![0u32; (W * H) as usize];
    let rect = Rect::new(100.0, 100.0, 300.0, 300.0);
    let corners = Corners::all(8.0);
    c.bench_function("fill_rounded_rect_200x200_r8", |bench| {
        bench.iter(|| {
            with_canvas!(&mut buf, |canvas| {
                canvas.fill_rounded_rect(black_box(rect), corners, Color::BLUE);
            })
        })
    });
}

fn bench_draw_line(c: &mut Criterion) {
    let mut buf = vec![0u32; (W * H) as usize];
    let from = Point::new(0.0, 0.0);
    let to = Point::new(W as f32, H as f32);
    c.bench_function("draw_line_diagonal_2px", |bench| {
        bench.iter(|| {
            with_canvas!(&mut buf, |canvas| {
                canvas.draw_line(black_box(from), black_box(to), 2.0, Color::WHITE);
            })
        })
    });
}

criterion_group!(
    benches,
    bench_fill_rect_opaque,
    bench_fill_rect_transparent,
    bench_fill_rounded_rect,
    bench_draw_line,
);
criterion_main!(benches);
