use crate::animatable::Animatable;
use crate::easing::Easing;
use crate::loop_mode::LoopMode;
use crate::tween::Tween;

/// Pre-configured animation presets with sensible defaults.
///
/// Each method returns a [`Tween<T>`] that can be further customized
/// via builder methods before use.
///
/// # Example
///
/// ```
/// use aurora_animate::Preset;
///
/// let mut anim = Preset::spring(0.0f32, 100.0);
/// anim.tick(0.3);
/// let value = anim.value();
/// ```
pub struct Preset;

impl Preset {
    /// Spring-like settle: overshoots the target then settles back.
    ///
    /// Duration: 0.6s, Easing: ElasticOut.
    pub fn spring<T: Animatable>(from: T, to: T) -> Tween<T> {
        Tween::new(from, to)
            .duration(0.6)
            .easing(Easing::ElasticOut)
    }

    /// Bounce landing: reaches the target, bounces off, settles.
    ///
    /// Duration: 0.8s, Easing: BounceOut.
    pub fn bounce<T: Animatable>(from: T, to: T) -> Tween<T> {
        Tween::new(from, to).duration(0.8).easing(Easing::BounceOut)
    }

    /// Quick ease-in-out, good for general UI transitions.
    ///
    /// Duration: 0.25s, Easing: CubicInOut.
    pub fn smooth<T: Animatable>(from: T, to: T) -> Tween<T> {
        Tween::new(from, to)
            .duration(0.25)
            .easing(Easing::CubicInOut)
    }

    /// Gentle linear transition, ideal for opacity.
    ///
    /// Duration: 0.3s, Easing: Linear.
    pub fn fade<T: Animatable>(from: T, to: T) -> Tween<T> {
        Tween::new(from, to).duration(0.3).easing(Easing::Linear)
    }

    /// Slow entrance with a soft landing.
    ///
    /// Duration: 0.4s, Easing: QuadOut.
    pub fn slide<T: Animatable>(from: T, to: T) -> Tween<T> {
        Tween::new(from, to).duration(0.4).easing(Easing::QuadOut)
    }

    /// Overshoots slightly then settles — feels like anticipation.
    ///
    /// Duration: 0.4s, Easing: BackOut.
    pub fn pop<T: Animatable>(from: T, to: T) -> Tween<T> {
        Tween::new(from, to).duration(0.4).easing(Easing::BackOut)
    }

    /// Quick elastic jiggle — good for attention-grabbing.
    ///
    /// Duration: 0.5s, Easing: ElasticOut, PingPong once.
    pub fn jiggle<T: Animatable>(from: T, to: T) -> Tween<T> {
        Tween::new(from, to)
            .duration(0.5)
            .easing(Easing::ElasticOut)
            .loop_mode(LoopMode::PingPong)
    }

    /// Oscillating shake — good for error indicators.
    ///
    /// Duration: 0.4s, Easing: SineInOut, PingPong 3 cycles.
    pub fn shake<T: Animatable>(from: T, to: T) -> Tween<T> {
        Tween::new(from, to)
            .duration(0.4)
            .easing(Easing::SineInOut)
            .loop_mode(LoopMode::PingPongCount(3))
    }

    /// Scale stretch: overshoots then compresses slightly before settling.
    ///
    /// Duration: 0.5s, Easing: BackInOut.
    pub fn stretch<T: Animatable>(from: T, to: T) -> Tween<T> {
        Tween::new(from, to).duration(0.5).easing(Easing::BackInOut)
    }

    /// Pulse: goes to target and back once, good for highlights.
    ///
    /// Duration: 0.6s, Easing: SineInOut, PingPong once.
    pub fn pulse<T: Animatable>(from: T, to: T) -> Tween<T> {
        Tween::new(from, to)
            .duration(0.6)
            .easing(Easing::SineInOut)
            .loop_mode(LoopMode::PingPong)
    }

    /// Breathe: continuous slow pulse, repeats forever.
    ///
    /// Duration: 1.5s, Easing: SineInOut, PingPong infinite.
    pub fn breathe<T: Animatable>(from: T, to: T) -> Tween<T> {
        Tween::new(from, to)
            .duration(1.5)
            .easing(Easing::SineInOut)
            .loop_mode(LoopMode::PingPongInfinite)
    }
}
