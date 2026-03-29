use criterion::{black_box, criterion_group, criterion_main, Criterion};
use aurora_core::color::Color;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;

fn bench_rect_intersection(c: &mut Criterion) {
    let a = Rect::new(0.0, 0.0, 100.0, 100.0);
    let b = Rect::new(50.0, 50.0, 150.0, 150.0);
    c.bench_function("rect_intersection", |bench| {
        bench.iter(|| black_box(a.intersection(&black_box(b))))
    });
}

fn bench_rect_contains(c: &mut Criterion) {
    let r = Rect::new(0.0, 0.0, 100.0, 100.0);
    let p = Point::new(50.0, 50.0);
    c.bench_function("rect_contains", |bench| {
        bench.iter(|| black_box(r.contains(&black_box(p))))
    });
}

fn bench_rect_inset(c: &mut Criterion) {
    let r = Rect::new(0.0, 0.0, 200.0, 200.0);
    let edges = Edges::new(10.0, 20.0, 30.0, 40.0);
    c.bench_function("rect_inset", |bench| {
        bench.iter(|| black_box(r.inset(&black_box(edges))))
    });
}

fn bench_color_lerp(c: &mut Criterion) {
    let a = Color::from_rgb(255, 0, 0);
    let b = Color::from_rgb(0, 0, 255);
    c.bench_function("color_lerp", |bench| {
        bench.iter(|| black_box(a.lerp(&b, black_box(0.5))))
    });
}

fn bench_color_lerp_many(c: &mut Criterion) {
    let colors = vec![Color::RED, Color::GREEN, Color::BLUE, Color::WHITE];
    c.bench_function("color_lerp_many_4", |bench| {
        bench.iter(|| black_box(Color::lerp_many(&colors, black_box(0.33))))
    });
}

criterion_group!(
    benches,
    bench_rect_intersection,
    bench_rect_contains,
    bench_rect_inset,
    bench_color_lerp,
    bench_color_lerp_many,
);
criterion_main!(benches);
