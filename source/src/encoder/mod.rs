//! Safe encoding configuration. All COM/D3D/Media Foundation ownership is platform-local.
pub mod benchmark;
pub mod capabilities;
pub mod color;
#[derive(Clone, Copy, Debug)]
pub struct VideoConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub bitrate: u32,
    pub hardware: bool,
}
/// Average bitrate that keeps screen text sharp: about 10 Mbps for 1080p at 30 FPS,
/// scaled by pixel count and sub-linearly by frame rate (motion between consecutive
/// frames costs less than whole new frames).
pub fn recommended_bitrate(width: u32, height: u32, fps: u32) -> u32 {
    let pixels = (width as f64 * height as f64) / (1920. * 1080.);
    let motion = (fps.max(1) as f64 / 30.).powf(0.65);
    let mbps = (10. * pixels * motion).clamp(4., 80.);
    ((mbps * 2.).round() / 2. * 1_000_000.) as u32
}
impl VideoConfig {
    pub fn from_settings(s: &crate::settings::Settings) -> Self {
        Self {
            width: s.width,
            height: s.height,
            fps: s.fps,
            bitrate: s.bitrate,
            hardware: s.hardware,
        }
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn bitrate_follows_resolution_and_frame_rate() {
        use super::recommended_bitrate as r;
        assert_eq!(r(1920, 1080, 30), 10_000_000);
        assert!(r(1920, 1080, 60) > r(1920, 1080, 30));
        assert!(r(1280, 720, 30) >= 4_000_000 && r(1280, 720, 30) < r(1920, 1080, 30));
        assert!(r(3840, 2160, 60) <= 80_000_000);
        assert_eq!(r(1920, 1080, 30) % 500_000, 0);
    }
}
