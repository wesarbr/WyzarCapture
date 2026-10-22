use super::{
    payload::{FileEntry, Manifest},
    *,
};
use crate::{persistence, platform::windows::setup as os};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    sync::atomic::{AtomicBool, Ordering},
};
const RECEIPT: &str = "install-state.ini";
const JOURNAL: &str = "install-transaction.ini";
const MARKER: &str = "WyzarCapture.InstallWork.1";
/// Only fixed installed product entries can be launched by the Setup.
/// This rechecks the committed receipt immediately before process creation.
pub fn verified_owned_file(root: &Path, name: &str) -> AppResult<PathBuf> {
    if !["WyzarCapture.exe", "WyzarCaptureUninstall.exe"].contains(&name) {
        return Err(crate::i18n::tr("Executável fora da allow-list do produto").into());
    }
    let receipt = read_state(root)?.ok_or(crate::i18n::tr("Instalação não encontrada"))?;
    let entry = entries(&receipt)?
        .into_iter()
        .find(|f| f.name == name)
        .ok_or(crate::i18n::tr("Executável ausente no recibo"))?;
    let path = root.join(name);
    guard(&path)?;
    os::verify_default_stream(&path)?;
    if fs::metadata(&path).map_err(|e| e.to_string())?.len() != entry.bytes
        || os::hash_file(&path)? != entry.hash
    {
        return Err(crate::i18n::tr("O executável instalado foi modificado. Repare a instalação antes de abri-lo pelo Setup.").into());
    }
    Ok(path)
}
pub const REG_NAMES: &[&str] = &[
    "DisplayName",
    "DisplayVersion",
    "Publisher",
    "InstallLocation",
    "DisplayIcon",
    "UninstallString",
    "QuietUninstallString",
    "EstimatedSize",
    "NoModify",
    "NoRepair",
];
#[derive(Clone, Debug)]
pub struct Context {
    pub registration: String,
    pub run_key: String,
    pub run_name: String,
    pub menu: PathBuf,
    pub desktop: PathBuf,
    pub fault_after: Option<usize>,
    pub free_override: Option<u64>,
}
impl Context {
    pub fn real(scope: Scope) -> AppResult<Self> {
        scope.ensure_available()?;
        use windows::Win32::UI::Shell::*;
        Ok(Self {
            registration: r"Software\Microsoft\Windows\CurrentVersion\Uninstall\WyzarCapture"
                .into(),
            run_key: r"Software\Microsoft\Windows\CurrentVersion\Run".into(),
            run_name: "WyzarCapture".into(),
            menu: os::known_folder(if scope == Scope::User {
                &FOLDERID_Programs
            } else {
                &FOLDERID_CommonPrograms
            })?
            .join("WyzarCapture.lnk"),
            desktop: os::known_folder(if scope == Scope::User {
                &FOLDERID_Desktop
            } else {
                &FOLDERID_PublicDesktop
            })?
            .join("WyzarCapture.lnk"),
            fault_after: None,
            free_override: None,
        })
    }
    pub fn test(root: &Path) -> Self {
        let id = format!("WyzarCapture.QA.{}", crate::settings::unique_stamp());
        Self {
            registration: format!(r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{id}"),
            run_key: format!(r"Software\WyzarCapture\InstallerQA\{id}"),
            run_name: "Startup".into(),
            menu: root.join("Menu Iniciar/WyzarCapture.lnk"),
            desktop: root.join("Área de Trabalho/WyzarCapture.lnk"),
            fault_after: None,
            free_override: None,
        }
    }
    pub fn test_file(file: &Path, target: &Path) -> AppResult<Self> {
        guard(file)?;
        guard(target)?;
        let root = file.parent().ok_or(crate::i18n::tr("Contexto sem raiz"))?;
        if !root.components().any(|c| c.as_os_str() == "validation") || !target.starts_with(root) {
            return Err("Contexto QA fora de validation".into());
        }
        let v = persistence::read(file)?;
        let ctx = Self {
            registration: v
                .get("registration")
                .ok_or("Contexto QA incompleto")?
                .clone(),
            run_key: v.get("run_key").ok_or("Contexto QA incompleto")?.clone(),
            run_name: "Startup".into(),
            menu: root.join("Menu Iniciar/WyzarCapture.lnk"),
            desktop: root.join("Área de Trabalho/WyzarCapture.lnk"),
            fault_after: None,
            free_override: None,
        };
        if !ctx
            .registration
            .starts_with(r"Software\Microsoft\Windows\CurrentVersion\Uninstall\WyzarCapture.QA.")
            || !ctx
                .run_key
                .starts_with(r"Software\WyzarCapture\InstallerQA\WyzarCapture.QA.")
        {
            return Err("Chave QA inválida".into());
        }
        Ok(ctx)
    }
    fn links(&self) -> [&Path; 2] {
        [&self.menu, &self.desktop]
    }
}
pub fn read_state(root: &Path) -> AppResult<Option<BTreeMap<String, String>>> {
    let p = root.join(RECEIPT);
    guard(&p)?;
    if !p.exists() {
        return Ok(None);
    }
    if p.metadata().map_err(|e| e.to_string())?.len() > 512 * 1024 {
        return Err(crate::i18n::tr("Registro da instalação excede o limite").into());
    }
    let v = persistence::read(&p)?;
    if v.get("product").map(String::as_str) != Some("WyzarCapture")
        || v.get("schema").map(String::as_str) != Some("1")
    {
        return Err(crate::i18n::tr("Destino não é uma instalação reconhecida").into());
    }
    Ok(Some(v))
}
pub fn read_uninstall_state(root: &Path) -> AppResult<Option<BTreeMap<String, String>>> {
    if let Some(state) = read_state(root)? {
        return Ok(Some(state));
    }
    let path = root.join(JOURNAL);
    guard(&path)?;
    if !path.is_file() {
        return Ok(None);
    }
    let v = persistence::read(&path)?;
    if v.get("product").map(String::as_str) == Some("WyzarCapture")
        && v.get("kind").map(String::as_str) == Some("uninstall")
    {
        Ok(Some(v))
    } else {
        Err(crate::i18n::tr("Diário não reconhecido para remoção").into())
    }
}
fn entries(v: &BTreeMap<String, String>) -> AppResult<Vec<FileEntry>> {
    let n = v
        .get("file_count")
        .and_then(|n| n.parse::<usize>().ok())
        .filter(|n| *n <= 513)
        .ok_or(crate::i18n::tr("Lista de arquivos inválida"))?;
    let mut out = vec![];
    let mut names = std::collections::BTreeSet::new();
    for i in 0..n {
        let name = v
            .get(&format!("file.{i}.name"))
            .ok_or(crate::i18n::tr("Arquivo ausente no registro"))?
            .clone();
        relative(&name)?;
        if !names.insert(name.to_lowercase()) {
            return Err(crate::i18n::tr("Arquivo duplicado no registro").into());
        }
        out.push(FileEntry {
            name,
            bytes: v
                .get(&format!("file.{i}.bytes"))
                .and_then(|n| n.parse().ok())
                .ok_or(crate::i18n::tr("Tamanho inválido"))?,
            hash: v
                .get(&format!("file.{i}.hash"))
                .filter(|s| s.len() == 64 && s.bytes().all(|c| c.is_ascii_hexdigit()))
                .ok_or(crate::i18n::tr("Hash inválido"))?
                .clone(),
        });
    }
    Ok(out)
}
fn encode_entries(v: &mut BTreeMap<String, String>, files: &[FileEntry]) {
    v.insert("file_count".into(), files.len().to_string());
    for (i, f) in files.iter().enumerate() {
        v.insert(format!("file.{i}.name"), f.name.clone());
        v.insert(format!("file.{i}.bytes"), f.bytes.to_string());
        v.insert(format!("file.{i}.hash"), f.hash.clone());
    }
}
/// The product registration, which may point to a folder other than the one chosen now.
#[derive(Clone, Debug, PartialEq)]
pub struct Registered {
    pub path: PathBuf,
    /// Present only while that folder still holds a recognized receipt.
    pub version: Option<String>,
}
pub fn registered(scope: Scope, ctx: &Context) -> AppResult<Option<Registered>> {
    let Some(location) = os::reg_get(scope, &ctx.registration, "InstallLocation")?
        .and_then(|v| os::text(&v))
        .filter(|s| !s.trim().is_empty())
    else {
        return Ok(None);
    };
    let path = PathBuf::from(location);
    // A deleted, foreign or unsafe folder cannot be updated or removed by this product.
    // Such a stale entry is replaced by the next installation instead of blocking it.
    let version = read_state(&path)
        .ok()
        .flatten()
        .and_then(|v| v.get("version").cloned());
    Ok(Some(Registered { path, version }))
}
/// A working installation in another folder, which this destination would move.
pub fn installed_elsewhere(
    scope: Scope,
    ctx: &Context,
    root: &Path,
) -> AppResult<Option<Registered>> {
    Ok(registered(scope, ctx)?.filter(|r| r.version.is_some() && !os::same_folder(&r.path, root)))
}
pub fn operation(root: &Path, new: &str) -> AppResult<&'static str> {
    Ok(
        match read_state(root)?.and_then(|v| v.get("version").cloned()) {
            None => crate::i18n::tr("Instalação nova"),
            Some(v) => match version(&v)?.cmp(&version(new)?) {
                std::cmp::Ordering::Less => crate::i18n::tr("Atualização"),
                std::cmp::Ordering::Equal => crate::i18n::tr("Reparo"),
                std::cmp::Ordering::Greater => {
                    crate::i18n::tr("Versão anterior — confirmação necessária")
                }
            },
        },
    )
}
pub fn required_bytes(m: &Manifest, setup: &Path) -> AppResult<u64> {
    m.size()
        .checked_mul(3)
        .and_then(|s| s.checked_add(setup.metadata().ok()?.len() * 2))
        .and_then(|s| s.checked_add(16 * 1024 * 1024))
        .ok_or(crate::i18n::tr("Payload excede o tamanho permitido").into())
}
struct Work<'a, F: FnMut(Progress)> {
    root: &'a Path,
    ctx: &'a Context,
    cancel: &'a AtomicBool,
    progress: Progress,
    emit: F,
}
impl<F: FnMut(Progress)> Work<'_, F> {
    fn check(&self) -> AppResult<()> {
        if self.cancel.load(Ordering::Relaxed) {
            return Err(crate::i18n::tr("Instalação cancelada").into());
        }
        if self
            .ctx
            .fault_after
            .is_some_and(|n| self.progress.completed >= n)
        {
            return Err(crate::i18n::tr("Falha controlada do teste de transação").into());
        }
        Ok(())
    }
    fn done(&mut self, op: &str, file: &str) -> AppResult<()> {
        self.progress.completed += 1;
        self.progress.operation = op.into();
        self.progress.file = file.into();
        (self.emit)(self.progress.clone());
        self.check()
    }
}
fn copy_file(source: &Path, dest: &Path, cancel: &AtomicBool) -> AppResult<()> {
    guard(source)?;
    guard(dest)?;
    fs::create_dir_all(dest.parent().ok_or(crate::i18n::tr("Destino sem pasta"))?)
        .map_err(|e| e.to_string())?;
    guard(dest)?;
    let mut input = fs::File::open(source).map_err(|e| e.to_string())?;
    let mut out = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dest)
        .map_err(|e| e.to_string())?;
    let mut block = [0; 65536];
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err(crate::i18n::tr("Instalação cancelada").into());
        }
        let n = input.read(&mut block).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        out.write_all(&block[..n]).map_err(|e| e.to_string())?;
    }
    out.sync_all().map_err(|e| e.to_string())
}
fn work_dir(root: &Path, name: &str) -> AppResult<PathBuf> {
    let dir = root.join(name);
    guard(&dir)?;
    if dir.exists() {
        return Err(crate::i18n::format_text(
            crate::i18n::tr("{name} já existe. Conclua a recuperação antes de continuar."),
            &[("name", format!("{}", name))],
        ));
    }
    fs::create_dir(&dir).map_err(|e| e.to_string())?;
    fs::write(dir.join(".wyzar-owned"), MARKER).map_err(|e| e.to_string())?;
    Ok(dir)
}
fn clean_work(root: &Path, name: &str) -> AppResult<()> {
    let dir = root.join(name);
    guard(&dir)?;
    if !dir.exists() {
        return Ok(());
    }
    if fs::read_to_string(dir.join(".wyzar-owned")).map_err(|e| e.to_string())? != MARKER {
        return Err(crate::i18n::tr("Pasta temporária sem identidade válida").into());
    }
    fn inspect(dir: &Path) -> AppResult<()> {
        guard(dir)?;
        for f in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let p = f.map_err(|e| e.to_string())?.path();
            guard(&p)?;
            if p.is_dir() {
                inspect(&p)?;
            }
        }
        Ok(())
    }
    inspect(&dir)?;
    fs::remove_dir_all(&dir).map_err(|e| e.to_string())
}
fn snapshot(v: &mut BTreeMap<String, String>, scope: Scope, ctx: &Context) -> AppResult<()> {
    for (i, name) in REG_NAMES
        .iter()
        .chain(std::iter::once(&ctx.run_name.as_str()))
        .enumerate()
    {
        let path = if i < REG_NAMES.len() {
            &ctx.registration
        } else {
            &ctx.run_key
        };
        if let Some(value) = os::reg_get(scope, path, name)? {
            v.insert(format!("reg.{i}.kind"), value.kind.0.to_string());
            v.insert(
                format!("reg.{i}.bytes"),
                value.bytes.iter().map(|b| format!("{b:02X}")).collect(),
            );
        }
    }
    Ok(())
}
fn restore_registry(v: &BTreeMap<String, String>, scope: Scope, ctx: &Context) -> AppResult<()> {
    use windows::Win32::System::Registry::REG_VALUE_TYPE;
    for (i, name) in REG_NAMES
        .iter()
        .chain(std::iter::once(&ctx.run_name.as_str()))
        .enumerate()
    {
        let path = if i < REG_NAMES.len() {
            &ctx.registration
        } else {
            &ctx.run_key
        };
        let value = if let Some(bytes) = v.get(&format!("reg.{i}.bytes")) {
            if bytes.len() > 131072 || bytes.len() % 2 != 0 {
                return Err(crate::i18n::tr("Snapshot de registro inválido").into());
            }
            let bytes = (0..bytes.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&bytes[i..i + 2], 16))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| crate::i18n::tr("Snapshot hexadecimal inválido"))?;
            Some(os::Value {
                kind: REG_VALUE_TYPE(
                    v.get(&format!("reg.{i}.kind"))
                        .and_then(|s| s.parse().ok())
                        .ok_or(crate::i18n::tr("Tipo do registro inválido"))?,
                ),
                bytes,
            })
        } else {
            None
        };
        os::reg_set(scope, path, name, value.as_ref())?;
    }
    os::reg_remove_empty(scope, &ctx.registration);
    Ok(())
}
fn backup_links(root: &Path, ctx: &Context, v: &mut BTreeMap<String, String>) -> AppResult<()> {
    for (i, p) in ctx.links().iter().enumerate() {
        guard(p)?;
        if p.exists() {
            let dest = root.join(format!("install-rollback/shortcut-{i}.lnk"));
            copy_file(p, &dest, &AtomicBool::new(false))?;
            v.insert(format!("link.{i}.old"), "true".into());
        }
    }
    Ok(())
}
fn restore_links(root: &Path, ctx: &Context, v: &BTreeMap<String, String>) -> AppResult<()> {
    for (i, p) in ctx.links().iter().enumerate() {
        guard(p)?;
        if v.get(&format!("link.{i}.old")).is_some_and(|s| s == "true") {
            let backup = root.join(format!("install-rollback/shortcut-{i}.lnk"));
            if backup.exists() {
                if p.exists() {
                    fs::remove_file(p).map_err(|e| e.to_string())?;
                }
                copy_file(&backup, p, &AtomicBool::new(false))?;
            }
        } else if v
            .get(&format!("link.{i}.touched"))
            .is_some_and(|s| s == "true")
            && p.exists()
        {
            fs::remove_file(p).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
pub fn recover(root: &Path, ctx: &Context) -> AppResult<()> {
    let journal = root.join(JOURNAL);
    guard(&journal)?;
    if !journal.exists() {
        return Ok(());
    }
    if journal.metadata().map_err(|e| e.to_string())?.len() > 2 * 1024 * 1024 {
        return Err(crate::i18n::tr("Diário inválido").into());
    }
    let v = persistence::read(&journal)?;
    if v.get("product").map(String::as_str) != Some("WyzarCapture")
        || v.get("registration") != Some(&ctx.registration)
    {
        return Err(crate::i18n::tr("Diário não pertence a esta instalação").into());
    }
    let scope = if v.get("scope").map(String::as_str) == Some("machine") {
        Scope::Machine
    } else {
        Scope::User
    };
    scope.ensure_available()?;
    if scope == Scope::Machine && !os::is_admin() {
        return Err(crate::i18n::tr("A recuperação desta instalação precisa de elevação").into());
    }
    if v.get("phase").map(String::as_str) == Some("committed") {
        clean_work(root, "install-staging")?;
        clean_work(root, "install-rollback")?;
        fs::remove_file(journal).map_err(|e| e.to_string())?;
        return Ok(());
    }
    let count = v
        .get("count")
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|n| *n <= 514)
        .ok_or(crate::i18n::tr("Diário sem lista válida"))?;
    for i in (0..count).rev() {
        let name = v
            .get(&format!("item.{i}"))
            .ok_or(crate::i18n::tr("Entrada de transação ausente"))?;
        let rel = relative(name)?;
        let dest = root.join(&rel);
        let backup = root.join("install-rollback").join(&rel);
        guard(&dest)?;
        guard(&backup)?;
        if backup.exists() {
            if dest.exists() {
                if v.get("kind").map(String::as_str) == Some("uninstall") {
                    return Err(crate::i18n::format_text(
                        crate::i18n::tr(
                            "Rollback preservou um arquivo criado durante a remoção: {name}. Verifique a pasta e repare a instalação.",
                        ),
                        &[("name", format!("{}", name))],
                    ));
                }
                fs::remove_file(&dest).map_err(|e| e.to_string())?;
            }
            fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
            fs::rename(backup, dest).map_err(|e| e.to_string())?;
        } else if v.get(&format!("old.{i}")).map(String::as_str) == Some("false") && dest.is_file()
        {
            let expected = v
                .get(&format!("new.{i}"))
                .ok_or(crate::i18n::tr("Hash ausente no diário"))?;
            if os::hash_file(&dest)? == *expected {
                fs::remove_file(dest).map_err(|e| e.to_string())?;
            }
        }
    }
    restore_registry(&v, scope, ctx)?;
    restore_links(root, ctx, &v)?;
    fs::remove_file(journal).map_err(|e| e.to_string())?;
    clean_work(root, "install-staging")?;
    clean_work(root, "install-rollback")?;
    Ok(())
}
pub fn install(
    manifest: &Manifest,
    package: &Path,
    setup: &Path,
    options: &Options,
    ctx: &Context,
    cancel: &AtomicBool,
    mut emit: impl FnMut(Progress),
) -> AppResult<()> {
    let Some(old) = install_files(manifest, package, setup, options, ctx, cancel, &mut emit)?
    else {
        return Ok(());
    };
    // The new folder is committed and registered. Only now remove the previous program
    // files, with the normal reversible removal; personal data there is preserved.
    let new = options.destination.to_string_lossy().into_owned();
    let previous = old.to_string_lossy().into_owned();
    let operation = match uninstall(&old, ctx, options.close_running, |_| {}) {
        Ok(()) => crate::i18n::format_text(
            crate::i18n::tr(
                "Instalação movida para {new}. Os arquivos do programa em {old} foram removidos; dados pessoais preservados.",
            ),
            &[("new", new), ("old", previous)],
        ),
        Err(e) => crate::i18n::format_text(
            crate::i18n::tr(
                "Instalação concluída em {new}. A pasta anterior {old} não pôde ser limpa: {e}",
            ),
            &[("new", new), ("old", previous), ("e", e)],
        ),
    };
    emit(Progress {
        completed: 1,
        total: 1,
        operation,
        file: String::new(),
    });
    Ok(())
}
/// Returns the previous folder when this installation moved a registered one.
fn install_files(
    manifest: &Manifest,
    package: &Path,
    setup: &Path,
    options: &Options,
    ctx: &Context,
    cancel: &AtomicBool,
    emit: impl FnMut(Progress),
) -> AppResult<Option<PathBuf>> {
    manifest.verify_payload(&package.join("payload"))?;
    os::validate_scope(options)?;
    let _metadata_lock = os::DestinationLock::metadata(&ctx.registration)?;
    let _lock = os::DestinationLock::acquire(&options.destination)?;
    guard(package)?;
    guard(setup)?;
    let root = &options.destination;
    if root.starts_with(package) || package.starts_with(root) {
        return Err(crate::i18n::tr("O destino não pode conter o pacote do instalador").into());
    }
    if root.join(JOURNAL).exists() {
        recover(root, ctx)?;
    }
    let prior = read_state(root)?;
    if prior.is_none()
        && root.exists()
        && fs::read_dir(root)
            .map_err(|e| e.to_string())?
            .next()
            .is_some()
    {
        return Err(crate::i18n::tr(
            "A pasta contém arquivos que não pertencem ao instalador. Escolha uma pasta vazia.",
        )
        .into());
    }
    if let Some(v) = &prior {
        if v.get("scope").map(String::as_str) != Some(options.scope.key()) {
            return Err(
                crate::i18n::tr("Mantenha o escopo original ao atualizar esta instalação").into(),
            );
        }
        let old = version(
            v.get("version")
                .ok_or(crate::i18n::tr("Versão instalada ausente"))?,
        )?;
        if old > version(&manifest.version)? && !options.allow_downgrade {
            return Err(crate::i18n::tr(
                "Uma versão superior está instalada. Confirme explicitamente o downgrade.",
            )
            .into());
        }
    }
    let moved = installed_elsewhere(options.scope, ctx, root)?;
    let mut moved_state = None;
    if let Some(existing) = &moved {
        if !options.move_existing {
            return Err(crate::i18n::format_text(
                crate::i18n::tr(
                    "Já existe uma instalação em {existing}. Atualize no mesmo destino ou remova-a primeiro.",
                ),
                &[("existing", existing.path.to_string_lossy().into_owned())],
            ));
        }
        let state =
            read_state(&existing.path)?.ok_or(crate::i18n::tr("Instalação não encontrada"))?;
        if state.get("scope").map(String::as_str) != Some(options.scope.key()) {
            return Err(
                crate::i18n::tr("Mantenha o escopo original ao atualizar esta instalação").into(),
            );
        }
        if version(existing.version.as_deref().unwrap_or("0.0.0"))? > version(&manifest.version)?
            && !options.allow_downgrade
        {
            return Err(crate::i18n::tr(
                "Uma versão superior está instalada. Confirme explicitamente o downgrade.",
            )
            .into());
        }
        moved_state = Some(state);
    }
    let available = ctx
        .free_override
        .or_else(|| crate::platform::windows::files::free_space(root).map(|v| v.0))
        .ok_or(crate::i18n::tr(
            "Não foi possível consultar o espaço disponível",
        ))?;
    if available < required_bytes(manifest, setup)? {
        return Err(
            crate::i18n::tr("Espaço insuficiente para preparar instalação e rollback").into(),
        );
    }
    let mut work = Work {
        root,
        ctx,
        cancel,
        progress: Progress {
            total: manifest.files.len() * 4 + 8,
            ..Default::default()
        },
        emit,
    };
    work.check()?;
    if let Some(media) = &options.media_root {
        // The explicit media intent is committed with the normal receipt.
        // Existing app preferences are never written by this transaction.
        if options.scope != Scope::User {
            return Err(crate::i18n::tr("A configuração de mídias é por usuário.").into());
        }
        if media.starts_with(root) || root.starts_with(media) {
            return Err(crate::i18n::tr(
                "Escolha uma pasta de mídia separada da pasta do aplicativo.",
            )
            .into());
        }
        crate::settings::bootstrap::prepare_media(media)?;
    }
    for entry in &manifest.files {
        manifest.verify_file(&package.join("payload"), entry)?;
        work.done(crate::i18n::tr("Verificando arquivos"), &entry.name)?;
    }
    for folder in std::iter::once(root.as_path()).chain(moved.as_ref().map(|r| r.path.as_path())) {
        if !os::running(folder).is_empty() {
            if !options.close_running {
                return Err(
                    crate::i18n::tr("O WyzarCapture precisa ser fechado para continuar.").into(),
                );
            }
            os::close_app(folder)?;
        }
    }
    fs::create_dir_all(root).map_err(|e| e.to_string())?;
    guard(root)?;
    let stage = work_dir(root, "install-staging")?;
    let _backup = match work_dir(root, "install-rollback") {
        Ok(p) => p,
        Err(e) => {
            let _ = clean_work(root, "install-staging");
            return Err(e);
        }
    };
    let mut journal = BTreeMap::from([
        ("product".into(), "WyzarCapture".into()),
        ("registration".into(), ctx.registration.clone()),
        ("scope".into(), options.scope.key().into()),
    ]);
    let result = (|| -> AppResult<()> {
        work.done(crate::i18n::tr("Preparando instalação"), "")?;
        for entry in &manifest.files {
            copy_file(
                &package.join("payload").join(relative(&entry.name)?),
                &stage.join(relative(&entry.name)?),
                cancel,
            )?;
            work.done(crate::i18n::tr("Copiando aplicativo"), &entry.name)?;
            manifest.verify_file(&stage, entry)?;
            work.done(crate::i18n::tr("Validando cópia"), &entry.name)?;
        }
        let uninstall = "WyzarCaptureUninstall.exe";
        let uninstaller_hash = os::hash_file(setup)?;
        copy_file(setup, &stage.join(uninstall), cancel)?;
        if os::hash_file(&stage.join(uninstall))? != uninstaller_hash {
            return Err(crate::i18n::tr("Falha na verificação do desinstalador").into());
        }
        work.done(crate::i18n::tr("Preparando desinstalador"), uninstall)?;
        let mut owned = manifest.files.clone();
        owned.push(FileEntry {
            name: uninstall.into(),
            bytes: setup.metadata().map_err(|e| e.to_string())?.len(),
            hash: uninstaller_hash,
        });
        let mut receipt = BTreeMap::from([
            ("schema".into(), "1".into()),
            ("product".into(), "WyzarCapture".into()),
            ("version".into(), manifest.version.clone()),
            ("architecture".into(), "x64".into()),
            ("scope".into(), options.scope.key().into()),
            ("build_date".into(), manifest.date.clone()),
            ("destination".into(), root.to_string_lossy().into()),
            ("desktop".into(), options.desktop.to_string()),
            ("start_menu".into(), options.start_menu.to_string()),
            ("startup".into(), options.startup.to_string()),
        ]);
        // A moved installation keeps the one-time requests it already carried.
        let carried = prior.as_ref().or(moved_state.as_ref());
        if let Some(media) = &options.media_root {
            receipt.insert("media_root".into(), media.to_string_lossy().into());
            receipt.insert(
                "storage_request_id".into(),
                crate::settings::unique_stamp().to_string(),
            );
        } else if let Some(prior) = carried {
            for key in ["media_root", "storage_request_id"] {
                if let Some(value) = prior.get(key) {
                    receipt.insert(key.into(), value.clone());
                }
            }
        }
        encode_entries(&mut receipt, &owned);
        if let Some(language) = options.language {
            receipt.insert("language".into(), language.code().into());
            receipt.insert(
                "language_request_id".into(),
                crate::settings::unique_stamp().to_string(),
            );
        } else if let Some(prior) = carried {
            for key in ["language", "language_request_id"] {
                if let Some(value) = prior.get(key) {
                    receipt.insert(key.into(), value.clone());
                }
            }
        }
        let old_files = if let Some(v) = &prior {
            entries(v)?
        } else {
            vec![]
        };
        let mut preserved = vec![];
        let mut plan: Vec<String> = owned.iter().map(|f| f.name.clone()).collect();
        for f in &old_files {
            if !plan.contains(&f.name) {
                let path = root.join(relative(&f.name)?);
                guard(&path)?;
                if path.exists()
                    && (path.metadata().map_err(|e| e.to_string())?.len() != f.bytes
                        || os::hash_file(&path)? != f.hash)
                {
                    preserved.push(f.name.clone());
                    continue;
                }
                plan.push(f.name.clone());
            }
        }
        if !preserved.is_empty() {
            receipt.insert("preserved_previous_files".into(), preserved.join(" | "));
        }
        persistence::write(&stage.join(RECEIPT), &receipt)?;
        plan.push(RECEIPT.into());
        journal.insert("count".into(), plan.len().to_string());
        snapshot(&mut journal, options.scope, ctx)?;
        backup_links(root, ctx, &mut journal)?;
        for (i, name) in plan.iter().enumerate() {
            let dest = root.join(relative(name)?);
            guard(&dest)?;
            if dest.exists() && name != RECEIPT && !old_files.iter().any(|f| &f.name == name) {
                return Err(crate::i18n::format_text(
                    crate::i18n::tr("Arquivo do usuário coincide com o payload: {name}"),
                    &[("name", format!("{}", name))],
                ));
            }
            journal.insert(format!("item.{i}"), name.clone());
            journal.insert(format!("old.{i}"), dest.exists().to_string());
            let new = stage.join(relative(name)?);
            journal.insert(
                format!("new.{i}"),
                if new.is_file() {
                    os::hash_file(&new)?
                } else {
                    String::new()
                },
            );
        }
        for i in 0..2 {
            journal.insert(format!("link.{i}.touched"), "true".into());
        }
        persistence::write(&root.join(JOURNAL), &journal)?;
        // No application file is touched before every payload hash and the journal are durable.
        if !os::running(root).is_empty() {
            return Err(crate::i18n::tr(
                "O aplicativo foi aberto novamente. Feche-o e tente outra vez.",
            )
            .into());
        }
        for name in &plan {
            work.check()?;
            let rel = relative(name)?;
            let dest = root.join(&rel);
            let backup = root.join("install-rollback").join(&rel);
            let from = stage.join(&rel);
            guard(&dest)?;
            if dest.is_file() {
                fs::create_dir_all(backup.parent().unwrap()).map_err(|e| e.to_string())?;
                fs::rename(&dest, &backup).map_err(|e| e.to_string())?;
            }
            if from.is_file() {
                fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
                fs::rename(from, dest).map_err(|e| e.to_string())?;
            }
            if manifest.files.iter().any(|f| &f.name == name) {
                work.done(crate::i18n::tr("Instalando arquivos verificados"), name)?;
            }
        }
        let app = root.join("WyzarCapture.exe");
        for (i, (link, on)) in [
            (ctx.menu.as_path(), options.start_menu),
            (ctx.desktop.as_path(), options.desktop),
        ]
        .into_iter()
        .enumerate()
        {
            guard(link)?;
            if on {
                fs::create_dir_all(link.parent().unwrap()).map_err(|e| e.to_string())?;
                os::shortcut(link, &app)?;
            } else if prior.as_ref().is_some_and(|v| {
                v.get(if i == 0 { "start_menu" } else { "desktop" })
                    .map(String::as_str)
                    == Some("true")
            }) && link.exists()
            {
                fs::remove_file(link).map_err(|e| e.to_string())?;
            }
        }
        work.done(crate::i18n::tr("Criando atalhos solicitados"), "")?;
        let quoted = format!(
            "{} --uninstall --target {}",
            os::quote_argument(&root.join(uninstall).to_string_lossy()),
            os::quote_argument(&root.to_string_lossy())
        );
        for (name, value) in [
            ("DisplayName", os::reg_text("WyzarCapture")),
            ("DisplayVersion", os::reg_text(&manifest.version)),
            ("Publisher", os::reg_text("WyzarCapture")),
            ("InstallLocation", os::reg_text(&root.to_string_lossy())),
            (
                "DisplayIcon",
                os::reg_text(&format!("\"{}\",0", app.display())),
            ),
            ("UninstallString", os::reg_text(&quoted)),
            (
                "QuietUninstallString",
                os::reg_text(&(quoted.clone() + " --quiet --yes")),
            ),
            (
                "EstimatedSize",
                os::dword(
                    ((manifest.size() + setup.metadata().map_err(|e| e.to_string())?.len()) / 1024)
                        .min(u32::MAX as u64) as u32,
                ),
            ),
            ("NoModify", os::dword(1)),
            ("NoRepair", os::dword(1)),
        ] {
            os::reg_set(options.scope, &ctx.registration, name, Some(&value))?;
        }
        work.done(crate::i18n::tr("Registrando desinstalador"), "")?;
        os::reg_set(
            options.scope,
            &ctx.run_key,
            &ctx.run_name,
            if options.startup {
                Some(os::reg_text(&format!("\"{}\"", app.display())))
            } else {
                None
            }
            .as_ref(),
        )?;
        work.done(
            crate::i18n::tr("Salvando informações da versão"),
            &manifest.version,
        )?;
        for entry in &manifest.files {
            manifest.verify_file(root, entry)?;
        }
        work.done(crate::i18n::tr("Validando instalação final"), "")?;
        // This durable state is the commit point. Recovery must never undo a committed install.
        journal.insert("phase".into(), "committed".into());
        persistence::write(&root.join(JOURNAL), &journal)?;
        let mut cleanup_pending = false;
        for name in ["install-staging", "install-rollback"] {
            cleanup_pending |= clean_work(root, name).is_err();
            work.progress.completed += 1;
            work.progress.operation = crate::i18n::tr("Finalizando instalação").into();
            work.progress.file = name.into();
            (work.emit)(work.progress.clone());
        }
        if !cleanup_pending {
            cleanup_pending = fs::remove_file(root.join(JOURNAL)).is_err();
        }
        for f in &old_files {
            if !owned.iter().any(|current| current.name == f.name) {
                let old = root.join(relative(&f.name)?);
                let mut parent = old.parent();
                while let Some(dir) = parent.filter(|dir| *dir != root) {
                    if guard(dir).is_err() || fs::remove_dir(dir).is_err() {
                        break;
                    }
                    parent = dir.parent();
                }
            }
        }
        work.progress.operation = if cleanup_pending {
            crate::i18n::tr("Instalação concluída; limpeza será retomada na próxima execução")
        } else {
            crate::i18n::tr("Instalação concluída")
        }
        .into();
        if !preserved.is_empty() {
            work.progress.operation.push_str(crate::i18n::tr(
                ". Alguns arquivos antigos modificados foram preservados.",
            ));
        }
        work.progress.file.clear();
        (work.emit)(work.progress.clone());
        Ok(())
    })();
    if let Err(error) = result {
        let rollback = if root.join(JOURNAL).exists() {
            recover(root, ctx)
        } else {
            clean_work(root, "install-staging").and_then(|_| clean_work(root, "install-rollback"))
        };
        return match rollback {
            Ok(()) => Err(crate::i18n::format_text(
                crate::i18n::tr("{error}. A instalação anterior foi preservada."),
                &[("error", format!("{}", error))],
            )),
            Err(e) => Err(crate::i18n::format_text(
                crate::i18n::tr("{error}. Recuperação pendente: {e}"),
                &[("error", format!("{}", error)), ("e", format!("{}", e))],
            )),
        };
    }
    Ok(moved.map(|r| r.path))
}
pub fn uninstall(
    root: &Path,
    ctx: &Context,
    close_running: bool,
    mut emit: impl FnMut(Progress),
) -> AppResult<()> {
    guard(root)?;
    let _metadata_lock = os::DestinationLock::metadata(&ctx.registration)?;
    let _lock = os::DestinationLock::acquire(root)?;
    let already_committed = if root.join(JOURNAL).is_file() {
        let v = persistence::read(&root.join(JOURNAL))?;
        v.get("kind").map(String::as_str) == Some("uninstall")
            && v.get("phase").map(String::as_str) == Some("committed")
    } else {
        false
    };
    if root.join(JOURNAL).exists() {
        recover(root, ctx)?;
    }
    if already_committed {
        return Ok(());
    }
    let state = read_state(root)?.ok_or(crate::i18n::tr("Instalação não encontrada"))?;
    let files = entries(&state)?;
    let scope = if state.get("scope").map(String::as_str) == Some("machine") {
        Scope::Machine
    } else {
        Scope::User
    };
    scope.ensure_available()?;
    if scope == Scope::Machine && !os::is_admin() {
        return Err(crate::i18n::tr("A remoção para todos os usuários precisa de elevação").into());
    }
    if !os::running(root).is_empty() {
        if !close_running {
            return Err(
                crate::i18n::tr("O WyzarCapture precisa ser fechado para continuar.").into(),
            );
        }
        os::close_app(root)?;
    }
    // Quarantine by same-volume rename, then commit registry/links/receipt together.
    // A modified owned file and every foreign file stay in place.
    let mut removable = vec![];
    let mut preserved = vec![];
    for entry in files {
        let path = root.join(relative(&entry.name)?);
        guard(&path)?;
        if path.exists() {
            if path.is_file()
                && path.metadata().map_err(|e| e.to_string())?.len() == entry.bytes
                && os::hash_file(&path)? == entry.hash
            {
                removable.push(entry);
            } else {
                preserved.push(entry.name);
            }
        }
    }
    let stage = work_dir(root, "install-staging")?;
    if let Err(e) = work_dir(root, "install-rollback") {
        clean_work(root, "install-staging")?;
        return Err(e);
    }
    let mut journal = BTreeMap::from([
        ("product".into(), "WyzarCapture".into()),
        ("kind".into(), "uninstall".into()),
        ("registration".into(), ctx.registration.clone()),
        ("scope".into(), scope.key().into()),
        (
            "version".into(),
            state.get("version").cloned().unwrap_or_default(),
        ),
        ("count".into(), (removable.len() + 1).to_string()),
    ]);
    let mut plan = removable.clone();
    plan.push(FileEntry {
        name: RECEIPT.into(),
        bytes: 0,
        hash: os::hash_file(&root.join(RECEIPT))?,
    });
    for (i, entry) in plan.iter().enumerate() {
        journal.insert(format!("item.{i}"), entry.name.clone());
        journal.insert(format!("old.{i}"), "true".into());
        journal.insert(format!("new.{i}"), String::new());
    }
    let total = plan.len() + 4;
    let mut done = 0usize;
    let mut step = |operation: &str, file: &str| -> AppResult<()> {
        done += 1;
        emit(Progress {
            completed: done,
            total,
            operation: operation.into(),
            file: file.into(),
        });
        if ctx.fault_after.is_some_and(|n| done >= n) {
            Err(crate::i18n::tr("Falha controlada na remoção transacional").into())
        } else {
            Ok(())
        }
    };
    let result = (|| -> AppResult<()> {
        snapshot(&mut journal, scope, ctx)?;
        backup_links(root, ctx, &mut journal)?;
        persistence::write(&root.join(JOURNAL), &journal)?;
        for entry in &plan {
            let from = root.join(relative(&entry.name)?);
            guard(&from)?;
            if os::hash_file(&from)? != entry.hash {
                return Err(
                    crate::i18n::tr("Arquivo mudou durante a remoção; tente novamente").into(),
                );
            }
            let backup = root.join("install-rollback").join(relative(&entry.name)?);
            fs::create_dir_all(backup.parent().unwrap()).map_err(|e| e.to_string())?;
            fs::rename(from, backup).map_err(|e| e.to_string())?;
            step(
                crate::i18n::tr("Preparando remoção reversível"),
                &entry.name,
            )?;
        }
        for (link, key) in [
            (ctx.menu.as_path(), "start_menu"),
            (ctx.desktop.as_path(), "desktop"),
        ] {
            if state.get(key).map(String::as_str) == Some("true")
                && link.exists()
                && os::shortcut_target(link)
                    .ok()
                    .is_some_and(|p| p == root.join("WyzarCapture.exe"))
            {
                guard(link)?;
                fs::remove_file(link).map_err(|e| e.to_string())?;
            }
        }
        step(crate::i18n::tr("Atualizando atalhos"), "")?;
        if os::reg_get(scope, &ctx.registration, "InstallLocation")?
            .and_then(|v| os::text(&v))
            .is_some_and(|p| Path::new(&p) == root)
        {
            for name in REG_NAMES {
                os::reg_set(scope, &ctx.registration, name, None)?;
            }
            os::reg_remove_empty(scope, &ctx.registration);
        }
        if os::reg_get(scope, &ctx.run_key, &ctx.run_name)?
            .and_then(|v| os::text(&v))
            .is_some_and(|v| v == format!("\"{}\"", root.join("WyzarCapture.exe").display()))
        {
            os::reg_set(scope, &ctx.run_key, &ctx.run_name, None)?;
            os::reg_remove_empty(scope, &ctx.run_key);
        }
        step(crate::i18n::tr("Atualizando registro"), "")?;
        if !preserved.is_empty() {
            let text = {
                let __wyzar_i18n_0 = &(preserved.join("\n"));
                crate::i18n::format_text(
                    crate::i18n::tr(
                        "Alguns arquivos da instalação foram modificados e foram preservados.\n\n{}\n",
                    ),
                    &[("0", format!("{}", __wyzar_i18n_0))],
                )
            };
            // Details belong to this transaction, never replace a user's existing report.
            let details = root.join("wyzar-uninstall-preserved.txt");
            if !details.exists() {
                fs::write(&details, text).map_err(|e| e.to_string())?;
            }
        }
        step(crate::i18n::tr("Confirmando remoção"), "")?;
        journal.insert("phase".into(), "committed".into());
        persistence::write(&root.join(JOURNAL), &journal)?;
        Ok(())
    })();
    if let Err(error) = result {
        let rollback = if root.join(JOURNAL).exists() {
            recover(root, ctx)
        } else {
            clean_work(root, "install-staging").and_then(|_| clean_work(root, "install-rollback"))
        };
        return Err(match rollback {
            Ok(()) => crate::i18n::format_text(
                crate::i18n::tr("{error}. A remoção foi revertida."),
                &[("error", format!("{}", error))],
            ),
            Err(e) => crate::i18n::format_text(
                crate::i18n::tr("{error}. Rollback pendente: {e}"),
                &[("error", format!("{}", error)), ("e", format!("{}", e))],
            ),
        });
    }
    drop(stage);
    let pending = recover(root, ctx).is_err();
    let mut directories: Vec<_> = plan
        .iter()
        .flat_map(|f| {
            root.join(&f.name)
                .ancestors()
                .skip(1)
                .map(Path::to_path_buf)
                .collect::<Vec<_>>()
        })
        .filter(|p| p.starts_with(root) && p != root)
        .collect();
    directories.sort_by_key(|p| std::cmp::Reverse(p.components().count()));
    directories.dedup();
    for p in directories {
        let _ = fs::remove_dir(p);
    }
    let _ = fs::remove_dir(root);
    emit(Progress {
        completed: total,
        total,
        operation: if pending {
            crate::i18n::tr("Remoção concluída; limpeza pendente na próxima execução.")
        } else if preserved.is_empty() {
            crate::i18n::tr("Remoção concluída. Arquivos pessoais preservados.")
        } else {
            crate::i18n::tr("Alguns arquivos da instalação foram modificados e foram preservados.")
        }
        .into(),
        file: String::new(),
    });
    Ok(())
}
