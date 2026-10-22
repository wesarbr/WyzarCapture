use crate::audio::wav::Writer as Wav;
pub mod metrics;
mod sink;
pub mod timing;
use crate::{
    core::{AppResult, Frame, RecordingInfo},
    encoder::VideoConfig,
    platform::windows::{
        self as win,
        capture::{Capture, Target},
        wasapi::AudioCapture,
    },
    settings::Settings,
};
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
#[derive(Clone, Debug)]
pub enum Event {
    Started,
    Warning(String),
    Metrics(metrics::Snapshot),
    Progress {
        seconds: f64,
        frames: u64,
        mic: f32,
        system: f32,
    },
    Saved(RecordingInfo),
    Error(String),
}
pub struct Session {
    stop: Arc<AtomicBool>,
    pause: Arc<AtomicBool>,
    pub events: Receiver<Event>,
    thread: Option<JoinHandle<()>>,
}
impl Session {
    pub fn start(
        settings: Settings,
        target: Target,
        path: PathBuf,
        preview: Arc<Mutex<Option<Frame>>>,
    ) -> Self {
        Self::start_inner(settings, Some(target), path, preview)
    }
    pub fn start_audio(settings: Settings, path: PathBuf) -> Self {
        Self::start_inner(settings, None, path, Arc::new(Mutex::new(None)))
    }
    fn start_inner(
        settings: Settings,
        target: Option<Target>,
        path: PathBuf,
        preview: Arc<Mutex<Option<Frame>>>,
    ) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let pause = Arc::new(AtomicBool::new(false));
        let (tx, events) = mpsc::channel();
        let (s, p) = (stop.clone(), pause.clone());
        let thread = thread::spawn(move || {
            let result = if let Some(target) = target {
                let mut settings = settings;
                if settings.recording_mode == 1 {
                    settings.microphone = false;
                    settings.system_audio = false;
                }
                record(&settings, &target, &path, &s, &p, &preview, |e| {
                    let _ = tx.send(e);
                })
            } else {
                crate::audio::recorder::record(&settings, &path, &s, &p, |e| {
                    let _ = tx.send(e);
                })
            };
            if let Err(e) = result {
                let _ = tx.send(Event::Error(e));
            }
        });
        Self {
            stop,
            pause,
            events,
            thread: Some(thread),
        }
    }
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
    pub fn pause(&self, value: bool) {
        self.pause.store(value, Ordering::Relaxed);
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        self.stop();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
fn record(
    settings: &Settings,
    target: &Target,
    path: &std::path::Path,
    stop: &AtomicBool,
    pause: &AtomicBool,
    preview: &Arc<Mutex<Option<Frame>>>,
    mut emit: impl FnMut(Event),
) -> AppResult<()> {
    let _runtime = win::Runtime::new(true)?;
    crate::diagnostics::event("capture_start", None);
    let mut capture = Capture::new(target)?;
    if settings.gpu_resize {
        capture.resize_on_gpu(settings.width, settings.height);
    }
    let mic = if settings.microphone {
        match AudioCapture::start(true, settings.microphone_id.clone()) {
            Ok(a) => Some(a),
            Err(e) => {
                emit(Event::Warning(crate::i18n::format_text(
                    crate::i18n::tr(
                        "Microfone indisponível: {e}. A captura continuará com as outras fontes.",
                    ),
                    &[("e", format!("{}", e))],
                )));
                None
            }
        }
    } else {
        None
    };
    let system = if settings.system_audio {
        match AudioCapture::start(false, settings.output_id.clone()) {
            Ok(a) => Some(a),
            Err(e) => {
                emit(Event::Warning(crate::i18n::format_text(
                    crate::i18n::tr(
                        "Áudio do sistema indisponível: {e}. O vídeo continuará sem essa fonte.",
                    ),
                    &[("e", format!("{}", e))],
                )));
                None
            }
        }
    } else {
        None
    };
    let mut webcam = settings.webcam.then(|| CameraFeed::start(settings));
    let mut effective = settings.clone();
    let forced_hardware = std::env::var("FORCE_HARDWARE_ENCODER").as_deref() == Ok("1");
    let forced_software = std::env::var("FORCE_SOFTWARE_ENCODER").as_deref() == Ok("1");
    if forced_hardware {
        effective.hardware = true;
    }
    if forced_software {
        effective.hardware = false;
    }
    if effective.hardware && !forced_hardware && !win::capabilities::inventory()?.hardware() {
        effective.hardware = false;
        emit(Event::Warning(crate::i18n::tr("Nenhum encoder H.264 de hardware foi encontrado. Usando software com a resolução e FPS solicitados.").into()));
    }
    let storage_budget = if settings.storage_limit_gb > 0 {
        let mut library = crate::library::Library::default();
        library.scan(&settings.videos);
        let used: u64 = library
            .items
            .iter()
            .filter(|r| !r.missing)
            .map(|r| r.bytes)
            .sum();
        Some((settings.storage_limit_gb as u64 * 1_073_741_824).saturating_sub(used))
    } else {
        None
    };
    win::storage::require(
        path,
        crate::core::storage::reserve(0, settings.recovery, settings.bitrate),
    )?;
    // The recovery journal can be configured on a different volume.
    win::storage::require(&settings.data_dir, 16 * crate::core::storage::MIB)?;
    let audio_enabled = settings.recording_mode != 2 && (mic.is_some() || system.is_some());
    let mut writer = match sink::RecordingSink::new(&effective, path, audio_enabled) {
        Ok(writer) => writer,
        Err(error) if settings.auto_fallback && !forced_hardware && !forced_software => {
            // First keep the requested quality and change only the encoder.
            effective.hardware = false;
            if let Ok(writer) = sink::RecordingSink::new(&effective, path, audio_enabled) {
                emit(Event::Warning(crate::i18n::tr("Hardware indisponível para este formato. Mantidos resolução, FPS e bitrate, com encoder por software.").into()));
                writer
            } else {
                effective.width = 1280;
                effective.height = 720;
                effective.fps = 30;
                effective.bitrate = 5_000_000;
                effective.hardware = false;
                let writer = sink::RecordingSink::new(&effective, path, audio_enabled).map_err(
                    |fallback| {
                        crate::i18n::format_text(
                            crate::i18n::tr(
                                "Perfil solicitado: {error}. Compatibilidade 720p/30: {fallback}",
                            ),
                            &[
                                ("error", format!("{}", error)),
                                ("fallback", format!("{}", fallback)),
                            ],
                        )
                    },
                )?;
                emit(Event::Warning(crate::i18n::tr("O encoder recusou a qualidade solicitada. Gravando em 1280 × 720, 30 FPS, H.264 por software. As preferências originais foram mantidas.").into()));
                if settings.gpu_resize {
                    capture.resize_on_gpu(effective.width, effective.height);
                }
                writer
            }
        }
        Err(error) => return Err(error),
    };
    let cfg = VideoConfig::from_settings(&effective);
    let mut separate = if settings.recording_mode == 2 && (mic.is_some() || system.is_some()) {
        Some(Wav::new(&path.with_extension("audio.wav"))?)
    } else {
        None
    };
    let frame_duration = Duration::from_secs_f64(1. / cfg.fps as f64);
    let ticks = 10_000_000 / cfg.fps as i64;
    let mut frames = 0u64;
    let mut last = None;
    let mut next = Instant::now();
    let mut last_progress = Instant::now();
    let mut empty_since = Instant::now();
    let mut last_cam = None;
    let mut recording_clock: Option<Instant> = None;
    let mut paused_for = Duration::ZERO;
    let mut pause_started: Option<Instant> = None;
    let mut audio_cursor = 0u64;
    let mut last_timestamp = 0i64;
    let mut checkpoint_at = Instant::now();
    let mut space_warned = false;
    let mut measurements = metrics::Measurements::new(cfg.fps);
    let mut limiter = crate::audio::limiter::Limiter::new();
    let mut voice =
        (settings.voice_filter && mic.is_some()).then(crate::audio::voice::VoiceFilter::new);
    measurements.snapshot.encoder = writer.encoder();
    emit(Event::Started);
    let mut mic_wav = if mic.is_some() {
        Some(Wav::new(&path.with_extension("mic.wav"))?)
    } else {
        None
    };
    let mut sys_wav = if system.is_some() {
        Some(Wav::new(&path.with_extension("system.wav"))?)
    } else {
        None
    };
    let mut space_checked = Instant::now();
    while !stop.load(Ordering::Relaxed) {
        if space_checked.elapsed() >= Duration::from_secs(1) {
            space_checked = Instant::now();
            let required =
                crate::core::storage::reserve(writer.bytes(), settings.recovery, effective.bitrate);
            if let Err(error) = win::storage::require(path, required).and_then(|_| {
                win::storage::require(&settings.data_dir, 16 * crate::core::storage::MIB)
            }) {
                emit(Event::Warning(error));
                break;
            }
        }
        if mic.as_ref().and_then(AudioCapture::take_error).is_some() {
            emit(Event::Warning(crate::i18n::tr("O microfone foi desconectado ou ficou indisponível. A gravação continuará com as outras fontes.").into()));
        }
        if system.as_ref().and_then(AudioCapture::take_error).is_some() {
            emit(Event::Warning(crate::i18n::tr("O áudio do sistema ficou indisponível. O vídeo e as outras fontes continuarão sendo gravados.").into()));
        }
        if pause.load(Ordering::Relaxed) {
            pause_started.get_or_insert_with(Instant::now);
            if let Some(a) = &mic {
                a.clear();
            }
            if let Some(a) = &system {
                a.clear();
            }
            thread::sleep(Duration::from_millis(12));
            next = Instant::now();
            continue;
        }
        if let Some(start) = pause_started.take() {
            paused_for += start.elapsed();
        }
        let processing_started = Instant::now();
        if let Some(frame) = capture.frame()? {
            measurements.snapshot.received += 1;
            last = Some(frame);
            empty_since = Instant::now();
        } else if last.is_none() && empty_since.elapsed() > Duration::from_secs(5) {
            return Err(crate::i18n::tr(
                "A fonte de captura não forneceu quadros. Verifique se a janela continua aberta.",
            )
            .into());
        }
        if let Some(warning) = capture.take_warning() {
            emit(Event::Warning(warning));
        }
        let Some(raw) = last.as_ref() else {
            thread::sleep(Duration::from_millis(4));
            continue;
        };
        let clock = *recording_clock.get_or_insert_with(|| {
            if let Some(a) = &mic {
                a.clear();
            }
            if let Some(a) = &system {
                a.clear();
            }
            Instant::now()
        });
        let timestamp = (clock.elapsed().saturating_sub(paused_for).as_nanos() / 100) as i64;
        let timestamp = if frames == 0 {
            0
        } else {
            timestamp.max(last_timestamp + 1)
        };
        let mut frame = if raw.width == cfg.width && raw.height == cfg.height {
            raw.clone()
        } else {
            raw.fit(cfg.width, cfg.height)
        };
        if let Some(cam) = webcam.as_ref() {
            if let Some((opened, error)) = cam.take_error() {
                webcam = None;
                last_cam = None;
                emit(Event::Warning(crate::i18n::format_text(
                    if opened {
                        crate::i18n::tr(
                            "Webcam indisponível: {error}. A gravação continuará sem câmera.",
                        )
                    } else {
                        crate::i18n::tr(
                            "A webcam não pôde ser aberta: {error}. A gravação continuará sem câmera.",
                        )
                    },
                    &[("error", error)],
                )));
            } else if let Some(latest) = cam.latest() {
                // Shape the tile once per camera frame, not once per video frame.
                last_cam = camera_tile(
                    &latest,
                    (cfg.width * settings.webcam_size / 100).min(cfg.width.saturating_sub(8)),
                    settings.webcam_shape,
                    settings.webcam_mirror,
                );
            }
        }
        if let Some(tile) = &last_cam {
            overlay(&mut frame, tile, settings.webcam_position);
        }
        if settings.click_effects {
            win::selection::click_overlay(&mut frame, target);
        }
        let encoding_started = Instant::now();
        writer.video(&frame, timestamp, ticks)?;
        let video_encoding_ms = encoding_started.elapsed().as_secs_f64() * 1000.;
        let audio_end = timing::audio_end(timestamp, ticks).max(audio_cursor);
        let audio_frames = (audio_end - audio_cursor) as usize;
        let mut a = mic
            .as_ref()
            .map(|x| x.take(audio_frames * 2))
            .unwrap_or_else(|| vec![0.; audio_frames * 2]);
        if let Some(filter) = voice.as_mut() {
            filter.process(&mut a);
        }
        let b = system
            .as_ref()
            .map(|x| x.take(audio_frames * 2))
            .unwrap_or_else(|| vec![0.; audio_frames * 2]);
        let mixed = if settings.audio_limiter {
            limiter.mix(
                &a,
                &b,
                settings.mic_gain as f32 / 100.,
                settings.system_gain as f32 / 100.,
            )
        } else {
            crate::audio::mix(
                &a,
                &b,
                settings.mic_gain as f32 / 100.,
                settings.system_gain as f32 / 100.,
            )
        };
        let audio_encoding_started = Instant::now();
        writer.audio(&mixed, audio_cursor as i64 * 10_000_000 / 48000)?;
        if let Some(w) = separate.as_mut() {
            w.write(&mixed)?;
        }
        let encoding_ms =
            video_encoding_ms + audio_encoding_started.elapsed().as_secs_f64() * 1000.;
        audio_cursor = audio_end;
        if frames > 0 {
            measurements.snapshot.missed_deadlines +=
                timing::missed_intervals(last_timestamp, timestamp, ticks);
        }
        last_timestamp = timestamp;
        if let Some(w) = mic_wav.as_mut() {
            w.write(&crate::audio::mix(&a, &[], 1., 0.))?;
        }
        if let Some(w) = sys_wav.as_mut() {
            w.write(&crate::audio::mix(&b, &[], 1., 0.))?;
        }
        if frames % 6 == 0 {
            if let Ok(mut p) = preview.lock() {
                *p = Some(frame.resize(960, 540));
            }
        }
        frames += 1;
        measurements.frame(
            processing_started.elapsed().as_secs_f64() * 1000.,
            encoding_ms,
        );
        measurements.snapshot.active_seconds = (last_timestamp + ticks) as f64 / 10_000_000.;
        measurements.snapshot.gpu_resize = capture.gpu_resize;
        measurements.snapshot.audio_frames = audio_cursor;
        measurements.snapshot.submitted_av_difference_ms =
            (audio_cursor as f64 / 48000. - measurements.snapshot.active_seconds) * 1000.;
        if checkpoint_at.elapsed() >= Duration::from_secs(5) {
            checkpoint_at = Instant::now();
            writer.checkpoint(frames)?;
            if let Some((free, _)) = win::files::free_space(path.parent().unwrap()) {
                if free < settings.low_space_gb as u64 * 1_073_741_824 && !space_warned {
                    emit(Event::Warning({
                        let __wyzar_i18n_0 = &(crate::core::size_label(free));
                        crate::i18n::format_text(
                            crate::i18n::tr("Restam {} no disco de gravação."),
                            &[("0", format!("{}", __wyzar_i18n_0))],
                        )
                    }));
                    space_warned = true;
                }
                if free < 256 * 1024 * 1024 {
                    emit(Event::Warning(
                        crate::i18n::tr("Espaço crítico. Finalizando os dados preservados.").into(),
                    ));
                    break;
                }
            }
            if storage_budget.is_some_and(|max| writer.bytes() >= max) {
                emit(Event::Warning(
                    crate::i18n::tr("Limite de armazenamento atingido. Finalizando a gravação.")
                        .into(),
                ));
                break;
            }
        }
        if last_progress.elapsed() >= Duration::from_millis(200) {
            if let Some(stats) = writer.statistics() {
                measurements.snapshot.encoder_queue_bytes = stats.queued_bytes;
                measurements.snapshot.encoder_delay_ms = stats.encoding_delay_ms;
                measurements.snapshot.encoder_samples_segment = stats.encoded;
            }
            measurements.sample_process();
            measurements.snapshot.bytes = writer.bytes();
            measurements.snapshot.free_bytes = win::files::free_space(path.parent().unwrap())
                .map(|s| s.0)
                .unwrap_or(0);
            let (a, b) = mic.as_ref().map(AudioCapture::counters).unwrap_or_default();
            let (c, d) = system
                .as_ref()
                .map(AudioCapture::counters)
                .unwrap_or_default();
            measurements.snapshot.audio_padded_samples = a + c;
            measurements.snapshot.audio_dropped_samples = b + d;
            measurements.snapshot.video_pts_end_100ns = last_timestamp + ticks;
            measurements.snapshot.audio_pts_end_100ns = audio_cursor as i64 * 10_000_000 / 48000;
            if let Some(mic) = &mic {
                let c = mic.clock();
                measurements.snapshot.mic_device_clock_drift_ms = c.drift_ms();
                measurements.snapshot.mic_discontinuities = c.discontinuities;
            }
            if let Some(system) = &system {
                let c = system.clock();
                measurements.snapshot.system_device_clock_drift_ms = c.drift_ms();
                measurements.snapshot.system_discontinuities = c.discontinuities;
            }
            emit(Event::Metrics(measurements.snapshot.clone()));
            emit(Event::Progress {
                seconds: (last_timestamp + ticks) as f64 / 10_000_000.,
                frames,
                mic: mic.as_ref().map(AudioCapture::peak).unwrap_or(0.),
                system: system.as_ref().map(AudioCapture::peak).unwrap_or(0.),
            });
            last_progress = Instant::now();
        }
        next += frame_duration;
        let now = Instant::now();
        if next > now {
            thread::sleep(next - now);
        } else if now.duration_since(next) > Duration::from_secs(1) {
            next = now;
        }
    }
    if frames == 0 {
        return Err(crate::i18n::tr("A gravação terminou antes do primeiro quadro").into());
    }
    measurements.sample_process();
    // Final diagnostics must describe the last submitted samples, not the last
    // 150 ms UI refresh before stopping.
    measurements.snapshot.video_pts_end_100ns = last_timestamp + ticks;
    measurements.snapshot.audio_pts_end_100ns = audio_cursor as i64 * 10_000_000 / 48000;
    if let Some(mic) = &mic {
        let c = mic.clock();
        measurements.snapshot.mic_device_clock_drift_ms = c.drift_ms();
        measurements.snapshot.mic_discontinuities = c.discontinuities;
    }
    if let Some(system) = &system {
        let c = system.clock();
        measurements.snapshot.system_device_clock_drift_ms = c.drift_ms();
        measurements.snapshot.system_discontinuities = c.discontinuities;
    }
    emit(Event::Metrics(measurements.snapshot.clone()));
    let duration = writer.finish()?;
    crate::diagnostics::event("capture_saved", None);
    if let Some(mut w) = mic_wav {
        w.finish()?;
    }
    if let Some(mut w) = sys_wav {
        w.finish()?;
    }
    if let Some(mut w) = separate {
        w.finish()?;
    }
    emit(Event::Saved(RecordingInfo {
        path: path.into(),
        duration: Duration::from_secs_f64(duration),
        frames,
        width: cfg.width,
        height: cfg.height,
    }));
    Ok(())
}
/// Reads the camera on its own thread. The recording loop takes the newest frame
/// and never waits for the device, so a slow or dark webcam cannot stall the screen.
struct CameraFeed {
    stop: Arc<AtomicBool>,
    latest: Arc<Mutex<Option<Frame>>>,
    error: Arc<Mutex<Option<(bool, String)>>>,
    thread: Option<JoinHandle<()>>,
}
impl CameraFeed {
    fn start(settings: &Settings) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let latest = Arc::new(Mutex::new(None));
        let error = Arc::new(Mutex::new(None));
        let (s, l, e, id) = (
            stop.clone(),
            latest.clone(),
            error.clone(),
            settings.webcam_id.clone(),
        );
        let thread = thread::spawn(move || {
            let mut opened = false;
            let result = (|| -> AppResult<()> {
                let _runtime = win::Runtime::new(true)?;
                let mut cam = win::media::Webcam::open(&id)?;
                opened = true;
                while !s.load(Ordering::Relaxed) {
                    match cam.next()? {
                        Some(frame) => {
                            if let Ok(mut slot) = l.lock() {
                                *slot = Some(frame);
                            }
                        }
                        None => thread::sleep(Duration::from_millis(4)),
                    }
                }
                Ok(())
            })();
            if let Err(message) = result {
                if let Ok(mut slot) = e.lock() {
                    *slot = Some((opened, message));
                }
            }
        });
        Self {
            stop,
            latest,
            error,
            thread: Some(thread),
        }
    }
    fn latest(&self) -> Option<Frame> {
        self.latest.lock().ok().and_then(|mut f| f.take())
    }
    fn take_error(&self) -> Option<(bool, String)> {
        self.error.lock().ok().and_then(|mut e| e.take())
    }
}
impl Drop for CameraFeed {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
/// Scale, crop and mask the camera image for the requested corner of the video.
/// Shape 0 is a rectangle, 1 has rounded corners and 2 is a circle; masked shapes
/// get a thin light edge so the camera reads clearly over any content.
fn camera_tile(cam: &Frame, width: u32, shape: usize, mirror: bool) -> Option<Frame> {
    if !cam.valid() || width < 8 {
        return None;
    }
    let source = if shape == 2 {
        let side = cam.width.min(cam.height);
        let (x0, y0) = ((cam.width - side) / 2, (cam.height - side) / 2);
        let mut square = Frame {
            width: side,
            height: side,
            bgra: vec![0; (side * side * 4) as usize],
        };
        for y in 0..side {
            let from = (((y + y0) * cam.width + x0) * 4) as usize;
            let to = (y * side * 4) as usize;
            square.bgra[to..to + (side * 4) as usize]
                .copy_from_slice(&cam.bgra[from..from + (side * 4) as usize]);
        }
        square
    } else {
        cam.clone()
    };
    let height = if shape == 2 {
        width
    } else {
        (width * source.height / source.width.max(1)).max(1)
    };
    let mut tile = source.resize(width, height);
    if mirror {
        for row in tile.bgra.chunks_exact_mut((width * 4) as usize) {
            let pixels: Vec<[u8; 4]> = row
                .chunks_exact(4)
                .rev()
                .map(|p| [p[0], p[1], p[2], p[3]])
                .collect();
            for (dst, src) in row.chunks_exact_mut(4).zip(pixels) {
                dst.copy_from_slice(&src);
            }
        }
    }
    if shape == 0 {
        return Some(tile);
    }
    let (w, h) = (width as f32, height as f32);
    let radius = if shape == 2 { w / 2. } else { w.min(h) * 0.12 };
    let edge = (w / 160.).clamp(1.5, 3.);
    for y in 0..height {
        for x in 0..width {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            // Signed distance to the rounded rectangle (a circle when radius = w/2).
            let qx = (px - w / 2.).abs() - (w / 2. - radius);
            let qy = (py - h / 2.).abs() - (h / 2. - radius);
            let outside = (qx.max(0.).powi(2) + qy.max(0.).powi(2)).sqrt() + qx.max(qy).min(0.);
            let distance = outside - radius;
            let alpha = (0.5 - distance).clamp(0., 1.);
            let border = (1. - (distance + edge).abs() / edge).clamp(0., 1.) * 0.75;
            let i = ((y * width + x) * 4) as usize;
            for c in 0..3 {
                let v = tile.bgra[i + c] as f32;
                tile.bgra[i + c] = (v + (245. - v) * border).round() as u8;
            }
            tile.bgra[i + 3] = (alpha * 255.).round() as u8;
        }
    }
    Some(tile)
}
fn overlay(frame: &mut Frame, tile: &Frame, position: usize) {
    let margin = (frame.width / 64).max(12);
    let x = if position % 2 == 0 {
        margin
    } else {
        frame.width.saturating_sub(tile.width + margin)
    };
    let y = if position < 2 {
        margin
    } else {
        frame.height.saturating_sub(tile.height + margin)
    };
    for yy in 0..tile.height.min(frame.height.saturating_sub(y)) {
        for xx in 0..tile.width.min(frame.width.saturating_sub(x)) {
            let src = ((yy * tile.width + xx) * 4) as usize;
            let dst = (((y + yy) * frame.width + x + xx) * 4) as usize;
            let a = tile.bgra[src + 3] as u32;
            if a == 0 {
                continue;
            }
            for c in 0..3 {
                frame.bgra[dst + c] =
                    ((tile.bgra[src + c] as u32 * a + frame.bgra[dst + c] as u32 * (255 - a) + 127)
                        / 255) as u8;
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn solid(width: u32, height: u32, bgra: [u8; 4]) -> Frame {
        Frame {
            width,
            height,
            bgra: bgra.repeat((width * height) as usize),
        }
    }
    #[test]
    fn camera_shapes_mask_corners_and_keep_center() {
        let cam = solid(64, 48, [10, 20, 30, 255]);
        let rect = camera_tile(&cam, 32, 0, false).unwrap();
        assert_eq!((rect.width, rect.height), (32, 24));
        assert!(rect.bgra.chunks_exact(4).all(|p| p[3] == 255));
        let circle = camera_tile(&cam, 40, 2, false).unwrap();
        assert_eq!((circle.width, circle.height), (40, 40));
        assert_eq!(circle.bgra[3], 0, "corner is transparent");
        let center = ((20 * 40 + 20) * 4) as usize;
        assert_eq!(&circle.bgra[center..center + 4], &[10, 20, 30, 255]);
        let rounded = camera_tile(&cam, 48, 1, false).unwrap();
        assert_eq!(rounded.bgra[3], 0);
        let mid = ((18 * 48 + 24) * 4) as usize;
        assert_eq!(rounded.bgra[mid + 3], 255);
    }
    #[test]
    fn mirrored_overlay_blends_inside_the_frame_bounds() {
        let mut cam = solid(8, 2, [0, 0, 0, 255]);
        cam.bgra[0..4].copy_from_slice(&[255, 255, 255, 255]);
        let flipped = camera_tile(&cam, 8, 0, true).unwrap();
        assert_eq!(&flipped.bgra[28..32], &[255, 255, 255, 255]);
        let mut frame = solid(320, 180, [50, 60, 70, 255]);
        let tile = camera_tile(&solid(64, 36, [200, 0, 0, 255]), 80, 1, false).unwrap();
        overlay(&mut frame, &tile, 3);
        assert!(frame.valid());
        assert_eq!(&frame.bgra[0..4], &[50, 60, 70, 255]);
        let inside = (((180 - 12 - 10) * 320 + (320 - 12 - 40)) * 4) as usize;
        assert_eq!(frame.bgra[inside], 200);
    }
}
