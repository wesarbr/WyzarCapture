//! Standalone PCM recording: no display, Direct3D or video encoder is opened.
use crate::{
    audio::{RATE, mix, wav::Writer},
    capture::Event,
    core::{AppResult, RecordingInfo},
    platform::windows::{self as win, wasapi::AudioCapture},
    settings::Settings,
};
use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};
pub fn record(
    settings: &Settings,
    path: &Path,
    stop: &AtomicBool,
    pause: &AtomicBool,
    mut emit: impl FnMut(Event),
) -> AppResult<()> {
    let _com = win::setup::Com::multithreaded()?;
    if !settings.microphone && !settings.system_audio {
        return Err(
            crate::i18n::tr("Ative o microfone ou o áudio do sistema para gravar áudio.").into(),
        );
    }
    let mut source = |enabled: bool, input: bool, id: &String| -> Option<AudioCapture> {
        if !enabled {
            return None;
        }
        match AudioCapture::start(input, id.clone()) {
            Ok(a) => Some(a),
            Err(e) => {
                emit(Event::Warning({
                    let __wyzar_i18n_0 = &(if input {
                        crate::i18n::tr("Microfone")
                    } else {
                        crate::i18n::tr("Áudio do sistema")
                    });
                    crate::i18n::format_text(
                        crate::i18n::tr("{} indisponível: {e}"),
                        &[
                            ("0", format!("{}", __wyzar_i18n_0)),
                            ("e", format!("{}", e)),
                        ],
                    )
                }));
                None
            }
        }
    };
    let mic = source(settings.microphone, true, &settings.microphone_id);
    let system = source(settings.system_audio, false, &settings.output_id);
    if mic.is_none() && system.is_none() {
        return Err(crate::i18n::tr(
            "Nenhuma fonte de áudio pôde ser aberta. Verifique os dispositivos e as permissões.",
        )
        .into());
    }
    win::storage::require(path, crate::core::storage::reserve(0, false, 0))?;
    let mut writer = Writer::new(path)?;
    let mut limiter = crate::audio::limiter::Limiter::new();
    let mut voice =
        (settings.voice_filter && mic.is_some()).then(crate::audio::voice::VoiceFilter::new);
    let mut mic_file = if settings.audio_sources && mic.is_some() {
        Some(Writer::new(&path.with_extension("mic.wav"))?)
    } else {
        None
    };
    let mut system_file = if settings.audio_sources && system.is_some() {
        Some(Writer::new(&path.with_extension("system.wav"))?)
    } else {
        None
    };
    if let Some(a) = &mic {
        a.clear();
    }
    if let Some(a) = &system {
        a.clear();
    }
    let clock = Instant::now();
    let mut paused = Duration::ZERO;
    let mut pause_start: Option<Instant> = None;
    let mut cursor = 0u64;
    let mut checkpoint = Instant::now();
    let mut progress = Instant::now();
    let mut warned = false;
    emit(Event::Started);
    loop {
        if pause.load(Ordering::Relaxed) {
            pause_start.get_or_insert_with(Instant::now);
            if let Some(a) = &mic {
                a.clear();
            }
            if let Some(a) = &system {
                a.clear();
            }
            if stop.load(Ordering::Relaxed) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
            continue;
        }
        if let Some(p) = pause_start.take() {
            paused += p.elapsed();
        }
        let end = (clock.elapsed().saturating_sub(paused).as_secs_f64() * RATE as f64) as u64;
        let frames = (end.saturating_sub(cursor)).min(4800) as usize;
        if frames > 0 {
            let mut a = mic
                .as_ref()
                .map(|a| a.take(frames * 2))
                .unwrap_or_else(|| vec![0.; frames * 2]);
            if let Some(filter) = voice.as_mut() {
                filter.process(&mut a);
            }
            let b = system
                .as_ref()
                .map(|a| a.take(frames * 2))
                .unwrap_or_else(|| vec![0.; frames * 2]);
            let mixed = if settings.audio_limiter {
                limiter.mix(
                    &a,
                    &b,
                    settings.mic_gain as f32 / 100.,
                    settings.system_gain as f32 / 100.,
                )
            } else {
                mix(
                    &a,
                    &b,
                    settings.mic_gain as f32 / 100.,
                    settings.system_gain as f32 / 100.,
                )
            };
            writer.write(&mixed)?;
            if let Some(w) = mic_file.as_mut() {
                w.write(&mix(&a, &[], 1., 0.))?;
            }
            if let Some(w) = system_file.as_mut() {
                w.write(&mix(&b, &[], 1., 0.))?;
            }
            cursor += frames as u64;
        }
        for (capture, name) in [
            (&mic, crate::i18n::tr("Microfone")),
            (&system, crate::i18n::tr("Áudio do sistema")),
        ] {
            if let Some(error) = capture.as_ref().and_then(AudioCapture::take_error) {
                emit(Event::Warning(crate::i18n::format_text(
                    crate::i18n::tr(
                        "{name} desconectado: {error}. Confira as fontes antes de continuar.",
                    ),
                    &[
                        ("name", format!("{}", name)),
                        ("error", format!("{}", error)),
                    ],
                )));
            }
        }
        if progress.elapsed().as_millis() >= 150 {
            emit(Event::Progress {
                seconds: cursor as f64 / RATE as f64,
                frames: 0,
                mic: mic.as_ref().map(AudioCapture::peak).unwrap_or(0.),
                system: system.as_ref().map(AudioCapture::peak).unwrap_or(0.),
            });
            progress = Instant::now();
        }
        if checkpoint.elapsed().as_secs() >= 1 {
            writer.finish()?;
            if let Some(w) = mic_file.as_mut() {
                w.finish()?;
            }
            if let Some(w) = system_file.as_mut() {
                w.finish()?;
            }
            if let Err(error) =
                win::storage::require(path, crate::core::storage::reserve(0, false, 0))
            {
                emit(Event::Warning(error));
                break;
            }
            if let Some((free, _)) =
                win::files::free_space(path.parent().ok_or(crate::i18n::tr("Pasta ausente"))?)
            {
                if free < 128 * 1024 * 1024 {
                    emit(Event::Warning(
                        crate::i18n::tr("Espaço crítico: o áudio foi finalizado com segurança")
                            .into(),
                    ));
                    break;
                }
                if !warned && free < settings.low_space_gb as u64 * 1_073_741_824 {
                    warned = true;
                    emit(Event::Warning(
                        crate::i18n::tr("Pouco espaço na pasta de áudio").into(),
                    ));
                }
            }
            checkpoint = Instant::now();
        }
        if stop.load(Ordering::Relaxed) && cursor >= end {
            break;
        }
        thread::sleep(Duration::from_millis(5));
    }
    writer.finish()?;
    if let Some(w) = mic_file.as_mut() {
        w.finish()?;
    }
    if let Some(w) = system_file.as_mut() {
        w.finish()?;
    }
    emit(Event::Saved(RecordingInfo {
        path: path.into(),
        duration: Duration::from_secs_f64(cursor as f64 / RATE as f64),
        frames: 0,
        width: 0,
        height: 0,
    }));
    Ok(())
}
