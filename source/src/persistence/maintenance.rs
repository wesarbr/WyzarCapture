//! Explicit, allow-listed cleanup. Media and project extensions are never selected.
use crate::{core::AppResult, platform::windows::setup as os, settings::Settings};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};
#[derive(Clone, Debug)]
pub struct Plan {
    pub label: String,
    pub files: Vec<(PathBuf, String, u64)>,
    pub bytes: u64,
}
impl Plan {
    fn new(label: &str) -> Self {
        Self {
            label: label.into(),
            files: vec![],
            bytes: 0,
        }
    }
    fn add(&mut self, path: PathBuf) -> AppResult<()> {
        os::guard(&path)?;
        if !path.is_file() {
            return Ok(());
        }
        if path.extension().is_some_and(|e| {
            ["mp4", "wav", "m4v", "wyzproj", "wyzar", "recording"]
                .iter()
                .any(|x| e.eq_ignore_ascii_case(x))
        }) {
            return Err(crate::i18n::tr("Mídia/projetos nunca fazem parte da limpeza").into());
        }
        let bytes = path.metadata().map_err(|e| e.to_string())?.len();
        self.bytes = self.bytes.saturating_add(bytes);
        self.files
            .push((path.clone(), os::hash_file(&path)?, bytes));
        Ok(())
    }
    pub fn cache(settings: &Settings, thumbnails: bool) -> AppResult<Self> {
        let mut p = Self::new(if thumbnails {
            crate::i18n::tr("Miniaturas recriáveis")
        } else {
            crate::i18n::tr("Cache temporário")
        });
        let root = settings.cache.join(if thumbnails {
            crate::i18n::tr("Thumbnails")
        } else {
            crate::i18n::tr("WyzarTemporary")
        });
        os::guard(&root)?;
        if fs::read_to_string(root.join(".wyzar-owned-cache"))
            .ok()
            .as_deref()
            != Some("WyzarCapture.Cache.1")
        {
            return Ok(p);
        }
        for e in fs::read_dir(&root).map_err(|e| e.to_string())?.take(10000) {
            let path = e.map_err(|e| e.to_string())?.path();
            if path.extension().is_some_and(|e| {
                ["png", "tmp", "bin"]
                    .iter()
                    .any(|x| e.eq_ignore_ascii_case(x))
            }) {
                p.add(path)?;
            }
        }
        Ok(p)
    }
    pub fn logs(data: &Path, old_only: bool) -> AppResult<Self> {
        let mut p = Self::new(if old_only {
            crate::i18n::tr("Logs antigos (mais de 7 dias)")
        } else {
            crate::i18n::tr("Logs locais")
        });
        for name in ["events.previous.log", "events.log"] {
            if old_only && name == "events.log" {
                continue;
            }
            let path = data.join("Logs").join(name);
            if path.is_file()
                && (!old_only
                    || path
                        .metadata()
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| SystemTime::now().duration_since(t).ok())
                        .is_some_and(|d| d > Duration::from_secs(7 * 86400)))
            {
                p.add(path)?;
            }
        }
        Ok(p)
    }
    pub fn preferences(data: &Path) -> AppResult<Self> {
        let mut p = Self::new(crate::i18n::tr(
            "Preferências locais (biblioteca e projetos preservados)",
        ));
        let path = data.join("settings.ini");
        if path.is_file() && path.metadata().map_err(|e| e.to_string())?.len() <= 524288 {
            let v = super::read(&path)?;
            if v.get("version")
                .is_some_and(|v| ["1", "2", "3"].contains(&v.as_str()))
            {
                p.add(path)?;
            }
        }
        Ok(p)
    }
    pub fn execute(&self, confirmed: bool) -> AppResult<()> {
        if !confirmed {
            return Err(crate::i18n::tr("Limpeza exige confirmação explícita").into());
        }
        // Same-directory quarantine ensures moves stay on the same volume. A
        // failed precommit move restores every file already moved.
        let id = crate::settings::unique_stamp();
        let mut moved: Vec<(PathBuf, PathBuf)> = vec![];
        let result = (|| -> AppResult<()> {
            for (index, (path, hash, bytes)) in self.files.iter().enumerate() {
                os::guard(path)?;
                if !path.is_file() {
                    continue;
                }
                if path.metadata().map_err(|e| e.to_string())?.len() != *bytes
                    || os::hash_file(path)? != *hash
                {
                    return Err(crate::i18n::tr(
                        "Um arquivo mudou. Reabra a confirmação de limpeza.",
                    )
                    .into());
                }
                let to = path.with_file_name(format!(".wyzar-clean-{id}-{index}.tmp"));
                if to.exists() {
                    return Err(crate::i18n::tr("Temporário de limpeza já existe").into());
                }
                fs::rename(path, &to).map_err(|e| e.to_string())?;
                moved.push((path.clone(), to));
            }
            Ok(())
        })();
        if let Err(e) = result {
            for (from, to) in moved.iter().rev() {
                if !from.exists() {
                    let _ = fs::rename(to, from);
                }
            }
            return Err(e);
        }
        for (_, to) in &moved {
            fs::remove_file(to).map_err(|e| {
                let __wyzar_i18n_0 = &(to.display());
                crate::i18n::format_text(
                    crate::i18n::tr("Limpeza confirmada; temporário pendente {}: {e}"),
                    &[
                        ("0", format!("{}", __wyzar_i18n_0)),
                        ("e", format!("{}", e)),
                    ],
                )
            })?;
        }
        Ok(())
    }
}
pub fn for_kind(s: &Settings, kind: usize) -> AppResult<Plan> {
    match kind {
        0 => Plan::cache(s, false),
        1 => Plan::cache(s, true),
        _ => Plan::logs(&s.data_dir, true),
    }
}
