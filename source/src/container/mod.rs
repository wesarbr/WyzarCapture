//! Container policy. MP4/H.264/AAC uses the OS Media Foundation muxer.
//! Other containers require an installed, verified Windows media sink before exposure.
pub const VIDEO_EXTENSION: &str = "mp4";
pub const VIDEO_DESCRIPTION: &str = "MP4 · H.264 / AAC";
