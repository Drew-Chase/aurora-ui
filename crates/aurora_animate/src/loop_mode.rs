/// Controls how an animation repeats after completing.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum LoopMode {
    /// Play once and stop.
    #[default]
    Once,
    /// Repeat `n` times then stop (e.g. `Count(3)` plays 3 times total).
    Count(u32),
    /// Repeat forever.
    Infinite,
    /// Play forward, then backward, then stop (one full cycle).
    PingPong,
    /// Play forward then backward, repeating for `n` full cycles.
    PingPongCount(u32),
    /// Play forward then backward, repeating forever.
    PingPongInfinite,
}

impl LoopMode {
    /// Returns true if this animation should stop at the given iteration count.
    pub(crate) fn is_done(&self, iteration: u32) -> bool {
        match self {
            Self::Once => iteration >= 1,
            Self::Count(n) => iteration >= *n,
            Self::Infinite => false,
            Self::PingPong => iteration >= 2,
            Self::PingPongCount(n) => iteration >= n * 2,
            Self::PingPongInfinite => false,
        }
    }

    /// Returns true if the animation should play in reverse for the given iteration.
    pub(crate) fn is_reversed(&self, iteration: u32) -> bool {
        match self {
            Self::PingPong | Self::PingPongCount(_) | Self::PingPongInfinite => iteration % 2 == 1,
            _ => false,
        }
    }
}
