pub mod bundle;
use crate::{
    core::{AppResult, Frame},
    persistence,
};
use std::{
    collections::{BTreeMap, HashSet},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
#[derive(Clone, Debug)]
pub struct Recording {
    pub path: PathBuf,
    pub title: String,
    pub bytes: u64,
    pub modified: u64,
    pub duration: f64,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub tags: Vec<String>,
    pub favorite: bool,
    pub missing: bool,
    pub problem: Option<String>,
    pub thumbnail: Option<Frame>,
}
impl Recording {
    pub fn sidecar(&self) -> PathBuf {
        self.path.with_extension("wyzar")
    }
}
#[derive(Default)]
pub struct Library {
    pub items: Vec<Recording>,
}
impl Library {
    pub fn scan(&mut self, root: &Path) {
        let mut files = vec![];
        gather(root, 0, &mut files);
        let mut old: BTreeMap<PathBuf, Recording> =
            self.items.drain(..).map(|x| (x.path.clone(), x)).collect();
        let mut seen = HashSet::new();
        for path in files {
            seen.insert(path.clone());
            let Ok(meta) = path.metadata() else {
                continue;
            };
            let modified = meta
                .modified()
                .unwrap_or(SystemTime::UNIX_EPOCH)
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let mut rec = old.remove(&path).unwrap_or_else(|| Recording {
                title: path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into(),
                path: path.clone(),
                bytes: 0,
                modified,
                duration: 0.,
                width: 0,
                height: 0,
                fps: 0.,
                tags: vec![],
                favorite: false,
                missing: false,
                problem: None,
                thumbnail: None,
            });
            rec.bytes = meta.len();
            rec.modified = modified;
            rec.missing = false;
            if let Ok(v) = persistence::read(&rec.sidecar()) {
                rec.title = v.get("title").cloned().unwrap_or(rec.title);
                rec.duration = v
                    .get("duration")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(rec.duration);
                rec.width = v
                    .get("width")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(rec.width);
                rec.height = v
                    .get("height")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(rec.height);
                rec.fps = v.get("fps").and_then(|v| v.parse().ok()).unwrap_or(rec.fps);
                rec.tags = v
                    .get("tags")
                    .map(|x| {
                        x.split(',')
                            .filter(|s| !s.is_empty())
                            .map(str::to_owned)
                            .collect()
                    })
                    .unwrap_or_default();
                rec.favorite = v.get("favorite").is_some_and(|s| s == "true");
            }
            self.items.push(rec);
        }
        for (_, mut rec) in old {
            if !seen.contains(&rec.path) {
                rec.missing = !rec.path.is_file();
                self.items.push(rec);
            }
        }
        self.items.sort_by(|a, b| b.modified.cmp(&a.modified));
    }
    pub fn import(&mut self, path: &Path) -> AppResult<()> {
        if !path.is_file() {
            return Err(crate::i18n::tr("Arquivo não encontrado").into());
        }
        if self.items.iter().any(|r| r.path == path) {
            return Ok(());
        }
        let meta = path.metadata().map_err(|e| e.to_string())?;
        self.items.insert(
            0,
            Recording {
                path: path.into(),
                title: path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into(),
                bytes: meta.len(),
                modified: meta
                    .modified()
                    .unwrap_or(UNIX_EPOCH)
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                duration: 0.,
                width: 0,
                height: 0,
                fps: 0.,
                tags: vec![crate::i18n::tr("Importado").into()],
                favorite: false,
                missing: false,
                problem: None,
                thumbnail: None,
            },
        );
        Ok(())
    }
    pub fn visible(&self, query: &str, favorites: bool, oldest: bool) -> Vec<usize> {
        let q = query.to_lowercase();
        let mut out: Vec<_> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                (!favorites || r.favorite)
                    && (q.is_empty()
                        || r.title.to_lowercase().contains(&q)
                        || r.tags.iter().any(|t| t.to_lowercase().contains(&q)))
            })
            .map(|(i, _)| i)
            .collect();
        if oldest {
            out.reverse();
        }
        out
    }
    pub fn rename(&mut self, index: usize, stem: &str) -> AppResult<()> {
        if !crate::settings::valid_pattern(stem) {
            return Err(crate::i18n::tr("Nome de arquivo inválido").into());
        }
        let r = self
            .items
            .get_mut(index)
            .ok_or(crate::i18n::tr("Selecione uma gravação"))?;
        let to = r.path.with_file_name(format!(
            "{stem}.{}",
            r.path.extension().unwrap_or_default().to_string_lossy()
        ));
        if to == r.path {
            r.title = stem.into();
            return Self::persist_record(r);
        }
        let mut moves = vec![(r.path.clone(), to.clone())];
        for ext in bundle::RELATED_EXTENSIONS {
            let from = r.path.with_extension(ext);
            if from.exists() {
                moves.push((from, to.with_extension(ext)));
            }
        }
        if moves.iter().any(|(_, target)| target.exists()) {
            return Err(crate::i18n::tr("Já existe um arquivo ou faixa com esse nome").into());
        }
        for (i, (from, target)) in moves.iter().enumerate() {
            if let Err(error) = std::fs::rename(from, target) {
                for (old, new) in moves[..i].iter().rev() {
                    let _ = std::fs::rename(new, old);
                }
                return Err(crate::i18n::format_text(
                    crate::i18n::tr("Falha ao renomear a gravação: {error}"),
                    &[("error", format!("{}", error))],
                ));
            }
        }
        r.path = to;
        r.title = stem.into();
        Self::persist_record(r)
    }
    pub fn persist_record(r: &Recording) -> AppResult<()> {
        let v = BTreeMap::from([
            ("version".into(), "1".into()),
            ("title".into(), r.title.clone()),
            ("duration".into(), r.duration.to_string()),
            ("width".into(), r.width.to_string()),
            ("height".into(), r.height.to_string()),
            ("fps".into(), r.fps.to_string()),
            ("tags".into(), r.tags.join(",")),
            ("favorite".into(), r.favorite.to_string()),
        ]);
        persistence::write(&r.sidecar(), &v)
    }
    pub fn save_index(&self, data: &Path) -> AppResult<()> {
        let v = self
            .items
            .iter()
            .enumerate()
            .map(|(i, r)| (format!("file.{i}"), r.path.to_string_lossy().into()))
            .collect();
        persistence::write(&data.join("library.ini"), &v)
    }
    pub fn restore_index(&mut self, data: &Path) {
        if let Ok(v) = persistence::read(&data.join("library.ini")) {
            for p in v.values() {
                let path = Path::new(p);
                if self.import(path).is_err() && !self.items.iter().any(|r| r.path == path) {
                    self.items.push(Recording {
                        path: path.into(),
                        title: path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into(),
                        bytes: 0,
                        modified: 0,
                        duration: 0.,
                        width: 0,
                        height: 0,
                        fps: 0.,
                        tags: vec![],
                        favorite: false,
                        missing: true,
                        problem: None,
                        thumbnail: None,
                    });
                }
            }
        }
    }
    pub fn relink(&mut self, index: usize, path: PathBuf) -> AppResult<()> {
        if !path.is_file() {
            return Err(crate::i18n::tr("Arquivo não encontrado").into());
        }
        let r = self
            .items
            .get_mut(index)
            .ok_or(crate::i18n::tr("Selecione um arquivo ausente"))?;
        r.bytes = path.metadata().map_err(|e| e.to_string())?.len();
        r.path = path;
        r.missing = false;
        r.problem = None;
        r.duration = 0.;
        r.thumbnail = None;
        Self::persist_record(r)
    }
}
fn gather(root: &Path, depth: u32, out: &mut Vec<PathBuf>) {
    if depth > 5 || out.len() > 10000 {
        return;
    }
    let Ok(dir) = std::fs::read_dir(root) else {
        return;
    };
    for e in dir.flatten() {
        let p = e.path();
        if e.file_type().is_ok_and(|f| f.is_symlink()) {
            continue;
        }
        if p.is_dir() {
            if p.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with(".wyzar-"))
            {
                continue;
            }
            gather(&p, depth + 1, out);
        } else if p.extension().is_some_and(|x| {
            ["mp4", "wmv", "mov", "m4v", "wav"]
                .contains(&x.to_string_lossy().to_lowercase().as_str())
        }) {
            if p.file_name().is_some_and(|n| {
                [".mic.wav", ".system.wav", ".audio.wav"]
                    .iter()
                    .any(|suffix| n.to_string_lossy().to_lowercase().ends_with(suffix))
            }) {
                continue;
            }
            out.push(p);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rename_preserves_audio_and_refuses_collisions() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("validation/unit")
            .join(format!("rename-{}", crate::settings::unique_stamp()));
        std::fs::create_dir_all(&root).unwrap();
        let original = root.join("clip.mp4");
        std::fs::write(&original, b"fixture").unwrap();
        std::fs::write(original.with_extension("mic.wav"), b"audio").unwrap();
        let mut library = Library::default();
        library.import(&original).unwrap();
        library.rename(0, "Renomeado").unwrap();
        assert!(root.join("Renomeado.mp4").exists());
        assert!(root.join("Renomeado.mic.wav").exists());
        std::fs::write(root.join("collision.mp4"), b"keep").unwrap();
        assert!(library.rename(0, "collision").is_err());
        assert_eq!(std::fs::read(root.join("collision.mp4")).unwrap(), b"keep");
        let resolved = std::fs::canonicalize(&root).unwrap();
        assert!(
            resolved.starts_with(
                std::fs::canonicalize(
                    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("validation/unit")
                )
                .unwrap()
            )
        );
        std::fs::remove_dir_all(resolved).unwrap();
    }
    #[test]
    fn imported_file_stays_available_outside_video_root_and_missing_survives_restart() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("validation/unit")
            .join(format!(
                "wyzar-library-{}-{}",
                std::process::id(),
                crate::settings::unique_stamp()
            ));
        let videos = root.join("videos");
        let data = root.join("data");
        std::fs::create_dir_all(&videos).unwrap();
        let file = root.join("external.mp4");
        std::fs::write(&file, b"fixture").unwrap();
        let mut library = Library::default();
        library.import(&file).unwrap();
        library.scan(&videos);
        assert!(!library.items[0].missing);
        library.save_index(&data).unwrap();
        std::fs::remove_file(&file).unwrap();
        let mut restored = Library::default();
        restored.restore_index(&data);
        assert_eq!(restored.items.len(), 1);
        assert!(restored.items[0].missing);
        let canonical = std::fs::canonicalize(&root).unwrap();
        assert!(
            canonical.starts_with(
                std::fs::canonicalize(
                    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("validation/unit")
                )
                .unwrap()
            )
        );
        std::fs::remove_dir_all(canonical).unwrap();
    }
}
