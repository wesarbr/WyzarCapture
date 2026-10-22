pub mod effects;
pub mod export;
pub mod tracks;
use crate::{core::AppResult, persistence};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
#[derive(Clone, Debug, PartialEq)]
pub struct Clip {
    pub source: PathBuf,
    pub start: f64,
    pub end: f64,
    pub speed: f64,
    pub volume: f32,
}
impl Clip {
    pub fn duration(&self) -> f64 {
        (self.end - self.start) / self.speed
    }
}
#[derive(Clone, Debug)]
pub struct TextOverlay {
    pub text: String,
    pub start: f64,
    pub end: f64,
}
#[derive(Clone, Debug)]
pub struct Project {
    pub audio_tracks: Vec<tracks::Track>,
    pub tracks_active: bool,
    pub selected_track: Option<usize>,
    pub name: String,
    pub clips: Vec<Clip>,
    pub text: Vec<TextOverlay>,
    pub playhead: f64,
    pub selected: usize,
    pub dirty: bool,
    pub fade: bool,
    pub crop: bool,
    pub blur: bool,
    pub arrow: bool,
}
impl Default for Project {
    fn default() -> Self {
        Self {
            audio_tracks: vec![],
            tracks_active: false,
            selected_track: None,
            name: crate::i18n::tr("Projeto sem título").into(),
            clips: vec![],
            text: vec![],
            playhead: 0.,
            selected: 0,
            dirty: false,
            fade: false,
            crop: false,
            blur: false,
            arrow: false,
        }
    }
}
impl Project {
    pub fn edit_track(&mut self, index: usize, command: usize) -> AppResult<()> {
        let mut t = self
            .audio_tracks
            .get(index)
            .cloned()
            .ok_or(crate::i18n::tr("Selecione uma trilha"))?;
        match command {
            0 => t.mute = !t.mute,
            1 => t.solo = !t.solo,
            2 => t.gain = (t.gain - 0.1).max(0.),
            3 => t.gain = (t.gain + 0.1).min(2.),
            4 => t.offset -= 0.05,
            5 => t.offset += 0.05,
            6 => {
                t.fade_in = if t.fade_in >= 2. { 0. } else { t.fade_in + 0.5 };
                t.fade_out = t.fade_in;
            }
            7 | 8 => {
                let cut = t.start + (self.playhead - t.offset) * t.speed;
                if cut - t.start < 0.02 || t.end - cut < 0.02 {
                    return Err(crate::i18n::tr(
                        "Posicione o cursor dentro da trilha para cortar/dividir",
                    )
                    .into());
                }
                if command == 7 {
                    t.start = cut;
                    t.offset = self.playhead;
                } else {
                    let mut right = t.clone();
                    right.start = cut;
                    right.offset = self.playhead;
                    t.end = cut;
                    self.audio_tracks.insert(index + 1, right);
                }
            }
            9 => {
                self.audio_tracks.remove(index);
                self.selected_track = None;
                self.dirty = true;
                return Ok(());
            }
            _ => return Err(crate::i18n::tr("Ação de trilha inválida").into()),
        }
        t.validate()?;
        self.audio_tracks[index] = t;
        self.selected_track = Some(index);
        self.dirty = true;
        Ok(())
    }
    pub fn duration(&self) -> f64 {
        self.clips.iter().map(Clip::duration).sum()
    }
    pub fn import(&mut self, path: PathBuf, duration: f64) {
        if duration.is_finite() && duration > 0. {
            let offset = self.duration();
            for (kind, ext) in ["mic.wav", "system.wav"].iter().enumerate() {
                let source = path.with_extension(ext);
                if let Ok(seconds) = crate::audio::wav::duration(&source) {
                    if seconds > 0. {
                        self.audio_tracks.push(tracks::Track {
                            source,
                            kind,
                            offset,
                            start: 0.,
                            end: duration.min(seconds),
                            speed: 1.,
                            gain: 1.,
                            mute: false,
                            solo: false,
                            fade_in: 0.,
                            fade_out: 0.,
                        });
                    }
                }
            }
            self.name = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into();
            self.clips.push(Clip {
                source: path,
                start: 0.,
                end: duration,
                speed: 1.,
                volume: 1.,
            });
            self.dirty = true;
        }
    }
    pub fn locate(&self, t: f64) -> Option<(usize, f64)> {
        let mut offset = 0.;
        for (i, c) in self.clips.iter().enumerate() {
            if t < offset + c.duration() {
                return Some((i, c.start + (t - offset) * c.speed));
            }
            offset += c.duration();
        }
        None
    }
    pub fn split(&mut self) -> AppResult<()> {
        let (i, t) = self
            .locate(self.playhead)
            .ok_or(crate::i18n::tr("Posicione o cursor dentro de um clipe"))?;
        let c = &mut self.clips[i];
        if t - c.start < 0.05 || c.end - t < 0.05 {
            return Err(crate::i18n::tr("O corte está muito próximo da borda").into());
        }
        let mut right = c.clone();
        right.start = t;
        c.end = t;
        self.clips.insert(i + 1, right);
        self.selected = i + 1;
        self.dirty = true;
        Ok(())
    }
    pub fn remove_selected(&mut self) -> AppResult<()> {
        if self.selected >= self.clips.len() {
            return Err(crate::i18n::tr("Selecione um clipe").into());
        }
        self.clips.remove(self.selected);
        self.selected = self.selected.min(self.clips.len().saturating_sub(1));
        self.playhead = self.playhead.min(self.duration());
        self.dirty = true;
        Ok(())
    }
    pub fn trim_start(&mut self) -> AppResult<()> {
        let (i, t) = self
            .locate(self.playhead)
            .ok_or(crate::i18n::tr("Posicione o cursor dentro de um clipe"))?;
        if self.clips[i].end - t < 0.05 {
            return Err(crate::i18n::tr("Trecho muito curto").into());
        }
        self.clips[i].start = t;
        self.selected = i;
        self.playhead = self.clips.iter().take(i).map(Clip::duration).sum();
        self.dirty = true;
        Ok(())
    }
    pub fn values(&self) -> BTreeMap<String, String> {
        let mut v = BTreeMap::from([
            ("version".into(), "2".into()),
            ("name".into(), self.name.clone()),
            ("clips".into(), self.clips.len().to_string()),
            ("fade".into(), self.fade.to_string()),
            ("crop".into(), self.crop.to_string()),
            ("blur".into(), self.blur.to_string()),
            ("arrow".into(), self.arrow.to_string()),
        ]);
        v.insert("tracks_active".into(), self.tracks_active.to_string());
        v.insert("audio_tracks".into(), self.audio_tracks.len().to_string());
        for (i, t) in self.audio_tracks.iter().enumerate() {
            for (key, value) in [
                ("source", t.source.to_string_lossy().into_owned()),
                ("kind", t.kind.to_string()),
                ("offset", t.offset.to_string()),
                ("start", t.start.to_string()),
                ("end", t.end.to_string()),
                ("speed", t.speed.to_string()),
                ("gain", t.gain.to_string()),
                ("mute", t.mute.to_string()),
                ("solo", t.solo.to_string()),
                ("fade_in", t.fade_in.to_string()),
                ("fade_out", t.fade_out.to_string()),
            ] {
                v.insert(format!("audio.{i}.{key}"), value);
            }
        }
        for (i, c) in self.clips.iter().enumerate() {
            for (k, val) in [
                ("source", c.source.to_string_lossy().into()),
                ("start", c.start.to_string()),
                ("end", c.end.to_string()),
                ("speed", c.speed.to_string()),
                ("volume", c.volume.to_string()),
            ] {
                v.insert(format!("clip.{i}.{k}"), val);
            }
        }
        v.insert("text.count".into(), self.text.len().to_string());
        for (i, t) in self.text.iter().enumerate() {
            v.insert(format!("text.{i}.value"), t.text.clone());
            v.insert(format!("text.{i}.start"), t.start.to_string());
            v.insert(format!("text.{i}.end"), t.end.to_string());
        }
        v
    }
    pub fn save(&mut self, path: &Path) -> AppResult<()> {
        persistence::write(path, &self.values())?;
        self.dirty = false;
        Ok(())
    }
    pub fn load(path: &Path) -> AppResult<Self> {
        let v = persistence::read(path)?;
        if v.get("version")
            .is_some_and(|version| !["1", "2"].contains(&version.as_str()))
        {
            return Err(crate::i18n::tr(
                "Este projeto usa uma versão mais recente. O arquivo foi preservado.",
            )
            .into());
        }
        let mut p = Self {
            name: v.get("name").cloned().unwrap_or_default(),
            ..Self::default()
        };
        let count = v
            .get("audio_tracks")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        if count > 1024 {
            return Err(crate::i18n::tr("Projeto com trilhas demais").into());
        }
        p.tracks_active = v.get("tracks_active").is_some_and(|s| s == "true");
        for i in 0..count {
            let get = |k: &str| {
                v.get(&format!("audio.{i}.{k}"))
                    .cloned()
                    .unwrap_or_default()
            };
            let t = tracks::Track {
                source: get("source").into(),
                kind: get("kind").parse().unwrap_or(2),
                offset: get("offset").parse().unwrap_or(f64::NAN),
                start: get("start").parse().unwrap_or(f64::NAN),
                end: get("end").parse().unwrap_or(f64::NAN),
                speed: get("speed").parse().unwrap_or(1.),
                gain: get("gain").parse().unwrap_or(1.),
                mute: get("mute") == "true",
                solo: get("solo") == "true",
                fade_in: get("fade_in").parse().unwrap_or(0.),
                fade_out: get("fade_out").parse().unwrap_or(0.),
            };
            t.validate()?;
            p.audio_tracks.push(t);
        }
        let count = v
            .get("clips")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        if count > 10000 {
            return Err(crate::i18n::tr("Projeto muito grande").into());
        }
        for i in 0..count {
            let get = |k: &str| v.get(&format!("clip.{i}.{k}")).cloned().unwrap_or_default();
            let c = Clip {
                source: get("source").into(),
                start: get("start").parse().unwrap_or(0.),
                end: get("end").parse().unwrap_or(0.),
                speed: get("speed").parse().unwrap_or(1.),
                volume: get("volume").parse().unwrap_or(1.),
            };
            if !c.start.is_finite()
                || !c.end.is_finite()
                || !c.speed.is_finite()
                || !c.volume.is_finite()
                || c.start < 0.
                || c.end <= c.start
                || !(0.25..=4.).contains(&c.speed)
                || !(0.0..=2.0).contains(&c.volume)
            {
                return Err(crate::i18n::tr("Projeto contém intervalo inválido").into());
            }
            p.clips.push(c);
        }
        p.fade = v.get("fade").is_some_and(|x| x == "true");
        p.crop = v.get("crop").is_some_and(|x| x == "true");
        p.blur = v.get("blur").is_some_and(|x| x == "true");
        p.arrow = v.get("arrow").is_some_and(|x| x == "true");
        let count = v
            .get("text.count")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0)
            .min(1000);
        for i in 0..count {
            let get = |k: &str| v.get(&format!("text.{i}.{k}")).cloned().unwrap_or_default();
            let overlay = TextOverlay {
                text: get("value"),
                start: get("start").parse().unwrap_or(0.),
                end: get("end").parse().unwrap_or(0.),
            };
            if !overlay.start.is_finite()
                || !overlay.end.is_finite()
                || overlay.start < 0.
                || overlay.end <= overlay.start
                || overlay.text.len() > 2000
            {
                return Err(crate::i18n::tr("Projeto contém legenda inválida.").into());
            }
            p.text.push(overlay);
        }
        Ok(p)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_project_keeps_clip_audio_and_upgrades_only_when_saved() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("validation/unit");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join(format!(
            "legacy-project-{}.wyzproj",
            crate::settings::unique_stamp()
        ));
        let mut original = Project::default();
        original.import("legacy.mp4".into(), 2.);
        original.clips[0].volume = 0.5;
        let mut values = original.values();
        values.insert("version".into(), "1".into());
        values.remove("audio_tracks");
        values.remove("tracks_active");
        persistence::write(&path, &values).unwrap();
        let mut loaded = Project::load(&path).unwrap();
        assert!(!loaded.tracks_active);
        assert!(loaded.audio_tracks.is_empty());
        assert_eq!(loaded.clips[0].volume, 0.5);
        loaded.save(&path).unwrap();
        assert_eq!(
            persistence::read(&path)
                .unwrap()
                .get("version")
                .map(String::as_str),
            Some("2")
        );
        assert!(!Project::load(&path).unwrap().tracks_active);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn split_preserves_duration() {
        let mut p = Project::default();
        p.import("a.mp4".into(), 10.);
        p.playhead = 4.;
        p.split().unwrap();
        assert_eq!(p.clips.len(), 2);
        assert_eq!(p.duration(), 10.);
        assert_eq!(p.clips[1].start, 4.);
        p.remove_selected().unwrap();
        assert_eq!(p.duration(), 4.);
    }
    #[test]
    fn trim_speed_mapping() {
        let mut p = Project::default();
        p.import("a.mp4".into(), 10.);
        p.clips[0].speed = 2.;
        p.playhead = 2.;
        p.trim_start().unwrap();
        assert_eq!(p.clips[0].start, 4.);
        assert_eq!(p.duration(), 3.);
    }
}
