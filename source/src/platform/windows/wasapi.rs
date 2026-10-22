use crate::{
    audio::{AudioQueue, resample::Resampler},
    core::AppResult,
};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use windows::Win32::{
    Media::{Audio::*, Multimedia::KSDATAFORMAT_SUBTYPE_IEEE_FLOAT},
    System::Com::CoTaskMemFree,
};
pub struct TestTone;
impl TestTone {
    pub fn start(path: &std::path::Path) -> AppResult<Self> {
        if !path.components().any(|c| c.as_os_str() == "validation") {
            return Err("Tom de teste fora de validation".into());
        }
        unsafe {
            use windows::Win32::Media::Audio::*;
            if !PlaySoundW(
                super::pcw(&super::wide(path)),
                None,
                SND_FILENAME | SND_ASYNC | SND_LOOP | SND_NODEFAULT,
            )
            .as_bool()
            {
                return Err(
                    crate::i18n::tr("Não foi possível reproduzir o tom local de teste").into(),
                );
            }
        }
        Ok(Self)
    }
}
impl Drop for TestTone {
    fn drop(&mut self) {
        unsafe {
            let _ = windows::Win32::Media::Audio::PlaySoundW(
                windows::core::PCWSTR::null(),
                None,
                windows::Win32::Media::Audio::SND_PURGE,
            );
        }
    }
}
pub struct AudioCapture {
    clock: Arc<Mutex<ClockStats>>,
    pub queue: Arc<Mutex<AudioQueue>>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    error: Arc<Mutex<Option<String>>>,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct ClockStats {
    pub rate: u32,
    pub first_device: Option<u64>,
    pub last_device: u64,
    pub first_qpc: u64,
    pub last_qpc: u64,
    pub discontinuities: u64,
    pub timestamp_errors: u64,
}
impl ClockStats {
    pub fn drift_ms(&self) -> Option<f64> {
        self.first_device
            .filter(|_| self.rate > 0 && self.last_qpc >= self.first_qpc)
            .map(|first| {
                ((self.last_device.saturating_sub(first)) as f64 / self.rate as f64
                    - (self.last_qpc - self.first_qpc) as f64 / 10_000_000.)
                    * 1000.
            })
    }
}
impl AudioCapture {
    pub fn start(input: bool, id: String) -> AppResult<Self> {
        let queue = Arc::new(Mutex::new(AudioQueue::default()));
        let clock = Arc::new(Mutex::new(ClockStats::default()));
        let clock_out = clock.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let q = queue.clone();
        let s = stop.clone();
        let error = Arc::new(Mutex::new(None));
        let error_out = error.clone();
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        let thread = thread::spawn(move || {
            let result = (|| -> AppResult<()> {
                let _runtime = super::setup::Com::multithreaded()?;
                unsafe {
                    let device =
                        super::devices::audio_endpoint(input, &id).map_err(|e| e.to_string())?;
                    let client: IAudioClient = device
                        .Activate(windows::Win32::System::Com::CLSCTX_INPROC_SERVER, None)
                        .map_err(|e| e.to_string())?;
                    let format = client.GetMixFormat().map_err(|e| e.to_string())?;
                    let f = *format;
                    let channels = f.nChannels as usize;
                    let rate = f.nSamplesPerSec;
                    let bits = f.wBitsPerSample;
                    let align = f.nBlockAlign as usize;
                    let subformat = if f.wFormatTag == 0xfffe && f.cbSize >= 22 {
                        Some((*(format as *const WAVEFORMATEXTENSIBLE)).SubFormat)
                    } else {
                        None
                    };
                    let float =
                        f.wFormatTag == 3 || subformat == Some(KSDATAFORMAT_SUBTYPE_IEEE_FLOAT);
                    let channel_mask = if f.wFormatTag == 0xfffe && f.cbSize >= 22 {
                        (*(format as *const WAVEFORMATEXTENSIBLE)).dwChannelMask
                    } else {
                        0
                    };
                    if !(1..=32).contains(&channels)
                        || !(8000..=384000).contains(&rate)
                        || ![16, 24, 32].contains(&bits)
                        || (float && bits != 32)
                        || align != channels * bits as usize / 8
                    {
                        CoTaskMemFree(Some(format.cast()));
                        return Err(crate::i18n::tr("Formato WASAPI não suportado. Selecione outro endpoint/formato no Windows.").into());
                    }
                    let mut resampler = Resampler::new(channels, rate, channel_mask);
                    let flags = if input {
                        0
                    } else {
                        AUDCLNT_STREAMFLAGS_LOOPBACK
                    };
                    let init = client.Initialize(
                        AUDCLNT_SHAREMODE_SHARED,
                        flags,
                        1_000_000,
                        0,
                        format,
                        None,
                    );
                    CoTaskMemFree(Some(format.cast()));
                    init.map_err(|e| e.to_string())?;
                    let capture: IAudioCaptureClient =
                        client.GetService().map_err(|e| e.to_string())?;
                    client.Start().map_err(|e| e.to_string())?;
                    let _ = tx.send(Ok(()));
                    while !s.load(Ordering::Relaxed) {
                        loop {
                            let packets = capture.GetNextPacketSize().map_err(|e| e.to_string())?;
                            if packets == 0 {
                                break;
                            }
                            let (mut data, mut frames, mut flags) = (std::ptr::null_mut(), 0, 0);
                            let (mut device_position, mut qpc_position) = (0u64, 0u64);
                            capture
                                .GetBuffer(
                                    &mut data,
                                    &mut frames,
                                    &mut flags,
                                    Some(&mut device_position),
                                    Some(&mut qpc_position),
                                )
                                .map_err(|e| e.to_string())?;
                            if let Ok(mut c) = clock_out.lock() {
                                c.rate = rate;
                                if flags & AUDCLNT_BUFFERFLAGS_DATA_DISCONTINUITY.0 as u32 != 0 {
                                    c.discontinuities += 1;
                                }
                                if flags & AUDCLNT_BUFFERFLAGS_TIMESTAMP_ERROR.0 as u32 != 0 {
                                    c.timestamp_errors += 1;
                                } else {
                                    if c.first_device.is_none() {
                                        c.first_device = Some(device_position);
                                        c.first_qpc = qpc_position;
                                    }
                                    c.last_device = device_position;
                                    c.last_qpc = qpc_position;
                                }
                            }
                            let mut samples = vec![0f32; frames as usize * channels];
                            if flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 == 0 && !data.is_null() {
                                let bytes =
                                    std::slice::from_raw_parts(data, frames as usize * align);
                                for (i, v) in samples.iter_mut().enumerate() {
                                    let p = i * bits as usize / 8;
                                    *v = if float && bits == 32 {
                                        f32::from_le_bytes(bytes[p..p + 4].try_into().unwrap())
                                    } else if bits == 16 {
                                        i16::from_le_bytes(bytes[p..p + 2].try_into().unwrap())
                                            as f32
                                            / 32768.
                                    } else if bits == 32 {
                                        i32::from_le_bytes(bytes[p..p + 4].try_into().unwrap())
                                            as f32
                                            / 2147483648.
                                    } else if bits == 24 {
                                        let n = i32::from_le_bytes([
                                            0,
                                            bytes[p],
                                            bytes[p + 1],
                                            bytes[p + 2],
                                        ]);
                                        n as f32 / 2147483648.
                                    } else {
                                        0.
                                    };
                                }
                            }
                            let release = capture.ReleaseBuffer(frames);
                            release.map_err(|e| e.to_string())?;
                            let resampled = resampler.push(&samples);
                            if let Ok(mut q) = q.lock() {
                                q.push(&resampled);
                            }
                        }
                        thread::sleep(Duration::from_millis(4));
                    }
                    let _ = client.Stop();
                }
                Ok(())
            })();
            if let Err(e) = result {
                if let Ok(mut last) = error_out.lock() {
                    *last = Some(e.clone());
                }
                let _ = tx.send(Err(e));
            }
        });
        match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(Ok(())) => Ok(Self {
                clock,
                queue,
                stop,
                thread: Some(thread),
                error,
            }),
            Ok(Err(e)) => {
                stop.store(true, Ordering::Relaxed);
                let _ = thread.join();
                Err(crate::i18n::format_text(
                    crate::i18n::tr("WASAPI: {e}"),
                    &[("e", format!("{}", e))],
                ))
            }
            Err(e) => {
                stop.store(true, Ordering::Relaxed);
                Err(crate::i18n::format_text(
                    crate::i18n::tr("Áudio não respondeu: {e}"),
                    &[("e", format!("{}", e))],
                ))
            }
        }
    }
    pub fn take(&self, n: usize) -> Vec<f32> {
        self.queue
            .lock()
            .map(|mut q| q.take(n))
            .unwrap_or_else(|_| vec![0.; n])
    }
    pub fn clock(&self) -> ClockStats {
        self.clock.lock().map(|v| *v).unwrap_or_default()
    }
    pub fn clear(&self) {
        if let Ok(mut q) = self.queue.lock() {
            q.clear();
        }
    }
    pub fn peak(&self) -> f32 {
        self.queue.lock().map(|q| q.peak).unwrap_or(0.)
    }
    pub fn take_error(&self) -> Option<String> {
        self.error.lock().ok()?.take()
    }
    pub fn counters(&self) -> (u64, u64) {
        self.queue
            .lock()
            .map(|q| (q.padded, q.dropped))
            .unwrap_or_default()
    }
}
impl Drop for AudioCapture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
