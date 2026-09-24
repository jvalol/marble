//! The sound a landing makes. See `specs/0005-landing-sound.md`.
//!
//! The thud is generated rather than loaded. A sine sweeping down from 150 Hz
//! to 55 Hz over 180 milliseconds, fading as it goes: short enough to read as
//! an impact, low enough to read as weight. Nothing here needs an audio
//! device, so all of it can be checked without one.

use rodio::source::{chirp, Source};
use rodio::SampleRate;
use std::time::Duration;

use crate::marble::MAX_FALL;

/// Under this downward speed a landing is a settle and makes no sound. About a
/// tenth of a second of falling, which is roughly the marble's own height.
pub const MIN_IMPACT: f32 = 3.0;
/// How loud the quietest audible landing is, against 1.0 for the longest drop.
pub const MIN_VOLUME: f32 = 0.25;

const LENGTH: Duration = Duration::from_millis(180);
const START_HZ: f32 = 150.0;
const END_HZ: f32 = 55.0;
const SAMPLE_RATE: u32 = 48_000;

/// How loud a landing at this speed is, or `None` when it is only a settle.
///
/// Rises linearly from [`MIN_VOLUME`] at the threshold to 1.0 at the terminal
/// fall speed, and stops there however far the marble fell.
pub fn volume(impact: f32) -> Option<f32> {
    if impact < MIN_IMPACT {
        return None;
    }

    let along = ((impact - MIN_IMPACT) / (MAX_FALL - MIN_IMPACT)).clamp(0.0, 1.0);

    Some(MIN_VOLUME + along * (1.0 - MIN_VOLUME))
}

/// The sound itself, at the volume [`volume`] gave.
pub fn thud(volume: f32) -> impl Source + Send + 'static {
    let rate = SampleRate::new(SAMPLE_RATE).expect("the sample rate is not zero");

    chirp(rate, START_HZ, END_HZ, LENGTH)
        .fade_out(LENGTH)
        .amplify(volume)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_soft_settle_makes_no_sound() {
        assert_eq!(volume(0.0), None);
        assert_eq!(volume(MIN_IMPACT - 0.1), None);
    }

    #[test]
    fn the_quietest_landing_is_still_audible() {
        let quietest = volume(MIN_IMPACT).expect("at the threshold it sounds");
        assert!(
            (quietest - MIN_VOLUME).abs() < 1e-5,
            "quietest was {}",
            quietest
        );
    }

    #[test]
    fn a_harder_landing_is_louder() {
        let gentle = volume(5.0).expect("audible");
        let hard = volume(25.0).expect("audible");
        assert!(hard > gentle, "{} should beat {}", hard, gentle);
    }

    #[test]
    fn volume_stops_at_the_cap() {
        let terminal = volume(MAX_FALL).expect("audible");
        let beyond = volume(MAX_FALL * 3.0).expect("audible");

        assert!((terminal - 1.0).abs() < 1e-5, "terminal was {}", terminal);
        assert_eq!(terminal, beyond);
    }

    #[test]
    fn the_thud_is_short_and_finite() {
        let samples: Vec<f32> = thud(1.0).collect();

        assert!(!samples.is_empty(), "it should make a sound");
        assert!(
            samples.iter().all(|s| s.abs() <= 1.0),
            "nothing should clip"
        );

        // fading out means it ends quieter than it starts
        let first = samples[..200].iter().fold(0.0f32, |a, s| a.max(s.abs()));
        let last = samples[samples.len() - 200..]
            .iter()
            .fold(0.0f32, |a, s| a.max(s.abs()));
        assert!(last < first, "it should decay: {} then {}", first, last);
    }

    #[test]
    fn a_quieter_thud_is_quieter() {
        let loud: Vec<f32> = thud(1.0).collect();
        let soft: Vec<f32> = thud(MIN_VOLUME).collect();

        let peak = |s: &[f32]| s.iter().fold(0.0f32, |a, v| a.max(v.abs()));
        assert!(peak(&soft) < peak(&loud));
    }
}
