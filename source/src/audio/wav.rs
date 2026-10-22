//! RIFF/RF64 PCM. Reserve JUNK for ds64 so crossing 4 GiB never moves audio.
use crate::core::AppResult;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};
pub const HEADER: u64 = 80;
#[derive(Clone, Copy, Debug)]
pub struct Info {
    pub offset: u64,
    pub bytes: u64,
    #[allow(dead_code)] // Diagnostic format information; short files remain RIFF.
    pub rf64: bool,
}
fn header(bytes: u64) -> AppResult<Vec<u8>> {
    let riff = bytes
        .checked_add(HEADER - 8)
        .ok_or(crate::i18n::tr("Áudio excede tamanho de 64 bits"))?;
    let large = riff > u32::MAX as u64;
    let mut h = if large {
        b"RF64".to_vec()
    } else {
        b"RIFF".to_vec()
    };
    h.extend(if large { u32::MAX } else { riff as u32 }.to_le_bytes());
    h.extend(b"WAVE");
    h.extend(if large { b"ds64" } else { b"JUNK" });
    h.extend(28u32.to_le_bytes());
    h.extend(if large { riff } else { 0 }.to_le_bytes());
    h.extend(if large { bytes } else { 0 }.to_le_bytes());
    h.extend(if large { bytes / 4 } else { 0 }.to_le_bytes());
    h.extend(0u32.to_le_bytes());
    h.extend(b"fmt ");
    h.extend(16u32.to_le_bytes());
    h.extend(1u16.to_le_bytes());
    h.extend(2u16.to_le_bytes());
    h.extend(48000u32.to_le_bytes());
    h.extend(192000u32.to_le_bytes());
    h.extend(4u16.to_le_bytes());
    h.extend(16u16.to_le_bytes());
    h.extend(b"data");
    h.extend(if large { u32::MAX } else { bytes as u32 }.to_le_bytes());
    Ok(h)
}
fn parse(file: &mut File, recovering: bool) -> AppResult<Info> {
    let total = file.metadata().map_err(|e| e.to_string())?.len();
    let mut first = [0u8; 12];
    file.read_exact(&mut first).map_err(|e| e.to_string())?;
    let rf64 = &first[..4] == b"RF64";
    if (!rf64 && &first[..4] != b"RIFF") || &first[8..] != b"WAVE" {
        return Err(crate::i18n::tr("Formato WAV desconhecido").into());
    }
    let (mut fmt, mut large, mut pos) = (false, None, 12u64);
    for _ in 0..128 {
        if pos + 8 > total {
            return Err(crate::i18n::tr("Cabeçalho WAV truncado").into());
        }
        file.seek(SeekFrom::Start(pos)).map_err(|e| e.to_string())?;
        let mut ch = [0u8; 8];
        file.read_exact(&mut ch).map_err(|e| e.to_string())?;
        let len = u32::from_le_bytes(ch[4..].try_into().unwrap()) as u64;
        if &ch[..4] == b"ds64" {
            if pos != 12 || len < 28 || len > 4096 {
                return Err(crate::i18n::tr("ds64 inválido").into());
            }
            let mut d = [0u8; 28];
            file.read_exact(&mut d).map_err(|e| e.to_string())?;
            let bytes = u64::from_le_bytes(d[8..16].try_into().unwrap());
            if u32::from_le_bytes(d[24..28].try_into().unwrap()) != 0
                || u64::from_le_bytes(d[16..24].try_into().unwrap()) != bytes / 4
            {
                return Err(
                    crate::i18n::tr("Tabela RF64 não suportada ou tamanho inválido").into(),
                );
            }
            large = Some(bytes);
        } else if &ch[..4] == b"fmt " {
            if !(16..=4096).contains(&len) {
                return Err(crate::i18n::tr("Formato PCM inválido").into());
            }
            let mut f = [0u8; 16];
            file.read_exact(&mut f).map_err(|e| e.to_string())?;
            fmt = u16::from_le_bytes(f[0..2].try_into().unwrap()) == 1
                && u16::from_le_bytes(f[2..4].try_into().unwrap()) == 2
                && u32::from_le_bytes(f[4..8].try_into().unwrap()) == 48000
                && u32::from_le_bytes(f[8..12].try_into().unwrap()) == 192000
                && u16::from_le_bytes(f[12..14].try_into().unwrap()) == 4
                && u16::from_le_bytes(f[14..16].try_into().unwrap()) == 16;
        } else if &ch[..4] == b"data" {
            if !fmt || (rf64 && (len != u32::MAX as u64 || large.is_none())) {
                return Err(crate::i18n::tr(
                    "WAV esperado: PCM estéreo 48 kHz/16 bits, RIFF ou RF64",
                )
                .into());
            }
            let offset = pos + 8;
            let bytes = if recovering {
                total.saturating_sub(offset) / 4 * 4
            } else if rf64 {
                large.unwrap()
            } else {
                len
            };
            if bytes % 4 != 0 || offset.checked_add(bytes).filter(|n| *n <= total).is_none() {
                return Err(crate::i18n::tr("WAV truncado").into());
            }
            return Ok(Info {
                offset,
                bytes,
                rf64,
            });
        }
        pos = pos
            .checked_add(8)
            .and_then(|p| p.checked_add(len + len % 2))
            .ok_or(crate::i18n::tr("Chunk WAV excede arquivo"))?;
    }
    Err(crate::i18n::tr("WAV com chunks demais").into())
}
pub fn info(path: &Path) -> AppResult<Info> {
    parse(&mut File::open(path).map_err(|e| e.to_string())?, false)
}
pub fn duration(path: &Path) -> AppResult<f64> {
    Ok(info(path)?.bytes as f64 / 192000.)
}
pub fn recover_copy(source: &Path, destination: &Path) -> AppResult<()> {
    let mut input = File::open(source).map_err(|e| e.to_string())?;
    let mut old = [0u8; 44];
    input.read_exact(&mut old).map_err(|e| e.to_string())?;
    input.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
    let i = if old.iter().all(|b| *b == 0) {
        Info {
            offset: 44,
            bytes: input
                .metadata()
                .map_err(|e| e.to_string())?
                .len()
                .saturating_sub(44)
                / 4
                * 4,
            rf64: false,
        }
    } else {
        parse(&mut input, true)?
    };
    input
        .seek(SeekFrom::Start(i.offset))
        .map_err(|e| e.to_string())?;
    let mut out = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(|e| e.to_string())?;
    let result = (|| -> AppResult<()> {
        out.write_all(&header(i.bytes)?)
            .map_err(|e| e.to_string())?;
        let copied =
            std::io::copy(&mut input.take(i.bytes), &mut out).map_err(|e| e.to_string())?;
        if copied != i.bytes {
            return Err(crate::i18n::tr("WAV mudou durante recuperação").into());
        }
        out.sync_all().map_err(|e| e.to_string())
    })();
    drop(out);
    if result.is_err() {
        let _ = std::fs::remove_file(destination);
    }
    result
}
pub struct Writer {
    file: File,
    samples: u64,
}
impl Drop for Writer {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}
impl Writer {
    pub fn new(path: &Path) -> AppResult<Self> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        file.write_all(&header(0)?).map_err(|e| e.to_string())?;
        Ok(Self { file, samples: 0 })
    }
    pub fn write(&mut self, values: &[i16]) -> AppResult<()> {
        if values.len() % 2 != 0 {
            return Err(crate::i18n::tr("PCM estéreo deve ter pares completos").into());
        }
        let samples = self
            .samples
            .checked_add(values.len() as u64)
            .filter(|n| {
                n.checked_mul(2)
                    .and_then(|b| b.checked_add(HEADER))
                    .is_some()
            })
            .ok_or(crate::i18n::tr("Áudio excede limite de 64 bits"))?;
        let mut b = Vec::with_capacity(values.len() * 2);
        for value in values {
            b.extend(value.to_le_bytes());
        }
        self.file.write_all(&b).map_err(|e| e.to_string())?;
        self.samples = samples;
        Ok(())
    }
    pub fn finish(&mut self) -> AppResult<()> {
        let h = header(self.samples * 2)?;
        self.file
            .seek(SeekFrom::Start(0))
            .and_then(|_| self.file.write_all(&h))
            .and_then(|_| self.file.seek(SeekFrom::End(0)).map(|_| ()))
            .and_then(|_| self.file.sync_all())
            .map_err(|e| e.to_string())
    }
}
/// Bounded reader, including RF64, for editor tracks and diagnostics.
pub struct PcmReader {
    file: File,
    pub info: Info,
}
impl PcmReader {
    pub fn open(path: &Path) -> AppResult<Self> {
        let mut file = File::open(path).map_err(|e| e.to_string())?;
        let info = parse(&mut file, false)?;
        Ok(Self { file, info })
    }
    pub fn read_frames(&mut self, frame: u64, count: usize) -> AppResult<Vec<f32>> {
        if count > 96000 {
            return Err(crate::i18n::tr("Leitura de áudio excede lote seguro").into());
        }
        let mut out = vec![0f32; count * 2];
        let start = frame.saturating_mul(4);
        if start >= self.info.bytes {
            return Ok(out);
        }
        let bytes = ((count as u64 * 4).min(self.info.bytes - start)) as usize;
        self.file
            .seek(SeekFrom::Start(self.info.offset + start))
            .map_err(|e| e.to_string())?;
        let mut data = vec![0u8; bytes];
        self.file.read_exact(&mut data).map_err(|e| e.to_string())?;
        for (i, b) in data.chunks_exact(2).enumerate() {
            out[i] = i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.;
        }
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rf64_crosses_4gib_on_sparse_fixture_without_filling_disk() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("validation/unit");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join(format!(
            "rf64-sparse-{}.wav",
            crate::settings::unique_stamp()
        ));
        let test = (|| -> AppResult<()> {
            let mut w = Writer::new(&path)?;
            crate::platform::windows::files::make_sparse_fixture(&w.file, &path)?;
            let data = 4_294_967_312u64;
            w.file
                .seek(SeekFrom::Start(HEADER + data - 4))
                .map_err(|e| e.to_string())?;
            w.samples = (data - 4) / 2;
            w.write(&[16000, -16000])?;
            w.finish()?;
            drop(w);
            let mut reader = PcmReader::open(&path)?;
            assert!(reader.info.rf64);
            assert_eq!(reader.info.bytes, data);
            let tail = reader.read_frames(data / 4 - 1, 1)?;
            assert!(tail[0] > 0.4 && tail[1] < -0.4);
            std::fs::write(root.join("rf64-synthetic.json"), format!("{{\"passed\":true,\"logical_bytes\":{},\"sparse\":true,\"decoded_tail\":true,\"long_recording\":false}}", data + HEADER)).map_err(|e| e.to_string())?;
            Ok(())
        })();
        let _ = std::fs::remove_file(path);
        test.unwrap();
    }
    #[test]
    fn checkpoint_append_and_rf64_synthetic_sizes() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("validation/unit");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join(format!(
            "checkpoint-{}.wav",
            crate::settings::unique_stamp()
        ));
        let mut w = Writer::new(&path).unwrap();
        w.write(&vec![12000; 960]).unwrap();
        w.finish().unwrap();
        w.write(&vec![-12000; 960]).unwrap();
        w.finish().unwrap();
        drop(w);
        assert_eq!(duration(&path).unwrap(), 0.02);
        let mut r = PcmReader::open(&path).unwrap();
        assert!(r.read_frames(0, 1).unwrap()[0] > 0.3);
        assert!(r.read_frames(480, 1).unwrap()[0] < -0.3);
        for size in [0, 4_294_967_296, 48_000_000_000] {
            let h = header(size).unwrap();
            assert_eq!(h.len(), 80);
            if size >= 4_294_967_296 {
                assert_eq!(&h[..4], b"RF64");
                assert_eq!(&h[12..16], b"ds64");
                assert_eq!(u64::from_le_bytes(h[28..36].try_into().unwrap()), size);
                assert_eq!(u64::from_le_bytes(h[36..44].try_into().unwrap()), size / 4);
            }
        }
        assert!(header(u64::MAX).is_err());
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn repairs_legacy_crash_header_and_refuses_collision() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("validation/unit")
            .join(format!("wav-{}", crate::settings::unique_stamp()));
        std::fs::create_dir_all(&root).unwrap();
        let source = root.join("Interrupted.wav");
        let dest = root.join("Recovered.wav");
        let mut data = vec![0; 44];
        for _ in 0..100 {
            data.extend(16000i16.to_le_bytes());
            data.extend((-16000i16).to_le_bytes());
        }
        std::fs::write(&source, &data).unwrap();
        recover_copy(&source, &dest).unwrap();
        assert!(
            crate::audio::waveform(&dest, 4)
                .unwrap()
                .iter()
                .all(|v| *v > 0.4)
        );
        let before = std::fs::read(&dest).unwrap();
        assert!(recover_copy(&source, &dest).is_err());
        assert_eq!(std::fs::read(dest).unwrap(), before);
        assert_eq!(std::fs::read(source).unwrap(), data);
        std::fs::remove_dir_all(root).unwrap();
    }
}
