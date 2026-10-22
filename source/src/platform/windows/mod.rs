//! Native boundary: all Win32, COM, Direct2D, capture and media unsafe code stays here.
#![allow(unsafe_op_in_unsafe_fn)]
pub mod accessibility;
mod brand_icon;
pub mod capabilities;
pub mod capture;
pub mod countdown;
pub mod devices;
pub mod diagnostics;
pub mod files;
mod gpu;
pub mod graphics;
pub mod host;
pub mod media;
pub mod playback;
pub mod selection;
#[allow(dead_code)] // Shared OS boundary used by the separate Setup executable.
pub mod setup;
#[allow(dead_code)]
pub mod setup_host;
pub mod storage;
mod uia_provider;
mod ux_probe;
pub mod wasapi;
use windows::{Win32::Media::MediaFoundation::*, core::PCWSTR};
pub fn wide(s: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    s.as_ref().encode_wide().chain(Some(0)).collect()
}
pub struct Runtime {
    owns_apartment: bool,
    _thread_bound: std::marker::PhantomData<*mut ()>,
}
impl Runtime {
    pub fn new(multithreaded: bool) -> crate::core::AppResult<Self> {
        unsafe {
            let initialized = windows::Win32::System::WinRT::RoInitialize(if multithreaded {
                windows::Win32::System::WinRT::RO_INIT_MULTITHREADED
            } else {
                windows::Win32::System::WinRT::RO_INIT_SINGLETHREADED
            });
            let owns_apartment = match initialized {
                Ok(()) => true,
                Err(e) if e.code() == windows::Win32::Foundation::RPC_E_CHANGED_MODE => false,
                Err(e) => return Err(e.to_string()),
            };
            if let Err(e) = MFStartup(MF_VERSION, MFSTARTUP_FULL) {
                if owns_apartment {
                    windows::Win32::System::WinRT::RoUninitialize();
                }
                return Err(e.to_string());
            }
            Ok(Self {
                owns_apartment,
                _thread_bound: std::marker::PhantomData,
            })
        }
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        unsafe {
            let _ = MFShutdown();
            if self.owns_apartment {
                windows::Win32::System::WinRT::RoUninitialize();
            }
        }
    }
}
pub fn pcw(s: &[u16]) -> PCWSTR {
    PCWSTR(s.as_ptr())
}
/// Stable UTF-16 storage for translated native captions/filter descriptors.
/// The boxed buffers are immutable and never removed; rehashing the map does
/// not move their allocations. Only compile-time message keys are accepted.
pub fn localized(source: &'static str) -> PCWSTR {
    localized_in(crate::i18n::current(), source)
}
fn localized_in(language: crate::i18n::Language, source: &'static str) -> PCWSTR {
    type Key = (crate::i18n::Language, &'static str);
    static TEXT: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<Key, Box<[u16]>>>> =
        std::sync::OnceLock::new();
    let mut cache = TEXT
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let buffer = cache.entry((language, source)).or_insert_with(|| {
        crate::i18n::translate(language, source)
            .encode_utf16()
            .chain(Some(0))
            .collect::<Vec<_>>()
            .into_boxed_slice()
    });
    PCWSTR(buffer.as_ptr())
}
#[cfg(test)]
mod localized_native_tests {
    use super::*;
    #[test]
    fn translated_native_buffers_remain_valid_after_cache_rehashing() {
        let pointer = localized_in(crate::i18n::Language::PtBr, "Escolher pasta");
        let before = unsafe { pointer.to_string().unwrap() };
        for language in crate::i18n::Language::ALL {
            for key in [
                "Idioma",
                "Instalador",
                "Desinstalador",
                "Escolher pasta",
                "Abrir projeto de edição",
                "Importar vídeo ou trilha de áudio",
                "Projeto WyzarCapture",
                "Vídeo ou áudio PCM",
                "Ícone vetorial SVG",
                "Abrir WyzarCapture",
                "Sair",
                "Selecionar região · WyzarCapture",
                "WyzarCapture · Contagem regressiva",
            ] {
                let text = localized_in(language, key);
                assert_eq!(
                    unsafe { text.to_string().unwrap() },
                    crate::i18n::translate(language, key)
                );
            }
        }
        assert_eq!(unsafe { pointer.to_string().unwrap() }, before);
        assert_eq!(
            pointer,
            localized_in(crate::i18n::Language::PtBr, "Escolher pasta")
        );
    }
}
