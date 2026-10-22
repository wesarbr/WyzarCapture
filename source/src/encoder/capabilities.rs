//! Format acceptance is distinct from sustained capture performance.
use crate::{
    core::{AppResult, Frame},
    encoder::VideoConfig,
    platform::windows::{
        Runtime,
        media::{Reader, Writer},
    },
};
use std::path::Path;
#[derive(Clone, Debug, Default)]
pub struct Inventory {
    pub logical_cpus: usize,
    pub dedicated_video_bytes: usize,
    pub memory_bytes: u64,
    pub primary_refresh_hz: u32,
    pub monitors: Vec<(u32, u32)>,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub cameras: Vec<String>,
    pub adapters: Vec<String>,
    pub encoders: Vec<Transform>,
    pub errors: Vec<String>,
}
#[derive(Clone, Debug)]
pub struct Transform {
    pub name: String,
    pub clsid: String,
    pub hardware: bool,
}
impl Inventory {
    pub fn hardware(&self) -> bool {
        self.encoders.iter().any(|e| e.hardware)
    }
    pub fn recommendation(&self) -> &'static str {
        let large = self
            .monitors
            .first()
            .is_some_and(|&(w, h)| w >= 1920 && h >= 1080);
        let fast = self.hardware()
            && self.dedicated_video_bytes >= 2 * 1024 * 1024 * 1024
            && self.logical_cpus >= 4
            && self.primary_refresh_hz >= 60;
        if large && fast {
            crate::i18n::tr(
                "1080p · 60 FPS · Hardware automático (provisório; confirme com Testar desempenho)",
            )
        } else if large && self.hardware() {
            crate::i18n::tr(
                "1080p · 30 FPS · Hardware automático (provisório; confirme com Testar desempenho)",
            )
        } else if fast {
            crate::i18n::tr(
                "720p · 60 FPS · Hardware automático (provisório; confirme com Testar desempenho)",
            )
        } else if self.hardware() {
            crate::i18n::tr(
                "720p · 30 FPS · Hardware automático (provisório; confirme com Testar desempenho)",
            )
        } else {
            crate::i18n::tr("720p · 30 FPS · Software (provisório; confirme com Testar desempenho)")
        }
    }
    pub fn json(&self) -> String {
        let adapters = self
            .adapters
            .iter()
            .map(|s| format!("{s:?}"))
            .collect::<Vec<_>>()
            .join(",");
        let encoders = self
            .encoders
            .iter()
            .map(|e| {
                format!(
                    "{{\"name\":{:?},\"clsid\":{:?},\"hardware\":{}}}",
                    e.name, e.clsid, e.hardware
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let monitors = self
            .monitors
            .iter()
            .map(|(w, h)| format!("[{w},{h}]"))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"adapters\":[{adapters}],\"h264_mfts\":[{encoders}],\"logical_cpus\":{},\"memory_bytes\":{},\"primary_refresh_hz\":{},\"monitors_width_height\":[{monitors}],\"audio_inputs\":{:?},\"audio_outputs\":{:?},\"cameras\":{:?},\"errors\":{:?},\"recommendation\":{:?},\"sustained_certified\":false}}",
            self.logical_cpus,
            self.memory_bytes,
            self.primary_refresh_hz,
            self.inputs,
            self.outputs,
            self.cameras,
            self.errors,
            self.recommendation()
        )
    }
}
#[derive(Clone, Debug)]
pub struct Profile {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub available: bool,
}
pub fn probe(cache: &Path, hardware: bool) -> AppResult<Vec<Profile>> {
    let _runtime = Runtime::new(true)?;
    let root = cache.join("CapabilityTests").join(format!(
        "{}-{}",
        crate::settings::unique_stamp(),
        std::process::id()
    ));
    std::fs::create_dir_all(root.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::create_dir(&root).map_err(|e| e.to_string())?;
    let mut profiles = vec![];
    for (width, height) in [(1280, 720), (1920, 1080), (2560, 1440), (3840, 2160)] {
        let frame = Frame {
            width,
            height,
            bgra: vec![0; (width * height * 4) as usize],
        };
        for fps in [24, 30, 60, 120] {
            let path = root.join(format!("{width}x{height}-{fps}.mp4"));
            let available = (|| -> AppResult<bool> {
                let writer = Writer::new(
                    &path,
                    VideoConfig {
                        width,
                        height,
                        fps,
                        bitrate: 20_000_000,
                        hardware,
                    },
                    false,
                )?;
                let ticks = 10_000_000 / fps as i64;
                for index in 0..3 {
                    writer.video(&frame, index * ticks, ticks)?;
                }
                writer.finalize()?;
                let mut reader = Reader::open(&path)?;
                Ok(reader.meta.width == width
                    && reader.meta.height == height
                    && reader.frame_at(0.)?.is_some())
            })()
            .unwrap_or(false);
            if path.is_file() {
                std::fs::remove_file(&path).map_err(|e| e.to_string())?;
            }
            profiles.push(Profile {
                width,
                height,
                fps,
                available,
            });
        }
    }
    std::fs::remove_dir(root).map_err(|e| e.to_string())?;
    Ok(profiles)
}
pub fn label(profiles: &[Profile], width: u32, height: u32, fps: u32) -> &'static str {
    match profiles
        .iter()
        .find(|p| p.width == width && p.height == height && p.fps == fps)
    {
        Some(p) if p.available => crate::i18n::tr("Formato verificado"),
        Some(_) => crate::i18n::tr("Indisponível"),
        None => crate::i18n::tr("Experimental"),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_capabilities_are_never_reported_as_verified() {
        assert_eq!(label(&[], 1920, 1080, 60), "Experimental");
        let profiles = [Profile {
            width: 1920,
            height: 1080,
            fps: 60,
            available: false,
        }];
        assert_eq!(label(&profiles, 1920, 1080, 60), "Indisponível");
    }
}
