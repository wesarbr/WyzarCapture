use std::collections::VecDeque;
pub mod limiter;
pub mod recorder;
pub mod resample;
pub mod voice;
pub mod wav;
pub const RATE: u32 = 48_000;
pub const MODE_LABELS: [&str; 4] = [
    "Vídeo com áudio",
    "Vídeo sem áudio",
    "Tela + áudio separado",
    "Somente áudio · WAV",
];
pub fn thumbnail(path: &std::path::Path) -> crate::core::AppResult<crate::core::Frame> {
    let levels = waveform(path, 96).map_err(|e| e.to_string())?;
    let mut f = crate::core::Frame {
        width: 480,
        height: 270,
        bgra: vec![0; 480 * 270 * 4],
    };
    for p in f.bgra.chunks_exact_mut(4) {
        p.copy_from_slice(&[22, 18, 14, 255]);
    }
    for (n, level) in levels.into_iter().enumerate() {
        let height = (level * 170.).max(3.) as u32;
        for y in 135 - height / 2..135 + height / 2 + 1 {
            for x in n as u32 * 5 + 1..n as u32 * 5 + 4 {
                let i = ((y * 480 + x) * 4) as usize;
                f.bgra[i..i + 4].copy_from_slice(&[35, 210, 255, 255]);
            }
        }
    }
    Ok(f)
}
pub fn waveform(path: &std::path::Path, points: usize) -> std::io::Result<Vec<f32>> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = std::fs::File::open(path)?;
    let info = wav::info(path).map_err(std::io::Error::other)?;
    let data = info.bytes;
    let mut peaks = Vec::with_capacity(points);
    let mut buffer = [0u8; 1024];
    for n in 0..points {
        let pos = info.offset + ((data * n as u64 / points.max(1) as u64) & !1);
        file.seek(SeekFrom::Start(pos))?;
        let count = file.read(&mut buffer)?;
        let peak = buffer[..count]
            .chunks_exact(2)
            .map(|s| (i16::from_le_bytes([s[0], s[1]]) as f32 / 32768.).abs())
            .fold(0f32, f32::max);
        peaks.push(peak);
    }
    Ok(peaks)
}
#[derive(Default)]
pub struct AudioQueue {
    samples: VecDeque<f32>,
    pub peak: f32,
    pub dropped: u64,
    pub padded: u64,
    pub received: u64,
}
impl AudioQueue {
    pub fn push(&mut self, values: &[f32]) {
        self.received += values.len() as u64;
        self.peak = values
            .iter()
            .filter(|v| v.is_finite())
            .fold(0f32, |a, v| a.max(v.abs()))
            .min(1.);
        self.samples.extend(
            values
                .iter()
                .map(|v| if v.is_finite() { v.clamp(-1., 1.) } else { 0. }),
        );
        let max = RATE as usize * 4;
        if self.samples.len() > max {
            let n = self.samples.len() - max;
            self.samples.drain(..n);
            self.dropped += n as u64;
        }
    }
    pub fn take(&mut self, n: usize) -> Vec<f32> {
        self.padded += n.saturating_sub(self.samples.len()) as u64;
        (0..n)
            .map(|_| self.samples.pop_front().unwrap_or(0.))
            .collect()
    }
    pub fn clear(&mut self) {
        self.samples.clear();
        self.peak = 0.;
    }
}
pub fn mix(a: &[f32], b: &[f32], gain_a: f32, gain_b: f32) -> Vec<i16> {
    (0..a.len().max(b.len()))
        .map(|i| {
            ((a.get(i).copied().unwrap_or(0.) * gain_a + b.get(i).copied().unwrap_or(0.) * gain_b)
                .clamp(-1., 1.)
                * 32767.) as i16
        })
        .collect()
}
#[cfg(test)] // Legacy one-shot reference; WASAPI uses a continuous sinc resampler.
pub fn stereo_resample(samples: &[f32], channels: usize, rate: u32) -> Vec<f32> {
    if channels == 0 || rate == 0 {
        return vec![];
    }
    let frames = samples.len() / channels;
    let count = frames * RATE as usize / rate as usize;
    let mut out = Vec::with_capacity(count * 2);
    for i in 0..count {
        let pos = i as f64 * rate as f64 / RATE as f64;
        let j = pos as usize;
        let t = (pos - j as f64) as f32;
        for c in 0..2 {
            let c = c.min(channels - 1);
            let a = samples.get(j * channels + c).copied().unwrap_or(0.);
            let b = samples
                .get((j + 1).min(frames.saturating_sub(1)) * channels + c)
                .copied()
                .unwrap_or(a);
            out.push(a + (b - a) * t);
        }
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clips_and_bounds() {
        assert_eq!(mix(&[1., -1.], &[1., -1.], 1., 1.), vec![32767, -32767]);
        let mut q = AudioQueue::default();
        q.push(&[f32::NAN, 0.5]);
        assert_eq!(q.take(3), vec![0., 0.5, 0.]);
    }
    #[test]
    fn resamples_mono() {
        let x = stereo_resample(&vec![0.5; 441], 1, 44100);
        assert_eq!(x.len(), 960);
        assert!(x.iter().all(|x| (*x - 0.5).abs() < 1e-6));
    }
}
