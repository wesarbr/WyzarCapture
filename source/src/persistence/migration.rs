//! Explicit import of recognised settings/index/profile formats; never copies media.
use super::*;
use std::path::PathBuf;
#[derive(Clone, Debug)]
pub struct Import {
    pub source: PathBuf,
    pub files: Vec<(String, BTreeMap<String, String>, String)>,
}
impl Import {
    pub fn inspect(source: PathBuf) -> AppResult<Self> {
        crate::platform::windows::setup::guard(&source)?;
        let main = source.join("settings.ini");
        let values = read_bounded(&main)?;
        if !recognised(&values) {
            return Err(crate::i18n::tr("Esta pasta não contém preferências reconhecidas do WyzarCapture (esquemas 1–3). Nenhum dado foi importado.").into());
        }
        let mut files = vec![(
            "settings.ini".into(),
            values,
            crate::platform::windows::setup::hash_file(&main)?,
        )];
        let index = source.join("library.ini");
        if index.is_file() {
            let values = read_bounded(&index)?;
            if values.len() <= 10000
                && values.iter().all(|(k, v)| {
                    k.strip_prefix("file.")
                        .is_some_and(|n| n.parse::<usize>().is_ok())
                        && std::path::Path::new(v).is_absolute()
                })
            {
                files.push((
                    "library.ini".into(),
                    values,
                    crate::platform::windows::setup::hash_file(&index)?,
                ));
            }
        }
        let profiles = source.join("Profiles");
        if profiles.is_dir() {
            crate::platform::windows::setup::guard(&profiles)?;
            for entry in std::fs::read_dir(profiles)
                .map_err(|e| e.to_string())?
                .take(256)
            {
                let file = entry.map_err(|e| e.to_string())?.path();
                if file
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("ini"))
                {
                    let values = read_bounded(&file)?;
                    if recognised(&values) {
                        let name =
                            format!("Profiles/{}", file.file_name().unwrap().to_string_lossy());
                        crate::installer::relative(&name)?;
                        files.push((
                            name,
                            values,
                            crate::platform::windows::setup::hash_file(&file)?,
                        ));
                    }
                }
            }
        }
        Ok(Self { source, files })
    }
    pub fn apply(&self, destination: &Path) -> AppResult<PathBuf> {
        crate::platform::windows::setup::guard(destination)?;
        if self.source.to_string_lossy().to_lowercase()
            == destination.to_string_lossy().to_lowercase()
        {
            return Err(crate::i18n::tr("Origem e destino de importação são iguais").into());
        }
        let backup = destination
            .join("Migrations")
            .join(format!("BeforeImport-{}", crate::settings::unique_stamp()));
        std::fs::create_dir_all(&backup).map_err(|e| e.to_string())?;
        let mut prior = vec![];
        for (name, _, hash) in &self.files {
            let rel = crate::installer::relative(name)?;
            let source = self.source.join(&rel);
            let target = destination.join(&rel);
            crate::platform::windows::setup::guard(&target)?;
            if crate::platform::windows::setup::hash_file(&source)? != *hash {
                return Err(
                    crate::i18n::tr("Dados de origem mudaram. Reabra a importação.").into(),
                );
            }
            let old = if target.exists() {
                let p = backup.join(&rel);
                std::fs::create_dir_all(p.parent().unwrap()).map_err(|e| e.to_string())?;
                std::fs::copy(&target, &p).map_err(|e| e.to_string())?;
                Some(p)
            } else {
                None
            };
            prior.push((target, old));
        }
        let result = (|| -> AppResult<()> {
            for (name, values, _) in &self.files {
                write(&destination.join(crate::installer::relative(name)?), values)?;
            }
            Ok(())
        })();
        if let Err(error) = result {
            for (target, old) in prior {
                if let Some(old) = old {
                    let _ = std::fs::copy(old, target);
                } else {
                    let _ = std::fs::remove_file(target);
                }
            }
            return Err(crate::i18n::format_text(
                crate::i18n::tr("Importação revertida: {error}"),
                &[("error", format!("{}", error))],
            ));
        }
        write(
            &backup.join("migration.ini"),
            &BTreeMap::from([
                ("product".into(), "WyzarCapture".into()),
                ("schema".into(), "1".into()),
                ("files".into(), self.files.len().to_string()),
            ]),
        )?;
        Ok(backup)
    }
}
fn read_bounded(path: &Path) -> AppResult<BTreeMap<String, String>> {
    crate::platform::windows::setup::guard(path)?;
    if path.metadata().map_err(|e| e.to_string())?.len() > 524288 {
        return Err(crate::i18n::tr("Dados de importação excedem limite").into());
    }
    read(path)
}
fn recognised(v: &BTreeMap<String, String>) -> bool {
    v.get("version")
        .is_some_and(|v| ["1", "2", "3"].contains(&v.as_str()))
        && ["videos", "fps", "width", "height"]
            .iter()
            .all(|k| v.contains_key(*k))
}
