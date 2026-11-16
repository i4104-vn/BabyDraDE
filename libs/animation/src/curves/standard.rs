use std::f64::consts::PI;

#[inline]
pub fn linear(t: f64) -> f64 {
    t
}

#[inline]
pub fn ease_in_quad(t: f64) -> f64 {
    t * t
}

#[inline]
pub fn ease_out_quad(t: f64) -> f64 {
    let p = 1.0 - t;
    1.0 - p * p
}

#[inline]
pub fn ease_in_out_quad(t: f64) -> f64 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        let p = 1.0 - t;
        1.0 - 2.0 * p * p
    }
}

#[inline]
pub fn ease_in_cubic(t: f64) -> f64 {
    t * t * t
}

#[inline]
pub fn ease_out_cubic(t: f64) -> f64 {
    let p = 1.0 - t;
    1.0 - p * p * p
}

#[inline]
pub fn ease_in_out_cubic(t: f64) -> f64 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let p = 1.0 - t;
        1.0 - 4.0 * p * p * p
    }
}

#[inline]
pub fn ease_in_sine(t: f64) -> f64 {
    1.0 - (t * (PI / 2.0)).cos()
}

#[inline]
pub fn ease_out_sine(t: f64) -> f64 {
    (t * (PI / 2.0)).sin()
}

#[inline]
pub fn ease_in_out_sine(t: f64) -> f64 {
    0.5 * (1.0 - (PI * t).cos())
}

#[inline]
pub fn ease_in_quart(t: f64) -> f64 {
    let t2 = t * t;
    t2 * t2
}

#[inline]
pub fn ease_out_quart(t: f64) -> f64 {
    let p = 1.0 - t;
    let p2 = p * p;
    1.0 - p2 * p2
}

#[inline]
pub fn ease_in_out_quart(t: f64) -> f64 {
    if t < 0.5 {
        let t2 = t * t;
        8.0 * t2 * t2
    } else {
        let p = 1.0 - t;
        let p2 = p * p;
        1.0 - 8.0 * p2 * p2
    }
}

#[inline]
pub fn ease_in_quint(t: f64) -> f64 {
    let t2 = t * t;
    t2 * t2 * t
}

#[inline]
pub fn ease_out_quint(t: f64) -> f64 {
    let p = 1.0 - t;
    let p2 = p * p;
    1.0 - p2 * p2 * p
}

#[inline]
pub fn ease_in_out_quint(t: f64) -> f64 {
    if t < 0.5 {
        let t2 = t * t;
        16.0 * t2 * t2 * t
    } else {
        let p = 1.0 - t;
        let p2 = p * p;
        1.0 - 16.0 * p2 * p2 * p
    }
}

#[inline]
pub fn ease_in_expo(t: f64) -> f64 {
    if t == 0.0 {
        0.0
    } else {
        (2.0f64).powf(10.0 * (t - 1.0))
    }
}

#[inline]
pub fn ease_out_expo(t: f64) -> f64 {
    if t == 1.0 {
        1.0
    } else {
        1.0 - (2.0f64).powf(-10.0 * t)
    }
}

#[inline]
pub fn ease_in_out_expo(t: f64) -> f64 {
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else if t < 0.5 {
        (2.0f64).powf(20.0 * t - 10.0) * 0.5
    } else {
        1.0 - (2.0f64).powf(-20.0 * t + 10.0) * 0.5
    }
}

#[inline]
pub fn ease_in_circ(t: f64) -> f64 {
    1.0 - (1.0 - t * t).max(0.0).sqrt()
}

#[inline]
pub fn ease_out_circ(t: f64) -> f64 {
    let p = 1.0 - t;
    (1.0 - p * p).max(0.0).sqrt()
}

#[inline]
pub fn ease_in_out_circ(t: f64) -> f64 {
    if t < 0.5 {
        0.5 * (1.0 - (1.0 - 4.0 * t * t).max(0.0).sqrt())
    } else {
        let p = 1.0 - t;
        0.5 * ((1.0 - 4.0 * p * p).max(0.0).sqrt() + 1.0)
    }
}
