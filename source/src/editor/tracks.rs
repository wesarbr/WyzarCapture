//! Independent non-destructive PCM tracks, shared by preview and export.
use crate::{audio::wav::PcmReader, core::AppResult};
use std::path::PathBuf;
#[derive(Clone, Debug)]
pub struct Track {
    pub source: PathBuf,
    pub kind: usize,
    pub offset: f64,
    pub start: f64,
    pub end: f64,
    pub speed: f64,
    pub gain: f32,
    pub mute: bool,
    pub solo: bool,
    pub fade_in: f64,
    pub fade_out: f64,
}
impl Track {
    pub fn duration(&self) -> f64 {
        (self.end - self.start) / self.speed
    }
    pub fn validate(&self) -> AppResult<()> {
        if !self.offset.is_finite()
            || self.offset.abs() > 86400.
            || !self.start.is_finite()
            || self.start < 0.
            || !self.end.is_finite()
            || self.end <= self.start
            || !self.speed.is_finite()
            || !(0.25..=4.).contains(&self.speed)
            || !self.gain.is_finite()
            || !(0.0..=2.).contains(&self.gain)
            || self.kind > 2
            || !self.fade_in.is_finite()
            || !self.fade_out.is_finite()
            || !(0.0..=3600.).contains(&self.fade_in)
            || !(0.0..=3600.).contains(&self.fade_out)
        {
            return Err(crate::i18n::tr("Trilha de áudio contém parâmetros inválidos").into());
        }
        Ok(())
    }
}
pub struct Mixer {
    tracks: Vec<(Track, PcmReader)>,
    solo: bool,
}
impl Mixer {
    pub fn new(tracks: &[Track]) -> AppResult<Self> {
        let solo = tracks.iter().any(|t| t.solo && !t.mute);
        let mut inputs = vec![];
        for track in tracks {
            track.validate()?;
            if track.mute || (solo && !track.solo) {
                continue;
            }
            let reader = PcmReader::open(&track.source).map_err(|e| {
                let __wyzar_i18n_0 = &(track.source.display());
                crate::i18n::format_text(
                    crate::i18n::tr("Trilha ausente ou inválida: {}: {e}"),
                    &[
                        ("0", format!("{}", __wyzar_i18n_0)),
                        ("e", format!("{}", e)),
                    ],
                )
            })?;
            inputs.push((track.clone(), reader));
        }
        Ok(Self {
            tracks: inputs,
            solo,
        })
    }
    pub fn take(&mut self, time: f64, count: usize) -> AppResult<Vec<i16>> {
        if count > 24000 {
            return Err(crate::i18n::tr("Bloco de mixagem excessivo").into());
        }
        let mut mixed = vec![0f32; count * 2];
        for (track, reader) in &mut self.tracks {
            if track.mute || (self.solo && !track.solo) {
                continue;
            }
            let source_time = track.start + (time - track.offset) * track.speed;
            let first_frame = (source_time.max(track.start) * 48000.).floor() as u64;
            let samples = reader.read_frames(
                first_frame,
                (count as f64 * track.speed).ceil() as usize + 2,
            )?;
            for frame in 0..count {
                let t = time + frame as f64 / 48000.;
                let local = t - track.offset;
                let source = (track.start + local * track.speed) * 48000.;
                if local < 0. || local >= track.duration() || source < 0. {
                    continue;
                }
                let pos = source - first_frame as f64;
                if pos < 0. {
                    continue;
                }
                let index = pos.floor() as usize;
                let fade_in = if track.fade_in > 0. {
                    (local / track.fade_in).min(1.)
                } else {
                    1.
                };
                let fade_out = if track.fade_out > 0. {
                    ((track.duration() - local) / track.fade_out).clamp(0., 1.)
                } else {
                    1.
                };
                for c in 0..2 {
                    let a = samples.get(index * 2 + c).copied().unwrap_or(0.);
                    let b = samples.get((index + 1) * 2 + c).copied().unwrap_or(a);
                    mixed[frame * 2 + c] += (a + (b - a) * pos.fract() as f32)
                        * track.gain
                        * (fade_in * fade_out) as f32;
                }
            }
        }
        Ok(mixed
            .into_iter()
            .map(|v| (v.clamp(-1., 1.) * 32767.) as i16)
            .collect())
    }
}
