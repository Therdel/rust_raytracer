#[cfg(test)]
pub fn assert_approx_eq(a: f32, b: f32) {
    float_eq::assert_float_eq!(a, b, rmax <= 2.0 * f32::EPSILON)
}