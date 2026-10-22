use crate::{
    core::AppResult,
    editor::Project,
    encoder::VideoConfig,
    platform::windows::{
        Runtime,
        media::{AudioReader, Reader, Writer},
    },
};
use std::path::Path;
pub fn export(
    project: &Project,
    cfg: VideoConfig,
    path: &Path,
    mut progress: impl FnMut(f32),
) -> AppResult<()> {
    let _runtime = Runtime::new(true)?;
    let total = project.duration();
    if total <= 0. {
        return Err(crate::i18n::tr("Projeto vazio").into());
    }
    let reserve = crate::core::storage::reserve(0, false, cfg.bitrate);
    crate::platform::windows::storage::require(path, reserve)?;
    let writer = Writer::new(path, cfg, true)?;
    let mut space_checked = std::time::Instant::now();
    let mut tracks = if project.tracks_active {
        Some(super::tracks::Mixer::new(&project.audio_tracks)?)
    } else {
        None
    };
    let mut offset = 0.;
    let mut frames = 0u64;
    let mut audio_cursor = 0u64;
    for clip in &project.clips {
        let count = end_frame(offset + clip.duration(), cfg.fps).saturating_sub(frames);
        if count == 0 {
            offset += clip.duration();
            continue;
        }
        let mut reader = Reader::open(&clip.source)?;
        reader.seek(clip.start)?;
        let source_start =
            clip.start + (frames as f64 / cfg.fps as f64 - offset).max(0.) * clip.speed;
        let mut audio = if reader.meta.audio && !project.tracks_active {
            Some(AudioReader::open(&clip.source, source_start)?)
        } else {
            None
        };
        let mut current = reader.next()?;
        let mut next = reader.next()?;
        for _ in 0..count {
            if space_checked.elapsed() >= std::time::Duration::from_secs(1) {
                space_checked = std::time::Instant::now();
                if let Err(error) = crate::platform::windows::storage::require(path, reserve) {
                    let _ = writer.finalize();
                    return Err(error);
                }
            }
            let timeline = frames as f64 / cfg.fps as f64;
            let source = clip.start + (timeline - offset).max(0.) * clip.speed;
            while next.as_ref().is_some_and(|(_, t)| *t <= source + 0.0001) {
                current = next.take();
                next = reader.next()?;
            }
            let raw = current.as_ref().map(|(f, _)| f).ok_or(crate::i18n::tr(
                "O vídeo não forneceu quadros para exportação",
            ))?;
            let mut frame = if raw.width != cfg.width || raw.height != cfg.height {
                raw.fit(cfg.width, cfg.height)
            } else {
                raw.clone()
            };
            super::effects::apply(&mut frame, project, timeline);
            let timestamp = frames as i64 * 10_000_000 / cfg.fps as i64;
            let next_timestamp = (frames + 1) as i64 * 10_000_000 / cfg.fps as i64;
            writer.video(&frame, timestamp, next_timestamp - timestamp)?;
            let audio_frames =
                ((frames + 1) * 48000 / cfg.fps as u64 - frames * 48000 / cfg.fps as u64) as usize;
            let samples = if let Some(mixer) = tracks.as_mut() {
                mixer.take(audio_cursor as f64 / 48000., audio_frames)?
            } else {
                audio
                    .as_mut()
                    .map(|a| a.take(audio_frames, clip.speed, clip.volume))
                    .unwrap_or_else(|| vec![0; audio_frames * 2])
            };
            writer.audio(&samples, audio_cursor as i64 * 10_000_000 / 48000)?;
            audio_cursor += audio_frames as u64;
            frames += 1;
            if frames % 10 == 0 {
                progress((timeline / total).min(0.99) as f32);
            }
        }
        offset += clip.duration();
    }
    writer.finalize()?;
    progress(1.);
    Ok(())
}
fn end_frame(seconds: f64, fps: u32) -> u64 {
    // Avoid an extra frame at an exact boundary reached through floating addition.
    (seconds * fps as f64 - 1e-8).ceil().max(0.) as u64
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn many_short_cuts_do_not_accumulate_rounding_frames() {
        let mut offset = 0.;
        let mut frames = 0;
        for _ in 0..40 {
            let next = end_frame(offset + 0.05, 30);
            frames += next - frames;
            offset += 0.05;
        }
        assert_eq!(frames, 60);
        assert_eq!(end_frame(1., 60), 60);
        assert_eq!(end_frame(1.001, 60), 61);
    }
}
