use crate::core::AppResult;
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
};

// A versioned, escaped UTF-8 key/value format. No executable or downloaded content.
pub fn encode(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace('\n', "%0A")
        .replace('\r', "%0D")
        .replace('=', "%3D")
}
pub fn decode(value: &str) -> String {
    value
        .replace("%3D", "=")
        .replace("%0D", "\r")
        .replace("%0A", "\n")
        .replace("%25", "%")
}
pub fn read(path: &Path) -> AppResult<BTreeMap<String, String>> {
    let text = fs::read_to_string(path).map_err(|e| {
        let __wyzar_i18n_0 = &(path.display());
        crate::i18n::format_text(
            crate::i18n::tr("{}: {e}"),
            &[
                ("0", format!("{}", __wyzar_i18n_0)),
                ("e", format!("{}", e)),
            ],
        )
    })?;
    if text.len() > 8 * 1024 * 1024 {
        return Err(crate::i18n::tr("Arquivo de preferências muito grande").into());
    }
    Ok(text
        .lines()
        .filter(|s| !s.starts_with('#'))
        .filter_map(|s| s.split_once('='))
        .map(|(k, v)| (k.into(), decode(v)))
        .collect())
}
pub fn write(path: &Path, values: &BTreeMap<String, String>) -> AppResult<()> {
    let body = values
        .iter()
        .map(|(k, v)| format!("{k}={}\n", encode(v)))
        .collect::<String>();
    atomic_write(path, body.as_bytes())
}
pub fn atomic_write(path: &Path, data: &[u8]) -> AppResult<()> {
    let parent = path
        .parent()
        .ok_or(crate::i18n::tr("Caminho sem pasta pai"))?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let temp = path.with_extension(format!("{}.tmp", std::process::id()));
    let mut f = File::create(&temp).map_err(|e| e.to_string())?;
    f.write_all(data)
        .and_then(|_| f.sync_all())
        .map_err(|e| e.to_string())?;
    drop(f);
    // MoveFileEx(REPLACE_EXISTING|WRITE_THROUGH) retains the old file until replacement succeeds.
    crate::platform::windows::files::replace_file(&temp, path)
}
pub fn test_folder(path: &Path) -> AppResult<()> {
    if !path.is_absolute() {
        return Err(crate::i18n::tr("Escolha um caminho absoluto para a pasta").into());
    }
    fs::create_dir_all(path)
        .map_err(|e| { let __wyzar_i18n_0 = &(path.display()); crate::i18n::format_text(crate::i18n::tr("Não foi possível criar {}: {e}. Escolha outra pasta gravável em Armazenamento; o destino pode estar indisponível, redirecionado ou protegido pelo Windows. Nenhuma proteção deve ser desativada."), &[("0", format!("{}", __wyzar_i18n_0)), ("e", format!("{}", e))]) })?;
    let probe = path.join(format!(
        ".wyzar-write-{}-{}",
        std::process::id(),
        crate::settings::unique_stamp()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
        .map_err(|e| {
            crate::i18n::format_text(
                crate::i18n::tr("Não foi possível gravar na pasta {path}: {e}. Confira a unidade e escolha outro destino disponível, se necessário."),
                &[("path", path.display().to_string()), ("e", format!("{}", e))],
            )
        })?;
    let written = file
        .write_all(b"WyzarCapture folder write probe")
        .and_then(|_| file.sync_all());
    drop(file);
    let removed = fs::remove_file(probe);
    written.and(removed).map_err(|e| {
        crate::i18n::format_text(
            crate::i18n::tr("Pasta sem permissão de escrita: {e}"),
            &[("e", e.to_string())],
        )
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escaped_roundtrip() {
        for s in ["D:\\Vídeos\\=clips", "um\ndois\r%0A%3D", "日本語"] {
            assert_eq!(decode(&encode(s)), s);
        }
    }
    #[test]
    fn invalid_folder() {
        assert!(test_folder(Path::new("relative")).is_err());
    }
}
pub mod maintenance;
pub mod migration;
