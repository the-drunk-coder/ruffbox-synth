#[inline(always)]
pub fn interpolate(frac: f32, y_m1: f32, y_0: f32, y_1: f32, y_2: f32, lvl: f32) -> f32 {
    // 4-point, 3rd-order Hermite
    let c0 = y_0;
    let c1 = 0.5 * (y_1 - y_m1);
    let c2 = y_m1 - 2.5 * y_0 + 2.0 * y_1 - 0.5 * y_2;
    let c3 = 0.5 * (y_2 - y_m1) + 1.5 * (y_0 - y_1);

    (((c3 * frac + c2) * frac + c1) * frac + c0) * lvl
}
