use crate::animatable::Animatable;
use crate::easing::Easing;
use crate::loop_mode::LoopMode;

/// A tween animation that interpolates between two values over a duration.
///
/// Create with the builder pattern, then call [`tick`](Self::tick) each frame
/// with the elapsed delta time (seconds) and read the current value with
/// [`value`](Self::value).
///
/// # Example
///
/// ```
/// use aurora_animate::{Tween, Easing};
///
/// let mut tween = Tween::new(0.0f32, 100.0)
///     .duration(0.5)
///     .easing(Easing::QuadOut);
///
/// tween.tick(0.25);
/// let current = tween.value(); // ~75.0 (QuadOut is fast-then-slow)
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tween<T: Animatable> {
    from: T,
    to: T,
    duration: f32,
    easing: Easing,
    loop_mode: LoopMode,
    delay: f32,
    elapsed: f32,
    iteration: u32,
}

impl<T: Animatable> Tween<T> {
    /// Creates a new tween from `from` to `to`.
    ///
    /// Defaults: 0.3s duration, linear easing, no looping, no delay.
    pub fn new(from: T, to: T) -> Self {
        Self {
            from,
            to,
            duration: 0.3,
            easing: Easing::Linear,
            loop_mode: LoopMode::Once,
            delay: 0.0,
            elapsed: 0.0,
            iteration: 0,
        }
    }

    /// Sets the animation duration in seconds.
    pub fn duration(mut self, seconds: f32) -> Self {
        self.duration = seconds.max(0.0);
        self
    }

    /// Sets the easing function.
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Sets the loop mode.
    pub fn loop_mode(mut self, mode: LoopMode) -> Self {
        self.loop_mode = mode;
        self
    }

    /// Sets an initial delay before the animation starts (seconds).
    pub fn delay(mut self, seconds: f32) -> Self {
        self.delay = seconds.max(0.0);
        self
    }

    /// Advances the animation by `dt` seconds.
    pub fn tick(&mut self, dt: f32) {
        if self.is_finished() {
            return;
        }
        self.elapsed += dt;
    }

    /// Returns the current interpolated value.
    pub fn value(&self) -> T {
        if self.elapsed < self.delay {
            return self.from;
        }

        if self.duration <= 0.0 {
            return self.to;
        }

        let active_time = self.elapsed - self.delay;
        let total_progress = active_time / self.duration;

        // Determine which iteration we're on and the local progress within it
        let (progress, done) = self.resolve_progress(total_progress);

        if done && !self.loop_mode.is_reversed(self.compute_iteration(total_progress)) {
            return self.to;
        }
        if done && self.loop_mode.is_reversed(self.compute_iteration(total_progress)) {
            return self.from;
        }

        let eased = self.easing.apply(progress);
        self.from.lerp(&self.to, eased)
    }

    /// Returns `true` if the animation has finished (respects loop mode).
    pub fn is_finished(&self) -> bool {
        if self.elapsed < self.delay {
            return false;
        }
        if self.duration <= 0.0 {
            return true;
        }
        let active_time = self.elapsed - self.delay;
        let total_progress = active_time / self.duration;
        let iteration = total_progress.floor() as u32;
        self.loop_mode.is_done(iteration) && total_progress >= iteration as f32
    }

    /// Returns the raw progress (0.0 to 1.0) within the current iteration, before easing.
    pub fn progress(&self) -> f32 {
        if self.elapsed < self.delay {
            return 0.0;
        }
        if self.duration <= 0.0 {
            return 1.0;
        }
        let active_time = self.elapsed - self.delay;
        let total_progress = active_time / self.duration;
        let (progress, _) = self.resolve_progress(total_progress);
        progress
    }

    /// Resets the animation to its initial state.
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.iteration = 0;
    }

    /// Resets and swaps `from` and `to`, effectively reversing the animation.
    pub fn reverse(&mut self) {
        std::mem::swap(&mut self.from, &mut self.to);
        self.reset();
    }

    /// Returns the `from` value.
    pub fn from_value(&self) -> T {
        self.from
    }

    /// Returns the `to` value.
    pub fn to_value(&self) -> T {
        self.to
    }

    /// Returns the configured duration in seconds.
    pub fn duration_secs(&self) -> f32 {
        self.duration
    }

    fn compute_iteration(&self, total_progress: f32) -> u32 {
        if total_progress <= 0.0 {
            0
        } else {
            total_progress.floor() as u32
        }
    }

    fn resolve_progress(&self, total_progress: f32) -> (f32, bool) {
        let iteration = self.compute_iteration(total_progress);

        if self.loop_mode.is_done(iteration) {
            return (1.0, true);
        }

        let local = total_progress - iteration as f32;
        let progress = local.clamp(0.0, 1.0);

        if self.loop_mode.is_reversed(iteration) {
            (1.0 - progress, false)
        } else {
            (progress, false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_tween() {
        let mut t = Tween::new(0.0f32, 100.0).duration(1.0);
        assert_eq!(t.value(), 0.0);

        t.tick(0.5);
        assert!((t.value() - 50.0).abs() < 0.01);

        t.tick(0.5);
        assert!((t.value() - 100.0).abs() < 0.01);
    }

    #[test]
    fn tween_with_delay() {
        let mut t = Tween::new(0.0f32, 100.0).duration(1.0).delay(0.5);
        assert_eq!(t.value(), 0.0);

        t.tick(0.5); // still in delay
        assert_eq!(t.value(), 0.0);

        t.tick(0.5); // 0.5s into animation
        assert!((t.value() - 50.0).abs() < 0.01);
    }

    #[test]
    fn tween_finished() {
        let mut t = Tween::new(0.0f32, 100.0).duration(1.0);
        assert!(!t.is_finished());

        t.tick(1.0);
        assert!(t.is_finished());

        // Should stay at end value
        assert!((t.value() - 100.0).abs() < 0.01);
    }

    #[test]
    fn tween_loop_count() {
        let mut t = Tween::new(0.0f32, 100.0)
            .duration(1.0)
            .loop_mode(LoopMode::Count(2));

        t.tick(1.5); // halfway through second iteration
        assert!(!t.is_finished());
        assert!((t.value() - 50.0).abs() < 0.01);

        t.tick(0.5);
        assert!(t.is_finished());
    }

    #[test]
    fn tween_infinite() {
        let mut t = Tween::new(0.0f32, 100.0)
            .duration(1.0)
            .loop_mode(LoopMode::Infinite);

        t.tick(2.5);
        assert!(!t.is_finished());
        assert!((t.value() - 50.0).abs() < 0.01);
    }

    #[test]
    fn tween_ping_pong() {
        let mut t = Tween::new(0.0f32, 100.0)
            .duration(1.0)
            .loop_mode(LoopMode::PingPong);

        t.tick(0.5); // forward, halfway
        assert!((t.value() - 50.0).abs() < 0.01);

        t.tick(0.75); // backward, 25% into reverse
        assert!((t.value() - 75.0).abs() < 0.01);

        t.tick(0.75);
        assert!(t.is_finished());
    }

    #[test]
    fn tween_reset() {
        let mut t = Tween::new(0.0f32, 100.0).duration(1.0);
        t.tick(0.5);
        t.reset();
        assert_eq!(t.value(), 0.0);
        assert!(!t.is_finished());
    }

    #[test]
    fn tween_reverse() {
        let mut t = Tween::new(0.0f32, 100.0).duration(1.0);
        t.reverse();
        assert_eq!(t.from_value(), 100.0);
        assert_eq!(t.to_value(), 0.0);
    }

    #[test]
    fn tween_with_easing() {
        let mut t = Tween::new(0.0f32, 100.0)
            .duration(1.0)
            .easing(Easing::QuadIn);

        t.tick(0.5);
        // QuadIn at 0.5 = 0.25, so value should be ~25
        assert!((t.value() - 25.0).abs() < 0.01);
    }

    #[test]
    fn zero_duration() {
        let t = Tween::new(0.0f32, 100.0).duration(0.0);
        assert_eq!(t.value(), 100.0);
        assert!(t.is_finished());
    }

    #[test]
    fn no_tick_after_finished() {
        let mut t = Tween::new(0.0f32, 100.0).duration(1.0);
        t.tick(2.0);
        let elapsed_before = t.elapsed;
        t.tick(1.0); // should be ignored
        assert_eq!(t.elapsed, elapsed_before);
    }
}
