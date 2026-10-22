//! Official RCDATA payload, with the previous overlay reader retained for comparison.
//! Both formats use the same compiled allow-list and bounded extraction; no payload
//! is executed from this temporary directory.
use super::{payload::Manifest, *};
use crate::platform::windows::setup as os;
use std::{
    fs,
    io::{Read, Seek, SeekFrom, Write},
};
const MAGIC: &[u8; 16] = b"WYZARPAYLOAD0001";
const FOOTER: u64 = 64;
const MARKER: &str = "WyzarCapture.EmbeddedPayload.1";
pub const RESOURCE_ID: u16 = 201;

pub struct Package {
    pub root: PathBuf,
    pub mode: &'static str,
    files: Vec<PathBuf>,
}
impl Drop for Package {
    fn drop(&mut self) {
        if guard(&self.root).is_err()
            || fs::read_to_string(self.root.join(".wyzar-payload-cache"))
                .ok()
                .as_deref()
                != Some(MARKER)
        {
            return;
        }
        for file in self.files.iter().rev() {
            if guard(file).is_ok() {
                let _ = fs::remove_file(file);
            }
        }
        let mut dirs: Vec<_> = self
            .files
            .iter()
            .flat_map(|p| p.ancestors().skip(1))
            .filter(|p| p.starts_with(&self.root) && *p != self.root)
            .map(Path::to_path_buf)
            .collect();
        dirs.sort_by_key(|p| std::cmp::Reverse(p.components().count()));
        dirs.dedup();
        for dir in dirs {
            let _ = fs::remove_dir(dir);
        }
        let _ = fs::remove_file(self.root.join(".wyzar-payload-cache"));
        let _ = fs::remove_dir(&self.root);
    }
}
fn bytes<const N: usize>(file: &mut fs::File, pos: u64) -> AppResult<[u8; N]> {
    file.seek(SeekFrom::Start(pos)).map_err(|e| e.to_string())?;
    let mut b = [0; N];
    file.read_exact(&mut b).map_err(|e| e.to_string())?;
    Ok(b)
}
fn u32_at(f: &mut fs::File, pos: u64) -> AppResult<u64> {
    Ok(u32::from_le_bytes(bytes(f, pos)?) as u64)
}
// PE32+ certificate directory is a file offset (not an RVA). A signed EXE can
// append an aligned certificate after our footer without changing payload offsets.
fn overlay_end(file: &mut fs::File) -> AppResult<(u64, u64)> {
    let size = file.metadata().map_err(|e| e.to_string())?.len();
    if bytes::<2>(file, 0)? != *b"MZ" {
        return Err(crate::i18n::tr("Setup não é PE").into());
    }
    let pe = u32_at(file, 0x3c)?;
    if pe > 1024 * 1024
        || bytes::<4>(file, pe)? != *b"PE\0\0"
        || bytes::<2>(file, pe + 24)? != 0x20bu16.to_le_bytes()
    {
        return Err(crate::i18n::tr("Setup PE x64 inválido").into());
    }
    let cert = u32_at(file, pe + 24 + 112 + 32)?;
    let cert_size = u32_at(file, pe + 24 + 112 + 36)?;
    let count = u16::from_le_bytes(bytes(file, pe + 6)?) as u64;
    let optional = u16::from_le_bytes(bytes(file, pe + 20)?) as u64;
    if count == 0 || count > 96 {
        return Err(crate::i18n::tr("Seções PE inválidas").into());
    }
    let mut image_end = 0;
    for i in 0..count {
        let section = pe + 24 + optional + i * 40;
        let end = u32_at(file, section + 20)?
            .checked_add(u32_at(file, section + 16)?)
            .ok_or(crate::i18n::tr("Seção excede o arquivo"))?;
        image_end = image_end.max(end);
    }
    let end = if cert == 0 && cert_size == 0 {
        size
    } else {
        if cert % 8 != 0 || cert < image_end || cert.checked_add(cert_size) != Some(size) {
            return Err(crate::i18n::tr("Tabela de assinatura PE inválida").into());
        }
        cert
    };
    if image_end > end || end < FOOTER {
        return Err(crate::i18n::tr("Setup truncado").into());
    }
    Ok((end, image_end))
}
pub fn extract(exe: &Path, embedded: &str) -> AppResult<Package> {
    let manifest = Manifest::parse(embedded)?;
    if let Some(package) = os::with_rcdata(exe, RESOURCE_ID, |data, _| {
        let entry = &manifest.files[0];
        if data.len() as u64 != entry.bytes || os::hash_bytes(data)? != entry.hash {
            return Err(crate::i18n::tr(
                "SHA-256/tamanho do recurso RCDATA divergente. Nada foi instalado.",
            )
            .into());
        }
        let mut package = prepare(&manifest, "rcdata-v1")?;
        let file = package.root.join("payload/WyzarCapture.exe");
        fs::create_dir(package.root.join("payload")).map_err(|e| e.to_string())?;
        guard(&file)?;
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&file)
            .map_err(|e| e.to_string())?;
        package.files.push(file);
        for chunk in data.chunks(65536) {
            output.write_all(chunk).map_err(|e| e.to_string())?;
        }
        output.sync_all().map_err(|e| e.to_string())?;
        drop(output);
        manifest.verify_payload(&package.root.join("payload"))?;
        Ok(package)
    })? {
        return Ok(package);
    }
    extract_overlay(exe, embedded)
}
fn extract_overlay(exe: &Path, embedded: &str) -> AppResult<Package> {
    let m = Manifest::parse(embedded)?;
    guard(exe)?;
    let mut input = fs::File::open(exe).map_err(|e| e.to_string())?;
    let (end, image_end) = overlay_end(&mut input)?;
    let footer = bytes::<64>(&mut input, end - FOOTER)?;
    if &footer[..16] != MAGIC {
        return Err(crate::i18n::tr(
            "Payload incorporado ausente ou truncado. Gere o Setup oficial pelo pipeline.",
        )
        .into());
    }
    let offset = u64::from_le_bytes(footer[16..24].try_into().unwrap());
    let length = u64::from_le_bytes(footer[24..32].try_into().unwrap());
    let footer_hash: String = footer[32..].iter().map(|b| format!("{b:02X}")).collect();
    let body_size = 24u64
        .checked_add(embedded.len() as u64)
        .and_then(|n| n.checked_add(m.size()))
        .ok_or(crate::i18n::tr("Tamanho do payload inválido"))?;
    if offset < image_end
        || offset.checked_add(length) != Some(end - FOOTER)
        || length < body_size
        || length - body_size > 7
        || footer_hash != os::hash_bytes(embedded.as_bytes())?
        || bytes::<16>(&mut input, offset)? != *MAGIC
        || u64::from_le_bytes(bytes(&mut input, offset + 16)?) != embedded.len() as u64
    {
        return Err(crate::i18n::tr("Estrutura do payload incorporado inválida").into());
    }
    let mut manifest = vec![0u8; embedded.len()];
    input.read_exact(&mut manifest).map_err(|e| e.to_string())?;
    if manifest != embedded.as_bytes() {
        return Err(crate::i18n::tr("Manifest incorporado adulterado").into());
    }
    let mut package = prepare(&m, "overlay-v1")?;
    let mut buffer = [0u8; 65536];
    for entry in &m.files {
        let file = package.root.join("payload").join(relative(&entry.name)?);
        guard(&file)?;
        fs::create_dir_all(file.parent().unwrap()).map_err(|e| e.to_string())?;
        let mut out = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&file)
            .map_err(|e| e.to_string())?;
        package.files.push(file.clone());
        let mut left = entry.bytes;
        while left != 0 {
            let count = left.min(buffer.len() as u64) as usize;
            input
                .read_exact(&mut buffer[..count])
                .map_err(|e| e.to_string())?;
            out.write_all(&buffer[..count]).map_err(|e| e.to_string())?;
            left -= count as u64;
        }
        out.sync_all().map_err(|e| e.to_string())?;
        drop(out);
        m.verify_file(&package.root.join("payload"), entry)?;
    }
    let mut padding = [0u8; 7];
    let pad = (length - body_size) as usize;
    input
        .read_exact(&mut padding[..pad])
        .map_err(|e| e.to_string())?;
    if padding[..pad].iter().any(|b| *b != 0) {
        return Err(crate::i18n::tr("Padding do payload inválido").into());
    }
    m.verify_payload(&package.root.join("payload"))?;
    Ok(package)
}
fn prepare(manifest: &Manifest, mode: &'static str) -> AppResult<Package> {
    let root = std::env::temp_dir().join(format!(
        "WyzarCapture-Payload-{}-{}",
        std::process::id(),
        crate::settings::unique_stamp()
    ));
    guard(&root)?;
    let free = crate::platform::windows::files::free_space(&root)
        .ok_or(crate::i18n::tr(
            "Não foi possível consultar espaço temporário",
        ))?
        .0;
    if free < manifest.size().saturating_add(1024 * 1024) {
        return Err(crate::i18n::tr("Espaço temporário insuficiente para validar o Setup").into());
    }
    fs::create_dir(&root).map_err(|e| e.to_string())?;
    fs::write(root.join(".wyzar-payload-cache"), MARKER).map_err(|e| e.to_string())?;
    let mut package = Package {
        root,
        mode,
        files: vec![],
    };
    let file = package.root.join("payload.manifest");
    package.files.push(file.clone());
    fs::write(file, &manifest.text).map_err(|e| e.to_string())?;
    Ok(package)
}
pub fn payload_start(exe: &Path, manifest: &str) -> AppResult<u64> {
    if let Some(offset) = os::with_rcdata(exe, RESOURCE_ID, |_, offset| {
        offset.ok_or(crate::i18n::tr("Offset de arquivo requer cópia RCDATA isolada").into())
    })? {
        return Ok(offset);
    }
    guard(exe)?;
    let mut f = fs::File::open(exe).map_err(|e| e.to_string())?;
    let (end, _) = overlay_end(&mut f)?;
    let footer = bytes::<64>(&mut f, end - FOOTER)?;
    if &footer[..16] != MAGIC {
        return Err(crate::i18n::tr("Footer ausente").into());
    }
    u64::from_le_bytes(footer[16..24].try_into().unwrap())
        .checked_add(24 + manifest.len() as u64)
        .ok_or(crate::i18n::tr("Overflow do payload").into())
}

/// Abandoned caches only: an active PID or an unrecognised file prevents deletion.
pub fn clean_abandoned() {
    let temp = std::env::temp_dir();
    let Ok(items) = fs::read_dir(&temp) else {
        return;
    };
    for item in items.flatten() {
        let name = item.file_name();
        let Some(name) = name.to_str() else { continue };
        let Some(rest) = name.strip_prefix("WyzarCapture-Payload-") else {
            continue;
        };
        let Some((pid, stamp)) = rest.split_once('-') else {
            continue;
        };
        let Ok(pid) = pid.parse::<u32>() else {
            continue;
        };
        if !stamp.bytes().all(|b| b.is_ascii_digit()) || os::process_alive(pid) {
            continue;
        }
        let root = item.path();
        if guard(&root).is_err()
            || fs::read_to_string(root.join(".wyzar-payload-cache"))
                .ok()
                .as_deref()
                != Some(MARKER)
        {
            continue;
        }
        let Ok(text) = fs::read_to_string(root.join("payload.manifest")) else {
            continue;
        };
        let Ok(m) = Manifest::parse(&text) else {
            continue;
        };
        let mut files = vec![root.join("payload.manifest")];
        files.extend(m.files.iter().map(|f| root.join("payload").join(&f.name)));
        drop(Package {
            root,
            files,
            mode: "abandoned",
        });
    }
}
