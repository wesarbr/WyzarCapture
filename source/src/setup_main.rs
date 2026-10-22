#![cfg_attr(not(test), windows_subsystem = "windows")]
#![allow(dead_code)]
mod audio;
mod capture;
mod container;
mod core;
mod diagnostics;
mod editor;
mod encoder;
mod i18n;
mod installer;
mod library;
mod license;
mod persistence;
mod platform;
mod recovery;
mod settings;
mod ui;
mod updates;
#[allow(unused_imports)]
mod validation;
include!(concat!(env!("OUT_DIR"), "/setup_payload.rs"));
fn main() {
    let result = run();
    if let Some(cleanup) = platform::windows::setup::finish_runner_cleanup() {
        let args: Vec<_> = std::env::args().collect();
        if let Some(path) = args
            .iter()
            .position(|s| s == "--report")
            .and_then(|i| args.get(i + 1))
        {
            let _ = std::fs::write(
                std::path::Path::new(path).with_extension("cleanup.ini"),
                cleanup,
            );
        }
    }
    if let Err(error) = result {
        let args: Vec<_> = std::env::args().collect();
        if let Some(i) = args.iter().position(|a| a == "--report") {
            if let Some(path) = args.get(i + 1) {
                let _ = std::fs::write(path, format!("{{\"passed\":false,\"error\":{error:?}}}"));
            }
        }
        if !args
            .iter()
            .any(|a| a == "--quiet" || a == "--qa" || a == "--render" || a == "--verify")
        {
            platform::windows::setup_host::message(&error);
        }
        std::process::exit(1);
    }
}
fn run() -> core::AppResult<()> {
    let _com = platform::windows::setup::Com::new()?;
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    i18n::configure_cli(&args)?;
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    if args.is_empty()
        && exe
            .file_name()
            .is_some_and(|n| n.eq_ignore_ascii_case("WyzarCaptureUninstall.exe"))
    {
        args.extend([
            "--uninstall".into(),
            "--target".into(),
            exe.parent()
                .ok_or(crate::i18n::tr("Desinstalador sem pasta"))?
                .to_string_lossy()
                .into_owned(),
        ]);
    }
    let value = |name: &str| {
        args.iter()
            .position(|v| v == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    platform::windows::setup::clean_old_runners(&exe);
    installer::bundle::clean_abandoned();
    if let Some(pid) = value("--wait-pid") {
        platform::windows::setup::wait_for_exit(
            pid.parse().map_err(|_| crate::i18n::tr("PID inválido"))?,
        )?;
    }
    if let Some(target) = value("--qa-hold-lock") {
        let target = std::path::PathBuf::from(target);
        if !target.components().any(|c| c.as_os_str() == "validation") {
            return Err("Lock QA exige validation".into());
        }
        let _metadata_lock = if let Some(file) = value("--qa-context") {
            let ctx = installer::engine::Context::test_file(std::path::Path::new(&file), &target)?;
            Some(platform::windows::setup::DestinationLock::metadata(
                &ctx.registration,
            )?)
        } else {
            None
        };
        let _lock = platform::windows::setup::DestinationLock::acquire(&target)?;
        let report = value("--report").ok_or("Relatório QA obrigatório")?;
        std::fs::write(report, "{\"locked\":true}").map_err(|e| e.to_string())?;
        std::thread::sleep(std::time::Duration::from_secs(3));
        return Ok(());
    }
    let uninstall = args.iter().any(|a| a == "--uninstall");
    let extracted = if !uninstall && value("--package").is_none() {
        Some(installer::bundle::extract(&exe, EMBEDDED_MANIFEST)?)
    } else {
        None
    };
    let package = value("--package")
        .map(std::path::PathBuf::from)
        .or_else(|| extracted.as_ref().map(|p| p.root.clone()))
        .unwrap_or_else(|| exe.parent().unwrap().into());
    if args.first().is_some_and(|s| s == "--qa") {
        return installer::qa::run(
            std::path::PathBuf::from(args.get(1).ok_or("Raiz de QA obrigatória")?),
            &package,
            &exe,
            EMBEDDED_MANIFEST,
            value("--old-app").map(Into::into),
        );
    }
    if args.first().is_some_and(|s| s == "--render") {
        return installer::view::render_matrix(
            std::path::PathBuf::from(
                args.get(1)
                    .ok_or(crate::i18n::tr("Raiz de renders obrigatória"))?,
            ),
            &package,
            EMBEDDED_MANIFEST,
            value("--trace").map(Into::into),
        );
    }
    let scope = if value("--scope").as_deref() == Some("machine") {
        installer::Scope::Machine
    } else {
        installer::Scope::User
    };
    let mut options = installer::Options {
        language: i18n::forced(),
        media_root: value("--media-root").map(Into::into),
        destination: value("--target")
            .map(Into::into)
            .unwrap_or_else(|| platform::windows::setup::default_destination(scope)),
        scope,
        desktop: args.iter().any(|a| a == "--desktop"),
        start_menu: args.iter().any(|a| a == "--menu"),
        startup: args.iter().any(|a| a == "--startup"),
        allow_downgrade: args.iter().any(|a| a == "--allow-downgrade"),
        close_running: args.iter().any(|a| a == "--close-app"),
        move_existing: args.iter().any(|a| a == "--move-existing"),
    };
    if uninstall {
        let state = installer::engine::read_uninstall_state(&options.destination)?
            .ok_or(crate::i18n::tr("Instalação não encontrada"))?;
        options.scope = if state.get("scope").map(String::as_str) == Some("machine") {
            installer::Scope::Machine
        } else {
            installer::Scope::User
        };
        options.scope.ensure_available()?;
        if exe.parent() == Some(options.destination.as_path()) {
            installer::engine::verified_owned_file(
                &options.destination,
                "WyzarCaptureUninstall.exe",
            )?;
            let cache = std::env::temp_dir().join(format!(
                "WyzarCapture-Uninstall-{}",
                settings::unique_stamp()
            ));
            installer::guard(&cache)?;
            std::fs::create_dir(&cache).map_err(|e| e.to_string())?;
            let runner = cache.join("WyzarCaptureSetup.exe");
            std::fs::copy(&exe, &runner).map_err(|e| e.to_string())?;
            let hash = platform::windows::setup::hash_file(&exe)?;
            if platform::windows::setup::hash_file(&runner)? != hash {
                return Err(crate::i18n::tr(
                    "Falha ao validar a cópia temporária do desinstalador",
                )
                .into());
            }
            persistence::write(
                &cache.join(".wyzar-uninstall-cache"),
                &std::collections::BTreeMap::from([
                    ("product".into(), "WyzarCapture.UninstallRunner".into()),
                    ("schema".into(), "2".into()),
                    ("sha256".into(), hash),
                    (
                        "bytes".into(),
                        std::fs::metadata(&runner)
                            .map_err(|e| e.to_string())?
                            .len()
                            .to_string(),
                    ),
                ]),
            )?;
            let mut forwarded = args.clone();
            forwarded.push("--runner".into());
            forwarded.extend(["--wait-pid".into(), std::process::id().to_string()]);
            platform::windows::setup::launch(&runner, &forwarded)?;
            return Ok(());
        }
    }
    options.scope.ensure_available()?;
    let manifest = if uninstall {
        None
    } else {
        Some(installer::payload::Manifest::trusted(
            &package,
            EMBEDDED_MANIFEST,
        )?)
    };
    if args.iter().any(|a| a == "--verify") {
        let m = manifest
            .as_ref()
            .ok_or(crate::i18n::tr("Payload ausente"))?;
        for f in &m.files {
            m.verify_file(&package.join("payload"), f)?;
        }
        if let Some(path) = value("--report") {
            std::fs::write(
                path,
                format!(
                    "{{\"passed\":true,\"version\":{:?},\"files\":{},\"bytes\":{},\"self_contained\":{},\"payload_mode\":{:?},\"all_users_enabled\":{},\"binary_sha256\":{:?}}}",
                    m.version,
                    m.files.len(),
                    m.size(),
                    extracted.is_some(),
                    extracted.as_ref().map(|p| p.mode).unwrap_or("external"),
                    cfg!(feature = "experimental-all-users"),
                    m.files.iter().find(|f| f.name == "WyzarCapture.exe").ok_or(crate::i18n::tr("Aplicativo ausente"))?.hash
                ),
            )
            .map_err(|e| e.to_string())?;
        }
        return Ok(());
    }
    if args.iter().any(|a| a == "--quiet") {
        if !args.iter().any(|a| a == "--yes") {
            return Err(crate::i18n::tr("Operação silenciosa exige --yes explícito").into());
        }
        let ctx = if let Some(test_file) = value("--qa-context") {
            if options.scope != installer::Scope::User {
                return Err("QA de remoção limitado a HKCU".into());
            }
            installer::engine::Context::test_file(
                std::path::Path::new(&test_file),
                &options.destination,
            )?
        } else {
            installer::engine::Context::real(options.scope)?
        };
        if uninstall {
            installer::engine::uninstall(
                &options.destination,
                &ctx,
                options.close_running,
                |_| {},
            )?;
            let data = platform::windows::files::data_folder();
            let settings = settings::Settings::load(data.clone());
            for arg in ["--remove-preferences", "--remove-cache", "--remove-logs"] {
                if !args.iter().any(|a| a == arg) {
                    continue;
                }
                let plan = match arg {
                    "--remove-preferences" => persistence::maintenance::Plan::preferences(&data)?,
                    "--remove-cache" => persistence::maintenance::Plan::cache(&settings, false)?,
                    _ => persistence::maintenance::Plan::logs(&data, false)?,
                };
                plan.execute(true)?;
            }
        } else {
            installer::engine::install(
                manifest.as_ref().unwrap(),
                &package,
                &exe,
                &options,
                &ctx,
                &std::sync::atomic::AtomicBool::new(false),
                |_| {},
            )?;
        }
        if let Some(report) = value("--report") {
            std::fs::write(report, "{\"passed\":true}").map_err(|e| e.to_string())?;
        }
        return Ok(());
    }
    let resume = args.iter().any(|a| a == "--continue-install");
    let mut wizard =
        installer::view::Wizard::new(package, exe, manifest, options, uninstall, resume)?;
    wizard.remove_data = ["--remove-preferences", "--remove-cache", "--remove-logs"]
        .map(|arg| args.iter().any(|a| a == arg));
    if let Some(root) = value("--window-smoke") {
        platform::windows::setup_host::smoke(wizard, root.into())
    } else {
        platform::windows::setup_host::run(wizard)
    }
}
