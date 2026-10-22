//! Conservative space budget, including the second MP4 written during assembly.
pub const MIB: u64 = 1024 * 1024;
pub const GIB: u64 = 1024 * MIB;
pub fn reserve(encoded_bytes: u64, recovery: bool, bitrate: u32) -> u64 {
    // Keep room for metadata/finalization and at least ten seconds of encoder work.
    (512 * MIB)
        .max((bitrate as u64 / 8).saturating_mul(10))
        .saturating_add(if recovery { encoded_bytes } else { 0 })
}
pub fn test_budget(seconds: u32, bitrate: u32) -> u64 {
    // Source segments + assembled MP4 + two stereo PCM companions, with headroom.
    let per_second = bitrate as u64 / 8 * 2 + 48_000 * 4 * 2;
    (seconds as u64)
        .saturating_mul(per_second)
        .saturating_add(GIB)
}
pub fn finalize_reserve(encoded_bytes: u64) -> u64 {
    // Recording stops with extra headroom; assembly can use that headroom after
    // the encoder closes. Requiring the recording threshold again would prevent
    // saving precisely when the low-space guard has stopped new frames.
    encoded_bytes.saturating_add(64 * MIB)
}
pub fn test_reserve(total: u64) -> u64 {
    (8 * GIB).max(total / 50)
}
pub fn available(free: Option<u64>, required: u64) -> bool {
    free.is_some_and(|bytes| bytes >= required)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn assembly_keeps_room_for_a_second_copy_and_encoder_headroom() {
        assert_eq!(reserve(3 * GIB, true, 20_000_000), 3 * GIB + 512 * MIB);
        assert_eq!(reserve(3 * GIB, false, 20_000_000), 512 * MIB);
        assert!(!available(
            Some(3 * GIB + 256 * MIB),
            reserve(3 * GIB, true, 20_000_000)
        ));
        assert!(available(
            Some(3 * GIB + 256 * MIB),
            finalize_reserve(3 * GIB)
        ));
        assert!(!available(None, 1));
        assert!(!available(Some(512 * MIB - 1), reserve(0, false, 1)));
        assert!(available(Some(512 * MIB), reserve(0, false, 1)));
    }
    #[test]
    fn bounded_test_budget_covers_all_outputs_and_preserves_disk_reserve() {
        assert!(test_budget(600, 20_000_000) > 3 * GIB);
        assert_eq!(test_reserve(1_000 * GIB), 20 * GIB);
        assert_eq!(test_reserve(100 * GIB), 8 * GIB);
        assert_eq!(reserve(u64::MAX, true, u32::MAX), u64::MAX);
    }
}
