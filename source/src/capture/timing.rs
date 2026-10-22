/// PCM scheduled from absolute media timestamps, without per-frame rounding accumulation.
pub fn audio_end(timestamp: i64, duration: i64) -> u64 {
    ((timestamp.saturating_add(duration).max(0) as u128 * 48_000) / 10_000_000)
        .min(u64::MAX as u128) as u64
}
pub const CONTAINER_TOLERANCE_MS: f64 = 150.;
pub fn missed_intervals(previous: i64, next: i64, period: i64) -> u64 {
    if period <= 0 {
        return 0;
    }
    (next.saturating_sub(previous).saturating_add(period / 2) / period)
        .saturating_sub(1)
        .max(0) as u64
}
pub fn container_timing_ok(audio_end: f64, video_end: f64) -> bool {
    audio_end.is_finite()
        && video_end.is_finite()
        && (audio_end - video_end).abs() * 1000. <= CONTAINER_TOLERANCE_MS
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sample_clock_does_not_accumulate_frame_rounding() {
        for fps in [24, 30, 60, 120] {
            let time = 8 * 3600 * 10_000_000i64;
            assert_eq!(audio_end(time, 0), 8 * 3600 * 48_000);
            assert_eq!(
                audio_end(time - 10_000_000 / fps, 10_000_000 / fps),
                8 * 3600 * 48_000
            );
        }
        assert!(!container_timing_ok(f64::NAN, 1.));
        assert!(!container_timing_ok(1.2, 1.));
        assert!(container_timing_ok(1.02, 1.));
        assert_eq!(missed_intervals(100, 200, 100), 0);
        assert_eq!(missed_intervals(100, 500, 100), 3);
    }
}
