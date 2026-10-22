use crate::{
    core::{AppResult, storage},
    platform::windows::files,
};
use std::path::Path;
pub fn require(path: &Path, required: u64) -> AppResult<()> {
    match files::free_space(path) {
        None => Err(crate::i18n::tr("Não foi possível verificar o espaço livre. Confira o destino antes de continuar.").into()),
        Some((free, _)) if !storage::available(Some(free), required) => Err(crate::i18n::tr("Espaço insuficiente para manter a reserva de segurança e finalizar o arquivo. Escolha outro destino ou libere espaço.").into()),
        _ => Ok(()),
    }
}
