use super::*;
use crate::platform::windows::setup::hash_file;
use std::{collections::BTreeSet, fs};
#[derive(Clone, Debug)]
pub struct FileEntry {
    pub name: String,
    pub bytes: u64,
    pub hash: String,
}
#[derive(Clone, Debug)]
pub struct Manifest {
    pub text: String,
    pub version: String,
    pub date: String,
    pub files: Vec<FileEntry>,
}
impl Manifest {
    pub fn parse(text: &str) -> AppResult<Self> {
        if text.len() > 512 * 1024 {
            return Err(crate::i18n::tr("Manifest excede o limite").into());
        }
        let mut m = Self {
            text: text.into(),
            version: String::new(),
            date: String::new(),
            files: vec![],
        };
        let mut keys = BTreeSet::new();
        let mut names = BTreeSet::new();
        let mut arch = false;
        let mut schema = false;
        for line in text.lines().filter(|l| !l.is_empty()) {
            let (k, v) = line
                .split_once('=')
                .ok_or(crate::i18n::tr("Manifest inválido"))?;
            if !keys.insert(k.to_owned()) {
                return Err(crate::i18n::tr("Chave duplicada no manifest").into());
            }
            match k {
                "schema" => schema = v == "1",
                "version" => {
                    version(v)?;
                    m.version = v.into();
                }
                "architecture" => arch = v == "x64",
                "build_date" => m.date = v.into(),
                "binary_sha256" => {}
                _ if k.starts_with("file.") => {
                    let parts: Vec<_> = v.split('|').collect();
                    if parts.len() != 3 {
                        return Err(crate::i18n::tr("Arquivo sem hash/tamanho").into());
                    }
                    relative(parts[0])?;
                    let first = parts[0].split('/').next().unwrap().to_ascii_lowercase();
                    if [
                        "install-staging",
                        "install-rollback",
                        "install-state.ini",
                        "install-transaction.ini",
                        "wyzarcaptureuninstall.exe",
                    ]
                    .contains(&first.as_str())
                    {
                        return Err(
                            crate::i18n::tr("Nome reservado ao instalador no payload").into()
                        );
                    }
                    if !names.insert(parts[0].to_lowercase()) {
                        return Err(crate::i18n::tr("Arquivo duplicado no pacote").into());
                    }
                    if parts[2].len() != 64 || !parts[2].bytes().all(|b| b.is_ascii_hexdigit()) {
                        return Err(crate::i18n::tr("SHA-256 inválido").into());
                    }
                    m.files.push(FileEntry {
                        name: parts[0].into(),
                        bytes: parts[1]
                            .parse()
                            .map_err(|_| crate::i18n::tr("Tamanho inválido"))?,
                        hash: parts[2].to_ascii_uppercase(),
                    });
                    if m.files.len() > 512
                        || m.files
                            .iter()
                            .try_fold(0u64, |n, f| n.checked_add(f.bytes))
                            .filter(|n| *n <= 4 * 1024 * 1024 * 1024)
                            .is_none()
                    {
                        return Err(crate::i18n::tr(
                            "Payload excede o limite de 4 GiB/512 arquivos",
                        )
                        .into());
                    }
                }
                _ => return Err(crate::i18n::tr("Chave desconhecida no manifest").into()),
            }
        }
        if !schema
            || !arch
            || m.version.is_empty()
            || m.date.is_empty()
            || m.files.len() != 1
            || m.files[0].name != "WyzarCapture.exe"
            || m.files[0].bytes == 0
        {
            return Err(crate::i18n::tr("Payload x64/versionado incompleto").into());
        }
        let declared = text
            .lines()
            .find_map(|l| l.strip_prefix("binary_sha256="))
            .ok_or(crate::i18n::tr("Hash principal ausente"))?;
        if !m
            .files
            .iter()
            .any(|f| f.name == "WyzarCapture.exe" && f.hash.eq_ignore_ascii_case(declared))
        {
            return Err(crate::i18n::tr("Hash principal divergente").into());
        }
        Ok(m)
    }
    /// The payload is a product allow-list, not a general purpose archive.
    /// Check the filesystem as well as the manifest so undeclared files, ADS,
    /// directories and reparse points cannot be silently accepted.
    pub fn verify_payload(&self, root: &Path) -> AppResult<()> {
        if self.files.len() != 1 || self.files[0].name != "WyzarCapture.exe" {
            return Err(crate::i18n::tr("Payload permitido: somente WyzarCapture.exe").into());
        }
        guard(root)?;
        let mut items = fs::read_dir(root).map_err(|e| e.to_string())?;
        let item = items
            .next()
            .ok_or(crate::i18n::tr("Payload vazio"))?
            .map_err(|e| e.to_string())?;
        if item.file_name() != "WyzarCapture.exe"
            || !item.file_type().map_err(|e| e.to_string())?.is_file()
            || items.next().is_some()
        {
            return Err(crate::i18n::tr("Arquivo ou diretório inesperado no payload").into());
        }
        guard(&item.path())?;
        self.verify_file(root, &self.files[0])
    }
    pub fn size(&self) -> u64 {
        self.files.iter().map(|f| f.bytes).sum()
    }
    pub fn verify_file(&self, root: &Path, entry: &FileEntry) -> AppResult<()> {
        let file = root.join(relative(&entry.name)?);
        guard(&file)?;
        crate::platform::windows::setup::verify_default_stream(&file)?;
        if fs::metadata(&file)
            .map_err(|e| {
                let __wyzar_i18n_0 = &(entry.name);
                crate::i18n::format_text(
                    crate::i18n::tr("{}: {e}"),
                    &[
                        ("0", format!("{}", __wyzar_i18n_0)),
                        ("e", format!("{}", e)),
                    ],
                )
            })?
            .len()
            != entry.bytes
            || hash_file(&file)? != entry.hash
        {
            return Err({
                let __wyzar_i18n_0 = &(entry.name);
                crate::i18n::format_text(
                    crate::i18n::tr("Hash/tamanho inválido: {}. Nada foi instalado."),
                    &[("0", format!("{}", __wyzar_i18n_0))],
                )
            });
        }
        Ok(())
    }
    pub fn trusted(package: &Path, embedded: &str) -> AppResult<Self> {
        if embedded.is_empty() {
            return Err(crate::i18n::tr("Este build de desenvolvimento não possui payload aprovado. Gere o pacote pelo pipeline de release.").into());
        }
        guard(package)?;
        guard(&package.join("payload.manifest"))?;
        let disk = fs::read_to_string(package.join("payload.manifest")).map_err(|e| {
            crate::i18n::format_text(
                crate::i18n::tr("Manifest não encontrado: {e}"),
                &[("e", format!("{}", e))],
            )
        })?;
        if disk != embedded {
            return Err(crate::i18n::tr(
                "O manifest externo não corresponde ao incorporado no instalador",
            )
            .into());
        }
        let manifest = Self::parse(embedded)?;
        manifest.verify_payload(&package.join("payload"))?;
        Ok(manifest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runtime_manifest_rejects_extra_files_paths_and_empty_app() {
        let base = format!(
            "schema=1\nversion=0.5.2\narchitecture=x64\nbuild_date=2026-10-21\nbinary_sha256={0}\nfile.0=WyzarCapture.exe|7|{0}\n",
            "A".repeat(64)
        );
        assert!(Manifest::parse(&base).is_ok());
        assert!(
            Manifest::parse(&format!("{base}file.1=extra.exe|7|{}\n", "B".repeat(64))).is_err()
        );
        for path in [
            "docs/WyzarCapture.exe",
            "WyzarCapture.exe:stream",
            "../WyzarCapture.exe",
            "wyzarcapture.exe",
        ] {
            assert!(
                Manifest::parse(
                    &base.replace("file.0=WyzarCapture.exe|", &format!("file.0={path}|"))
                )
                .is_err()
            );
        }
        assert!(Manifest::parse(&base.replace("|7|", "|0|")).is_err());
    }
    #[test]
    fn filesystem_allow_list_rejects_undeclared_file_and_ads() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("validation/unit")
            .join(format!("payload-{}", crate::settings::unique_stamp()));
        fs::create_dir_all(&root).unwrap();
        let hash = crate::platform::windows::setup::hash_bytes(b"fixture").unwrap();
        let text = format!(
            "schema=1\nversion=0.5.2\narchitecture=x64\nbuild_date=2026-10-21\nbinary_sha256={hash}\nfile.0=WyzarCapture.exe|7|{hash}\n"
        );
        let manifest = Manifest::parse(&text).unwrap();
        fs::write(root.join("WyzarCapture.exe"), b"fixture").unwrap();
        assert!(manifest.verify_payload(&root).is_ok());
        fs::write(root.join("extra.txt"), b"unexpected").unwrap();
        assert!(manifest.verify_payload(&root).is_err());
        fs::remove_file(root.join("extra.txt")).unwrap();
        fs::write(root.join("WyzarCapture.exe:unexpected"), b"unexpected ADS").unwrap();
        assert!(manifest.verify_payload(&root).is_err());
        fs::remove_file(root.join("WyzarCapture.exe")).unwrap();
        fs::remove_dir(root).unwrap();
    }
}
