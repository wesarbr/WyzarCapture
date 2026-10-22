pub mod appearance;
pub mod bootstrap;
pub mod shortcuts;
use crate::{
    core::{AppResult, Source},
    persistence,
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Debug)]
pub struct Settings {
    pub language: crate::i18n::Language,
    pub appearance: appearance::Appearance,
    pub data_dir: PathBuf,
    pub videos: PathBuf,
    pub audios: PathBuf,
    /// 0: mixed video; 1: silent video; 2: video + separate WAV; 3: audio only.
    pub recording_mode: usize,
    pub mic_gain: u32,
    pub system_gain: u32,
    pub audio_sources: bool,
    pub audio_limiter: bool,
    pub auto_fallback: bool,
    pub screenshots: PathBuf,
    pub cache: PathBuf,
    pub exports: PathBuf,
    pub projects: PathBuf,
    pub source: Source,
    pub fps: u32,
    pub width: u32,
    pub height: u32,
    pub bitrate: u32,
    pub microphone: bool,
    pub system_audio: bool,
    pub webcam: bool,
    pub cursor: bool,
    pub click_effects: bool,
    pub microphone_id: String,
    pub output_id: String,
    pub webcam_id: String,
    pub monitor: usize,
    pub countdown: u32,
    pub naming: String,
    pub subfolders: String,
    pub project_name: String,
    pub recovery: bool,
    pub segment_seconds: u32,
    pub notifications: bool,
    pub tray: bool,
    pub startup: bool,
    pub hardware: bool,
    pub gpu_resize: bool,
    pub low_space_gb: u32,
    pub storage_limit_gb: u32,
    pub webcam_size: u32,
    pub webcam_position: usize,
    pub window_width: u32,
    pub window_height: u32,
    pub last_page: String,
    pub last_seen_version: String,
    pub notified_version: String,
    pub install_storage_request: String,
    pub install_language_request: String,
    pub record_hotkey: u32,
    pub pause_hotkey: u32,
    pub stop_hotkey: u32,
    pub screenshot_hotkey: u32,
    pub screenshot_mode: usize,
    pub screenshot_delay: u32,
    pub screenshot_clipboard: bool,
    pub screenshot_cursor: bool,
    /// 0 rectangle, 1 rounded corners, 2 circle.
    pub webcam_shape: usize,
    pub webcam_mirror: bool,
    /// Microphone rumble filter and gentle noise reduction between phrases.
    pub voice_filter: bool,
    /// Transient explanation of a first-use fallback; never overwrites a saved choice.
    pub storage_notice: Option<String>,
}
impl Settings {
    pub fn defaults(data_dir: PathBuf) -> Self {
        let [videos, audios, screenshots, exports, projects] =
            crate::platform::windows::files::media_folders();
        Self {
            language: crate::i18n::current(),
            appearance: appearance::Appearance::default(),
            videos,
            audios,
            recording_mode: 0,
            mic_gain: 100,
            system_gain: 100,
            audio_sources: true,
            audio_limiter: true,
            auto_fallback: true,
            screenshots,
            cache: data_dir.join("Cache"),
            exports,
            projects,
            data_dir,
            source: Source::Display,
            fps: 60,
            width: 1920,
            height: 1080,
            bitrate: 20_000_000,
            microphone: false,
            system_audio: true,
            webcam: false,
            cursor: true,
            click_effects: false,
            microphone_id: String::new(),
            output_id: String::new(),
            webcam_id: String::new(),
            monitor: 0,
            countdown: 3,
            naming: "Wyzar_{data}_{hora}".into(),
            subfolders: "none".into(),
            project_name: crate::i18n::tr("Meu projeto").into(),
            recovery: true,
            segment_seconds: 10,
            notifications: true,
            tray: false,
            startup: false,
            hardware: true,
            gpu_resize: true,
            low_space_gb: 5,
            storage_limit_gb: 0,
            webcam_size: 22,
            webcam_position: 3,
            window_width: 1560,
            window_height: 900,
            last_page: "home".into(),
            last_seen_version: String::new(),
            notified_version: String::new(),
            install_storage_request: String::new(),
            install_language_request: String::new(),
            record_hotkey: 0x77,
            pause_hotkey: 0x78,
            stop_hotkey: 0x79,
            screenshot_hotkey: 0x2c,
            screenshot_mode: 0,
            screenshot_delay: 0,
            screenshot_clipboard: false,
            screenshot_cursor: false,
            webcam_shape: 1,
            webcam_mirror: false,
            voice_filter: true,
            storage_notice: None,
        }
    }
    pub fn load(data_dir: PathBuf) -> Self {
        let mut s = Self::defaults(data_dir);
        if let Ok(v) = persistence::read(&s.file()) {
            s.apply(&v);
        }
        s
    }
    pub fn file(&self) -> PathBuf {
        self.data_dir.join("settings.ini")
    }
    pub fn load_checked(data_dir: PathBuf) -> AppResult<Self> {
        let mut settings = Self::defaults(data_dir);
        if settings.file().exists() {
            let values = persistence::read(&settings.file())?;
            if values
                .get("version")
                .is_some_and(|v| !["1", "2", "3"].contains(&v.as_str()))
            {
                return Err(crate::i18n::tr("As preferências pertencem a uma versão mais recente. O arquivo original foi preservado.").into());
            }
            settings.apply(&values);
        }
        Ok(settings)
    }
    fn apply(&mut self, v: &BTreeMap<String, String>) {
        // Version 1 is migrated by additive defaults; never reinterpret an unknown format.
        if v.get("version")
            .is_some_and(|v| !["1", "2", "3"].contains(&v.as_str()))
        {
            return;
        }
        self.appearance = appearance::Appearance::load(v);
        self.language = crate::i18n::forced()
            .or_else(|| {
                v.get("language")
                    .and_then(|s| crate::i18n::Language::parse(s))
            })
            .unwrap_or_default();
        macro_rules! paths { ($($f:ident),*)=>{$(if let Some(p)=v.get(stringify!($f)){if Path::new(p).is_absolute(){self.$f=p.into();}})*}; }
        macro_rules! nums { ($($f:ident),*)=>{$(if let Some(n)=v.get(stringify!($f)).and_then(|n|n.parse().ok()){self.$f=n;})*}; }
        macro_rules! bools { ($($f:ident),*)=>{$(if let Some(b)=v.get(stringify!($f)){self.$f=b=="true";})*}; }
        macro_rules! strings { ($($f:ident),*)=>{$(if let Some(n)=v.get(stringify!($f)){self.$f=n.clone();})*}; }
        paths!(videos, audios, screenshots, cache, exports, projects);
        nums!(
            recording_mode,
            mic_gain,
            system_gain,
            fps,
            width,
            height,
            bitrate,
            monitor,
            countdown,
            segment_seconds,
            low_space_gb,
            storage_limit_gb,
            webcam_size,
            webcam_position,
            window_width,
            window_height,
            record_hotkey,
            pause_hotkey,
            stop_hotkey,
            screenshot_hotkey,
            screenshot_mode,
            screenshot_delay,
            webcam_shape
        );
        bools!(
            audio_limiter,
            audio_sources,
            auto_fallback,
            microphone,
            system_audio,
            webcam,
            cursor,
            click_effects,
            recovery,
            notifications,
            tray,
            startup,
            hardware,
            gpu_resize,
            screenshot_clipboard,
            screenshot_cursor,
            webcam_mirror,
            voice_filter
        );
        strings!(
            naming,
            subfolders,
            project_name,
            microphone_id,
            output_id,
            webcam_id,
            last_page,
            last_seen_version,
            notified_version,
            install_storage_request,
            install_language_request
        );
        self.source = Source::parse(v.get("source").map(String::as_str).unwrap_or("display"));
        if ![24, 30, 60, 120].contains(&self.fps) {
            self.fps = 60;
        }
        self.width = self.width.clamp(320, 3840) & !1;
        self.height = self.height.clamp(240, 2160) & !1;
        self.bitrate = self.bitrate.clamp(1_000_000, 100_000_000);
        self.countdown = self.countdown.min(10);
        self.segment_seconds = self.segment_seconds.clamp(2, 60);
        self.webcam_size = self.webcam_size.clamp(10, 50);
        self.webcam_position = self.webcam_position.min(3);
        self.screenshot_mode = self.screenshot_mode.min(3);
        self.screenshot_delay = self.screenshot_delay.min(10);
        self.webcam_shape = self.webcam_shape.min(2);
        let keys = self.shortcut_keys();
        if shortcuts::validate(&keys).is_err() {
            [
                self.record_hotkey,
                self.pause_hotkey,
                self.stop_hotkey,
                self.screenshot_hotkey,
            ] = shortcuts::DEFAULTS;
        }
        self.recording_mode = self.recording_mode.min(3);
        self.mic_gain = self.mic_gain.min(200);
        self.system_gain = self.system_gain.min(200);
        self.window_width = self.window_width.clamp(1100, 3840);
        self.window_height = self.window_height.clamp(700, 2160);
        if !valid_pattern(&self.naming) {
            self.naming = "Wyzar_{data}_{hora}".into();
        }
        if !valid_pattern(&self.project_name) {
            self.project_name = crate::i18n::tr("Meu projeto").into();
        }
    }
    pub fn values(&self) -> BTreeMap<String, String> {
        let mut v = BTreeMap::from([
            ("version".into(), "3".into()),
            ("source".into(), self.source.key().into()),
            ("language".into(), self.language.code().into()),
        ]);
        macro_rules! fields { ($($f:ident),*)=>{$(v.insert(stringify!($f).into(),self.$f.to_string());)*}; }
        for (k, p) in [
            ("videos", &self.videos),
            ("audios", &self.audios),
            ("screenshots", &self.screenshots),
            ("cache", &self.cache),
            ("exports", &self.exports),
            ("projects", &self.projects),
        ] {
            v.insert(k.into(), p.to_string_lossy().into_owned());
        }
        fields!(
            recording_mode,
            mic_gain,
            system_gain,
            audio_sources,
            audio_limiter,
            auto_fallback,
            fps,
            width,
            height,
            bitrate,
            microphone,
            system_audio,
            webcam,
            cursor,
            click_effects,
            microphone_id,
            output_id,
            webcam_id,
            monitor,
            countdown,
            naming,
            subfolders,
            project_name,
            recovery,
            segment_seconds,
            notifications,
            tray,
            startup,
            hardware,
            gpu_resize,
            low_space_gb,
            storage_limit_gb,
            webcam_size,
            webcam_position,
            window_width,
            window_height,
            last_page,
            last_seen_version,
            notified_version,
            install_storage_request,
            install_language_request,
            record_hotkey,
            pause_hotkey,
            stop_hotkey,
            screenshot_hotkey,
            screenshot_mode,
            screenshot_delay,
            screenshot_clipboard,
            screenshot_cursor,
            webcam_shape,
            webcam_mirror,
            voice_filter
        );
        self.appearance.store(&mut v);
        v
    }
    pub fn save(&self) -> AppResult<()> {
        persistence::write(&self.file(), &self.values())
    }
    pub fn shortcut_keys(&self) -> [u32; 4] {
        [
            self.record_hotkey,
            self.pause_hotkey,
            self.stop_hotkey,
            self.screenshot_hotkey,
        ]
    }
    pub fn apply_profile(&mut self, v: &BTreeMap<String, String>) {
        self.apply(v);
    }
    pub fn prepare(&self) -> AppResult<()> {
        for path in [
            &self.videos,
            &self.audios,
            &self.screenshots,
            &self.cache,
            &self.exports,
            &self.projects,
        ] {
            persistence::test_folder(path)?;
        }
        Ok(())
    }
    pub fn save_profile(&self) -> AppResult<PathBuf> {
        let p = self
            .data_dir
            .join("Profiles")
            .join(format!("Perfil_{}.ini", unique_stamp()));
        persistence::write(&p, &self.values())?;
        Ok(p)
    }
    pub fn output_path(&self, folder: &Path, extension: &str) -> AppResult<PathBuf> {
        if !valid_pattern(&self.naming) {
            return Err(crate::i18n::tr(
                "Nome inválido. Use apenas um nome, sem separadores de pasta.",
            )
            .into());
        }
        let (date, time) = crate::platform::windows::files::local_date_time();
        let folder = match self.subfolders.as_str() {
            "date" => folder.join(&date),
            "type" => folder.join(extension.to_uppercase()),
            "project" => {
                if !valid_pattern(&self.project_name) {
                    return Err(crate::i18n::tr("Nome do projeto inválido").into());
                }
                folder.join(&self.project_name)
            }
            _ => folder.to_owned(),
        };
        persistence::test_folder(&folder)?;
        for n in 1..100_000 {
            let name = self
                .naming
                .replace("{data}", &date)
                .replace("{hora}", &time)
                .replace("{contador}", &format!("{n:04}"));
            let suffix = if n == 1 || self.naming.contains("{contador}") {
                String::new()
            } else {
                format!("_{n}")
            };
            let p = folder.join(format!("{name}{suffix}.{extension}"));
            if !p.exists() {
                return Ok(p);
            }
        }
        Err(crate::i18n::tr("Não foi possível gerar um nome único").into())
    }
}
pub fn valid_pattern(s: &str) -> bool {
    let stem = s.split('.').next().unwrap_or("").to_ascii_uppercase();
    !s.is_empty()
        && s.len() <= 140
        && !s
            .chars()
            .any(|c| c.is_control() || "<>:\"/\\|?*".contains(c))
        && !s.ends_with(['.', ' '])
        && ![
            "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
            "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ]
        .contains(&stem.as_str())
}
pub fn unique_stamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn names_cannot_escape() {
        for n in ["../x", "C:\\x", "x/y", "NUL", "COM1.mp4", "x.", "a\nb", ""] {
            assert!(!valid_pattern(n), "{n}");
        }
        assert!(valid_pattern("Wyzar_{data}_{hora}"));
    }
    #[test]
    fn clamps_untrusted_values() {
        let mut s = Settings::defaults(PathBuf::from("D:\\test"));
        s.apply(&BTreeMap::from([
            ("fps".into(), "999".into()),
            ("width".into(), "4294967295".into()),
            ("countdown".into(), "500".into()),
        ]));
        assert_eq!(s.fps, 60);
        assert_eq!(s.width, 3840);
        assert_eq!(s.countdown, 10);
    }
    #[test]
    fn all_languages_roundtrip_and_legacy_defaults_to_portuguese() {
        for language in crate::i18n::Language::ALL {
            let mut original = Settings::defaults(PathBuf::from("validation/language-fixture"));
            original.language = language;
            original.naming = "Minha captura 日本語_{contador}".into();
            original.install_language_request = "once".into();
            let mut reloaded = Settings::defaults(original.data_dir.clone());
            reloaded.apply(&original.values());
            assert_eq!(reloaded.language, language);
            assert_eq!(reloaded.naming, original.naming);
            assert_eq!(reloaded.install_language_request, "once");
        }
        let mut legacy = Settings::defaults(PathBuf::from("validation/language-fixture"));
        legacy.language = crate::i18n::Language::Ja;
        legacy.apply(&BTreeMap::new());
        assert_eq!(legacy.language, crate::i18n::Language::PtBr);
    }
    #[test]
    fn migrates_legacy_preferences_and_preserves_unknown_versions() {
        let root = std::env::current_dir()
            .unwrap()
            .join("validation/unit")
            .join(format!("settings-{}", unique_stamp()));
        let old = BTreeMap::from([
            ("version".into(), "1".into()),
            ("naming".into(), "Antigo_{contador}".into()),
            ("fps".into(), "30".into()),
        ]);
        persistence::write(&root.join("settings.ini"), &old).unwrap();
        let mut settings = Settings::load_checked(root.clone()).unwrap();
        assert_eq!(settings.fps, 30);
        assert_eq!(settings.naming, "Antigo_{contador}");
        assert_eq!(settings.segment_seconds, 10);
        settings.last_seen_version = "0.2.0".into();
        settings.save().unwrap();
        assert_eq!(
            Settings::load_checked(root.clone())
                .unwrap()
                .last_seen_version,
            "0.2.0"
        );
        let future = BTreeMap::from([("version".into(), "999".into())]);
        persistence::write(&root.join("settings.ini"), &future).unwrap();
        assert!(Settings::load_checked(root.clone()).is_err());
        assert_eq!(
            persistence::read(&root.join("settings.ini")).unwrap(),
            future
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
