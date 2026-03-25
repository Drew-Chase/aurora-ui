#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aurora_ui::prelude::*;
use std::time::Instant;

fn main() {
    App::new()
        .title("Animation Example")
        .size((800, 600))
        .position(WindowPosition::Center)
        .use_system_fonts()
        .font_options(FontOptions::new().family("Segoe UI"))
        .run(|window, _frame_info| {
            window.root(animation_demo());
        })
        .expect("Failed to run app");
}

#[derive(Clone)]
struct AnimState {
    pos_tween: Tween<f32>,
    color_tween: Tween<Color>,
    breathe: Tween<f32>,
    bounce_tween: Tween<f32>,
    color_cycle: KeyframeAnimation<Color>,
    last_frame: Instant,
    started: bool,
}

impl Default for AnimState {
    fn default() -> Self {
        Self {
            pos_tween: Tween::new(0.0f32, 500.0)
                .duration(2.0)
                .easing(Easing::QuadOut),
            color_tween: Tween::new(
                Color::new(66, 133, 244, 255),
                Color::new(234, 67, 53, 255),
            )
            .duration(3.0)
            .easing(Easing::CubicInOut)
            .loop_mode(LoopMode::PingPongInfinite),
            breathe: Preset::breathe(0.3f32, 1.0),
            bounce_tween: Preset::bounce(0.0f32, 300.0),
            color_cycle: KeyframeAnimation::new(vec![
                Keyframe::new(0.0, Color::new(255, 87, 34, 255)).easing(Easing::CubicInOut),
                Keyframe::new(0.25, Color::new(76, 175, 80, 255)).easing(Easing::CubicInOut),
                Keyframe::new(0.5, Color::new(33, 150, 243, 255)).easing(Easing::CubicInOut),
                Keyframe::new(0.75, Color::new(156, 39, 176, 255)).easing(Easing::CubicInOut),
                Keyframe::new(1.0, Color::new(255, 87, 34, 255)),
            ])
            .duration(4.0)
            .loop_mode(LoopMode::Infinite),
            last_frame: Instant::now(),
            started: false,
        }
    }
}

fn animation_demo() -> impl Widget {
    Composite::new(AnimState::default(), move |state, set_state| {
        let now = Instant::now();
        let dt = if state.started {
            now.duration_since(state.last_frame).as_secs_f32()
        } else {
            0.0
        };

        let setter = set_state.clone();
        setter.set(move |s| {
            s.last_frame = now;
            s.started = true;
            s.pos_tween.tick(dt);
            s.color_tween.tick(dt);
            s.breathe.tick(dt);
            s.bounce_tween.tick(dt);
            s.color_cycle.tick(dt);
        });

        let pos_x = state.pos_tween.value();
        let anim_color = state.color_tween.value();
        let opacity = state.breathe.value();
        let bounce_x = state.bounce_tween.value();
        let cycle_color = state.color_cycle.value();

        Box::new(
            col!()
                .spacing(20.0)
                .padding(Edges::all(20.0))
                .child(
                    Text::new("Aurora Animate — Demo")
                        .font_size(28.0)
                        .font_weight(FontWeight::Bold),
                )
                // Row 1: Position tween (box slides right)
                .child(
                    col!()
                        .spacing(4.0)
                        .child(Text::new("Tween<f32> — Position (QuadOut)").font_size(14.0))
                        .child(
                            Stack::new()
                                .height(50.0)
                                .child(
                                    Positioned::absolute((pos_x, 0.0)).child(
                                        BoxWidget::new()
                                            .width(50)
                                            .height(50)
                                            .background_color(Color::new(66, 133, 244, 255))
                                            .corners(Corners::all(8.0)),
                                    ),
                                ),
                        ),
                )
                // Row 2: Color tween (ping-pong)
                .child(
                    col!()
                        .spacing(4.0)
                        .child(
                            Text::new("Tween<Color> — Ping-Pong (CubicInOut)").font_size(14.0),
                        )
                        .child(
                            BoxWidget::new()
                                .height(50)
                                .background_color(anim_color)
                                .corners(Corners::all(8.0)),
                        ),
                )
                // Row 3: Breathing opacity preset
                .child(
                    col!()
                        .spacing(4.0)
                        .child(Text::new("Preset::breathe — Opacity Pulse").font_size(14.0))
                        .child(
                            BoxWidget::new()
                                .height(50)
                                .background_color(Color::new(76, 175, 80, 255).opacity(opacity))
                                .corners(Corners::all(8.0)),
                        ),
                )
                // Row 4: Bounce preset
                .child(
                    col!()
                        .spacing(4.0)
                        .child(
                            Text::new("Preset::bounce — Drop-In (BounceOut)").font_size(14.0),
                        )
                        .child(
                            Stack::new()
                                .height(50.0)
                                .child(
                                    Positioned::absolute((bounce_x, 0.0)).child(
                                        BoxWidget::new()
                                            .width(50)
                                            .height(50)
                                            .background_color(Color::new(255, 152, 0, 255))
                                            .corners(Corners::all(25.0)),
                                    ),
                                ),
                        ),
                )
                // Row 5: Keyframe color cycle
                .child(
                    col!()
                        .spacing(4.0)
                        .child(
                            Text::new("KeyframeAnimation<Color> — 4-Stop Cycle").font_size(14.0),
                        )
                        .child(
                            BoxWidget::new()
                                .height(50)
                                .background_color(cycle_color)
                                .corners(Corners::all(8.0)),
                        ),
                ),
        )
    })
}
