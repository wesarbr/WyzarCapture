use std::{path::PathBuf, time::Duration};
pub mod window;

pub type AppResult<T> = Result<T, String>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Source {
    #[default]
    Display,
    Window,
    Region,
    Game,
}
impl Source {
    pub const ALL: [Self; 4] = [Self::Display, Self::Window, Self::Region, Self::Game];
    pub fn label(self) -> &'static str {
        match self {
            Self::Display => crate::i18n::tr("Tela inteira"),
            Self::Window => crate::i18n::tr("Janela"),
            Self::Region => crate::i18n::tr("Região"),
            Self::Game => crate::i18n::tr("Jogo"),
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Self::Display => "display",
            Self::Window => "window",
            Self::Region => "region",
            Self::Game => "game",
        }
    }
    pub fn parse(s: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|v| v.key() == s)
            .unwrap_or_default()
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RecordingState {
    #[default]
    Idle,
    Countdown,
    Starting,
    Recording,
    Paused,
    Finalizing,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
impl Region {
    pub fn valid(self) -> bool {
        self.width >= 16 && self.height >= 16 && self.width <= 16384 && self.height <= 16384
    }
}
#[derive(Clone, Debug)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}
impl Frame {
    pub fn from_strided_rgb32(
        data: &[u8],
        width: u32,
        height: u32,
        stride: i32,
    ) -> AppResult<Self> {
        if width == 0 || height == 0 || width > 4096 || height > 4096 {
            return Err(crate::i18n::tr("Dimensões da câmera fora do limite seguro.").into());
        }
        let row = width as usize * 4;
        let pitch = stride.unsigned_abs() as usize;
        let need = pitch
            .checked_mul(height as usize - 1)
            .and_then(|n| n.checked_add(row));
        if pitch < row || need.is_none_or(|n| n > data.len()) {
            return Err(crate::i18n::tr(
                "A câmera retornou um quadro RGB32 truncado ou stride inválido.",
            )
            .into());
        }
        let mut bgra = vec![0; row * height as usize];
        for y in 0..height as usize {
            let source = if stride < 0 {
                height as usize - 1 - y
            } else {
                y
            };
            bgra[y * row..(y + 1) * row]
                .copy_from_slice(&data[source * pitch..source * pitch + row]);
        }
        for pixel in bgra.chunks_exact_mut(4) {
            pixel[3] = 255;
        }
        Ok(Self {
            width,
            height,
            bgra,
        })
    }
    pub fn valid(&self) -> bool {
        self.width > 0
            && self.height > 0
            && (self.width as usize)
                .checked_mul(self.height as usize)
                .and_then(|n| n.checked_mul(4))
                == Some(self.bgra.len())
    }
    pub fn fit(&self, width: u32, height: u32) -> Self {
        if self.width == width && self.height == height {
            return self.clone();
        }
        let ratio = (width as f64 / self.width.max(1) as f64)
            .min(height as f64 / self.height.max(1) as f64);
        let rw = (self.width as f64 * ratio).round().max(1.) as u32;
        let rh = (self.height as f64 * ratio).round().max(1.) as u32;
        let scaled = self.resize(rw.min(width), rh.min(height));
        let mut out = Self {
            width,
            height,
            bgra: vec![0; (width * height * 4) as usize],
        };
        for a in out.bgra.chunks_exact_mut(4) {
            a[3] = 255;
        }
        let (x, y) = ((width - scaled.width) / 2, (height - scaled.height) / 2);
        for row in 0..scaled.height {
            let from = (row * scaled.width * 4) as usize;
            let to = (((row + y) * width + x) * 4) as usize;
            out.bgra[to..to + (scaled.width * 4) as usize]
                .copy_from_slice(&scaled.bgra[from..from + (scaled.width * 4) as usize]);
        }
        out
    }
    /// High-quality resize: a tent filter widened by the reduction ratio averages
    /// every source pixel when shrinking (no aliasing on text/UI), and becomes
    /// center-aligned bilinear when enlarging. Separable, fixed-point, split
    /// across a few threads so weak CPUs keep up without the GPU path.
    pub fn resize(&self, width: u32, height: u32) -> Self {
        if !self.valid() || width == 0 || height == 0 {
            return Self {
                width,
                height,
                bgra: vec![0; (width as usize) * (height as usize) * 4],
            };
        }
        if self.width == width && self.height == height {
            return self.clone();
        }
        let columns = filter(self.width, width);
        let rows = filter(self.height, height);
        let (sw, dw) = (self.width as usize, width as usize);
        // Horizontal pass into 8.8 fixed point, one source row at a time.
        let mut middle = vec![0u16; dw * 3 * self.height as usize];
        let src = &self.bgra;
        let threads = std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1)
            .clamp(1, 4);
        let band = (self.height as usize).div_ceil(threads).max(1);
        std::thread::scope(|scope| {
            for (index, chunk) in middle.chunks_mut(band * dw * 3).enumerate() {
                let columns = &columns;
                scope.spawn(move || {
                    for (local, line) in chunk.chunks_mut(dw * 3).enumerate() {
                        let y = index * band + local;
                        let row = &src[y * sw * 4..(y + 1) * sw * 4];
                        for (x, (start, weights)) in columns.iter().enumerate() {
                            let mut acc = [0i32; 3];
                            for (k, w) in weights.iter().enumerate() {
                                let p = (start + k) * 4;
                                acc[0] += *w * row[p] as i32;
                                acc[1] += *w * row[p + 1] as i32;
                                acc[2] += *w * row[p + 2] as i32;
                            }
                            for c in 0..3 {
                                line[x * 3 + c] = ((acc[c] + (1 << 5)) >> 6).clamp(0, 65535) as u16;
                            }
                        }
                    }
                });
            }
        });
        let mut out = vec![0u8; dw * height as usize * 4];
        let band = (height as usize).div_ceil(threads).max(1);
        std::thread::scope(|scope| {
            for (index, chunk) in out.chunks_mut(band * dw * 4).enumerate() {
                let (rows, middle) = (&rows, &middle);
                scope.spawn(move || {
                    for (local, line) in chunk.chunks_mut(dw * 4).enumerate() {
                        let (start, weights) = &rows[index * band + local];
                        for x in 0..dw {
                            let mut acc = [0i32; 3];
                            for (k, w) in weights.iter().enumerate() {
                                let p = ((start + k) * dw + x) * 3;
                                acc[0] += *w * middle[p] as i32;
                                acc[1] += *w * middle[p + 1] as i32;
                                acc[2] += *w * middle[p + 2] as i32;
                            }
                            for c in 0..3 {
                                line[x * 4 + c] = ((acc[c] + (1 << 21)) >> 22).clamp(0, 255) as u8;
                            }
                            line[x * 4 + 3] = 255;
                        }
                    }
                });
            }
        });
        Self {
            width,
            height,
            bgra: out,
        }
    }
}
/// Per-output tap ranges with 14-bit weights that sum exactly to 1 << 14.
fn filter(source: u32, target: u32) -> Vec<(usize, Vec<i32>)> {
    let scale = source as f64 / target as f64;
    let support = scale.max(1.);
    (0..target)
        .map(|i| {
            let center = (i as f64 + 0.5) * scale - 0.5;
            let first = (center - support).ceil().max(0.) as usize;
            let last = ((center + support).floor() as usize).min(source as usize - 1);
            let first = first.min(last);
            let raw: Vec<f64> = (first..=last)
                .map(|x| (1. - (x as f64 - center).abs() / support).max(0.))
                .collect();
            let total: f64 = raw.iter().sum();
            let mut weights: Vec<i32> = if total > 0. {
                raw.iter()
                    .map(|w| (w / total * 16384.).round() as i32)
                    .collect()
            } else {
                vec![16384]
            };
            // Put the rounding remainder on the largest tap so flat colors stay exact.
            let remainder = 16384 - weights.iter().sum::<i32>();
            if let Some(max) = weights
                .iter()
                .enumerate()
                .max_by_key(|(_, w)| **w)
                .map(|(i, _)| i)
            {
                weights[max] += remainder;
            }
            (first, weights)
        })
        .collect()
}
#[derive(Clone, Debug)]
pub struct RecordingInfo {
    pub path: PathBuf,
    pub duration: Duration,
    pub frames: u64,
    pub width: u32,
    pub height: u32,
}
pub fn duration_label(seconds: f64) -> String {
    let s = seconds.max(0.) as u64;
    if s >= 3600 {
        format!("{:02}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
    } else {
        format!("{:02}:{:02}", s / 60, s % 60)
    }
}
pub fn size_label(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.)
    } else {
        format!("{:.1} MB", bytes as f64 / 1_048_576.)
    }
}
pub fn timecode(seconds: f64) -> String {
    let millis = (seconds.max(0.) * 1000.).round() as u64;
    format!(
        "{}.{:03}",
        duration_label((millis / 1000) as f64),
        millis % 1000
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fit_preserves_aspect_with_opaque_bars() {
        let frame = Frame {
            width: 4,
            height: 2,
            bgra: [20, 40, 200, 255].repeat(8),
        };
        let out = frame.fit(4, 4);
        assert!(out.valid());
        assert_eq!(&out.bgra[..4], &[0, 0, 0, 255]);
        assert_eq!(&out.bgra[16..20], &[20, 40, 200, 255]);
        assert_eq!(&out.bgra[48..52], &[0, 0, 0, 255]);
    }
    #[test]
    fn resize_keeps_flat_colors_and_averages_fine_detail() {
        let flat = Frame {
            width: 37,
            height: 21,
            bgra: [12, 200, 77, 255].repeat(37 * 21),
        };
        for (w, h) in [(20, 11), (80, 45), (37, 21), (1, 1)] {
            let out = flat.resize(w, h);
            assert!(out.valid());
            assert!(out.bgra.chunks_exact(4).all(|p| p == [12, 200, 77, 255]));
        }
        // One-pixel checkerboard: shrinking must average, not pick alternate pixels.
        let mut checker = Frame {
            width: 64,
            height: 64,
            bgra: vec![0; 64 * 64 * 4],
        };
        for (i, p) in checker.bgra.chunks_exact_mut(4).enumerate() {
            let v = if (i % 64 + i / 64) % 2 == 0 { 255 } else { 0 };
            p.copy_from_slice(&[v, v, v, 255]);
        }
        let small = checker.resize(21, 21);
        let inner: Vec<u8> = small
            .bgra
            .chunks_exact(4)
            .enumerate()
            .filter(|(i, _)| (2..19).contains(&(i % 21)) && (2..19).contains(&(i / 21)))
            .map(|(_, p)| p[0])
            .collect();
        assert!(inner.iter().all(|v| (100..=156).contains(v)), "{inner:?}");
    }
}

#[cfg(test)]
mod webcam_buffer_tests {
    use super::Frame;
    #[test]
    fn camera_rows_accept_padding_and_bottom_up_without_reading_padding() {
        let data = [
            1, 2, 3, 0, 4, 5, 6, 0, 99, 99, 99, 99, 7, 8, 9, 0, 10, 11, 12, 0,
        ];
        let top = Frame::from_strided_rgb32(&data, 2, 2, 12).unwrap();
        assert_eq!(
            top.bgra,
            [1, 2, 3, 255, 4, 5, 6, 255, 7, 8, 9, 255, 10, 11, 12, 255]
        );
        let bottom = Frame::from_strided_rgb32(&data, 2, 2, -12).unwrap();
        assert_eq!(&bottom.bgra[..8], &top.bgra[8..]);
        assert_eq!(&bottom.bgra[8..], &top.bgra[..8]);
    }
    #[test]
    fn camera_rows_reject_truncation_invalid_pitch_and_unbounded_size() {
        assert!(Frame::from_strided_rgb32(&[0; 15], 2, 2, 8).is_err());
        assert!(Frame::from_strided_rgb32(&[0; 16], 2, 2, 4).is_err());
        assert!(Frame::from_strided_rgb32(&[], u32::MAX, u32::MAX, i32::MIN).is_err());
        assert!(Frame::from_strided_rgb32(&[], 0, 1, 0).is_err());
    }
}
pub mod storage;
