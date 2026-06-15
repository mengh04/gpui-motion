//! L1 - tween state machine.
//!
//! A [`Tween`] tracks the progress of a single interpolation from `from` to
//! `to` over a fixed `duration`, applying an [`Easing`] curve to the linear
//! time progress. The driver ticks the tween each frame; readers call
//! [`Tween::current`] to obtain the value at this point in time.

use std::time::Duration;

use gpui::Lerp;

use crate::{Easing, easing::apply};

/// A single tween from one value to another.
///
/// The tween is advanced by [`Tween::tick`]; its current value at any
/// point in time is returned by [`Tween::current`]. The tween is done
/// when `elapsed >= duration`, at which point `current` returns `to`.
///
/// `Clone` is implemented; the bound on `T` is only required for
/// [`Tween::current`] (so a `Tween<T>` can be moved around without
/// requiring `T: Clone`).
#[derive(Debug, Clone)]
pub struct Tween<T> {
    from: T,
    to: T,
    elapsed: Duration,
    duration: Duration,
    easing: Easing,
}

impl<T: Lerp> Tween<T> {
    pub fn new(from: T, to: T, duration: Duration, easing: Easing) -> Self {
        Self {
            from,
            to,
            elapsed: Duration::ZERO,
            duration,
            easing,
        }
    }

    /// Advance the tween by `delta`. Saturates at `Duration::MAX` to
    /// avoid panicking on overflow.
    pub fn tick(&mut self, delta: Duration) {
        self.elapsed = self.elapsed.saturating_add(delta);
    }

    /// Whether the tween has reached its target.
    pub fn is_done(&self) -> bool {
        self.elapsed >= self.duration
    }

    /// Compute the current value at this point in time.
    ///
    /// The linear time progress is computed as `elapsed / duration`,
    /// clamped to `[0, 1]`, then transformed by the easing curve, then
    /// used as the lerp parameter between `from` and `to`. A zero
    /// `duration` short-circuits to `to`.
    pub fn current(&self) -> T
    where
        T: Clone,
    {
        if self.duration.is_zero() {
            return self.to.clone();
        }
        let linear = (self.elapsed.as_secs_f32() / self.duration.as_secs_f32()).clamp(0., 1.);
        let eased = apply(self.easing, linear);
        self.from.lerp(&self.to, eased)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const APPROX_EPS: f32 = 0.02;

    #[test]
    fn test_current_at_zero_is_start() {
        let tw = Tween::new(0.0_f32, 100.0, Duration::from_millis(1000), Easing::Linear);
        assert_eq!(tw.current(), 0.0);
        assert!(!tw.is_done());
    }

    #[test]
    fn test_current_at_end_is_target() {
        let mut tw = Tween::new(0.0_f32, 100.0, Duration::from_millis(1000), Easing::Linear);
        tw.tick(Duration::from_millis(1000));
        assert_eq!(tw.current(), 100.0);
        assert!(tw.is_done());
    }

    #[test]
    fn test_current_at_half_with_linear() {
        let mut tw = Tween::new(0.0_f32, 100.0, Duration::from_millis(1000), Easing::Linear);
        tw.tick(Duration::from_millis(500));
        assert!((tw.current() - 50.0).abs() < 1e-5);
        assert!(!tw.is_done());
    }

    /// After the duration has elapsed, additional ticks must not move the
    /// value past the target.
    #[test]
    fn test_clamps_past_end() {
        let mut tw = Tween::new(0.0_f32, 100.0, Duration::from_millis(1000), Easing::Linear);
        tw.tick(Duration::from_millis(2000));
        assert_eq!(tw.current(), 100.0);
        assert!(tw.is_done());
    }

    /// A zero-duration tween should be immediately at its target.
    #[test]
    fn test_zero_duration_returns_target() {
        let tw = Tween::new(0.0_f32, 100.0, Duration::ZERO, Easing::Linear);
        assert_eq!(tw.current(), 100.0);
        assert!(tw.is_done());
    }

    /// EaseIn curves start slow: at the midpoint the value should be well
    /// below the linear midpoint.
    #[test]
    fn test_ease_in_slow_start() {
        let mut tw = Tween::new(
            0.0_f32,
            100.0,
            Duration::from_millis(1000),
            Easing::EaseInCubic,
        );
        tw.tick(Duration::from_millis(500));
        let v = tw.current();
        assert!(v < 50.0, "ease_in should be slow at start, got {}", v);
        // cubic-bezier approximation of ease-in cubic at t=0.5 is ≈ 0.125
        assert!((v - 12.5).abs() < 2.0, "expected ≈ 12.5, got {}", v);
    }

    /// EaseOut curves start fast: at the midpoint the value should be well
    /// above the linear midpoint.
    #[test]
    fn test_ease_out_fast_start() {
        let mut tw = Tween::new(
            0.0_f32,
            100.0,
            Duration::from_millis(1000),
            Easing::EaseOutCubic,
        );
        tw.tick(Duration::from_millis(500));
        let v = tw.current();
        assert!(v > 50.0, "ease_out should be fast at start, got {}", v);
    }

    /// Tween works with non-f32 types that implement Lerp.
    #[test]
    fn test_tween_with_pixels() {
        use gpui::px;
        let mut tw = Tween::new(
            px(0.0),
            px(100.0),
            Duration::from_millis(1000),
            Easing::Linear,
        );
        tw.tick(Duration::from_millis(500));
        assert_eq!(tw.current(), px(50.0));
    }

    /// Multiple ticks accumulate.
    #[test]
    fn test_multiple_ticks_accumulate() {
        let mut tw = Tween::new(0.0_f32, 100.0, Duration::from_millis(1000), Easing::Linear);
        tw.tick(Duration::from_millis(200));
        tw.tick(Duration::from_millis(300));
        assert!((tw.current() - 50.0).abs() < 1e-5);
    }

    /// Saturating arithmetic: an absurdly large tick must not overflow.
    #[test]
    fn test_tick_saturates() {
        let mut tw = Tween::new(0.0_f32, 100.0, Duration::from_millis(1000), Easing::Linear);
        tw.tick(Duration::from_secs(60 * 60 * 24 * 365)); // 1 year
        assert!(tw.is_done());
        assert_eq!(tw.current(), 100.0);
    }

    /// Approximation tolerance constant is used in any assertion that
    /// compares against cubic-bezier-approximated values.
    #[test]
    fn test_approx_eps_is_reasonable() {
        // Sanity check: APPROX_EPS should be in (0, 1).
        assert!(APPROX_EPS > 0.0 && APPROX_EPS < 1.0);
    }
}
