//! First-use defaults and the explicit storage choice made in Setup.
//! Existing preferences are preserved unless a new installer request is present.
use super::Settings;
use crate::{core::AppResult, encoder::capabilities::Inventory, persistence};
use std::path::{Path, PathBuf};

pub fn media_paths(root: &Path) -> AppResult<[PathBuf; 5]> {
    if !root.is_absolute() {
        return Err(crate::i18n::tr("Escolha uma pasta de mídia com caminho absoluto.").into());
    }
    Ok([
        root.to_owned(),
        root.join("Áudios"),
        root.join("Capturas"),
        root.join("Exportações"),
        root.join("Projetos"),
    ])
}
pub fn prepare_media(root: &Path) -> AppResult<()> {
    for path in media_paths(root)? {
        persistence::test_folder(&path)?;
    }
    Ok(())
}
pub fn set_media(settings: &mut Settings, root: &Path) -> AppResult<()> {
    set_paths(settings, media_paths(root)?);
    Ok(())
}
pub fn paths(settings: &Settings) -> [PathBuf; 5] {
    [
        settings.videos.clone(),
        settings.audios.clone(),
        settings.screenshots.clone(),
        settings.exports.clone(),
        settings.projects.clone(),
    ]
}
fn set_paths(settings: &mut Settings, paths: [PathBuf; 5]) {
    let [videos, audios, screenshots, exports, projects] = paths;
    settings.videos = videos;
    settings.audios = audios;
    settings.screenshots = screenshots;
    settings.exports = exports;
    settings.projects = projects;
}
pub fn restore_windows_media(settings: &mut Settings) -> AppResult<()> {
    let defaults = crate::platform::windows::files::media_folders();
    crate::platform::windows::files::prepare_known_media(&defaults);
    restore_media_paths(settings, defaults)
}
pub(crate) fn restore_media_paths(
    settings: &mut Settings,
    defaults: [PathBuf; 5],
) -> AppResult<()> {
    for path in &defaults {
        persistence::test_folder(path)?;
    }
    set_paths(settings, defaults);
    Ok(())
}
pub fn windows_media_paths(
    videos: PathBuf,
    music: PathBuf,
    pictures: PathBuf,
    documents: PathBuf,
) -> [PathBuf; 5] {
    let folder = |p: PathBuf| {
        if p.as_os_str().is_empty() {
            p
        } else {
            p.join("WyzarCapture")
        }
    };
    let videos = folder(videos);
    let projects = folder(documents);
    [
        videos.clone(),
        folder(music),
        folder(pictures),
        if videos.as_os_str().is_empty() {
            PathBuf::new()
        } else {
            videos.join("Exports")
        },
        if projects.as_os_str().is_empty() {
            PathBuf::new()
        } else {
            projects.join("Projects")
        },
    ]
}
// Match the complete old automatic layout, never an individual user-selected
// destination. Media bytes remain in place; only future destinations change.
fn obsolete_local_defaults(settings: &Settings) -> bool {
    media_paths(&settings.data_dir.join("Media")).is_ok_and(|old| old == paths(settings))
}
fn backup_preferences(settings: &Settings) -> AppResult<()> {
    use std::io::Write;
    if !settings.file().is_file() {
        return Ok(());
    }
    let bytes = std::fs::read(settings.file()).map_err(|e| e.to_string())?;
    let target = settings.data_dir.join(format!(
        "settings-before-media-0.6.3-{}.ini",
        crate::settings::unique_stamp()
    ));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target)
        .map_err(|e| e.to_string())?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())
}
pub fn detected_profile(settings: &mut Settings, inventory: &Inventory) {
    let large = inventory
        .monitors
        .first()
        .is_some_and(|&(w, h)| w >= 1920 && h >= 1080);
    (settings.width, settings.height) = if large && inventory.hardware() {
        (1920, 1080)
    } else {
        (1280, 720)
    };
    settings.hardware = inventory.hardware();
    settings.fps = if inventory.hardware()
        && inventory.logical_cpus >= 4
        && inventory.dedicated_video_bytes >= 2 * 1024 * 1024 * 1024
        && inventory.primary_refresh_hz >= 60
    {
        60
    } else {
        30
    };
    settings.bitrate =
        crate::encoder::recommended_bitrate(settings.width, settings.height, settings.fps);
    settings.monitor = 0;
}
pub fn initialize(
    settings: &mut Settings,
    inventory: &Inventory,
    request: Option<(String, PathBuf)>,
) -> AppResult<bool> {
    initialize_preferences(settings, inventory, request, None)
}
pub fn initialize_preferences(
    settings: &mut Settings,
    inventory: &Inventory,
    request: Option<(String, PathBuf)>,
    language: Option<(String, crate::i18n::Language)>,
) -> AppResult<bool> {
    initialize_using_media(
        settings,
        inventory,
        request,
        language,
        crate::platform::windows::files::media_folders(),
    )
}
fn initialize_using_media(
    settings: &mut Settings,
    inventory: &Inventory,
    request: Option<(String, PathBuf)>,
    language: Option<(String, crate::i18n::Language)>,
    defaults: [PathBuf; 5],
) -> AppResult<bool> {
    let fresh = !settings.file().is_file();
    let mut next = settings.clone();
    if fresh {
        detected_profile(&mut next, inventory);
    }
    let mut changed = fresh;
    let migrated = !fresh && request.is_none() && obsolete_local_defaults(&next);
    if migrated {
        backup_preferences(&next)?;
        set_paths(&mut next, defaults);
        next.storage_notice = Some(crate::i18n::tr("Os novos arquivos serão salvos nas pastas de mídia do Windows. Os arquivos anteriores foram preservados; uma cópia das preferências antigas está na pasta de dados.").into());
        changed = true;
    }
    if let Some((identity, language)) = language {
        if identity != next.install_language_request {
            next.language = language;
            next.install_language_request = identity;
            changed = true;
        }
    }
    if let Some((identity, root)) = request {
        if identity != next.install_storage_request {
            set_media(&mut next, &root)?;
            next.install_storage_request = identity;
            changed = true;
        }
    }
    if changed {
        crate::platform::windows::files::prepare_known_media(&paths(&next));
        if let Err(error) = next.prepare() {
            // Leave Windows destinations visible/configurable when unavailable.
            // Never silently redirect user media to AppData or another volume.
            if (!fresh && !migrated) || (fresh && !next.install_storage_request.is_empty()) {
                return Err(error);
            }
            next.storage_notice = Some(crate::i18n::format_text(
                crate::i18n::tr(
                    "Confira as pastas em Armazenamento. O Windows não disponibilizou um dos destinos: {e}",
                ),
                &[("e", error)],
            ));
        }
        next.save()?;
        *settings = next;
    }
    Ok(changed)
}
#[cfg(test)]
mod tests {
    use super::*;
    static FIXTURE_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    #[test]
    fn old_fallback_migrates_once_preserving_media_and_exact_preferences_backup() {
        let (root, mut settings) = isolated();
        let old_root = settings.data_dir.join("Media");
        set_media(&mut settings, &old_root).unwrap();
        settings.prepare().unwrap();
        let old_video = old_root.join("user-video.mp4");
        std::fs::write(&old_video, b"user-sentinel").unwrap();
        settings.save().unwrap();
        let old_bytes = std::fs::read(settings.file()).unwrap();
        let defaults = windows_media_paths(
            root.join("Videos"),
            root.join("Music"),
            root.join("Pictures"),
            root.join("Documents"),
        );
        assert!(
            initialize_using_media(
                &mut settings,
                &Inventory::default(),
                None,
                None,
                defaults.clone()
            )
            .unwrap()
        );
        assert_eq!(paths(&settings), defaults);
        assert_eq!(std::fs::read(&old_video).unwrap(), b"user-sentinel");
        let backup = std::fs::read_dir(&settings.data_dir)
            .unwrap()
            .filter_map(Result::ok)
            .find(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("settings-before-media-")
            })
            .unwrap();
        assert_eq!(std::fs::read(backup.path()).unwrap(), old_bytes);
        settings.audios = root.join("Subsequent user choice");
        settings.save().unwrap();
        let bytes = std::fs::read(settings.file()).unwrap();
        assert!(
            !initialize_using_media(&mut settings, &Inventory::default(), None, None, defaults)
                .unwrap()
        );
        assert_eq!(std::fs::read(settings.file()).unwrap(), bytes);
    }
    #[test]
    fn windows_types_follow_redirected_roots_without_using_data_directory() {
        let (root, mut settings) = isolated();
        let expected = windows_media_paths(
            root.join("Redirected Videos"),
            root.join("Redirected Music"),
            root.join("Redirected Pictures"),
            root.join("Redirected Documents"),
        );
        restore_media_paths(&mut settings, expected.clone()).unwrap();
        assert_eq!(paths(&settings), expected);
        assert!(settings.videos.ends_with("Redirected Videos/WyzarCapture"));
        assert!(
            settings
                .screenshots
                .ends_with("Redirected Pictures/WyzarCapture")
        );
        assert!(settings.audios.ends_with("Redirected Music/WyzarCapture"));
        assert!(
            settings
                .projects
                .ends_with("Redirected Documents/WyzarCapture/Projects")
        );
        assert!(
            !paths(&settings)
                .iter()
                .any(|p| p.starts_with(&settings.data_dir))
        );
        let old = paths(&settings);
        let sentinel = root.join("unavailable-new-folder");
        std::fs::write(&sentinel, b"preserve").unwrap();
        let invalid = windows_media_paths(
            root.join("V"),
            root.join("M"),
            sentinel.join("Pictures"),
            root.join("D"),
        );
        assert!(restore_media_paths(&mut settings, invalid).is_err());
        assert_eq!(paths(&settings), old);
        assert_eq!(std::fs::read(sentinel).unwrap(), b"preserve");
    }
    #[test]
    fn only_the_complete_obsolete_fallback_is_migratable() {
        let (_root, mut settings) = isolated();
        let fallback = settings.data_dir.join("Media");
        set_media(&mut settings, &fallback).unwrap();
        assert!(obsolete_local_defaults(&settings));
        settings.screenshots = settings.data_dir.join("My chosen pictures");
        assert!(!obsolete_local_defaults(&settings));
        settings.save().unwrap();
        let bytes = std::fs::read(settings.file()).unwrap();
        backup_preferences(&settings).unwrap();
        let backup = std::fs::read_dir(&settings.data_dir)
            .unwrap()
            .filter_map(Result::ok)
            .find(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("settings-before-media-")
            })
            .unwrap();
        assert_eq!(std::fs::read(backup.path()).unwrap(), bytes);
        assert_eq!(std::fs::read(settings.file()).unwrap(), bytes);
    }
    #[test]
    fn installer_language_is_applied_once_and_failed_storage_preserves_preferences() {
        let (root, mut settings) = isolated();
        let language = ("language-request".to_owned(), crate::i18n::Language::En);
        initialize_preferences(
            &mut settings,
            &Inventory::default(),
            None,
            Some(language.clone()),
        )
        .unwrap();
        assert_eq!(settings.language, crate::i18n::Language::En);
        settings.language = crate::i18n::Language::Ja;
        settings.save().unwrap();
        assert!(
            !initialize_preferences(&mut settings, &Inventory::default(), None, Some(language))
                .unwrap()
        );
        assert_eq!(settings.language, crate::i18n::Language::Ja);
        let before = std::fs::read(settings.file()).unwrap();
        let blocked = root.join("file-not-directory");
        std::fs::write(&blocked, b"preserve").unwrap();
        assert!(
            initialize_preferences(
                &mut settings,
                &Inventory::default(),
                Some(("bad-media".into(), blocked.join("media"))),
                Some(("new-language".into(), crate::i18n::Language::Es))
            )
            .is_err()
        );
        assert_eq!(std::fs::read(settings.file()).unwrap(), before);
        assert_eq!(settings.language, crate::i18n::Language::Ja);
    }
    #[test]
    fn initial_profile_and_recommendation_match_detected_monitor_and_encoder() {
        let mut settings = Settings::defaults(PathBuf::from("D:\\validation-fixture"));
        let mut inventory = Inventory::default();
        detected_profile(&mut settings, &inventory);
        assert_eq!(
            (
                settings.width,
                settings.height,
                settings.fps,
                settings.hardware
            ),
            (1280, 720, 30, false)
        );
        assert!(inventory.recommendation().starts_with("720p"));
        inventory.monitors = vec![(1366, 768)];
        inventory.logical_cpus = 8;
        inventory.primary_refresh_hz = 60;
        inventory.dedicated_video_bytes = 4 * 1024 * 1024 * 1024;
        inventory
            .encoders
            .push(crate::encoder::capabilities::Transform {
                name: "synthetic hardware".into(),
                clsid: "fixture".into(),
                hardware: true,
            });
        detected_profile(&mut settings, &inventory);
        assert_eq!(
            (settings.width, settings.height, settings.fps),
            (1280, 720, 60)
        );
        assert!(inventory.recommendation().starts_with("720p · 60"));
        inventory.monitors = vec![(1920, 1080)];
        detected_profile(&mut settings, &inventory);
        assert_eq!(
            (settings.width, settings.height, settings.fps),
            (1920, 1080, 60)
        );
    }
    fn isolated() -> (PathBuf, Settings) {
        let root = std::env::current_dir()
            .unwrap()
            .join("validation/unit")
            .join(format!(
                "first-use-{}-{}-{}",
                std::process::id(),
                super::super::unique_stamp(),
                FIXTURE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
        let mut settings = Settings::defaults(root.join("Data"));
        set_media(
            &mut settings,
            &root.join("Mídias com espaços — ação 日本語"),
        )
        .unwrap();
        settings.cache = root.join("Cache");
        (root, settings)
    }
    #[test]
    fn first_use_creates_writable_unicode_destinations_and_preserves_preferences() {
        let (root, mut settings) = isolated();
        assert!(initialize(&mut settings, &Inventory::default(), None).unwrap());
        assert_eq!(
            (settings.width, settings.height, settings.fps),
            (1280, 720, 30)
        );
        settings.fps = 24;
        settings.videos = root.join("Escolha manual");
        settings.save().unwrap();
        let before = std::fs::read(settings.file()).unwrap();
        assert!(!initialize(&mut settings, &Inventory::default(), None).unwrap());
        assert_eq!(std::fs::read(settings.file()).unwrap(), before);
        assert_eq!(settings.fps, 24);
    }
    #[test]
    fn unavailable_initial_default_stays_visible_without_appdata_fallback() {
        let (root, mut settings) = isolated();
        let sentinel = root.join("unavailable-default");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&sentinel, b"keep").unwrap();
        set_media(&mut settings, &sentinel.join("Media")).unwrap();
        initialize(&mut settings, &Inventory::default(), None).unwrap();
        assert_eq!(settings.videos, sentinel.join("Media"));
        assert!(settings.storage_notice.is_some());
        assert!(settings.prepare().is_err());
        assert!(!settings.data_dir.join("Media").exists());
        assert_eq!(std::fs::read(sentinel).unwrap(), b"keep");
    }
    #[test]
    fn explicit_installer_choice_is_applied_once_without_replacing_later_user_choice() {
        let (root, mut settings) = isolated();
        let request = ("0.5.3|choice".to_owned(), root.join("Escolhido no Setup"));
        initialize(&mut settings, &Inventory::default(), Some(request.clone())).unwrap();
        assert_eq!(settings.videos, request.1);
        settings.videos = root.join("Escolhido depois no aplicativo");
        settings.save().unwrap();
        assert!(!initialize(&mut settings, &Inventory::default(), Some(request)).unwrap());
        assert!(settings.videos.ends_with("Escolhido depois no aplicativo"));
    }
    #[test]
    fn failed_storage_request_preserves_settings_bytes_and_original_destinations() {
        let (root, mut settings) = isolated();
        initialize(&mut settings, &Inventory::default(), None).unwrap();
        let before = std::fs::read(settings.file()).unwrap();
        let old = settings.videos.clone();
        let blocked = root.join("arquivo no lugar de pasta");
        std::fs::write(&blocked, b"keep").unwrap();
        assert!(
            initialize(
                &mut settings,
                &Inventory::default(),
                Some(("blocked".into(), blocked.join("Mídias")))
            )
            .is_err()
        );
        assert_eq!(std::fs::read(settings.file()).unwrap(), before);
        assert_eq!(settings.videos, old);
        assert_eq!(std::fs::read(blocked).unwrap(), b"keep");
        assert!(media_paths(Path::new("relative")).is_err());
    }
}
