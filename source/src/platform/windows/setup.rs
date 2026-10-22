//! Installer OS boundary: CNG, COM/Shell, registry and process lifecycle.
use super::{pcw, wide};
use crate::{
    core::AppResult,
    installer::{Options, Scope},
};
use std::{
    fs::File,
    io::Read,
    os::windows::fs::MetadataExt,
    path::{Path, PathBuf},
};
use windows::{
    Win32::{
        Foundation::*,
        Security::Cryptography::*,
        System::{Com::*, ProcessStatus::EnumProcesses, Registry::*, Threading::*},
        UI::{Shell::*, WindowsAndMessaging::*},
    },
    core::{GUID, Interface, PWSTR, w},
};
pub const CLOSE_FOR_INSTALL: u32 = WM_APP + 42;
pub struct Com;
impl Com {
    pub fn new() -> AppResult<Self> {
        Self::apartment(COINIT_APARTMENTTHREADED)
    }
    pub fn multithreaded() -> AppResult<Self> {
        Self::apartment(COINIT_MULTITHREADED)
    }
    fn apartment(kind: COINIT) -> AppResult<Self> {
        unsafe {
            CoInitializeEx(None, kind).ok().map_err(|e| e.to_string())?;
        }
        Ok(Self)
    }
}
impl Drop for Com {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
struct Algorithm(BCRYPT_ALG_HANDLE);
impl Drop for Algorithm {
    fn drop(&mut self) {
        unsafe {
            let _ = BCryptCloseAlgorithmProvider(self.0, 0);
        }
    }
}
struct Hash(BCRYPT_HASH_HANDLE);
impl Drop for Hash {
    fn drop(&mut self) {
        unsafe {
            let _ = BCryptDestroyHash(self.0);
        }
    }
}
fn hash_reader(mut r: impl Read) -> AppResult<String> {
    unsafe {
        let mut alg = BCRYPT_ALG_HANDLE::default();
        BCryptOpenAlgorithmProvider(
            &mut alg,
            BCRYPT_SHA256_ALGORITHM,
            None,
            BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0),
        )
        .ok()
        .map_err(|e| e.to_string())?;
        let alg = Algorithm(alg);
        let mut hash = BCRYPT_HASH_HANDLE::default();
        BCryptCreateHash(alg.0, &mut hash, None, None, 0)
            .ok()
            .map_err(|e| e.to_string())?;
        let hash = Hash(hash);
        let mut buffer = [0u8; 65536];
        loop {
            let count = r.read(&mut buffer).map_err(|e| e.to_string())?;
            if count == 0 {
                break;
            }
            BCryptHashData(hash.0, &buffer[..count], 0)
                .ok()
                .map_err(|e| e.to_string())?;
        }
        let mut digest = [0u8; 32];
        BCryptFinishHash(hash.0, &mut digest, 0)
            .ok()
            .map_err(|e| e.to_string())?;
        Ok(digest.iter().map(|b| format!("{b:02X}")).collect())
    }
}
pub fn hash_file(path: &Path) -> AppResult<String> {
    guard(path)?;
    hash_reader(File::open(path).map_err(|e| e.to_string())?)
}
pub fn hash_bytes(bytes: &[u8]) -> AppResult<String> {
    hash_reader(bytes)
}
pub fn verify_default_stream(path: &Path) -> AppResult<()> {
    use windows::Win32::Storage::FileSystem::*;
    guard(path)?;
    struct Streams(HANDLE);
    impl Drop for Streams {
        fn drop(&mut self) {
            unsafe {
                let _ = FindClose(self.0);
            }
        }
    }
    unsafe {
        let mut data = WIN32_FIND_STREAM_DATA::default();
        let handle = match FindFirstStreamW(
            pcw(&wide(path)),
            FindStreamInfoStandard,
            (&mut data as *mut WIN32_FIND_STREAM_DATA).cast(),
            None,
        ) {
            Ok(handle) => Streams(handle),
            // FAT/exFAT do not support alternate streams.
            Err(e) if e.code() == windows::core::HRESULT::from_win32(ERROR_INVALID_FUNCTION.0) => {
                return Ok(());
            }
            Err(e) => {
                return Err(crate::i18n::format_text(
                    crate::i18n::tr("Não foi possível validar streams do payload: {e}"),
                    &[("e", format!("{}", e))],
                ));
            }
        };
        loop {
            let end = data
                .cStreamName
                .iter()
                .position(|c| *c == 0)
                .ok_or(crate::i18n::tr("Nome de stream inválido"))?;
            if String::from_utf16_lossy(&data.cStreamName[..end]) != "::$DATA" {
                return Err(
                    crate::i18n::tr("Stream alternativo (ADS) inesperado no payload").into(),
                );
            }
            if let Err(e) =
                FindNextStreamW(handle.0, (&mut data as *mut WIN32_FIND_STREAM_DATA).cast())
            {
                if e.code() == windows::core::HRESULT::from_win32(ERROR_HANDLE_EOF.0) {
                    return Ok(());
                }
                return Err(e.to_string());
            }
        }
    }
}
/// Map an EXE strictly as a data file: no DllMain, imports or payload execution.
/// The borrowed resource is valid only inside `read`; no full payload Vec is made.
pub fn with_rcdata<T>(
    exe: &Path,
    id: u16,
    read: impl FnOnce(&[u8], Option<u64>) -> AppResult<T>,
) -> AppResult<Option<T>> {
    use windows::Win32::Media::KernelStreaming::RT_RCDATA;
    use windows::Win32::System::LibraryLoader::*;
    guard(exe)?;
    struct ResourceModule(HMODULE, bool);
    impl Drop for ResourceModule {
        fn drop(&mut self) {
            if self.1 {
                unsafe {
                    let _ = FreeLibrary(self.0);
                }
            }
        }
    }
    unsafe {
        let own_image = std::env::current_exe().ok().as_deref() == Some(exe);
        let module = if own_image {
            // Borrow the already running Setup. Loading it again may return an
            // image mapping; never interpret an RVA as a raw file offset.
            ResourceModule(GetModuleHandleW(None).map_err(|e| e.to_string())?, false)
        } else {
            ResourceModule(
                LoadLibraryExW(pcw(&wide(exe)), None, LOAD_LIBRARY_AS_DATAFILE_EXCLUSIVE).map_err(
                    |e| {
                        crate::i18n::format_text(
                            crate::i18n::tr("Mapeamento de recurso: {e}"),
                            &[("e", format!("{}", e))],
                        )
                    },
                )?,
                true,
            )
        };
        let tagged = module.0.0 as usize;
        if !own_image && tagged & 1 == 0 {
            return Err(crate::i18n::tr("Recurso não foi mapeado como arquivo de dados").into());
        }
        let resource = FindResourceW(
            Some(module.0),
            windows::core::PCWSTR(id as usize as *const u16),
            RT_RCDATA,
        );
        if resource.0.is_null() {
            let error = GetLastError();
            if error == ERROR_RESOURCE_NAME_NOT_FOUND || error == ERROR_RESOURCE_TYPE_NOT_FOUND {
                return Ok(None);
            }
            return Err({
                let __wyzar_i18n_0 = &(error.0);
                crate::i18n::format_text(
                    crate::i18n::tr("Recurso RCDATA inválido: {}"),
                    &[("0", format!("{}", __wyzar_i18n_0))],
                )
            });
        }
        let size = SizeofResource(Some(module.0), resource) as usize;
        let data = LockResource(LoadResource(Some(module.0), resource).map_err(|e| e.to_string())?);
        if size == 0 || data.is_null() {
            return Err(crate::i18n::tr("Recurso RCDATA vazio").into());
        }
        let offset = (data as usize)
            .checked_sub(tagged & !3)
            .ok_or(crate::i18n::tr("Offset RCDATA inválido"))? as u64;
        let bound = if own_image {
            use windows::Win32::System::ProcessStatus::{GetModuleInformation, MODULEINFO};
            let mut info = MODULEINFO::default();
            GetModuleInformation(
                GetCurrentProcess(),
                module.0,
                &mut info,
                std::mem::size_of_val(&info) as u32,
            )
            .map_err(|e| e.to_string())?;
            info.SizeOfImage as u64
        } else {
            std::fs::metadata(exe).map_err(|e| e.to_string())?.len()
        };
        if offset
            .checked_add(size as u64)
            .filter(|end| *end <= bound)
            .is_none()
        {
            return Err(crate::i18n::tr("Recurso RCDATA excede o arquivo").into());
        }
        // Windows validated the raw mapping and resource bounds; module remains
        // alive throughout the closure and all copying/hashing stays bounded.
        read(
            std::slice::from_raw_parts(data.cast::<u8>(), size),
            if own_image { None } else { Some(offset) },
        )
        .map(Some)
    }
}
/// A destination lock covers recovery as well as mutations. The owner is thread-bound.
pub struct DestinationLock {
    handle: HANDLE,
    _thread: std::marker::PhantomData<std::rc::Rc<()>>,
}
impl DestinationLock {
    pub fn acquire(destination: &Path) -> AppResult<Self> {
        let normalized = normalized_folder(destination)?;
        Self::named(&format!(
            "Global\\WyzarCapture.Setup.{}",
            hash_bytes(normalized.as_bytes())?
        ))
    }
    /// Different destinations can still share uninstall registration and shortcuts.
    /// Acquire this before the destination lock to give every operation one order.
    pub fn metadata(registration: &str) -> AppResult<Self> {
        Self::named(&format!(
            "Global\\WyzarCapture.Setup.Registry.{}",
            hash_bytes(registration.to_lowercase().as_bytes())?
        ))
    }
    fn named(name: &str) -> AppResult<Self> {
        unsafe {
            let handle = CreateMutexW(None, false, pcw(&wide(name))).map_err(|_| crate::i18n::tr("Outra instalação do WyzarCapture já está em andamento ou o bloqueio não pôde ser obtido").to_string())?;
            let result = WaitForSingleObject(handle, 0);
            if result != WAIT_OBJECT_0 && result != WAIT_ABANDONED {
                let _ = CloseHandle(handle);
                return Err(crate::i18n::tr(
                    "Outra instalação do WyzarCapture já está em andamento.",
                )
                .into());
            }
            Ok(Self {
                handle,
                _thread: std::marker::PhantomData,
            })
        }
    }
}
/// Resolve existing ancestors, including 8.3 aliases, and compare without case,
/// as Windows does for folder names.
pub fn normalized_folder(destination: &Path) -> AppResult<String> {
    guard(destination)?;
    let ancestor = destination
        .ancestors()
        .find(|p| p.exists())
        .ok_or(crate::i18n::tr("Destino sem volume acessível"))?;
    let resolved = std::fs::canonicalize(ancestor)
        .map_err(|e| e.to_string())?
        .join(
            destination
                .strip_prefix(ancestor)
                .map_err(|e| e.to_string())?,
        );
    Ok(resolved
        .to_string_lossy()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase())
}
/// An unresolvable path is never treated as the same folder.
pub fn same_folder(a: &Path, b: &Path) -> bool {
    match (normalized_folder(a), normalized_folder(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}
impl Drop for DestinationLock {
    fn drop(&mut self) {
        unsafe {
            let _ = ReleaseMutex(self.handle);
            let _ = CloseHandle(self.handle);
        }
    }
}
pub fn process_alive(pid: u32) -> bool {
    unsafe {
        match OpenProcess(PROCESS_SYNCHRONIZE, false, pid) {
            Ok(handle) => {
                let active = WaitForSingleObject(handle, 0) != WAIT_OBJECT_0;
                let _ = CloseHandle(handle);
                active
            }
            Err(e) => e.code() != windows::core::HRESULT::from_win32(ERROR_INVALID_PARAMETER.0),
        }
    }
}
pub fn guard(path: &Path) -> AppResult<()> {
    let s = path
        .to_str()
        .ok_or(crate::i18n::tr("O caminho precisa ser Unicode válido"))?;
    let b = s.as_bytes();
    if !path.is_absolute()
        || b.len() < 4
        || !b[0].is_ascii_alphabetic()
        || b[1] != b':'
        || !matches!(b[2], b'\\' | b'/')
        || s[2..].contains(':')
        || s.chars().any(|c| c < ' ')
    {
        return Err(
            crate::i18n::tr("Escolha um caminho local absoluto, fora da raiz do disco").into(),
        );
    }
    for component in path.components().skip(2) {
        match component {
            std::path::Component::Normal(v) => {
                let v = v
                    .to_str()
                    .ok_or(crate::i18n::tr("Caminho Unicode inválido"))?;
                crate::installer::relative(v)?;
            }
            _ => return Err(crate::i18n::tr("Travessia de caminho recusada").into()),
        }
    }
    for entry in path.ancestors() {
        if let Ok(meta) = std::fs::symlink_metadata(entry) {
            if meta.file_attributes() & 0x400 != 0 {
                return Err(
                    crate::i18n::tr("Links/junctions não são permitidos na instalação").into(),
                );
            }
        }
    }
    Ok(())
}
pub fn known_folder(id: &GUID) -> AppResult<PathBuf> {
    unsafe {
        let p = SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None).map_err(|e| e.to_string())?;
        let s = p.to_string().map_err(|e| e.to_string());
        CoTaskMemFree(Some(p.0.cast()));
        s.map(PathBuf::from)
    }
}
pub fn default_destination(scope: Scope) -> PathBuf {
    if scope == Scope::Machine {
        known_folder(&FOLDERID_ProgramFiles)
            .unwrap_or_else(|_| PathBuf::from(r"C:\Program Files"))
            .join("WyzarCapture")
    } else {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("Programs")
            .join("WyzarCapture")
    }
}
pub fn is_admin() -> bool {
    unsafe { IsUserAnAdmin().as_bool() }
}
pub fn validate_scope(options: &Options) -> AppResult<()> {
    options.scope.ensure_available()?;
    validate_destination(options)?;
    if options.scope == Scope::Machine && !is_admin() {
        return Err(
            crate::i18n::tr("Esta instalação precisa de permissão de administrador").into(),
        );
    }
    Ok(())
}
pub fn validate_destination(options: &Options) -> AppResult<()> {
    guard(&options.destination)?;
    if options.destination.components().count() < 3 {
        return Err(crate::i18n::tr("Não instale na raiz do disco").into());
    }
    if options.scope == Scope::Machine {
        let base = default_destination(Scope::Machine)
            .parent()
            .unwrap()
            .to_path_buf();
        if !options.destination.starts_with(&base) || options.destination == base {
            return Err(crate::i18n::tr("Para todos os usuários, use uma subpasta de Program Files. Outras pastas podem ser usadas em Somente para mim.").into());
        }
    }
    Ok(())
}
/// Selecting a volume means choosing our own subfolder, never owning the root.
pub fn install_folder(path: PathBuf) -> AppResult<PathBuf> {
    let path = if path.is_absolute() && path.components().count() == 2 {
        path.join("WyzarCapture")
    } else {
        path
    };
    guard(&path)?;
    Ok(path)
}
#[derive(Clone, Debug, PartialEq)]
pub struct Value {
    pub kind: REG_VALUE_TYPE,
    pub bytes: Vec<u8>,
}
struct Key(HKEY);
impl Drop for Key {
    fn drop(&mut self) {
        unsafe {
            let _ = RegCloseKey(self.0);
        }
    }
}
fn hive(scope: Scope) -> HKEY {
    if scope == Scope::Machine {
        HKEY_LOCAL_MACHINE
    } else {
        HKEY_CURRENT_USER
    }
}
fn key(scope: Scope, path: &str, create: bool, write: bool) -> AppResult<Option<Key>> {
    unsafe {
        let mut k = HKEY::default();
        let status = if create {
            RegCreateKeyExW(
                hive(scope),
                pcw(&wide(path)),
                Some(0),
                None,
                REG_OPTION_NON_VOLATILE,
                KEY_READ | KEY_WRITE | KEY_WOW64_64KEY,
                None,
                &mut k,
                None,
            )
        } else {
            RegOpenKeyExW(
                hive(scope),
                pcw(&wide(path)),
                Some(0),
                (if write {
                    KEY_READ | KEY_WRITE
                } else {
                    KEY_READ
                }) | KEY_WOW64_64KEY,
                &mut k,
            )
        };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        status.ok().map_err(|e| e.to_string())?;
        Ok(Some(Key(k)))
    }
}
pub fn reg_get(scope: Scope, path: &str, name: &str) -> AppResult<Option<Value>> {
    let Some(k) = key(scope, path, false, false)? else {
        return Ok(None);
    };
    unsafe {
        let mut length = 0;
        let mut kind = REG_NONE;
        let status = RegQueryValueExW(
            k.0,
            pcw(&wide(name)),
            None,
            Some(&mut kind),
            None,
            Some(&mut length),
        );
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        status.ok().map_err(|e| e.to_string())?;
        if length > 65536 {
            return Err(crate::i18n::tr("Valor de registro excede o limite").into());
        }
        let mut bytes = vec![0; length as usize];
        RegQueryValueExW(
            k.0,
            pcw(&wide(name)),
            None,
            Some(&mut kind),
            Some(bytes.as_mut_ptr()),
            Some(&mut length),
        )
        .ok()
        .map_err(|e| e.to_string())?;
        bytes.truncate(length as usize);
        Ok(Some(Value { kind, bytes }))
    }
}
pub fn reg_set(scope: Scope, path: &str, name: &str, value: Option<&Value>) -> AppResult<()> {
    let k = key(scope, path, value.is_some(), true)?;
    if let Some(k) = k {
        unsafe {
            let status = if let Some(v) = value {
                RegSetValueExW(k.0, pcw(&wide(name)), Some(0), v.kind, Some(&v.bytes))
            } else {
                RegDeleteValueW(k.0, pcw(&wide(name)))
            };
            if status != ERROR_FILE_NOT_FOUND {
                status.ok().map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}
pub fn reg_text(value: &str) -> Value {
    let bytes = wide(value).iter().flat_map(|c| c.to_le_bytes()).collect();
    Value {
        kind: REG_SZ,
        bytes,
    }
}
pub fn text(value: &Value) -> Option<String> {
    if value.kind != REG_SZ {
        return None;
    }
    let chars = value
        .bytes
        .chunks_exact(2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .take_while(|c| *c != 0)
        .collect::<Vec<_>>();
    String::from_utf16(&chars).ok()
}
pub fn dword(value: u32) -> Value {
    Value {
        kind: REG_DWORD,
        bytes: value.to_le_bytes().to_vec(),
    }
}
pub fn reg_remove_empty(scope: Scope, path: &str) {
    unsafe {
        if let Ok(Some(k)) = key(scope, path, false, false) {
            let (mut subkeys, mut values) = (0, 0);
            if RegQueryInfoKeyW(
                k.0,
                None,
                None,
                None,
                Some(&mut subkeys),
                None,
                None,
                Some(&mut values),
                None,
                None,
                None,
                None,
            )
            .is_ok()
                && subkeys == 0
                && values == 0
            {
                drop(k);
                let _ = RegDeleteKeyExW(hive(scope), pcw(&wide(path)), KEY_WOW64_64KEY.0, Some(0));
            }
        }
    }
}
pub fn shortcut(path: &Path, target: &Path) -> AppResult<()> {
    guard(path)?;
    guard(target)?;
    unsafe {
        let link: IShellLinkW =
            CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).map_err(|e| e.to_string())?;
        link.SetPath(pcw(&wide(target)))
            .map_err(|e| e.to_string())?;
        link.SetWorkingDirectory(pcw(&wide(
            target
                .parent()
                .ok_or(crate::i18n::tr("Destino sem pasta"))?,
        )))
        .map_err(|e| e.to_string())?;
        link.SetDescription(crate::platform::windows::localized(
            "WyzarCapture — captura de tela e áudio",
        ))
        .map_err(|e| e.to_string())?;
        link.SetIconLocation(pcw(&wide(target)), 0)
            .map_err(|e| e.to_string())?;
        let persist: IPersistFile = link.cast().map_err(|e| e.to_string())?;
        persist
            .Save(pcw(&wide(path)), true)
            .map_err(|e| e.to_string())
    }
}
pub fn shortcut_target(path: &Path) -> AppResult<PathBuf> {
    unsafe {
        let link: IShellLinkW =
            CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).map_err(|e| e.to_string())?;
        let persist: IPersistFile = link.cast().map_err(|e| e.to_string())?;
        persist
            .Load(pcw(&wide(path)), STGM_READ)
            .map_err(|e| e.to_string())?;
        let mut buffer = [0u16; 32768];
        link.GetPath(&mut buffer, std::ptr::null_mut(), SLGP_RAWPATH.0 as u32)
            .map_err(|e| e.to_string())?;
        Ok(PathBuf::from(String::from_utf16_lossy(
            &buffer[..buffer.iter().position(|c| *c == 0).unwrap_or(buffer.len())],
        )))
    }
}
pub fn launch(path: &Path, args: &[String]) -> AppResult<()> {
    // Per-user production installs never need to give the recorder an elevated
    // token. Do not silently inherit it if the owner ran Setup as administrator.
    if process_elevated()? {
        return Err(crate::i18n::tr("Não é possível abrir o aplicativo automaticamente a partir de um Setup elevado. Feche o Setup e use o atalho do WyzarCapture na sua sessão normal.").into());
    }
    use std::os::windows::process::CommandExt;
    std::process::Command::new(path)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW.0)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}
pub fn process_elevated() -> AppResult<bool> {
    use windows::Win32::Security::*;
    unsafe {
        let mut token = HANDLE::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)
            .map_err(|e| e.to_string())?;
        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = 0;
        let result = GetTokenInformation(
            token,
            TokenElevation,
            Some((&mut elevation as *mut TOKEN_ELEVATION).cast()),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        );
        let _ = CloseHandle(token);
        result.map_err(|e| e.to_string())?;
        Ok(elevation.TokenIsElevated != 0)
    }
}
pub fn elevate(exe: &Path, args: &[String]) -> AppResult<()> {
    Scope::Machine.ensure_available()?;
    unsafe {
        let file = wide(exe);
        let parameters = wide(
            args.iter()
                .map(|s| quote_argument(s))
                .collect::<Vec<_>>()
                .join(" "),
        );
        let mut info = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            lpVerb: w!("runas"),
            lpFile: pcw(&file),
            lpParameters: pcw(&parameters),
            nShow: SW_SHOWNORMAL.0,
            ..Default::default()
        };
        ShellExecuteExW(&mut info).map_err(|e| {
            if e.code() == windows::core::HRESULT::from_win32(ERROR_CANCELLED.0) {
                crate::i18n::tr("Elevação cancelada. Nenhum arquivo da instalação foi alterado.")
                    .into()
            } else {
                e.to_string()
            }
        })
    }
}
pub fn quote_argument(value: &str) -> String {
    let mut out = String::from("\"");
    let mut slashes = 0;
    for c in value.chars() {
        if c == '\\' {
            slashes += 1;
            continue;
        }
        if c == '"' {
            out.extend(std::iter::repeat_n('\\', slashes * 2 + 1));
            out.push(c);
        } else {
            out.extend(std::iter::repeat_n('\\', slashes));
            out.push(c);
        }
        slashes = 0;
    }
    out.extend(std::iter::repeat_n('\\', slashes * 2));
    out.push('"');
    out
}
#[derive(Clone, Debug)]
pub struct Running {
    pub hwnd: isize,
    pub pid: u32,
}
pub fn running(destination: &Path) -> Vec<Running> {
    struct Search<'a> {
        root: &'a Path,
        out: Vec<Running>,
    }
    unsafe extern "system" fn enumerate(hwnd: HWND, l: LPARAM) -> windows::core::BOOL {
        let s = &mut *(l.0 as *mut Search<'_>);
        let mut class = [0u16; 128];
        let length = GetClassNameW(hwnd, &mut class);
        if String::from_utf16_lossy(&class[..length.max(0) as usize]) != "WyzarCapture.Native" {
            return true.into();
        }
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            let mut buffer = [0u16; 32768];
            let mut size = buffer.len() as u32;
            if QueryFullProcessImageNameW(
                process,
                PROCESS_NAME_WIN32,
                PWSTR(buffer.as_mut_ptr()),
                &mut size,
            )
            .is_ok()
            {
                let file = PathBuf::from(String::from_utf16_lossy(&buffer[..size as usize]));
                if file
                    .file_name()
                    .is_some_and(|f| f.eq_ignore_ascii_case("WyzarCapture.exe"))
                    && file.starts_with(s.root)
                    && !s.out.iter().any(|r| r.pid == pid)
                {
                    s.out.push(Running {
                        hwnd: hwnd.0 as isize,
                        pid,
                    });
                }
            }
            let _ = CloseHandle(process);
        }
        true.into()
    }
    let mut s = Search {
        root: destination,
        out: vec![],
    };
    unsafe {
        let _ = EnumWindows(
            Some(enumerate),
            LPARAM((&mut s as *mut Search<'_>) as isize),
        );
        // Include tray-only/headless instances; never replace a mapped executable silently.
        let mut pids = [0u32; 8192];
        let mut bytes = 0;
        if EnumProcesses(
            pids.as_mut_ptr(),
            std::mem::size_of_val(&pids) as u32,
            &mut bytes,
        )
        .is_ok()
        {
            for pid in &pids[..(bytes as usize / 4).min(pids.len())] {
                if s.out.iter().any(|r| r.pid == *pid) {
                    continue;
                }
                if let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, *pid) {
                    let mut path = [0u16; 32768];
                    let mut length = path.len() as u32;
                    if QueryFullProcessImageNameW(
                        handle,
                        PROCESS_NAME_WIN32,
                        PWSTR(path.as_mut_ptr()),
                        &mut length,
                    )
                    .is_ok()
                    {
                        let file =
                            PathBuf::from(String::from_utf16_lossy(&path[..length as usize]));
                        if file == destination.join("WyzarCapture.exe") {
                            s.out.push(Running { hwnd: 0, pid: *pid });
                        }
                    }
                    let _ = CloseHandle(handle);
                }
            }
        }
    }
    s.out
}
pub fn close_app(destination: &Path) -> AppResult<()> {
    for r in running(destination) {
        if r.hwnd == 0 {
            return Err(crate::i18n::tr(
                "Há uma instância sem janela. Aguarde sua conclusão antes de continuar.",
            )
            .into());
        }
        unsafe {
            let hwnd = HWND(r.hwnd as *mut _);
            let _ = PostMessageW(Some(hwnd), CLOSE_FOR_INSTALL, WPARAM(0), LPARAM(0));
            let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }
    let start = std::time::Instant::now();
    while !running(destination).is_empty() {
        if start.elapsed().as_secs() > 45 {
            return Err(crate::i18n::tr("O WyzarCapture continua ativo. Feche-o pela bandeja e tente novamente. Nenhum processo foi morto.").into());
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    Ok(())
}
pub fn wait_for_exit(pid: u32) -> AppResult<()> {
    unsafe {
        if let Ok(handle) = OpenProcess(PROCESS_SYNCHRONIZE, false, pid) {
            let status = WaitForSingleObject(handle, 15000);
            let _ = CloseHandle(handle);
            if status != WAIT_OBJECT_0 {
                return Err(
                    crate::i18n::tr("O desinstalador anterior ainda está encerrando").into(),
                );
            }
        }
    }
    Ok(())
}
fn runner_receipt(root: &Path) -> AppResult<std::collections::BTreeMap<String, String>> {
    let marker = root.join(".wyzar-uninstall-cache");
    let runner = root.join("WyzarCaptureSetup.exe");
    guard(&marker)?;
    guard(&runner)?;
    if std::fs::metadata(&marker).map_err(|e| e.to_string())?.len() > 8192 {
        return Err(crate::i18n::tr("Recibo temporário inválido").into());
    }
    let receipt = crate::persistence::read(&marker)?;
    if receipt.get("product").map(String::as_str) != Some("WyzarCapture.UninstallRunner")
        || receipt.get("schema").map(String::as_str) != Some("2")
        || receipt.get("sha256") != Some(&hash_file(&runner)?)
        || receipt.get("bytes")
            != Some(
                &std::fs::metadata(&runner)
                    .map_err(|e| e.to_string())?
                    .len()
                    .to_string(),
            )
    {
        return Err(crate::i18n::tr("Cópia temporária não corresponde ao recibo").into());
    }
    Ok(receipt)
}
pub fn clean_old_runners(current: &Path) {
    let temp = std::env::temp_dir();
    if guard(&temp).is_err() {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(&temp) {
        for entry in entries.flatten().take(20000) {
            let dir = entry.path();
            let Some(suffix) = entry
                .file_name()
                .to_str()
                .and_then(|s| s.strip_prefix("WyzarCapture-Uninstall-"))
                .map(str::to_owned)
            else {
                continue;
            };
            if suffix.parse::<u128>().is_err() || current.starts_with(&dir) || guard(&dir).is_err()
            {
                continue;
            }
            let marker = dir.join(".wyzar-uninstall-cache");
            let runner = dir.join("WyzarCaptureSetup.exe");
            let Ok(receipt) = runner_receipt(&dir) else {
                continue;
            };
            if receipt
                .get("runner_pid")
                .and_then(|p| p.parse().ok())
                .is_some_and(process_alive)
            {
                continue;
            }
            // Never recursively delete an untrusted temporary directory.
            if std::fs::remove_file(runner).is_ok() {
                let _ = std::fs::remove_file(dir.join(".wyzar-cleanup-pending"));
                let _ = std::fs::remove_file(marker);
                let _ = std::fs::remove_dir(dir);
            }
        }
    }
}
/// A running image cannot be reliably removed on supported Windows versions.
/// Record cleanup for the next Setup instead of attempting generic self-delete,
/// creating a helper shell/process, or registering persistence/reboot actions.
pub fn finish_runner_cleanup() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let root = exe.parent()?;
    let marker = root.join(".wyzar-uninstall-cache");
    if root.parent() != Some(std::env::temp_dir().as_path())
        || guard(&exe).is_err()
        || !root
            .file_name()?
            .to_str()?
            .starts_with("WyzarCapture-Uninstall-")
    {
        return None;
    }
    let mut receipt = runner_receipt(root).ok()?;
    receipt.insert("runner_pid".into(), std::process::id().to_string());
    crate::persistence::write(&marker, &receipt).ok()?;
    let status = "pending-next-setup";
    let _ = std::fs::write(root.join(".wyzar-cleanup-pending"), status);
    Some(format!("status={status}\npath={}\n", exe.display()))
}
/// Test-only effect constrained to a fresh validation folder. Restores the original DACL.
pub fn with_write_denied(path: &Path, check: impl FnOnce() -> AppResult<()>) -> AppResult<()> {
    use windows::Win32::Security::{Authorization::*, *};
    guard(path)?;
    if !path.components().any(|c| c.as_os_str() == "validation") {
        return Err("ACL de teste fora de validation".into());
    }
    unsafe {
        let name = wide(path);
        let mut original = PSECURITY_DESCRIPTOR::default();
        let mut dacl = std::ptr::null_mut();
        GetNamedSecurityInfoW(
            pcw(&name),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(&mut dacl),
            None,
            &mut original,
        )
        .ok()
        .map_err(|e| e.to_string())?;
        let mut control = 0;
        let mut revision = 0;
        GetSecurityDescriptorControl(original, &mut control, &mut revision)
            .map_err(|e| e.to_string())?;
        let mut denied = PSECURITY_DESCRIPTOR::default();
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            w!("D:P(D;;GW;;;WD)(A;;GRGX;;;WD)"),
            SDDL_REVISION_1,
            &mut denied,
            None,
        )
        .map_err(|e| e.to_string())?;
        let mut deny_acl = std::ptr::null_mut();
        let mut present = false.into();
        let mut defaulted = false.into();
        GetSecurityDescriptorDacl(denied, &mut present, &mut deny_acl, &mut defaulted)
            .map_err(|e| e.to_string())?;
        let applied = SetNamedSecurityInfoW(
            pcw(&name),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(deny_acl),
            None,
        )
        .ok()
        .map_err(|e| e.to_string());
        let result = if applied.is_ok() { check() } else { applied };
        let inherited = if control & 0x1000 != 0 {
            PROTECTED_DACL_SECURITY_INFORMATION
        } else {
            UNPROTECTED_DACL_SECURITY_INFORMATION
        };
        let restore = SetNamedSecurityInfoW(
            pcw(&name),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | inherited,
            None,
            None,
            Some(dacl),
            None,
        )
        .ok()
        .map_err(|e| e.to_string());
        let _ = LocalFree(Some(HLOCAL(original.0)));
        let _ = LocalFree(Some(HLOCAL(denied.0)));
        restore?;
        result
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn rcdata_distinguishes_loaded_image_from_raw_file_mapping() {
        let exe = std::env::current_exe().unwrap();
        let loaded = super::with_rcdata(&exe, 200, |bytes, offset| {
            assert!(offset.is_none());
            Ok((bytes.len(), super::hash_bytes(bytes)?))
        })
        .unwrap()
        .expect("Avisos RCDATA vinculados no build");
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("validation/unit")
            .join(format!("resource-{}", crate::settings::unique_stamp()));
        std::fs::create_dir_all(&root).unwrap();
        let copy = root.join("resource-test.exe");
        std::fs::copy(exe, &copy).unwrap();
        let raw = super::with_rcdata(&copy, 200, |bytes, offset| {
            assert!(offset.is_some());
            Ok((bytes.len(), super::hash_bytes(bytes)?))
        })
        .unwrap()
        .unwrap();
        assert_eq!(loaded, raw);
        assert!(
            super::with_rcdata(&copy, 201, |_, _| Ok(()))
                .unwrap()
                .is_none()
        );
        std::fs::remove_file(copy).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    fn elevation_quotes_trailing_slashes_and_literal_quotes() {
        assert_eq!(
            super::quote_argument("C:\\Program Files\\WyzarCapture\\"),
            "\"C:\\Program Files\\WyzarCapture\\\\\""
        );
        assert_eq!(super::quote_argument("a\"b"), "\"a\\\"b\"");
        unsafe {
            let values = [
                r"C:\Program Files\WyzarCapture\",
                "ação 日本語 com espaços",
                "literal\"quote",
                "",
            ];
            let line = format!(
                "Setup.exe {}",
                values
                    .iter()
                    .map(|v| super::quote_argument(v))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            let mut count = 0;
            let args = super::CommandLineToArgvW(super::pcw(&super::wide(&line)), &mut count);
            assert!(!args.is_null());
            assert_eq!(count, 5);
            for (i, value) in values.iter().enumerate() {
                assert_eq!((*args.add(i + 1)).to_string().unwrap(), *value);
            }
            let _ = super::LocalFree(Some(super::HLOCAL(args.cast())));
        }
    }
    #[test]
    fn cng_matches_sha256_vectors() {
        assert_eq!(
            super::hash_bytes(b"abc").unwrap(),
            "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD"
        );
        assert_eq!(
            super::hash_bytes(b"").unwrap(),
            "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"
        );
    }
}
