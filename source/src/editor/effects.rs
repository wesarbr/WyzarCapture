use crate::{core::Frame, editor::Project};
pub fn apply(f: &mut Frame, p: &Project, time: f64) {
    if !f.valid() {
        return;
    }
    if p.crop {
        let margin = f.width / 12;
        let mut out = f.clone();
        for y in 0..f.height {
            for x in 0..f.width {
                let sx = margin + x * (f.width - margin * 2) / f.width;
                let sy = f.height / 12 + y * (f.height - f.height / 6) / f.height;
                let dst = ((y * f.width + x) * 4) as usize;
                let src = ((sy * f.width + sx) * 4) as usize;
                out.bgra[dst..dst + 4].copy_from_slice(&f.bgra[src..src + 4]);
            }
        }
        *f = out;
    }
    if p.blur {
        let (x0, y0, w, h) = (f.width / 3, f.height / 3, f.width / 3, f.height / 4);
        blur_region(f, x0, y0, w, h, (f.width / 100).max(4));
    }
    if p.arrow {
        let (x0, y0, x1, y1) = (
            f.width as i32 / 4,
            f.height as i32 * 3 / 4,
            f.width as i32 * 3 / 4,
            f.height as i32 / 4,
        );
        let thickness = (f.width / 250).max(2) as i32;
        line(f, x0, y0, x1, y1, thickness);
        line(f, x1, y1, x1 - f.width as i32 / 10, y1, thickness);
        line(f, x1, y1, x1, y1 + f.height as i32 / 7, thickness);
    }
    if p.fade {
        let gain = (time / 0.6).min((p.duration() - time) / 0.6).clamp(0., 1.) as f32;
        for px in f.bgra.chunks_exact_mut(4) {
            for v in &mut px[..3] {
                *v = (*v as f32 * gain) as u8;
            }
        }
    }
    let texts: Vec<_> = p
        .text
        .iter()
        .filter(|t| time >= t.start && time < t.end)
        .map(|t| t.text.clone())
        .collect();
    if !texts.is_empty() {
        if let Ok(frame) = crate::platform::windows::graphics::text_overlay(f, &texts) {
            *f = frame;
        }
    }
}
fn blur_region(f: &mut Frame, x0: u32, y0: u32, w: u32, h: u32, r: u32) {
    if w == 0 || h == 0 {
        return;
    }
    let mut horizontal = vec![0u8; (w * h * 3) as usize];
    for y in 0..h {
        let mut sum = [0u32; 3];
        let mut count = 0u32;
        for x in 0..=r.min(w - 1) {
            let i = (((y0 + y) * f.width + x0 + x) * 4) as usize;
            for c in 0..3 {
                sum[c] += f.bgra[i + c] as u32;
            }
            count += 1;
        }
        for x in 0..w {
            for c in 0..3 {
                horizontal[((y * w + x) * 3) as usize + c] = (sum[c] / count) as u8;
            }
            if x >= r {
                let i = (((y0 + y) * f.width + x0 + x - r) * 4) as usize;
                for c in 0..3 {
                    sum[c] -= f.bgra[i + c] as u32;
                }
                count -= 1;
            }
            if x + r + 1 < w {
                let i = (((y0 + y) * f.width + x0 + x + r + 1) * 4) as usize;
                for c in 0..3 {
                    sum[c] += f.bgra[i + c] as u32;
                }
                count += 1;
            }
        }
    }
    for x in 0..w {
        let mut sum = [0u32; 3];
        let mut count = 0u32;
        for y in 0..=r.min(h - 1) {
            for c in 0..3 {
                sum[c] += horizontal[((y * w + x) * 3) as usize + c] as u32;
            }
            count += 1;
        }
        for y in 0..h {
            let i = (((y0 + y) * f.width + x0 + x) * 4) as usize;
            for c in 0..3 {
                f.bgra[i + c] = (sum[c] / count) as u8;
            }
            if y >= r {
                for c in 0..3 {
                    sum[c] -= horizontal[(((y - r) * w + x) * 3) as usize + c] as u32;
                }
                count -= 1;
            }
            if y + r + 1 < h {
                for c in 0..3 {
                    sum[c] += horizontal[(((y + r + 1) * w + x) * 3) as usize + c] as u32;
                }
                count += 1;
            }
        }
    }
}
fn line(f: &mut Frame, x0: i32, y0: i32, x1: i32, y1: i32, r: i32) {
    let count = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
    for t in 0..=count {
        let x = x0 + (x1 - x0) * t / count;
        let y = y0 + (y1 - y0) * t / count;
        for yy in y - r..=y + r {
            for xx in x - r..=x + r {
                if xx >= 0 && yy >= 0 && xx < f.width as i32 && yy < f.height as i32 {
                    let i = ((yy as u32 * f.width + xx as u32) * 4) as usize;
                    f.bgra[i..i + 4].copy_from_slice(&[26, 210, 255, 255]);
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn blur_smooths_only_the_selected_region() {
        let mut f = Frame {
            width: 48,
            height: 48,
            bgra: vec![0; 48 * 48 * 4],
        };
        for (y, row) in f.bgra.chunks_exact_mut(48 * 4).enumerate() {
            for (x, p) in row.chunks_exact_mut(4).enumerate() {
                p.copy_from_slice(&[if (x + y) % 2 == 0 { 0 } else { 255 }, 0, 0, 255]);
            }
        }
        let outside = f.bgra[..4].to_vec();
        let mut p = Project::default();
        p.blur = true;
        apply(&mut f, &p, 0.);
        assert_eq!(&f.bgra[..4], outside.as_slice());
        let index = ((20 * 48 + 20) * 4) as usize;
        assert!((1..255).contains(&f.bgra[index]));
    }
    #[test]
    fn tiny_blur_is_safe() {
        let mut f = Frame {
            width: 1,
            height: 1,
            bgra: vec![0, 0, 0, 255],
        };
        let mut p = Project::default();
        p.blur = true;
        apply(&mut f, &p, 0.);
        assert!(f.valid());
    }
}
