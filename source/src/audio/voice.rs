//! Microphone clean-up for interleaved stereo at 48 kHz: a second-order high-pass
//! at 80 Hz removes rumble, desk bumps and fan hum, and a gentle downward expander
//! lowers steady background hiss between phrases by at most 12 dB. Speech above the
//! threshold passes unchanged; nothing is gated to silence.
const RATE: f32 = super::RATE as f32;
#[derive(Clone, Copy, Default)]
struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}
impl Biquad {
    fn high_pass(cutoff: f32) -> Self {
        let w = std::f32::consts::TAU * cutoff / RATE;
        let alpha = w.sin() / (2. * std::f32::consts::FRAC_1_SQRT_2);
        let cos = w.cos();
        let a0 = 1. + alpha;
        Self {
            b0: (1. + cos) / 2. / a0,
            b1: -(1. + cos) / a0,
            b2: (1. + cos) / 2. / a0,
            a1: -2. * cos / a0,
            a2: (1. - alpha) / a0,
            ..Default::default()
        }
    }
    fn run(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        // Flush denormals so long silences cost nothing.
        self.y1 = if y.abs() < 1e-20 { 0. } else { y };
        self.y1
    }
}
pub struct VoiceFilter {
    filters: [Biquad; 2],
    envelope: f32,
    gain: f32,
    hold: u32,
}
impl Default for VoiceFilter {
    fn default() -> Self {
        Self::new()
    }
}
impl VoiceFilter {
    const THRESHOLD: f32 = 0.006; // about -44 dBFS
    const FLOOR: f32 = 0.25; // -12 dB at most
    const HOLD: u32 = (RATE * 0.12) as u32;
    pub fn new() -> Self {
        Self {
            filters: [Biquad::high_pass(80.); 2],
            envelope: 0.,
            gain: 1.,
            hold: 0,
        }
    }
    /// Process interleaved stereo samples in place.
    pub fn process(&mut self, samples: &mut [f32]) {
        let attack = 1. - (-1. / (RATE * 0.004)).exp();
        let release = 1. - (-1. / (RATE * 0.15)).exp();
        let smooth_open = 1. - (-1. / (RATE * 0.005)).exp();
        let smooth_close = 1. - (-1. / (RATE * 0.08)).exp();
        for frame in samples.chunks_exact_mut(2) {
            let left = self.filters[0].run(frame[0]);
            let right = self.filters[1].run(frame[1]);
            let level = left.abs().max(right.abs());
            let k = if level > self.envelope {
                attack
            } else {
                release
            };
            self.envelope += (level - self.envelope) * k;
            let target = if self.envelope >= Self::THRESHOLD {
                self.hold = Self::HOLD;
                1.
            } else if self.hold > 0 {
                self.hold -= 1;
                1.
            } else {
                // 2:1 downward expansion below the threshold.
                (self.envelope / Self::THRESHOLD).clamp(Self::FLOOR, 1.)
            };
            let k = if target > self.gain {
                smooth_open
            } else {
                smooth_close
            };
            self.gain += (target - self.gain) * k;
            frame[0] = left * self.gain;
            frame[1] = right * self.gain;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn tone(frequency: f32, amplitude: f32, seconds: f32) -> Vec<f32> {
        (0..(RATE * seconds) as usize)
            .flat_map(|n| {
                let v = amplitude * (std::f32::consts::TAU * frequency * n as f32 / RATE).sin();
                [v, v]
            })
            .collect()
    }
    fn peak(samples: &[f32]) -> f32 {
        samples.iter().fold(0f32, |a, v| a.max(v.abs()))
    }
    #[test]
    fn rumble_is_removed_and_voice_band_passes() {
        let mut filter = VoiceFilter::new();
        let mut hum = tone(30., 0.5, 1.);
        filter.process(&mut hum);
        assert!(
            peak(&hum[hum.len() / 2..]) < 0.1,
            "30 Hz should be strongly reduced"
        );
        let mut filter = VoiceFilter::new();
        let mut voice = tone(440., 0.3, 1.);
        filter.process(&mut voice);
        let tail = peak(&voice[voice.len() / 2..]);
        assert!(
            (0.27..=0.33).contains(&tail),
            "speech level preserved: {tail}"
        );
    }
    #[test]
    fn quiet_hiss_is_lowered_by_at_most_twelve_db() {
        let mut filter = VoiceFilter::new();
        let mut hiss = tone(3000., 0.002, 2.);
        filter.process(&mut hiss);
        let tail = peak(&hiss[hiss.len() * 3 / 4..]);
        assert!(tail < 0.0015 && tail >= 0.002 * 0.24, "{tail}");
        let mut silence = vec![0.; 9600];
        filter.process(&mut silence);
        assert!(silence.iter().all(|v| v.is_finite() && v.abs() < 0.001));
        assert!(
            peak(&silence[silence.len() - 200..]) < 1e-6,
            "filter tail settles"
        );
    }
}
