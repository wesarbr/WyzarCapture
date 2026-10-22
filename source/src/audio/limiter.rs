//! Peak protection without changing samples below the ceiling. Attack uses the
//! already available block; release is gradual. Raw source sidecars stay untouched.
#[derive(Default)]
pub struct Limiter {
    gain: f32,
}
impl Limiter {
    pub fn new() -> Self {
        Self { gain: 1. }
    }
    pub fn mix(&mut self, a: &[f32], b: &[f32], gain_a: f32, gain_b: f32) -> Vec<i16> {
        let raw: Vec<f32> = (0..a.len().max(b.len()))
            .map(|i| {
                let value = a.get(i).copied().unwrap_or(0.) * gain_a
                    + b.get(i).copied().unwrap_or(0.) * gain_b;
                if value.is_finite() { value } else { 0. }
            })
            .collect();
        let peak = raw.iter().fold(0f32, |p, x| p.max(x.abs()));
        let ceiling = if peak > 0.98 { 0.98 / peak } else { 1. };
        self.gain = self.gain.min(ceiling);
        raw.into_iter()
            .map(|v| {
                self.gain = (self.gain + 1. / (48000. * 2. * 0.15)).min(ceiling);
                (v * self.gain * 32767.).clamp(-32767., 32767.) as i16
            })
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_quiet_signal_and_limits_overload() {
        let mut l = Limiter::new();
        assert_eq!(l.mix(&[0.25, -0.25], &[], 1., 0.), vec![8191, -8191]);
        let x = l.mix(&[1., -1.], &[1., -1.], 2., 2.);
        assert!(x.iter().all(|v| v.abs() < 32200));
        let y = l.mix(&[0.1; 100], &[], 1., 0.);
        assert!(y.windows(2).all(|p| p[1] >= p[0]));
    }
}
