#![cfg_attr(not(test), windows_subsystem = "windows")]
mod audio;
mod capture;
mod container;
mod core;
mod diagnostics;
mod editor;
mod encoder;
mod i18n;
#[allow(dead_code)]
mod installer;
mod library;
mod license;
mod persistence;
mod platform;
mod recovery;
mod settings;
mod ui;
mod updates;
mod validation;
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let data = if args
        .get(1)
        .is_some_and(|v| v.starts_with("--") && v != "--language" && v != "--news")
    {
        args.get(2)
            .map(std::path::PathBuf::from)
            .unwrap_or_else(platform::windows::files::data_folder)
    } else {
        platform::windows::files::data_folder()
    };
    if !args.get(1).is_some_and(|v| {
        v == "--endurance"
            || v == "--recovery-test"
            || v == "--audio-qa"
            || v == "--hardening-qa"
            || v == "--startup-qa"
            || v == "--usability-qa"
            || v == "--i18n-qa"
            || v == "--accessibility-qa"
    }) {
        diagnostics::init(&data);
    }
    let result = (|| -> core::AppResult<()> {
        i18n::configure_cli(&args)?;
        let _runtime = platform::windows::Runtime::new(false)?;
        if args.get(1).is_some_and(|s| s == "--accessibility-qa") {
            return validation::accessibility_qa(data.clone());
        }
        if args.get(1).is_some_and(|s| s == "--i18n-qa") {
            return validation::localization::run(data.clone());
        }
        if args.get(1).is_some_and(|s| s == "--hardening-qa") {
            return validation::hardening::run(data.clone());
        }
        if args.get(1).is_some_and(|s| s == "--startup-qa") {
            return validation::startup::run(data.clone());
        }
        if args.get(1).is_some_and(|s| s == "--usability-qa") {
            return validation::usability::run(data.clone());
        }
        if args.get(1).is_some_and(|s| s == "--encoder-test") {
            return validation::hardening::encoder_test(data.clone());
        }
        if args.get(1).is_some_and(|s| s == "--audio-qa") {
            return validation::audio_qa(data.clone());
        }
        if args.get(1).is_some_and(|s| s == "--performance-test") {
            let mut s = settings::Settings::defaults(data.join("Data"));
            s.cache = data.join("Cache");
            let report = encoder::benchmark::run(s)?;
            return std::fs::write(
                data.join("performance-result.json"),
                format!(
                    "{{\"passed\":true,\"report\":{:?},\"recommendation\":{:?}}}",
                    report.path, report.recommendation
                ),
            )
            .map_err(|e| e.to_string());
        }
        if args.get(1).is_some_and(|s| s == "--render-ux") {
            return validation::render_ux(
                data.clone(),
                args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1366),
                args.get(4).and_then(|s| s.parse().ok()).unwrap_or(768),
            );
        }
        if args.get(1).is_some_and(|s| s == "--editor-qa") {
            return validation::editor_qa(data.clone());
        }
        if args.get(1).is_some_and(|s| s == "--ux-native") {
            return validation::native_qa(data.clone());
        }
        if args.get(1).is_some_and(|s| s == "--source-loss-test") {
            return validation::source_loss_test(
                std::path::absolute(
                    args.get(2)
                        .ok_or(crate::i18n::tr("Diretório obrigatório"))?,
                )
                .map_err(|e| e.to_string())?,
            );
        }
        if args.get(1).is_some_and(|s| s == "--capabilities") {
            return validation::capabilities_test(
                std::path::absolute(
                    args.get(2)
                        .ok_or(crate::i18n::tr("Diretório obrigatório"))?,
                )
                .map_err(|e| e.to_string())?,
            );
        }
        if args.get(1).is_some_and(|s| s == "--gpu-test") {
            let root = std::path::absolute(
                args.get(2)
                    .ok_or(crate::i18n::tr("Diretório obrigatório"))?,
            )
            .map_err(|e| e.to_string())?;
            std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
            let frame = platform::windows::capture::validate_gpu_scaler()?;
            platform::windows::graphics::save_frame(&frame, &root.join("gpu-colors.png"))?;
            return std::fs::write(root.join("gpu-report.json"),"{\"passed\":true,\"d3d11_video_resize\":true,\"aspect_and_colors\":true,\"zero_copy\":false}").map_err(|e|e.to_string());
        }
        if args.get(1).is_some_and(|s| s == "--endurance") {
            let root = std::path::absolute(
                args.get(2)
                    .ok_or(crate::i18n::tr("Diretório isolado obrigatório"))?,
            )
            .map_err(|e| e.to_string())?;
            let number = |index: usize| {
                args.get(index)
                    .and_then(|v| v.parse::<u32>().ok())
                    .ok_or_else(|| {
                        crate::i18n::tr("Dimensões, FPS e duração são obrigatórios").to_string()
                    })
            };
            return validation::endurance(root, number(3)?, number(4)?, number(5)?, number(6)?);
        }
        if args
            .get(1)
            .is_some_and(|s| s == "--recovery-test" || s == "--crash-record")
        {
            let root = std::path::absolute(
                args.get(2)
                    .ok_or(crate::i18n::tr("Diretório isolado obrigatório"))?,
            )
            .map_err(|e| e.to_string())?;
            return if args[1] == "--recovery-test" {
                validation::recovery_test(root)
            } else {
                validation::crash_record(root)
            };
        }
        if args.get(1).is_some_and(|s| s == "--smoke") {
            let root = std::path::absolute(
                args.get(2)
                    .ok_or(crate::i18n::tr("Diretório isolado obrigatório"))?,
            )
            .map_err(|e| e.to_string())?;
            let mut s = settings::Settings::defaults(root.join("Data"));
            s.videos = root.join("Videos");
            s.audios = s.videos.join("Áudios");
            s.screenshots = root.join("Pictures");
            s.cache = root.join("Cache");
            s.exports = root.join("Exports");
            s.projects = root.join("Projects");
            if s.file().is_file() {
                s = settings::Settings::load_checked(s.data_dir.clone())?;
            }
            s.tray = false;
            s.startup = false;
            s.notifications = false;
            let seconds = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(2);
            if args.iter().any(|s| s == "--news") {
                platform::windows::host::open_news(s.clone(), Some(seconds))?;
            } else {
                platform::windows::host::smoke(s.clone(), seconds)?;
            }
            let reloaded = settings::Settings::load(s.data_dir.clone());
            if reloaded.videos != s.videos
                || reloaded.appearance != s.appearance
                || !reloaded.file().is_file()
            {
                return Err(crate::i18n::tr("Persistência após fechar a janela falhou").into());
            }
            if s.appearance.mode().compact()
                && (s.window_width != reloaded.window_width
                    || s.window_height != reloaded.window_height)
            {
                return Err(crate::i18n::tr(
                    "Reabrir compacto alterou o tamanho salvo da interface padrão",
                )
                .into());
            }
            return std::fs::write(root.join("smoke-report.json"), format!("{{\"passed\":true,\"version\":{:?},\"native_window\":true,\"settings_reload\":true,\"appearance_reload\":true,\"theme\":{},\"fps\":{},\"compact_mode\":{},\"compact_focus\":{},\"standard_width\":{},\"standard_height\":{}}}", updates::VERSION,s.appearance.theme,s.fps,s.appearance.compact,s.appearance.compact_focus,reloaded.window_width,reloaded.window_height)).map_err(|e| e.to_string());
        }
        if args.get(1).is_some_and(|s| s == "--validate") {
            return validation::run(
                args.get(2)
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| std::path::PathBuf::from("validation/runtime")),
            );
        }
        if args.get(1).is_some_and(|s| s == "--render-all") {
            return validation::render_at(
                args.get(2)
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| std::path::PathBuf::from("validation/screens")),
                args.get(3).and_then(|v| v.parse().ok()).unwrap_or(1600),
                args.get(4).and_then(|v| v.parse().ok()).unwrap_or(900),
            );
        }
        if args.get(1).is_some_and(|s| s == "--probe") {
            return validation::probe(
                args.get(2)
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| std::path::PathBuf::from("validation/native")),
            );
        }
        let data = platform::windows::files::data_folder();
        let mut s = settings::Settings::load_checked(data)?;
        i18n::set(s.language);
        let setup_notice = (|| -> core::AppResult<()> {
            let inventory = platform::windows::capabilities::inventory()?;
            let request = platform::windows::files::installed_media_request()?;
            let language = platform::windows::files::installed_language_request()?;
            settings::bootstrap::initialize_preferences(&mut s, &inventory, request, language)?;
            Ok(())
        })()
        .err();
        if let Some(language) = i18n::forced() {
            if s.language != language {
                s.language = language;
                s.save()?;
            }
        }
        i18n::set(s.language);
        std::fs::create_dir_all(&s.data_dir).map_err(|e| e.to_string())?;
        if args.iter().any(|a| a == "--news") {
            platform::windows::host::open_news(s, None)
        } else {
            platform::windows::host::run_with_notice(s, setup_notice)
        }
    })();
    if let Err(error) = result {
        let _ = std::fs::create_dir_all(&data);
        diagnostics::event("failure", diagnostics::code(&error));
        eprintln!(
            "{}",
            crate::i18n::format_text(
                crate::i18n::tr("WyzarCapture: {error}"),
                &[("error", error)]
            )
        );
        std::process::exit(1);
    }
}
