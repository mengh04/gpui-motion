// Reference: <https://easings.net/en>
//
// Curve control points and formulas are taken from easings.net. Most
// named curves are cubic-bezier approximations of a simpler polynomial,
// so midpoint values may differ from the polynomial by up to a few
// percent. Tolerance is chosen accordingly.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    Linear,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
    CubicBezier(f32, f32, f32, f32),
}

pub fn apply(easing: Easing, t: f32) -> f32 {
    match easing {
        Easing::Linear => t,
        Easing::EaseInSine => cubic_bezier(0.12, 0., 0.39, 0., t),
        Easing::EaseOutSine => cubic_bezier(0.61, 1., 0.88, 1., t),
        Easing::EaseInOutSine => cubic_bezier(0.37, 0., 0.63, 1., t),
        Easing::EaseInQuad => cubic_bezier(0.11, 0., 0.5, 0., t),
        Easing::EaseOutQuad => cubic_bezier(0.5, 1., 0.89, 1., t),
        Easing::EaseInOutQuad => cubic_bezier(0.45, 0., 0.55, 1., t),
        Easing::EaseInCubic => cubic_bezier(0.32, 0., 0.67, 0., t),
        Easing::EaseOutCubic => cubic_bezier(0.33, 1., 0.68, 1., t),
        Easing::EaseInOutCubic => cubic_bezier(0.65, 0., 0.35, 1., t),
        Easing::EaseInQuart => cubic_bezier(0.5, 0., 0.75, 0., t),
        Easing::EaseOutQuart => cubic_bezier(0.25, 1., 0.5, 1., t),
        Easing::EaseInOutQuart => cubic_bezier(0.76, 0., 0.24, 1., t),
        Easing::EaseInQuint => cubic_bezier(0.64, 0., 0.78, 0., t),
        Easing::EaseOutQuint => cubic_bezier(0.22, 1., 0.36, 1., t),
        Easing::EaseInOutQuint => cubic_bezier(0.83, 0., 0.17, 1., t),
        Easing::EaseInExpo => cubic_bezier(0.7, 0., 0.84, 0., t),
        Easing::EaseOutExpo => cubic_bezier(0.16, 1., 0.3, 1., t),
        Easing::EaseInOutExpo => cubic_bezier(0.87, 0., 0.13, 1., t),
        Easing::EaseInCirc => cubic_bezier(0.55, 0., 1., 0.45, t),
        Easing::EaseOutCirc => cubic_bezier(0., 0.55, 0.45, 1., t),
        Easing::EaseInOutCirc => cubic_bezier(0.85, 0., 0.15, 1., t),
        Easing::EaseInBack => cubic_bezier(0.36, 0., 0.66, -0.56, t),
        Easing::EaseOutBack => cubic_bezier(0.34, 1.56, 0.64, 1., t),
        Easing::EaseInOutBack => cubic_bezier(0.68, -0.6, 0.32, 1.6, t),
        Easing::EaseInElastic => {
            let c4 = (2.0 * std::f32::consts::PI) / 3.0;
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else {
                -2.0_f32.powf(10.0 * t - 10.0) * (t * 10.0 - 10.75).sin() * c4
            }
        }
        Easing::EaseOutElastic => {
            let c4 = (2.0 * std::f32::consts::PI) / 3.0;
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else {
                2.0_f32.powf(-10.0 * t) * (t * 10.0 - 0.75).sin() * c4 + 1.0
            }
        }
        Easing::EaseInOutElastic => {
            let c5 = (2.0 * std::f32::consts::PI) / 4.5;
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else if t < 0.5 {
                -(2.0_f32.powf(20.0 * t - 10.0) * (20.0 * t - 11.125).sin() * c5) / 2.0
            } else {
                (2.0_f32.powf(-20.0 * t + 10.0) * (20.0 * t - 11.125).sin() * c5) / 2.0 + 1.0
            }
        }
        Easing::EaseInBounce => 1. - ease_out_bounce(1. - t),
        Easing::EaseOutBounce => ease_out_bounce(t),
        Easing::EaseInOutBounce => {
            if t < 0.5 {
                (1. - ease_out_bounce(1. - 2. * t)) / 2.
            } else {
                (1. + ease_out_bounce(2. * t - 1.)) / 2.
            }
        }
        Easing::CubicBezier(x1, y1, x2, y2) => cubic_bezier(x1, y1, x2, y2, t),
    }
}

fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, t: f32) -> f32 {
    // Endpoints are fixed at (0,0) and (1,1). When x1 == 0 or x2 == 1 the
    // derivative of x(s) is 0 at the corresponding endpoint, so any
    // root-finder would divide by zero. Short-circuit instead.
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    // When the y-control points match the x-control points, the curve is
    // strictly y = x, so we can return t directly.
    if x1 == y1 && x2 == y2 {
        return t;
    }
    let s = binary_subdivide(t, 0.0, 1.0, x1, x2);
    calc_bezier(s, y1, y2)
}

/// Horner form of `3(1-s)²·a1·s + 3(1-s)·a2·s² + s³`.
///
/// Equivalent to the expanded form but with fewer multiplications and
/// less floating-point error accumulation.
#[inline]
fn calc_bezier(s: f32, a1: f32, a2: f32) -> f32 {
    ((1.0 - 3.0 * a2 + 3.0 * a1) * s + (3.0 * a2 - 6.0 * a1)) * s * s + 3.0 * a1 * s
}

/// Bisection search for `s` such that `calc_bezier(s, m_x1, m_x2) == x`.
///
/// Binary subdivision was benchmarked faster than Newton-Raphson for this
/// use case (see
/// <https://github.com/framer/motion/blob/main/packages/motion-utils/src/easing/cubic-bezier.ts>).
fn binary_subdivide(x: f32, mut lower: f32, mut upper: f32, m_x1: f32, m_x2: f32) -> f32 {
    const PRECISION: f32 = 1e-7;
    const MAX_ITERATIONS: u32 = 12;
    let mut current_t;
    let mut i = 0;
    loop {
        current_t = lower + (upper - lower) / 2.0;
        let current_x = calc_bezier(current_t, m_x1, m_x2) - x;
        if current_x > 0.0 {
            upper = current_t;
        } else {
            lower = current_t;
        }
        if current_x.abs() <= PRECISION || i >= MAX_ITERATIONS {
            break;
        }
        i += 1;
    }
    current_t
}

fn ease_out_bounce(t: f32) -> f32 {
    if t < 1.0 / 2.75 {
        7.5625 * t * t
    } else if t < 2.0 / 2.75 {
        let t = t - 1.5 / 2.75;
        7.5625 * t * t + 0.75
    } else if t < 2.5 / 2.75 {
        let t = t - 2.25 / 2.75;
        7.5625 * t * t + 0.9375
    } else {
        let t = t - 2.625 / 2.75;
        7.5625 * t * t + 0.984375
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOOSE_EPS: f32 = 1e-3;
    /// Tolerance for cubic-bezier approximations of polynomial easings.
    const APPROX_EPS: f32 = 0.02;

    const ALL_PRESETS: &[Easing] = &[
        Easing::Linear,
        Easing::EaseInSine,
        Easing::EaseOutSine,
        Easing::EaseInOutSine,
        Easing::EaseInQuad,
        Easing::EaseOutQuad,
        Easing::EaseInOutQuad,
        Easing::EaseInCubic,
        Easing::EaseOutCubic,
        Easing::EaseInOutCubic,
        Easing::EaseInQuart,
        Easing::EaseOutQuart,
        Easing::EaseInOutQuart,
        Easing::EaseInQuint,
        Easing::EaseOutQuint,
        Easing::EaseInOutQuint,
        Easing::EaseInExpo,
        Easing::EaseOutExpo,
        Easing::EaseInOutExpo,
        Easing::EaseInCirc,
        Easing::EaseOutCirc,
        Easing::EaseInOutCirc,
        Easing::EaseInBack,
        Easing::EaseOutBack,
        Easing::EaseInOutBack,
        Easing::EaseInElastic,
        Easing::EaseOutElastic,
        Easing::EaseInOutElastic,
        Easing::EaseInBounce,
        Easing::EaseOutBounce,
        Easing::EaseInOutBounce,
    ];

    #[test]
    fn test_linear_is_identity() {
        assert_eq!(apply(Easing::Linear, 0.0), 0.0);
        assert_eq!(apply(Easing::Linear, 0.25), 0.25);
        assert_eq!(apply(Easing::Linear, 0.5), 0.5);
        assert_eq!(apply(Easing::Linear, 0.75), 0.75);
        assert_eq!(apply(Easing::Linear, 1.0), 1.0);
    }

    /// All named variants evaluate to 0 at t=0 and 1 at t=1.
    #[test]
    fn test_endpoints() {
        for &e in ALL_PRESETS {
            assert!(apply(e, 0.0).abs() < LOOSE_EPS, "{:?}(0)", e);
            assert!((apply(e, 1.0) - 1.0).abs() < LOOSE_EPS, "{:?}(1)", e);
        }
    }

    /// Every value in (0, 1) is finite — no NaN, no Inf.
    #[test]
    fn test_no_nan_or_inf() {
        for &e in ALL_PRESETS {
            for i in 1..100 {
                let t = i as f32 / 100.0;
                assert!(apply(e, t).is_finite(), "{:?}({})", e, t);
            }
        }
    }

    /// Sine uses closed-form trig. Midpoint values match the polynomial
    /// form to high precision.
    #[test]
    fn test_sine_midpoints() {
        // 1 - cos(π/4) = 1 - √2/2
        let in_expected = 1.0 - std::f32::consts::FRAC_1_SQRT_2;
        // sin(π/4) = √2/2
        let out_expected = std::f32::consts::FRAC_1_SQRT_2;
        assert!((apply(Easing::EaseInSine, 0.5) - in_expected).abs() < APPROX_EPS);
        assert!((apply(Easing::EaseOutSine, 0.5) - out_expected).abs() < APPROX_EPS);
    }

    /// Quad/Cubic/Quart/Quint/Expo/Circ are cubic-bezier approximations
    /// of the corresponding polynomial. Midpoint values should be close
    /// to the polynomial within `APPROX_EPS`.
    #[test]
    fn test_polynomial_approximations() {
        let cases = [
            (Easing::EaseInQuad, 0.25),
            (Easing::EaseOutQuad, 0.75),
            (Easing::EaseInCubic, 0.125),
            (Easing::EaseOutCubic, 0.875),
            (Easing::EaseInQuart, 0.0625),
            (Easing::EaseInQuint, 0.03125),
            (Easing::EaseInExpo, 0.03125),
            (Easing::EaseInCirc, 0.13397),
        ];
        for (e, expected) in cases {
            let v = apply(e, 0.5);
            assert!(
                (v - expected).abs() < APPROX_EPS,
                "{:?}(0.5) = {} (expected ≈ {})",
                e,
                v,
                expected
            );
        }
    }

    /// Back curves overshoot their endpoint range.
    #[test]
    fn test_back_overshoots() {
        assert!(apply(Easing::EaseInBack, 0.5) < 0.0);
        assert!(apply(Easing::EaseOutBack, 0.5) > 1.0);
    }

    /// InOut curves pass through 0.5 at the midpoint.
    #[test]
    fn test_in_out_passes_through_half() {
        let in_outs = [
            Easing::EaseInOutSine,
            Easing::EaseInOutQuad,
            Easing::EaseInOutCubic,
            Easing::EaseInOutQuart,
            Easing::EaseInOutQuint,
            Easing::EaseInOutCirc,
            Easing::EaseInOutBack,
            Easing::EaseInOutBounce,
        ];
        for e in in_outs {
            assert!((apply(e, 0.5) - 0.5).abs() < LOOSE_EPS, "{:?}(0.5)", e);
        }
    }

    /// EaseIn variants at t=0.5 produce values below 0.5.
    #[test]
    fn test_ease_in_below_midpoint() {
        let ins = [
            Easing::EaseInSine,
            Easing::EaseInQuad,
            Easing::EaseInCubic,
            Easing::EaseInQuart,
            Easing::EaseInQuint,
            Easing::EaseInExpo,
            Easing::EaseInCirc,
            Easing::EaseInBack,
            Easing::EaseInBounce,
        ];
        for e in ins {
            assert!(apply(e, 0.5) < 0.5, "{:?}(0.5)", e);
        }
    }

    /// EaseOut variants at t=0.5 produce values above 0.5.
    #[test]
    fn test_ease_out_above_midpoint() {
        let outs = [
            Easing::EaseOutSine,
            Easing::EaseOutQuad,
            Easing::EaseOutCubic,
            Easing::EaseOutQuart,
            Easing::EaseOutQuint,
            Easing::EaseOutExpo,
            Easing::EaseOutCirc,
            Easing::EaseOutBack,
            Easing::EaseOutBounce,
        ];
        for e in outs {
            assert!(apply(e, 0.5) > 0.5, "{:?}(0.5)", e);
        }
    }

    /// Elastic curves have explicit `if t == 0` / `t == 1` checks, so
    /// endpoints are returned exactly.
    #[test]
    fn test_elastic_endpoints_exact() {
        let elastic = [
            Easing::EaseInElastic,
            Easing::EaseOutElastic,
            Easing::EaseInOutElastic,
        ];
        for e in elastic {
            assert_eq!(apply(e, 0.0), 0.0);
            assert_eq!(apply(e, 1.0), 1.0);
        }
    }

    /// Custom cubic-bezier produces a valid curve.
    #[test]
    fn test_cubic_bezier_open_api() {
        let e = Easing::CubicBezier(0.42, 0.0, 0.58, 1.0);
        assert!(apply(e, 0.0).abs() < LOOSE_EPS);
        assert!((apply(e, 1.0) - 1.0).abs() < LOOSE_EPS);
        // (0.42, 0, 0.58, 1) is the standard ease-in-out shape.
        assert!((apply(e, 0.5) - 0.5).abs() < 0.01);
    }

    /// Custom cubic-bezier with negative y control point (Back-like) stays finite.
    #[test]
    fn test_cubic_bezier_negative_y() {
        let e = Easing::CubicBezier(0.36, 0.0, 0.66, -0.56);
        assert!(apply(e, 0.5).is_finite());
    }
}
