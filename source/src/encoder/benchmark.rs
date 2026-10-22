//! Optional five-second desktop sample, never an endurance certificate.
use crate::{
    capture::{Event, Session, metrics::Snapshot},
    core::{AppResult, Frame},
    platform::windows::{self as win, capture::Target, media::Reader},
    settings::Settings,
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
#[derive(Clone, Debug)]
pub struct Report {
    pub metrics: Snapshot,
    pub recommendation: String,
    pub path: PathBuf,
}
pub fn run(mut settings: Settings) -> AppResult<Report> {
    let _runtime = win::Runtime::new(true)?;
    let monitor = win::devices::monitors()
        .get(settings.monitor)
        .cloned()
        .ok_or(crate::i18n::tr("Monitor indisponível para o diagnóstico"))?;
    let root = settings
        .cache
        .join("PerformanceTests")
        .join(crate::settings::unique_stamp().to_string());
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    settings.data_dir = root.join("Data");
    settings.videos = root.clone();
    settings.audios = root.join("Audio");
    settings.screenshots = root.join("Pictures");
    settings.exports = root.join("Exports");
    settings.projects = root.join("Projects");
    settings.microphone = false;
    settings.system_audio = false;
    settings.webcam = false;
    settings.recording_mode = 1;
    settings.recovery = false;
    let path = root.join("performance.mp4");
    let report_path = root.join("performance-report.json");
    let result = (|| -> AppResult<Report> {
        let session = Session::start(
            settings.clone(),
            Target {
                monitor,
                window: None,
                region: None,
                cursor: false,
            },
            path.clone(),
            Arc::new(Mutex::new(None::<Frame>)),
        );
        let started = Instant::now();
        let mut clock = None;
        let mut saved = None;
        let mut metrics = None;
        while started.elapsed() < Duration::from_secs(25) {
            for event in session.events.try_iter() {
                match event {
                    Event::Started => clock = Some(Instant::now()),
                    Event::Metrics(m) => metrics = Some(m),
                    Event::Saved(info) => saved = Some(info),
                    Event::Error(e) => return Err(e),
                    _ => {}
                }
            }
            if saved.is_some() {
                break;
            }
            if clock.is_some_and(|c| c.elapsed() >= Duration::from_secs(5)) {
                session.stop();
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        if saved.is_none() {
            return Err(crate::i18n::tr("O teste de desempenho excedeu o tempo previsto").into());
        }
        drop(session);
        let m = metrics.ok_or(crate::i18n::tr("Métricas de desempenho indisponíveis"))?;
        let mut reader = Reader::open(&path)?;
        if reader.frame_at(reader.meta.duration / 2.)?.is_none() {
            return Err(crate::i18n::tr("Teste não decodificou vídeo").into());
        }
        let recommendation=if m.effective_fps() < settings.fps as f64*0.92 || m.missed_deadlines > m.encoded/10 || m.cpu_mean > 70. { crate::i18n::tr("Recomenda-se reduzir manualmente para 1080p30 ou 720p30. As preferências não foram alteradas.") } else { crate::i18n::tr("O perfil respondeu bem nesta amostra de 5 s. Monitore atrasos e espaço em gravações maiores.") }.to_string();
        let hash = win::setup::hash_file(&path)?;
        std::fs::write(&report_path,format!("{{\"passed\":true,\"sample_seconds\":5,\"long_stability_certified\":false,\"video_sha256\":{hash:?},\"recommendation\":{recommendation:?},\"metrics\":{}}}",m.json())).map_err(|e|e.to_string())?;
        Ok(Report {
            metrics: m,
            recommendation,
            path: report_path,
        })
    })();
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    result
}
