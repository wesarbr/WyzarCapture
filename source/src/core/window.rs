#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}
pub fn fit(requested: Bounds, work: Bounds) -> Bounds {
    let width = (requested.right as i64 - requested.left as i64)
        .max(1)
        .min((work.right as i64 - work.left as i64).max(1));
    let height = (requested.bottom as i64 - requested.top as i64)
        .max(1)
        .min((work.bottom as i64 - work.top as i64).max(1));
    let x = (requested.left as i64).clamp(
        work.left as i64,
        (work.right as i64 - width).max(work.left as i64),
    );
    let y = (requested.top as i64).clamp(
        work.top as i64,
        (work.bottom as i64 - height).max(work.top as i64),
    );
    Bounds {
        left: x as i32,
        top: y as i32,
        right: (x + width) as i32,
        bottom: (y + height) as i32,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dpi_resize_suggestions_fit_the_work_area_in_25_model_cases() {
        for (width, height) in [
            (1366, 768),
            (1600, 900),
            (1920, 1080),
            (2560, 1440),
            (3840, 2160),
        ] {
            for dpi in [100, 125, 150, 175, 200] {
                let work = Bounds {
                    left: -width,
                    top: 0,
                    right: 0,
                    bottom: height - 48,
                };
                let requested = Bounds {
                    left: -300,
                    top: 20,
                    right: -300 + 1334 * dpi / 100,
                    bottom: 20 + 688 * dpi / 100,
                };
                let result = fit(requested, work);
                assert!(
                    result.left >= work.left
                        && result.top >= work.top
                        && result.right <= work.right
                        && result.bottom <= work.bottom
                );
            }
        }
    }
}
