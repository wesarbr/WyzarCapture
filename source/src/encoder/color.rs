//! Explicit BT.709 limited-range NV12. Avoid converter-dependent RGB range or
//! matrix defaults between hardware/software MFTs. Alpha is not encoded.
use crate::core::{AppResult, Frame};
pub fn nv12(frame: &Frame) -> AppResult<Vec<u8>> {
    if !frame.valid() || frame.width % 2 != 0 || frame.height % 2 != 0 {
        return Err(crate::i18n::tr("NV12 requer quadro válido com dimensões pares").into());
    }
    let w = frame.width as usize;
    let h = frame.height as usize;
    let size = w * h;
    let mut out = vec![0; size + size / 2];
    for y in 0..h {
        for x in 0..w {
            let p = (y * w + x) * 4;
            let b = frame.bgra[p] as i32;
            let g = frame.bgra[p + 1] as i32;
            let r = frame.bgra[p + 2] as i32;
            out[y * w + x] = (16 + ((47 * r + 157 * g + 16 * b + 128) >> 8)).clamp(16, 235) as u8;
        }
    }
    for y in (0..h).step_by(2) {
        for x in (0..w).step_by(2) {
            let (mut r, mut g, mut b) = (0, 0, 0);
            for dy in 0..2 {
                for dx in 0..2 {
                    let p = ((y + dy) * w + x + dx) * 4;
                    b += frame.bgra[p] as i32;
                    g += frame.bgra[p + 1] as i32;
                    r += frame.bgra[p + 2] as i32;
                }
            }
            let r = (r + 2) / 4;
            let g = (g + 2) / 4;
            let b = (b + 2) / 4;
            let uv = size + y / 2 * w + x;
            out[uv] = (128 + ((-26 * r - 86 * g + 112 * b + 128) >> 8)).clamp(16, 240) as u8;
            out[uv + 1] = (128 + ((112 * r - 102 * g - 10 * b + 128) >> 8)).clamp(16, 240) as u8;
        }
    }
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nv12_preserves_endpoints_and_size() {
        for (rgb, y) in [(0, 16), (255, 235)] {
            let f = Frame {
                width: 2,
                height: 2,
                bgra: vec![rgb; 16],
            };
            let data = nv12(&f).unwrap();
            assert_eq!(data, vec![y, y, y, y, 128, 128]);
        }
    }
}
