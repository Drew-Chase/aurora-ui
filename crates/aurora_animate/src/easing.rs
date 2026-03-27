use std::f32::consts::PI;

/// An easing function that transforms linear progress into curved motion.
///
/// All variants map an input `t` in 0.0..=1.0 to an output value.
/// Most outputs stay in 0.0..=1.0, but `Elastic` and `Back` variants
/// intentionally overshoot, producing values outside that range.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Easing {
    /// Constant speed. `f(t) = t`.
    #[default]
    Linear,

    // --- Quadratic ---
    /// Slow start, accelerating.
    QuadIn,
    /// Fast start, decelerating.
    QuadOut,
    /// Slow start and end.
    QuadInOut,

    // --- Cubic ---
    CubicIn,
    CubicOut,
    CubicInOut,

    // --- Quartic ---
    QuartIn,
    QuartOut,
    QuartInOut,

    // --- Quintic ---
    QuintIn,
    QuintOut,
    QuintInOut,

    // --- Sine ---
    /// Gentle acceleration using a sine curve.
    SineIn,
    SineOut,
    SineInOut,

    // --- Exponential ---
    /// Sharp acceleration. Near-zero at start, explosive at end.
    ExpoIn,
    ExpoOut,
    ExpoInOut,

    // --- Circular ---
    CircIn,
    CircOut,
    CircInOut,

    // --- Back (overshoots target, then settles) ---
    BackIn,
    BackOut,
    BackInOut,

    // --- Elastic (springy overshoot) ---
    ElasticIn,
    ElasticOut,
    ElasticInOut,

    // --- Bounce (simulates bouncing off a surface) ---
    BounceIn,
    BounceOut,
    BounceInOut,
}

const C1: f32 = 1.70158;
const C2: f32 = C1 * 1.525;
const C3: f32 = C1 + 1.0;
const C4: f32 = (2.0 * PI) / 3.0;
const C5: f32 = (2.0 * PI) / 4.5;

fn bounce_out(t: f32) -> f32 {
    const N1: f32 = 7.5625;
    const D1: f32 = 2.75;
    if t < 1.0 / D1 {
        N1 * t * t
    } else if t < 2.0 / D1 {
        let t = t - 1.5 / D1;
        N1 * t * t + 0.75
    } else if t < 2.5 / D1 {
        let t = t - 2.25 / D1;
        N1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / D1;
        N1 * t * t + 0.984375
    }
}

impl Easing {
    /// Applies this easing function to a linear progress value.
    ///
    /// `t` is clamped to 0.0..=1.0 before evaluation.
    /// The return value may exceed that range for `Elastic` and `Back` easings.
    pub fn apply(&self, t: f32) -> f32 {
        if !t.is_finite() {
            return 0.0;
        }
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,

            // Quadratic
            Self::QuadIn => t * t,
            Self::QuadOut => 1.0 - (1.0 - t) * (1.0 - t),
            Self::QuadInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }

            // Cubic
            Self::CubicIn => t * t * t,
            Self::CubicOut => 1.0 - (1.0 - t).powi(3),
            Self::CubicInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }

            // Quartic
            Self::QuartIn => t * t * t * t,
            Self::QuartOut => 1.0 - (1.0 - t).powi(4),
            Self::QuartInOut => {
                if t < 0.5 {
                    8.0 * t * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(4) / 2.0
                }
            }

            // Quintic
            Self::QuintIn => t * t * t * t * t,
            Self::QuintOut => 1.0 - (1.0 - t).powi(5),
            Self::QuintInOut => {
                if t < 0.5 {
                    16.0 * t * t * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(5) / 2.0
                }
            }

            // Sine
            Self::SineIn => 1.0 - (t * PI / 2.0).cos(),
            Self::SineOut => (t * PI / 2.0).sin(),
            Self::SineInOut => -((PI * t).cos() - 1.0) / 2.0,

            // Exponential
            Self::ExpoIn => {
                if t == 0.0 {
                    0.0
                } else {
                    (2.0f32).powf(10.0 * t - 10.0)
                }
            }
            Self::ExpoOut => {
                if t == 1.0 {
                    1.0
                } else {
                    1.0 - (2.0f32).powf(-10.0 * t)
                }
            }
            Self::ExpoInOut => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    (2.0f32).powf(20.0 * t - 10.0) / 2.0
                } else {
                    (2.0 - (2.0f32).powf(-20.0 * t + 10.0)) / 2.0
                }
            }

            // Circular
            Self::CircIn => 1.0 - (1.0 - t * t).sqrt(),
            Self::CircOut => (1.0 - (t - 1.0).powi(2)).sqrt(),
            Self::CircInOut => {
                if t < 0.5 {
                    (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt()) / 2.0
                } else {
                    ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0
                }
            }

            // Back
            Self::BackIn => C3 * t * t * t - C1 * t * t,
            Self::BackOut => {
                let t1 = t - 1.0;
                1.0 + C3 * t1 * t1 * t1 + C1 * t1 * t1
            }
            Self::BackInOut => {
                if t < 0.5 {
                    ((2.0 * t).powi(2) * ((C2 + 1.0) * 2.0 * t - C2)) / 2.0
                } else {
                    ((2.0 * t - 2.0).powi(2) * ((C2 + 1.0) * (t * 2.0 - 2.0) + C2) + 2.0) / 2.0
                }
            }

            // Elastic
            Self::ElasticIn => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    -(2.0f32).powf(10.0 * t - 10.0) * ((10.0 * t - 10.75) * C4).sin()
                }
            }
            Self::ElasticOut => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    (2.0f32).powf(-10.0 * t) * ((10.0 * t - 0.75) * C4).sin() + 1.0
                }
            }
            Self::ElasticInOut => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    -((2.0f32).powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * C5).sin()) / 2.0
                } else {
                    ((2.0f32).powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * C5).sin()) / 2.0
                        + 1.0
                }
            }

            // Bounce
            Self::BounceOut => bounce_out(t),
            Self::BounceIn => 1.0 - bounce_out(1.0 - t),
            Self::BounceInOut => {
                if t < 0.5 {
                    (1.0 - bounce_out(1.0 - 2.0 * t)) / 2.0
                } else {
                    (1.0 + bounce_out(2.0 * t - 1.0)) / 2.0
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-5;

    fn assert_near(a: f32, b: f32, msg: &str) {
        assert!(
            (a - b).abs() < EPSILON,
            "{msg}: expected {b}, got {a}"
        );
    }

    #[test]
    fn all_easings_start_at_zero_and_end_at_one() {
        let easings = [
            Easing::Linear,
            Easing::QuadIn, Easing::QuadOut, Easing::QuadInOut,
            Easing::CubicIn, Easing::CubicOut, Easing::CubicInOut,
            Easing::QuartIn, Easing::QuartOut, Easing::QuartInOut,
            Easing::QuintIn, Easing::QuintOut, Easing::QuintInOut,
            Easing::SineIn, Easing::SineOut, Easing::SineInOut,
            Easing::ExpoIn, Easing::ExpoOut, Easing::ExpoInOut,
            Easing::CircIn, Easing::CircOut, Easing::CircInOut,
            Easing::BackIn, Easing::BackOut, Easing::BackInOut,
            Easing::ElasticIn, Easing::ElasticOut, Easing::ElasticInOut,
            Easing::BounceIn, Easing::BounceOut, Easing::BounceInOut,
        ];
        for easing in &easings {
            assert_near(easing.apply(0.0), 0.0, &format!("{easing:?} at t=0"));
            assert_near(easing.apply(1.0), 1.0, &format!("{easing:?} at t=1"));
        }
    }

    #[test]
    fn linear_is_identity() {
        for i in 0..=10 {
            let t = i as f32 / 10.0;
            assert_near(Easing::Linear.apply(t), t, "linear");
        }
    }

    #[test]
    fn quad_in_is_t_squared() {
        assert_near(Easing::QuadIn.apply(0.5), 0.25, "quad_in 0.5");
    }

    #[test]
    fn quad_out_is_fast_start() {
        assert_near(Easing::QuadOut.apply(0.5), 0.75, "quad_out 0.5");
    }

    #[test]
    fn bounce_out_stays_in_range() {
        for i in 0..=100 {
            let t = i as f32 / 100.0;
            let v = Easing::BounceOut.apply(t);
            assert!(v >= 0.0 && v <= 1.0, "bounce_out({t}) = {v} out of range");
        }
    }

    #[test]
    fn elastic_out_overshoots() {
        // ElasticOut should overshoot 1.0 at some point
        let mut overshoots = false;
        for i in 1..100 {
            let t = i as f32 / 100.0;
            if Easing::ElasticOut.apply(t) > 1.0 {
                overshoots = true;
                break;
            }
        }
        assert!(overshoots, "ElasticOut should overshoot 1.0");
    }

    #[test]
    fn back_in_goes_negative() {
        // BackIn should go negative at some point
        let mut goes_negative = false;
        for i in 1..100 {
            let t = i as f32 / 100.0;
            if Easing::BackIn.apply(t) < 0.0 {
                goes_negative = true;
                break;
            }
        }
        assert!(goes_negative, "BackIn should go negative");
    }

    #[test]
    fn default_is_linear() {
        assert_eq!(Easing::default(), Easing::Linear);
    }

    #[test]
    fn clamps_input() {
        assert_near(Easing::Linear.apply(-0.5), 0.0, "clamp negative");
        assert_near(Easing::Linear.apply(1.5), 1.0, "clamp above 1");
    }
}
