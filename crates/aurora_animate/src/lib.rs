#![doc = include_str!("../README.md")]

pub mod animatable;
pub mod easing;
pub mod keyframes;
pub mod loop_mode;
pub mod preset;
pub mod timeline;
pub mod tween;

pub use animatable::Animatable;
pub use easing::Easing;
pub use keyframes::{Keyframe, KeyframeAnimation};
pub use loop_mode::LoopMode;
pub use preset::Preset;
pub use timeline::Timeline;
pub use tween::Tween;
