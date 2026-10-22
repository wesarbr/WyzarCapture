//! Offline installer model/transaction. Windows effects live behind platform/windows/setup.
pub mod bundle;
pub mod engine;
pub mod payload;
pub mod qa;
pub mod view;
use crate::core::AppResult;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    User,
    Machine,
}
impl Scope {
    pub fn ensure_available(self) -> AppResult<()> {
        if self == Self::Machine && !cfg!(feature = "experimental-all-users") {
            return Err(crate::i18n::tr("Esta build beta permite instalar somente para o usuário atual. A instalação para todos os usuários ainda está em validação.").into());
        }
        Ok(())
    }
    pub fn key(self) -> &'static str {
        if self == Self::User {
            "user"
        } else {
            "machine"
        }
    }
}
#[derive(Clone, Debug)]
pub struct Options {
    pub language: Option<crate::i18n::Language>,
    pub destination: PathBuf,
    pub media_root: Option<PathBuf>,
    pub scope: Scope,
    pub desktop: bool,
    pub start_menu: bool,
    pub startup: bool,
    pub allow_downgrade: bool,
    pub close_running: bool,
    /// Explicit consent to move a working installation registered in another folder.
    pub move_existing: bool,
}
#[derive(Clone, Debug, Default)]
pub struct Progress {
    pub completed: usize,
    pub total: usize,
    pub operation: String,
    pub file: String,
}
impl Progress {
    pub fn percent(&self) -> u32 {
        if self.total == 0 {
            0
        } else {
            (self.completed.min(self.total) * 100 / self.total) as u32
        }
    }
}
pub fn relative(value: &str) -> AppResult<PathBuf> {
    if value.is_empty()
        || value.len() > 220
        || value.contains([':', '\\', '\0'])
        || value.starts_with('/')
    {
        return Err(crate::i18n::tr("Caminho inválido no pacote").into());
    }
    for part in value.split('/') {
        let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        if part.is_empty()
            || matches!(part, "." | "..")
            || part.ends_with(['.', ' '])
            || part.chars().any(|c| c < ' ' || "<>\"|?*".contains(c))
            || [
                "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
                "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8",
                "LPT9",
            ]
            .contains(&stem.as_str())
        {
            return Err(crate::i18n::tr("Componente de caminho recusado").into());
        }
    }
    Ok(PathBuf::from(value))
}
pub fn guard(path: &Path) -> AppResult<()> {
    crate::platform::windows::setup::guard(path)
}
pub fn version(value: &str) -> AppResult<[u32; 3]> {
    let numbers = value
        .split('.')
        .map(str::parse::<u32>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| crate::i18n::tr("Versão inválida"))?;
    numbers
        .try_into()
        .map_err(|_| crate::i18n::tr("A versão precisa ter três números").into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn production_scope_never_requires_machine_installation() {
        assert!(Scope::User.ensure_available().is_ok());
        assert_eq!(
            Scope::Machine.ensure_available().is_ok(),
            cfg!(feature = "experimental-all-users")
        );
    }
    #[test]
    fn package_paths_and_versions_are_bounded() {
        for p in [
            "../x",
            "a/../../x",
            "C:/Windows/x",
            "//server/x",
            "a\\b",
            "a:stream",
            "A/CON.txt",
            "docs/aux",
            "docs/test. /x",
            "docs//a",
        ] {
            assert!(relative(p).is_err(), "{p}");
        }
        assert!(relative("docs/Notas de atualização.md").is_ok());
        assert!(version("0.4.0").unwrap() > version("0.3.1").unwrap());
        assert!(version("0.4.0/../../x").is_err());
    }
}
