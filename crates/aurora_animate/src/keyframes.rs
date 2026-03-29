use crate::animatable::Animatable;
use crate::easing::Easing;
use crate::loop_mode::LoopMode;

/// A single keyframe stop in a [`KeyframeAnimation`].
///
/// Each keyframe specifies a value at a normalized time position (0.0–1.0)
/// and an easing function used to interpolate from this keyframe to the next.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Keyframe<T: Animatable> {
    /// The value at this keyframe.
    pub value: T,
    /// Normalized time position (0.0 to 1.0) within the total duration.
    pub time: f32,
    /// Easing function for the segment from this keyframe to the next.
    pub easing: Easing,
}

impl<T: Animatable> Keyframe<T> {
    /// Creates a new keyframe at the given normalized time with the given value.
    pub fn new(time: f32, value: T) -> Self {
        Self {
            value,
            time: time.clamp(0.0, 1.0),
            easing: Easing::Linear,
        }
    }

    /// Sets the easing function for the segment starting at this keyframe.
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }
}

/// A multi-stop animation defined by keyframes at specific time positions.
///
/// Each keyframe has a normalized time (0.0–1.0), a value, and an easing
/// function that controls interpolation to the next keyframe.
///
/// # Example
///
/// ```
/// use aurora_animate::{KeyframeAnimation, Keyframe, Easing};
///
/// let mut anim = KeyframeAnimation::new(vec![
///     Keyframe::new(0.0, 0.0f32),
///     Keyframe::new(0.3, 100.0).easing(Easing::CubicOut),
///     Keyframe::new(0.7, 50.0).easing(Easing::QuadInOut),
///     Keyframe::new(1.0, 80.0),
/// ])
/// .duration(2.0);
///
/// anim.tick(1.0); // 50% through
/// let value = anim.value();
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct KeyframeAnimation<T: Animatable> {
    keyframes: Vec<Keyframe<T>>,
    duration: f32,
    loop_mode: LoopMode,
    delay: f32,
    elapsed: f32,
    iteration: u32,
}

impl<T: Animatable> KeyframeAnimation<T> {
    /// Creates a new keyframe animation from a list of keyframes.
    ///
    /// Keyframes are sorted by time automatically. Panics if fewer than 2 keyframes are provided.
    pub fn new(mut keyframes: Vec<Keyframe<T>>) -> Self {
        assert!(
            keyframes.len() >= 2,
            "KeyframeAnimation requires at least 2 keyframes"
        );
        keyframes.sort_by(|a, b| {
            a.time
                .partial_cmp(&b.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Self {
            keyframes,
            duration: 1.0,
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
            return self.keyframes[0].value;
        }

        if self.duration <= 0.0 {
            return self.keyframes.last().unwrap().value;
        }

        let active_time = self.elapsed - self.delay;
        let total_progress = active_time / self.duration;
        let (progress, done) = self.resolve_progress(total_progress);

        if done {
            let iteration = self.compute_iteration(total_progress);
            if self.loop_mode.is_reversed(iteration) {
                return self.keyframes[0].value;
            }
            return self.keyframes.last().unwrap().value;
        }

        self.interpolate(progress)
    }

    /// Returns `true` if the animation has finished.
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

    /// Returns the raw progress (0.0 to 1.0) within the current iteration.
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

    fn interpolate(&self, progress: f32) -> T {
        // Find the two keyframes that bracket `progress`
        let kf = &self.keyframes;
        if progress <= kf[0].time {
            return kf[0].value;
        }
        if progress >= kf[kf.len() - 1].time {
            return kf[kf.len() - 1].value;
        }

        for i in 0..kf.len() - 1 {
            if progress >= kf[i].time && progress <= kf[i + 1].time {
                let segment_length = kf[i + 1].time - kf[i].time;
                if segment_length <= 0.0 {
                    return kf[i + 1].value;
                }
                let local_t = (progress - kf[i].time) / segment_length;
                let eased_t = kf[i].easing.apply(local_t);
                return kf[i].value.lerp(&kf[i + 1].value, eased_t);
            }
        }

        kf.last().unwrap().value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_keyframes() {
        let mut anim =
            KeyframeAnimation::new(vec![Keyframe::new(0.0, 0.0f32), Keyframe::new(1.0, 100.0)])
                .duration(1.0);

        assert!((anim.value() - 0.0).abs() < 0.01);

        anim.tick(0.5);
        assert!((anim.value() - 50.0).abs() < 0.01);

        anim.tick(0.5);
        assert!((anim.value() - 100.0).abs() < 0.01);
    }

    #[test]
    fn three_keyframes() {
        let mut anim = KeyframeAnimation::new(vec![
            Keyframe::new(0.0, 0.0f32),
            Keyframe::new(0.5, 100.0),
            Keyframe::new(1.0, 50.0),
        ])
        .duration(1.0);

        anim.tick(0.25); // 25% through first segment (0.0 -> 0.5)
        assert!((anim.value() - 50.0).abs() < 0.01);

        anim.tick(0.5); // 75% total, 50% through second segment (0.5 -> 1.0)
        assert!((anim.value() - 75.0).abs() < 0.01);
    }

    #[test]
    fn keyframes_with_easing() {
        let mut anim = KeyframeAnimation::new(vec![
            Keyframe::new(0.0, 0.0f32).easing(Easing::QuadIn),
            Keyframe::new(1.0, 100.0),
        ])
        .duration(1.0);

        anim.tick(0.5);
        // QuadIn at 0.5 = 0.25, so value should be ~25
        assert!((anim.value() - 25.0).abs() < 0.01);
    }

    #[test]
    fn keyframes_finished() {
        let mut anim =
            KeyframeAnimation::new(vec![Keyframe::new(0.0, 0.0f32), Keyframe::new(1.0, 100.0)])
                .duration(1.0);

        anim.tick(1.0);
        assert!(anim.is_finished());
    }

    #[test]
    fn keyframes_loop() {
        let mut anim =
            KeyframeAnimation::new(vec![Keyframe::new(0.0, 0.0f32), Keyframe::new(1.0, 100.0)])
                .duration(1.0)
                .loop_mode(LoopMode::Infinite);

        anim.tick(1.5);
        assert!(!anim.is_finished());
        assert!((anim.value() - 50.0).abs() < 0.01);
    }

    #[test]
    fn keyframes_reset() {
        let mut anim =
            KeyframeAnimation::new(vec![Keyframe::new(0.0, 0.0f32), Keyframe::new(1.0, 100.0)])
                .duration(1.0);

        anim.tick(0.5);
        anim.reset();
        assert!((anim.value() - 0.0).abs() < 0.01);
    }

    #[test]
    #[should_panic(expected = "at least 2 keyframes")]
    fn requires_two_keyframes() {
        KeyframeAnimation::new(vec![Keyframe::new(0.0, 0.0f32)]);
    }
}
