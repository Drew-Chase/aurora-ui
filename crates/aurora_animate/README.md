# aurora_animate

Animation library for AuroraUI — tweens, easing, keyframes, timelines, and presets.

## Feature Gate

Enable with the `animate` feature on `aurora_ui`:

```toml
aurora_ui = { version = "0.1", features = ["software", "animate"] }
```

Or use the crate directly:

```toml
aurora_animate = "0.1"
```

## Quick Start

```rust
use aurora_animate::{Tween, Easing};
use std::time::Instant;

let mut tween = Tween::new(0.0f32, 100.0)
    .duration(0.5)
    .easing(Easing::QuadOut);

// In your render loop:
let mut last = Instant::now();
loop {
    let now = Instant::now();
    let dt = now.duration_since(last).as_secs_f32();
    last = now;

    tween.tick(dt);
    let value = tween.value();
    // use value...

    if tween.is_finished() { break; }
}
```

## Tween

Interpolates between two values over a duration with an easing function.

```rust
use aurora_animate::{Tween, Easing, LoopMode};
use aurora_core::color::Color;

// Simple color transition
let mut color_anim = Tween::new(Color::RED, Color::BLUE)
    .duration(1.0)
    .easing(Easing::CubicInOut);

// Looping position animation
let mut pos = Tween::new(0.0f32, 200.0)
    .duration(2.0)
    .easing(Easing::SineInOut)
    .loop_mode(LoopMode::PingPongInfinite);
```

## Easing Functions

31 easing functions across 10 families:

| Family | Variants |
|---|---|
| Linear | `Linear` |
| Quad | `QuadIn`, `QuadOut`, `QuadInOut` |
| Cubic | `CubicIn`, `CubicOut`, `CubicInOut` |
| Quart | `QuartIn`, `QuartOut`, `QuartInOut` |
| Quint | `QuintIn`, `QuintOut`, `QuintInOut` |
| Sine | `SineIn`, `SineOut`, `SineInOut` |
| Expo | `ExpoIn`, `ExpoOut`, `ExpoInOut` |
| Circ | `CircIn`, `CircOut`, `CircInOut` |
| Back | `BackIn`, `BackOut`, `BackInOut` |
| Elastic | `ElasticIn`, `ElasticOut`, `ElasticInOut` |
| Bounce | `BounceIn`, `BounceOut`, `BounceInOut` |

## Keyframes

Multi-stop animations with per-segment easing:

```rust
use aurora_animate::{KeyframeAnimation, Keyframe, Easing};

let mut anim = KeyframeAnimation::new(vec![
    Keyframe::new(0.0, 0.0f32),
    Keyframe::new(0.3, 100.0).easing(Easing::CubicOut),
    Keyframe::new(0.7, 50.0).easing(Easing::QuadInOut),
    Keyframe::new(1.0, 80.0),
])
.duration(2.0);
```

## Timeline

Sequence or overlap multiple tweens:

```rust
use aurora_animate::{Timeline, Tween, Easing};

let mut timeline = Timeline::new()
    .add(0.0, Tween::new(0.0f32, 100.0).duration(0.5))
    .add(0.3, Tween::new(100.0f32, 200.0).duration(0.5).easing(Easing::CubicIn));
```

## Presets

One-liner animations with tuned defaults:

| Preset | Duration | Easing | Loop | Use Case |
|---|---|---|---|---|
| `spring` | 0.6s | ElasticOut | Once | Button press, arrival |
| `bounce` | 0.8s | BounceOut | Once | Drop-in, notification |
| `smooth` | 0.25s | CubicInOut | Once | General transitions |
| `fade` | 0.3s | Linear | Once | Opacity changes |
| `slide` | 0.4s | QuadOut | Once | Panel/drawer open |
| `pop` | 0.4s | BackOut | Once | Toast, scale-up |
| `jiggle` | 0.5s | ElasticOut | PingPong | Attention grab |
| `shake` | 0.4s | SineInOut | PingPong x3 | Error indicator |
| `stretch` | 0.5s | BackInOut | Once | Squash & stretch |
| `pulse` | 0.6s | SineInOut | PingPong | Highlight |
| `breathe` | 1.5s | SineInOut | PingPong ∞ | Loading/idle |

```rust
use aurora_animate::Preset;

let mut anim = Preset::spring(0.0f32, 100.0);
// Customize further if needed:
let mut anim = Preset::bounce(0.0f32, 100.0).duration(1.2);
```

## Animatable Trait

Built-in support for `f32`, `Color`, `Point`, `Size`, `Rect`, `Edges`, and `Corners`.

Implement for custom types:

```rust
use aurora_animate::Animatable;

#[derive(Clone, Copy)]
struct Opacity(f32);

impl Animatable for Opacity {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Opacity(self.0 + (other.0 - self.0) * t)
    }
}
```

## Loop Modes

| Mode | Behavior |
|---|---|
| `Once` | Play once and stop |
| `Count(n)` | Repeat n times |
| `Infinite` | Repeat forever |
| `PingPong` | Forward then backward, once |
| `PingPongCount(n)` | Ping-pong n cycles |
| `PingPongInfinite` | Ping-pong forever |
