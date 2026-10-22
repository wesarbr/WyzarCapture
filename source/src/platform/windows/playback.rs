//! WASAPI preview output. The worker owns every COM interface it creates.
use crate::{core::AppResult, editor::Project};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use windows::Win32::{Media::Audio::*, System::Com::CLSCTX_INPROC_SERVER};
pub struct Playback {
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    errors: std::sync::mpsc::Receiver<String>,
}
impl Playback {
    pub fn start(project: Project, position: f64) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let (tx, errors) = std::sync::mpsc::channel();
        let worker = thread::spawn(move || {
            if let Err(e) = play(project, position, &flag) {
                let _ = tx.send(e);
            }
        });
        Self {
            stop,
            worker: Some(worker),
            errors,
        }
    }
    pub fn error(&self) -> Option<String> {
        self.errors.try_recv().ok()
    }
}
impl Drop for Playback {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
fn play(project: Project, position: f64, stop: &AtomicBool) -> AppResult<()> {
    let _runtime = super::Runtime::new(true)?;
    unsafe {
        let device = super::devices::audio_endpoint(false, "").map_err(|e| e.to_string())?;
        let client: IAudioClient = device
            .Activate(CLSCTX_INPROC_SERVER, None)
            .map_err(|e| e.to_string())?;
        let format = WAVEFORMATEX {
            wFormatTag: 1,
            nChannels: 2,
            nSamplesPerSec: 48000,
            nAvgBytesPerSec: 192000,
            nBlockAlign: 4,
            wBitsPerSample: 16,
            cbSize: 0,
        };
        client
            .Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY,
                1_000_000,
                0,
                &format,
                None,
            )
            .map_err(|e| e.to_string())?;
        let capacity = client.GetBufferSize().map_err(|e| e.to_string())?;
        let render: IAudioRenderClient = client.GetService().map_err(|e| e.to_string())?;
        let Some((first, source_position)) = project.locate(position) else {
            return Ok(());
        };
        client.Start().map_err(|e| e.to_string())?;
        let mut mixer = if project.tracks_active {
            Some(crate::editor::tracks::Mixer::new(&project.audio_tracks)?)
        } else {
            None
        };
        let mut timeline_position = position;
        for (index, clip) in project.clips.iter().enumerate().skip(first) {
            let source_start = if index == first {
                source_position
            } else {
                clip.start
            };
            let mut reader = super::media::AudioReader::open(&clip.source, source_start).ok();
            let target_frames = ((clip.end - source_start) / clip.speed * 48000.).max(0.) as u64;
            let mut written = 0;
            while written < target_frames && !stop.load(Ordering::Relaxed) {
                let padding = client.GetCurrentPadding().map_err(|e| e.to_string())?;
                let count =
                    (capacity - padding).min((target_frames - written).min(u32::MAX as u64) as u32);
                if count > 0 {
                    let buffer = render.GetBuffer(count).map_err(|e| e.to_string())?;
                    let samples = if let Some(mixer) = mixer.as_mut() {
                        Some(mixer.take(timeline_position, count as usize)?)
                    } else {
                        reader
                            .as_mut()
                            .map(|r| r.take(count as usize, clip.speed, clip.volume))
                    };
                    if let Some(samples) = samples {
                        for (i, s) in samples.iter().enumerate() {
                            std::ptr::write_unaligned(buffer.add(i * 2).cast::<i16>(), *s);
                        }
                        render.ReleaseBuffer(count, 0).map_err(|e| e.to_string())?;
                    } else {
                        render
                            .ReleaseBuffer(count, AUDCLNT_BUFFERFLAGS_SILENT.0 as u32)
                            .map_err(|e| e.to_string())?;
                    }
                    written += count as u64;
                    timeline_position += count as f64 / 48000.;
                }
                thread::sleep(Duration::from_millis(5));
            }
            if stop.load(Ordering::Relaxed) {
                break;
            }
        }
        if !stop.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(100));
        }
        let _ = client.Stop();
    }
    Ok(())
}
