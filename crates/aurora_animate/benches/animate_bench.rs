use aurora_animate::easing::Easing;
use aurora_animate::timeline::Timeline;
use aurora_animate::tween::Tween;
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn bench_easing_all(c: &mut Criterion) {
    let easings = [
        Easing::Linear,
        Easing::QuadIn,
        Easing::QuadOut,
        Easing::QuadInOut,
        Easing::CubicIn,
        Easing::CubicOut,
        Easing::CubicInOut,
        Easing::SineIn,
        Easing::SineOut,
        Easing::SineInOut,
        Easing::ElasticOut,
        Easing::BounceOut,
        Easing::BackIn,
    ];
    c.bench_function("easing_13_variants", |bench| {
        bench.iter(|| {
            for e in &easings {
                black_box(e.apply(black_box(0.5)));
            }
        })
    });
}

fn bench_tween_tick(c: &mut Criterion) {
    c.bench_function("tween_tick_100_frames", |bench| {
        bench.iter(|| {
            let mut tween = Tween::new(0.0f32, 100.0)
                .duration(1.0)
                .easing(Easing::CubicInOut);
            for _ in 0..100 {
                tween.tick(black_box(0.01));
                black_box(tween.value());
            }
        })
    });
}

fn bench_timeline_tick(c: &mut Criterion) {
    c.bench_function("timeline_10_tracks", |bench| {
        bench.iter(|| {
            let mut timeline = Timeline::new();
            for i in 0..10 {
                timeline = timeline.add(i as f32 * 0.1, Tween::new(0.0f32, 100.0).duration(0.5));
            }
            for _ in 0..60 {
                timeline.tick(black_box(1.0 / 60.0));
                black_box(timeline.value());
            }
        })
    });
}

criterion_group!(
    benches,
    bench_easing_all,
    bench_tween_tick,
    bench_timeline_tick,
);
criterion_main!(benches);
