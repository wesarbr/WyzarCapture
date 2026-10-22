//! Packet-continuous windowed-sinc resampling. Native 48 kHz is unchanged.
//! Surround downmix retains the centre channel instead of dropping all but L/R.
pub struct Resampler {
    channels: usize,
    rate: u32,
    mask: u32,
    frames: Vec<[f32; 2]>,
    position: f64,
    kernel: Vec<[f32; 32]>,
}
impl Resampler {
    pub fn new(channels: usize, rate: u32, mask: u32) -> Self {
        let cutoff = (48000. / rate.max(1) as f64).min(1.) * 0.94;
        let mut kernel = vec![[0f32; 32]; 256];
        for (phase, taps) in kernel.iter_mut().enumerate() {
            for (i, tap) in taps.iter_mut().enumerate() {
                let x = i as f64 - 15. - phase as f64 / 256.;
                let a = std::f64::consts::PI * x * cutoff;
                let sinc = if a.abs() < 1e-12 { 1. } else { a.sin() / a };
                let window = 0.5 + 0.5 * (std::f64::consts::PI * x / 16.).cos();
                *tap = (sinc * window * cutoff) as f32;
            }
            let sum = taps.iter().sum::<f32>();
            for tap in taps {
                *tap /= sum;
            }
        }
        Self {
            channels,
            rate,
            mask,
            frames: vec![[0.; 2]; 16],
            position: 16.,
            kernel,
        }
    }
    fn stereo(&self, frame: &[f32]) -> [f32; 2] {
        let value = |i: usize| {
            frame
                .get(i)
                .copied()
                .filter(|v| v.is_finite())
                .unwrap_or(0.)
                .clamp(-1., 1.)
        };
        if self.channels <= 2 {
            return [value(0), value(if self.channels == 1 { 0 } else { 1 })];
        }
        let mask = if self.mask != 0 {
            self.mask
        } else {
            match self.channels {
                6 => 0x3f,
                8 => 0x63f,
                _ => (1u32 << self.channels.min(31)) - 1,
            }
        };
        let (mut out, mut weight) = ([0f32; 2], [0f32; 2]);
        let mut channel = 0;
        for bit in 0..32 {
            if mask & (1 << bit) == 0 {
                continue;
            }
            let w = match bit {
                0 => [1., 0.],
                1 => [0., 1.],
                2 => [0.7071, 0.7071],
                3 => [0.3162, 0.3162],
                4 | 6 | 9 => [0.7071, 0.],
                5 | 7 | 10 => [0., 0.7071],
                _ => [0.5, 0.5],
            };
            for c in 0..2 {
                out[c] += value(channel) * w[c];
                weight[c] += w[c];
            }
            channel += 1;
            if channel >= self.channels {
                break;
            }
        }
        [out[0] / weight[0].max(1.), out[1] / weight[1].max(1.)]
    }
    pub fn push(&mut self, input: &[f32]) -> Vec<f32> {
        if self.channels == 0 || self.rate == 0 {
            return vec![];
        }
        let stereo: Vec<_> = input
            .chunks_exact(self.channels)
            .map(|f| self.stereo(f))
            .collect();
        if self.rate == 48000 {
            return stereo.into_iter().flatten().collect();
        }
        self.frames.extend(stereo);
        let mut out = Vec::new();
        while self.position.floor() as usize + 17 < self.frames.len() {
            let index = self.position.floor() as usize;
            let phase = ((self.position.fract() * 256.) as usize).min(255);
            for c in 0..2 {
                out.push(
                    (0..32)
                        .map(|tap| self.frames[index - 15 + tap][c] * self.kernel[phase][tap])
                        .sum(),
                );
            }
            self.position += self.rate as f64 / 48000.;
        }
        let discard = (self.position.floor() as usize)
            .saturating_sub(16)
            .min(self.frames.len().saturating_sub(16));
        self.frames.drain(..discard);
        self.position -= discard as f64;
        out
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn packet_boundaries_preserve_phase_and_surround_dialogue() {
        let signal: Vec<f32> = (0..44100).map(|n| (n as f32 * 0.1).sin() * 0.5).collect();
        let mut full = Resampler::new(1, 44100, 0);
        let expected = full.push(&signal);
        let mut chunked = Resampler::new(1, 44100, 0);
        let actual: Vec<_> = signal.chunks(137).flat_map(|c| chunked.push(c)).collect();
        assert_eq!(expected.len(), actual.len());
        assert!(
            actual
                .iter()
                .zip(expected)
                .all(|(a, b)| (*a - b).abs() < 0.001)
        );
        assert!((actual.len() as i64 / 2 - 48000).abs() < 20);
        let mut surround = Resampler::new(6, 48000, 0x3f);
        let dialogue = surround.push(&[0., 0., 1., 0., 0., 0.]);
        assert!(dialogue[0] > 0.2);
        assert_eq!(dialogue[0], dialogue[1]);
    }
    #[test]
    fn downsampling_rejects_alias_energy() {
        let tone = |hz: f64| {
            (0..9600)
                .map(|n| ((n as f64 * hz * 2. * std::f64::consts::PI / 96000.).sin() * 0.5) as f32)
                .collect::<Vec<_>>()
        };
        let rms = |x: Vec<f32>| {
            (x.iter().skip(128).map(|x| x * x).sum::<f32>() / (x.len() - 128) as f32).sqrt()
        };
        let pass = rms(Resampler::new(1, 96000, 0).push(&tone(1000.)));
        let stop = rms(Resampler::new(1, 96000, 0).push(&tone(36000.)));
        assert!(pass > 0.3 && stop < pass * 0.03);
    }
}
