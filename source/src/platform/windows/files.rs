use super::{pcw, wide};
use crate::core::AppResult;
use std::path::{Path, PathBuf};
use windows::{
    Win32::{
        Foundation::*,
        Storage::FileSystem::*,
        System::{Com::*, SystemInformation::*},
        UI::{Shell::Common::COMDLG_FILTERSPEC, Shell::*, WindowsAndMessaging::*},
    },
    core::{HRESULT, w},
};
pub fn data_folder() -> PathBuf {
    std::env::var_os("WYZAR_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            if let Some(root) = portable_root() {
                return root.join("Data");
            }
            std::env::var_os("LOCALAPPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(std::env::temp_dir)
                .join("WyzarCapture")
        })
}
pub fn portable_root() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let root = exe.parent()?;
    let marker = root.join("wyzar-portable.ini");
    super::setup::guard(&marker).ok()?;
    if marker.metadata().ok()?.len() > 4096 {
        return None;
    }
    let v = crate::persistence::read(&marker).ok()?;
    (v.get("product").map(String::as_str) == Some("WyzarCapture")
        && v.get("schema").map(String::as_str) == Some("1"))
    .then(|| root.into())
}
fn user_folder(id: &windows::core::GUID, name: &str) -> Option<PathBuf> {
    unsafe {
        // DONT_VERIFY resolves redirected and not-yet-created folders. Creation
        // and write permission are checked separately; never substitute AppData.
        if let Ok(p) = SHGetKnownFolderPath(id, KF_FLAG_DONT_VERIFY, None) {
            let s = p.to_string().ok();
            CoTaskMemFree(Some(p.0.cast()));
            if let Some(path) = s.map(PathBuf::from).filter(|p| p.is_absolute()) {
                return Some(path);
            }
        }
    }
    // An unregistered known folder may use the actual user's profile. Do not
    // replace an existing redirected location merely because it is unavailable.
    std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .map(|p| p.join(name))
}
pub fn media_folders() -> [PathBuf; 5] {
    let base = |id, name| user_folder(id, name).unwrap_or_default();
    crate::settings::bootstrap::windows_media_paths(
        base(&FOLDERID_Videos, "Videos"),
        base(&FOLDERID_Music, "Music"),
        base(&FOLDERID_Pictures, "Pictures"),
        base(&FOLDERID_Documents, "Documents"),
    )
}
/// Ask Shell to materialize a registered Windows folder before the normal
/// create/write probe. This is best effort; it never changes ACLs or protection.
pub fn prepare_known_media(paths: &[PathBuf; 5]) {
    for (index, id, name) in [
        (0, &FOLDERID_Videos, "Videos"),
        (1, &FOLDERID_Music, "Music"),
        (2, &FOLDERID_Pictures, "Pictures"),
        (4, &FOLDERID_Documents, "Documents"),
    ] {
        if user_folder(id, name).is_some_and(|p| paths[index].starts_with(p)) {
            unsafe {
                if let Ok(p) = SHGetKnownFolderPath(id, KF_FLAG_CREATE, None) {
                    CoTaskMemFree(Some(p.0.cast()));
                }
            }
        }
    }
}
pub fn key_name(key: u32) -> String {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    if key == 0x2c {
        return "PrintScreen".into();
    }
    unsafe {
        let scan = MapVirtualKeyW(key, MAPVK_VK_TO_VSC);
        let extended = matches!(key, 0x21..=0x28 | 0x2d | 0x2e | 0x6f);
        let mut name = [0u16; 128];
        let len = GetKeyNameTextW(((scan << 16) | ((extended as u32) << 24)) as i32, &mut name);
        if len > 0 {
            String::from_utf16_lossy(&name[..len as usize])
        } else {
            format!("VK {key:02X}")
        }
    }
}
/// Copies an uncompressed top-down DIB. Clipboard ownership transfers only on success.
pub fn set_clipboard_frame(owner: HWND, frame: &crate::core::Frame) -> AppResult<()> {
    use windows::Win32::{
        Graphics::Gdi::*,
        System::{DataExchange::*, Memory::*},
    };
    unsafe {
        struct ClipboardOwner(Option<HWND>);
        impl Drop for ClipboardOwner {
            fn drop(&mut self) {
                if let Some(hwnd) = self.0 {
                    unsafe {
                        let _ = DestroyWindow(hwnd);
                    }
                }
            }
        }
        // A NULL owner followed by EmptyClipboard prevents SetClipboardData.
        let temporary = ClipboardOwner(if owner.is_invalid() {
            Some(
                CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    w!("STATIC"),
                    w!(""),
                    WINDOW_STYLE::default(),
                    0,
                    0,
                    0,
                    0,
                    Some(HWND_MESSAGE),
                    None,
                    None,
                    None,
                )
                .map_err(|e| e.to_string())?,
            )
        } else {
            None
        });
        let owner = temporary.0.unwrap_or(owner);
        let header = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: frame.width as i32,
            biHeight: -(frame.height as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        };
        let head = std::mem::size_of_val(&header);
        let memory =
            GlobalAlloc(GMEM_MOVEABLE, head + frame.bgra.len()).map_err(|e| e.to_string())?;
        let data = GlobalLock(memory).cast::<u8>();
        if data.is_null() {
            let _ = GlobalFree(Some(memory));
            return Err(windows::core::Error::from_win32().to_string());
        }
        std::ptr::copy_nonoverlapping(
            (&header as *const BITMAPINFOHEADER).cast::<u8>(),
            data,
            head,
        );
        std::ptr::copy_nonoverlapping(frame.bgra.as_ptr(), data.add(head), frame.bgra.len());
        let _ = GlobalUnlock(memory);
        if let Err(e) = OpenClipboard(Some(owner)) {
            let _ = GlobalFree(Some(memory));
            return Err(e.to_string());
        }
        let result = (|| -> windows::core::Result<()> {
            EmptyClipboard()?;
            SetClipboardData(8, Some(HANDLE(memory.0)))?;
            Ok(())
        })();
        let _ = CloseClipboard();
        if result.is_err() {
            let _ = GlobalFree(Some(memory));
        }
        result.map_err(|e| e.to_string())
    }
}
pub fn installed_media_request() -> AppResult<Option<(String, PathBuf)>> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let root = exe
        .parent()
        .ok_or(crate::i18n::tr("Aplicativo sem pasta pai"))?;
    let Some(state) = crate::installer::engine::read_state(root)? else {
        return Ok(None);
    };
    let (Some(media), Some(id)) = (state.get("media_root"), state.get("storage_request_id")) else {
        return Ok(None);
    };
    crate::installer::engine::verified_owned_file(root, "WyzarCapture.exe")?;
    // 0.6.1/0.6.2 could record their automatic recovery under AppData/Media as
    // an installer choice. Do not reapply that obsolete fallback on an update.
    if Path::new(media) == data_folder().join("Media") {
        return Ok(None);
    }
    Ok(Some((id.clone(), PathBuf::from(media))))
}
pub fn installed_language_request() -> AppResult<Option<(String, crate::i18n::Language)>> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let root = exe
        .parent()
        .ok_or(crate::i18n::tr("Aplicativo sem pasta pai"))?;
    let Some(state) = crate::installer::engine::read_state(root)? else {
        return Ok(None);
    };
    let (Some(code), Some(identity)) = (state.get("language"), state.get("language_request_id"))
    else {
        return Ok(None);
    };
    let Some(language) = crate::i18n::Language::parse(code) else {
        return Ok(None);
    };
    crate::installer::engine::verified_owned_file(root, "WyzarCapture.exe")?;
    Ok(Some((identity.clone(), language)))
}
pub fn replace_file(from: &Path, to: &Path) -> AppResult<()> {
    unsafe {
        MoveFileExW(
            pcw(&wide(from)),
            pcw(&wide(to)),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
        .map_err(|e| e.to_string())
    }
}
pub fn local_date_time() -> (String, String) {
    unsafe {
        let t = GetLocalTime();
        (
            format!("{:04}-{:02}-{:02}", t.wYear, t.wMonth, t.wDay),
            format!("{:02}-{:02}-{:02}", t.wHour, t.wMinute, t.wSecond),
        )
    }
}
pub fn timestamp_label(seconds: u64) -> String {
    if seconds == 0 {
        return "—".into();
    }
    let Some(ticks) = seconds
        .checked_mul(10_000_000)
        .and_then(|n| n.checked_add(116_444_736_000_000_000))
    else {
        return "—".into();
    };
    unsafe {
        let ft = FILETIME {
            dwLowDateTime: ticks as u32,
            dwHighDateTime: (ticks >> 32) as u32,
        };
        let (mut utc, mut local) = (SYSTEMTIME::default(), SYSTEMTIME::default());
        if windows::Win32::System::Time::FileTimeToSystemTime(&ft, &mut utc).is_err()
            || windows::Win32::System::Time::SystemTimeToTzSpecificLocalTime(None, &utc, &mut local)
                .is_err()
        {
            return "—".into();
        }
        format!(
            "{:02}/{:02}/{:04} {:02}:{:02}",
            local.wDay, local.wMonth, local.wYear, local.wHour, local.wMinute
        )
    }
}
/// Windows "Animation effects" (Accessibility > Visual effects). Off means static UI.
pub fn animations_enabled() -> bool {
    if std::env::var_os("WYZAR_REDUCED_MOTION").is_some() {
        return false;
    }
    let mut enabled = windows::core::BOOL(1);
    unsafe {
        if SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            Some((&mut enabled as *mut windows::core::BOOL).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
        .is_err()
        {
            return true;
        }
    }
    enabled.as_bool()
}
pub fn free_space(path: &Path) -> Option<(u64, u64)> {
    let path = path.ancestors().find(|p| p.is_dir())?;
    unsafe {
        let (mut available, mut total, mut free) = (0, 0, 0);
        GetDiskFreeSpaceExW(
            pcw(&wide(path)),
            Some(&mut available),
            Some(&mut total),
            Some(&mut free),
        )
        .ok()?;
        Some((available, total))
    }
}
pub fn choose_folder(owner: HWND, current: &Path) -> AppResult<Option<PathBuf>> {
    unsafe {
        let dialog: IFileOpenDialog = CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)
            .map_err(|e| e.to_string())?;
        dialog
            .SetOptions(FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST | FOS_NOCHANGEDIR)
            .map_err(|e| e.to_string())?;
        dialog
            .SetTitle(crate::platform::windows::localized(
                "Escolha a pasta do WyzarCapture",
            ))
            .map_err(|e| e.to_string())?;
        if let Ok(item) = SHCreateItemFromParsingName::<_, _, IShellItem>(pcw(&wide(current)), None)
        {
            let _ = dialog.SetFolder(&item);
        }
        if let Err(e) = dialog.Show(Some(owner)) {
            if e.code() == HRESULT(0x800704c7u32 as i32) {
                return Ok(None);
            }
            return Err(e.to_string());
        }
        let item = dialog.GetResult().map_err(|e| e.to_string())?;
        let p = item
            .GetDisplayName(SIGDN_FILESYSPATH)
            .map_err(|e| e.to_string())?;
        let result = p.to_string().map(PathBuf::from).map_err(|e| e.to_string());
        CoTaskMemFree(Some(p.0.cast()));
        result.map(Some)
    }
}
pub fn choose_file(owner: HWND, project: bool) -> AppResult<Option<PathBuf>> {
    unsafe {
        let dialog: IFileOpenDialog = CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)
            .map_err(|e| e.to_string())?;
        dialog
            .SetOptions(FOS_FORCEFILESYSTEM | FOS_FILEMUSTEXIST | FOS_NOCHANGEDIR)
            .map_err(|e| e.to_string())?;
        dialog
            .SetTitle(if project {
                crate::platform::windows::localized("Abrir projeto de edição")
            } else {
                crate::platform::windows::localized("Importar vídeo ou trilha de áudio")
            })
            .map_err(|e| e.to_string())?;
        let spec = [COMDLG_FILTERSPEC {
            pszName: if project {
                crate::platform::windows::localized("Projeto WyzarCapture")
            } else {
                crate::platform::windows::localized("Vídeo ou áudio PCM")
            },
            pszSpec: if project {
                w!("*.wyzproj")
            } else {
                w!("*.mp4;*.wmv;*.mov;*.m4v;*.wav")
            },
        }];
        dialog.SetFileTypes(&spec).map_err(|e| e.to_string())?;
        if let Err(e) = dialog.Show(Some(owner)) {
            if e.code() == HRESULT(0x800704c7u32 as i32) {
                return Ok(None);
            }
            return Err(e.to_string());
        }
        let item = dialog.GetResult().map_err(|e| e.to_string())?;
        let p = item
            .GetDisplayName(SIGDN_FILESYSPATH)
            .map_err(|e| e.to_string())?;
        let result = p.to_string().map(PathBuf::from).map_err(|e| e.to_string());
        CoTaskMemFree(Some(p.0.cast()));
        result.map(Some)
    }
}
pub fn choose_svg(owner: HWND) -> AppResult<Option<PathBuf>> {
    unsafe {
        let dialog: IFileOpenDialog = CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)
            .map_err(|e| e.to_string())?;
        dialog
            .SetOptions(FOS_FORCEFILESYSTEM | FOS_FILEMUSTEXIST | FOS_NOCHANGEDIR)
            .map_err(|e| e.to_string())?;
        dialog
            .SetTitle(crate::platform::windows::localized(
                "Personalizar ícone · SVG local 24 × 24",
            ))
            .map_err(|e| e.to_string())?;
        dialog
            .SetFileTypes(&[COMDLG_FILTERSPEC {
                pszName: crate::platform::windows::localized("Ícone vetorial SVG"),
                pszSpec: w!("*.svg"),
            }])
            .map_err(|e| e.to_string())?;
        if let Err(e) = dialog.Show(Some(owner)) {
            if e.code() == HRESULT(0x800704c7u32 as i32) {
                return Ok(None);
            }
            return Err(e.to_string());
        }
        let p = dialog
            .GetResult()
            .and_then(|i| i.GetDisplayName(SIGDN_FILESYSPATH))
            .map_err(|e| e.to_string())?;
        let result = p.to_string().map(PathBuf::from).map_err(|e| e.to_string());
        CoTaskMemFree(Some(p.0.cast()));
        result.map(Some)
    }
}
pub fn open_path(owner: HWND, path: &Path) {
    unsafe {
        ShellExecuteW(
            Some(owner),
            w!("open"),
            pcw(&wide(path)),
            None,
            None,
            SW_SHOWNORMAL,
        );
    }
}
pub fn reveal(owner: HWND, path: &Path) {
    unsafe {
        let args = format!("/select,\"{}\"", path.display());
        ShellExecuteW(
            Some(owner),
            w!("open"),
            w!("explorer.exe"),
            pcw(&wide(args)),
            None,
            SW_SHOWNORMAL,
        );
    }
}
pub fn recycle_many(owner: HWND, paths: &[PathBuf]) -> AppResult<()> {
    if paths.is_empty() {
        return Ok(());
    }
    unsafe {
        let op: IFileOperation = CoCreateInstance(&FileOperation, None, CLSCTX_INPROC_SERVER)
            .map_err(|e| e.to_string())?;
        op.SetOwnerWindow(owner).map_err(|e| e.to_string())?;
        op.SetOperationFlags(FOF_ALLOWUNDO | FOF_NOCONFIRMMKDIR | FOFX_RECYCLEONDELETE)
            .map_err(|e| e.to_string())?;
        for path in paths {
            let item: IShellItem =
                SHCreateItemFromParsingName(pcw(&wide(path)), None).map_err(|e| e.to_string())?;
            op.DeleteItem(&item, None).map_err(|e| e.to_string())?;
        }
        op.PerformOperations().map_err(|e| e.to_string())?;
        if op
            .GetAnyOperationsAborted()
            .map_err(|e| e.to_string())?
            .as_bool()
        {
            return Err(crate::i18n::tr("Exclusão cancelada").into());
        }
        Ok(())
    }
}
pub fn clipboard_text(owner: HWND) -> Option<String> {
    use windows::Win32::System::{DataExchange::*, Memory::*};
    unsafe {
        OpenClipboard(Some(owner)).ok()?;
        struct Clipboard;
        impl Drop for Clipboard {
            fn drop(&mut self) {
                unsafe {
                    let _ = CloseClipboard();
                }
            }
        }
        let _guard = Clipboard;
        let handle = GetClipboardData(13).ok()?;
        let memory = HGLOBAL(handle.0);
        let count = (GlobalSize(memory) / 2).min(1_000_000);
        let data = GlobalLock(memory).cast::<u16>();
        if data.is_null() {
            return None;
        }
        let slice = std::slice::from_raw_parts(data, count);
        let len = slice.iter().position(|c| *c == 0).unwrap_or(count);
        let text = String::from_utf16_lossy(&slice[..len]);
        let _ = GlobalUnlock(memory);
        Some(text)
    }
}
pub fn set_clipboard_text(owner: HWND, text: &str) -> AppResult<()> {
    use windows::Win32::System::{DataExchange::*, Memory::*};
    unsafe {
        let data = wide(text);
        let memory = GlobalAlloc(GMEM_MOVEABLE, data.len() * 2).map_err(|e| e.to_string())?;
        let pointer = GlobalLock(memory).cast::<u16>();
        if pointer.is_null() {
            let _ = GlobalFree(Some(memory));
            return Err(
                crate::i18n::tr("Não foi possível reservar a área de transferência").into(),
            );
        }
        std::ptr::copy_nonoverlapping(data.as_ptr(), pointer, data.len());
        let _ = GlobalUnlock(memory);
        if let Err(error) = OpenClipboard(Some(owner)) {
            let _ = GlobalFree(Some(memory));
            return Err(error.to_string());
        }
        let result = (|| -> windows::core::Result<()> {
            EmptyClipboard()?;
            SetClipboardData(13, Some(HANDLE(memory.0)))?;
            Ok(())
        })();
        let _ = CloseClipboard();
        if result.is_err() {
            let _ = GlobalFree(Some(memory));
        }
        result.map_err(|e| e.to_string())
    }
}
/// Synthetic >4 GiB audio fixture without allocating that disk space.
/// Never used by recording; constrained to validation paths.
#[cfg(test)]
pub fn make_sparse_fixture(
    file: &std::fs::File,
    path: &std::path::Path,
) -> crate::core::AppResult<()> {
    use std::os::windows::io::AsRawHandle;
    if !path.components().any(|c| c.as_os_str() == "validation") {
        return Err("Sparse QA fora de validation".into());
    }
    unsafe {
        let mut returned = 0;
        windows::Win32::System::IO::DeviceIoControl(
            windows::Win32::Foundation::HANDLE(file.as_raw_handle()),
            590020, // FSCTL_SET_SPARSE, Windows SDK winioctl.h
            None,
            0,
            None,
            0,
            Some(&mut returned),
            None,
        )
        .map_err(|e| e.to_string())
    }
}
