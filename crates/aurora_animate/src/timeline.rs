use crate::animatable::Animatable;
use crate::tween::Tween;

/// A container that sequences or overlaps multiple tweens of the same type
/// with time offsets.
///
/// Each track is a `Tween<T>` that starts at a given offset (seconds) from the
/// timeline's beginning. Tracks can overlap.
///
/// For animating multiple properties of different types simultaneously,
/// use separate `Timeline` instances and tick them together.
///
/// # Example
///
/// ```
/// use aurora_animate::{Timeline, Tween, Easing};
///
/// let mut timeline = Timeline::new()
///     .add(0.0, Tween::new(0.0f32, 100.0).duration(0.5).easing(Easing::QuadOut))
///     .add(0.3, Tween::new(100.0f32, 200.0).duration(0.5).easing(Easing::CubicIn));
///
/// timeline.tick(0.4);
/// let value = timeline.value(); // value from the latest active track
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Timeline<T: Animatable> {
    tracks: Vec<Track<T>>,
    elapsed: f32,
    total_duration: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Track<T: Animatable> {
    offset: f32,
    tween: Tween<T>,
}

impl<T: Animatable> Timeline<T> {
    /// Creates a new empty timeline.
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            elapsed: 0.0,
            total_duration: 0.0,
        }
    }

    /// Adds a tween starting at `offset` seconds from the timeline's beginning.
    pub fn add(mut self, offset: f32, tween: Tween<T>) -> Self {
        let end = offset + tween.duration_secs();
        if end > self.total_duration {
            self.total_duration = end;
        }
        self.tracks.push(Track {
            offset: offset.max(0.0),
            tween,
        });
        self
    }

    /// Advances all tracks by `dt` seconds.
    pub fn tick(&mut self, dt: f32) {
        let prev_elapsed = self.elapsed;
        self.elapsed += dt;
        for track in &mut self.tracks {
            let track_time = self.elapsed - track.offset;
            if track_time > 0.0 {
                let prev_track_time = prev_elapsed - track.offset;
                if prev_track_time <= 0.0 {
                    // Track just became active — set to absolute position
                    track.tween.reset();
                    track.tween.tick(track_time);
                } else {
                    // Track already active — advance by delta only
                    track.tween.tick(dt);
                }
            }
        }
    }

    /// Returns the value from the latest active (most recently started, not finished) track.
    ///
    /// If multiple tracks overlap, the one with the latest offset wins.
    /// If no tracks are active yet, returns the `from` value of the first track.
    /// If all tracks are finished, returns the final value of the last track.
    pub fn value(&self) -> T {
        assert!(!self.tracks.is_empty(), "Timeline has no tracks");

        // Find the latest active track (highest offset that has started)
        let mut best: Option<&Track<T>> = None;
        for track in &self.tracks {
            if self.elapsed >= track.offset {
                match best {
                    None => best = Some(track),
                    Some(current) => {
                        if track.offset >= current.offset {
                            best = Some(track);
                        }
                    }
                }
            }
        }

        match best {
            Some(track) => track.tween.value(),
            None => self.tracks[0].tween.from_value(),
        }
    }

    /// Returns the current value of every track.
    ///
    /// Tracks that haven't started yet return their `from` value.
    pub fn values(&self) -> Vec<T> {
        self.tracks
            .iter()
            .map(|track| {
                if self.elapsed < track.offset {
                    track.tween.from_value()
                } else {
                    track.tween.value()
                }
            })
            .collect()
    }

    /// Returns `true` when all tracks have finished.
    pub fn is_finished(&self) -> bool {
        if self.tracks.is_empty() {
            return true;
        }
        self.tracks.iter().all(|track| {
            let track_time = self.elapsed - track.offset;
            track_time > 0.0 && track.tween.is_finished()
        })
    }

    /// Resets all tracks to their initial state.
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        for track in &mut self.tracks {
            track.tween.reset();
        }
    }

    /// The total duration of the timeline (max of offset + tween duration across all tracks).
    pub fn total_duration(&self) -> f32 {
        self.total_duration
    }
}

impl<T: Animatable> Default for Timeline<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_track() {
        let mut tl = Timeline::new().add(0.0, Tween::new(0.0f32, 100.0).duration(1.0));

        tl.tick(0.5);
        assert!((tl.value() - 50.0).abs() < 0.01);

        tl.tick(0.5);
        assert!((tl.value() - 100.0).abs() < 0.01);
        assert!(tl.is_finished());
    }

    #[test]
    fn sequential_tracks() {
        let mut tl = Timeline::new()
            .add(0.0, Tween::new(0.0f32, 100.0).duration(1.0))
            .add(1.0, Tween::new(100.0f32, 0.0).duration(1.0));

        tl.tick(0.5);
        assert!((tl.value() - 50.0).abs() < 0.01);

        tl.tick(1.0); // 1.5s total — second track at 0.5
        assert!((tl.value() - 50.0).abs() < 0.01);

        tl.tick(0.5); // 2.0s total — both done
        assert!(tl.is_finished());
    }

    #[test]
    fn overlapping_tracks() {
        let mut tl = Timeline::new()
            .add(0.0, Tween::new(0.0f32, 100.0).duration(1.0))
            .add(0.5, Tween::new(200.0f32, 300.0).duration(1.0));

        tl.tick(0.75); // both active, second has higher offset
        // Second track at 0.25s into its 1.0s duration = 25% = 225.0
        assert!((tl.value() - 225.0).abs() < 0.01);
    }

    #[test]
    fn values_returns_all() {
        let mut tl = Timeline::new()
            .add(0.0, Tween::new(0.0f32, 100.0).duration(1.0))
            .add(0.5, Tween::new(200.0f32, 300.0).duration(1.0));

        tl.tick(0.5);
        let vals = tl.values();
        assert_eq!(vals.len(), 2);
    }

    #[test]
    fn reset_timeline() {
        let mut tl = Timeline::new().add(0.0, Tween::new(0.0f32, 100.0).duration(1.0));

        tl.tick(0.5);
        tl.reset();
        assert!((tl.value() - 0.0).abs() < 0.01);
    }

    #[test]
    fn total_duration() {
        let tl = Timeline::new()
            .add(0.0, Tween::new(0.0f32, 100.0).duration(1.0))
            .add(0.5, Tween::new(0.0f32, 100.0).duration(2.0));

        assert!((tl.total_duration() - 2.5).abs() < 0.01);
    }

    #[test]
    fn before_any_track_starts() {
        let mut tl = Timeline::new().add(1.0, Tween::new(50.0f32, 100.0).duration(1.0));

        tl.tick(0.5); // before the track starts
        assert!((tl.value() - 50.0).abs() < 0.01); // from value
    }
}
