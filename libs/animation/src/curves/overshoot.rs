use std::f64::consts::PI;

const BACK_C1: f64 = 1.70158;
const BACK_C2: f64 = BACK_C1 * 1.525;
const BACK_C3: f64 = BACK_C1 + 1.0;
/// Gentler overshoot constant used by the GTK capsule transitions.
const SOFT_BACK_C1: f64 = 1.25;
const SOFT_BACK_C3: f64 = SOFT_BACK_C1 + 1.0;

/// Ease-out curve with a gentle overshoot before settling at the target.
///
/// # Arguments
/// * `t` - Normalized linear progress in `0.0..=1.0`.
///
/// # Returns
/// Eased progress, briefly above `1.0` before settling.
#[inline]
pub fn ease_out_back_soft(t: f64) -> f64 {
    1.0 + SOFT_BACK_C3 * (t - 1.0).powi(3) + SOFT_BACK_C1 * (t - 1.0).powi(2)
}

#[inline]
pub fn ease_in_back(t: f64) -> f64 {
    (BACK_C3 * t * t * t) - (BACK_C1 * t * t)
}

#[inline]
pub fn ease_out_back(t: f64) -> f64 {
    let p = 1.0 - t;
    1.0 - ((BACK_C3 * p * p * p) - (BACK_C1 * p * p))
}

#[inline]
pub fn ease_in_out_back(t: f64) -> f64 {
    if t < 0.5 {
        let t2 = 2.0 * t;
        (t2 * t2 * ((BACK_C2 + 1.0) * t2 - BACK_C2)) * 0.5
    } else {
        let p = 2.0 - 2.0 * t;
        1.0 - (p * p * ((BACK_C2 + 1.0) * p - BACK_C2)) * 0.5
    }
}

#[inline]
pub fn ease_in_elastic(t: f64) -> f64 {
    match true {
        _ if t == 0.0 => 0.0,
        _ if t == 1.0 => 1.0,
        _ => {
            let c4 = (2.0 * PI) / 3.0;
            -((2.0f64).powf(10.0 * t - 10.0)) * ((t * 10.0 - 10.75) * c4).sin()
        }
    }
}

#[inline]
pub fn ease_out_elastic(t: f64) -> f64 {
    match true {
        _ if t == 0.0 => 0.0,
        _ if t == 1.0 => 1.0,
        _ => {
            let c4 = (2.0 * PI) / 3.0;
            (2.0f64).powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
        }
    }
}

#[inline]
pub fn ease_in_out_elastic(t: f64) -> f64 {
    match true {
        _ if t == 0.0 => 0.0,
        _ if t == 1.0 => 1.0,
        _ => {
            let c5 = (2.0 * PI) / 4.5;
            if t < 0.5 {
                -0.5 * ((2.0f64).powf(20.0 * t - 10.0)) * ((20.0 * t - 11.125) * c5).sin()
            } else {
                0.5 * ((2.0f64).powf(-20.0 * t + 10.0)) * ((20.0 * t - 11.125) * c5).sin() + 1.0
            }
        }
    }
}

#[inline]
pub fn ease_out_bounce(t: f64) -> f64 {
    let n1 = 7.5625;
    let d1 = 2.75;
    match true {
        _ if t < 1.0 / d1 => n1 * t * t,
        _ if t < 2.0 / d1 => {
            let t = t - 1.5 / d1;
            n1 * t * t + 0.75
        }
        _ if t < 2.5 / d1 => {
            let t = t - 2.25 / d1;
            n1 * t * t + 0.9375
        }
        _ => {
            let t = t - 2.625 / d1;
            n1 * t * t + 0.984375
        }
    }
}

#[inline]
pub fn ease_in_bounce(t: f64) -> f64 {
    1.0 - ease_out_bounce(1.0 - t)
}

#[inline]
pub fn ease_in_out_bounce(t: f64) -> f64 {
    if t < 0.5 {
        (1.0 - ease_out_bounce(1.0 - 2.0 * t)) * 0.5
    } else {
        (1.0 + ease_out_bounce(2.0 * t - 1.0)) * 0.5
    }
}
