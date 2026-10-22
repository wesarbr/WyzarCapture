//! Backward-compatible bundles: existing MP4/WAV/metadata stay in place.
use std::path::{Path, PathBuf};
pub const RELATED_EXTENSIONS: [&str; 6] = [
    "audio.wav",
    "mic.wav",
    "system.wav",
    "webcam.mp4",
    "thumbnail.png",
    "wyzar",
];
pub fn files(primary: &Path) -> Vec<PathBuf> {
    std::iter::once(primary.to_path_buf())
        .chain(RELATED_EXTENSIONS.map(|ext| primary.with_extension(ext)))
        .filter(|path| path.is_file())
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bundles_include_only_known_companions_and_preserve_other_files() {
        let root = std::env::current_dir()
            .unwrap()
            .join("validation/unit")
            .join(format!("bundle-{}", crate::settings::unique_stamp()));
        std::fs::create_dir_all(&root).unwrap();
        let primary = root.join("Clipe.mp4");
        for path in [
            &primary,
            &primary.with_extension("mic.wav"),
            &primary.with_extension("system.wav"),
            &root.join("Outro.mp4"),
            &root.join("Clipe.png"),
        ] {
            std::fs::write(path, b"fixture").unwrap();
        }
        let bundle = files(&primary);
        assert_eq!(bundle.len(), 3);
        assert!(!bundle.contains(&root.join("Outro.mp4")));
        assert!(!bundle.contains(&root.join("Clipe.png")));
        std::fs::remove_dir_all(root).unwrap();
    }
}
