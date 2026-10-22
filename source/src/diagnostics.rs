//! Local, bounded log. Free-form errors, paths, device names and media are excluded.
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};
static LOG: OnceLock<Mutex<PathBuf>> = OnceLock::new();
pub fn init(data: &Path) {
    let root = data.join("Logs");
    if fs::create_dir_all(&root).is_ok() {
        let _ = LOG.set(Mutex::new(root));
    }
    event("startup", None);
}
pub fn code(error: &str) -> Option<u32> {
    error.match_indices("0x").find_map(|(i, _)| {
        error
            .get(i + 2..i + 10)
            .and_then(|s| u32::from_str_radix(s, 16).ok())
    })
}
pub fn os_code(error: &str) -> Option<u32> {
    error
        .split("(os error ")
        .nth(1)?
        .split(')')
        .next()?
        .parse()
        .ok()
}
pub fn event(kind: &'static str, hresult: Option<u32>) {
    let Some(root) = LOG.get().and_then(|m| m.lock().ok()) else {
        return;
    };
    let path = root.join("events.log");
    if path.metadata().is_ok_and(|m| m.len() > 1_048_576) {
        let previous = root.join("events.previous.log");
        let _ = fs::remove_file(&previous);
        let _ = fs::rename(&path, &previous);
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let _ = writeln!(
            file,
            "time={timestamp} version={} event={kind} hresult={}",
            crate::updates::VERSION,
            hresult.map(|c| format!("0x{c:08X}")).unwrap_or_default()
        );
    }
}
pub fn summary(metrics: Option<&crate::capture::metrics::Snapshot>, error: Option<&str>) -> String {
    {
        let __wyzar_i18n_0 = &(crate::updates::VERSION);
        let __wyzar_i18n_1 = &(error
            .and_then(code)
            .map(|c| format!("0x{c:08X}"))
            .unwrap_or_else(|| crate::i18n::tr("unavailable").into()));
        let __wyzar_i18n_2 = &(error
            .and_then(os_code)
            .map(|c| c.to_string())
            .unwrap_or_else(|| crate::i18n::tr("unavailable").into()));
        let __wyzar_i18n_3 = &(metrics
            .map(|m| m.json())
            .unwrap_or_else(|| crate::i18n::tr("not recorded").into()));
        crate::i18n::format_text(
            crate::i18n::tr(
                "WyzarCapture {}\nWindows native; offline; H.264/AAC; encoder selection=automatic request\nLast HRESULT: {}\nLast OS error: {}\nMetrics: {}\nGPU utilization and physical A/V synchronization: not measured\n",
            ),
            &[
                ("0", format!("{}", __wyzar_i18n_0)),
                ("1", format!("{}", __wyzar_i18n_1)),
                ("2", format!("{}", __wyzar_i18n_2)),
                ("3", format!("{}", __wyzar_i18n_3)),
            ],
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagnostics_exclude_paths_and_private_error_text() {
        let private = "C:\\Pessoa\\Vídeos\\reunião.mp4: erro (0x80070005)";
        let summary = summary(None, Some(private));
        assert!(summary.contains("0x80070005"));
        assert!(!summary.contains("Pessoa"));
        assert!(!summary.contains("reunião"));
        let os = super::summary(None, Some("C:\\Pessoa\\Vídeos: falha (os error 2)"));
        assert!(os.contains("Último erro do sistema: 2"));
        assert!(!os.contains("Pessoa"));
    }
}
