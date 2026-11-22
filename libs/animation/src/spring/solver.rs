use super::config::SpringConfig;

/// Solves harmonic oscillator displacement and velocity analytically.
#[inline]
pub fn solve_spring(config: &SpringConfig, y0: f64, v0: f64, elapsed: f64) -> (f64, f64) {
    let k = config.stiffness;
    let m = config.mass;
    let c = config.damping;

    let omega0 = (k / m).sqrt();
    let zeta = c / (2.0 * (k * m).sqrt());

    match true {
        _ if zeta < 0.9999 => {
            // Underdamped (oscillatory)
            let omega_d = omega0 * (1.0 - zeta * zeta).sqrt();
            let c1 = y0;
            let c2 = (v0 + zeta * omega0 * y0) / omega_d;

            let decay = (-zeta * omega0 * elapsed).exp();
            let sin_dt = (omega_d * elapsed).sin();
            let cos_dt = (omega_d * elapsed).cos();

            let y = decay * (c1 * cos_dt + c2 * sin_dt);
            let v = decay
                * ((c2 * omega_d - c1 * zeta * omega0) * cos_dt
                    - (c1 * omega_d + c2 * zeta * omega0) * sin_dt);
            (y, v)
        }
        _ if zeta <= 1.0001 => {
            // Critically damped
            let c1 = y0;
            let c2 = v0 + omega0 * y0;
            let decay = (-omega0 * elapsed).exp();

            let y = decay * (c1 + c2 * elapsed);
            let v = decay * (c2 - omega0 * (c1 + c2 * elapsed));
            (y, v)
        }
        _ => {
            // Overdamped
            let omega_d = omega0 * (zeta * zeta - 1.0).sqrt();
            let r1 = -zeta * omega0 - omega_d;
            let r2 = -zeta * omega0 + omega_d;

            let c1 = (v0 - r2 * y0) / (r1 - r2);
            let c2 = y0 - c1;

            let e1 = (r1 * elapsed).exp();
            let e2 = (r2 * elapsed).exp();

            let y = c1 * e1 + c2 * e2;
            let v = c1 * r1 * e1 + c2 * r2 * e2;
            (y, v)
        }
    }
}
