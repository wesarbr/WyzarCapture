use crate::{
    core::{AppResult, Frame},
    encoder::VideoConfig,
    platform::windows::media::{self, EncoderWorker as Writer},
    recovery::Journal,
    settings::Settings,
};
use std::path::{Path, PathBuf};

enum PreparedEvent {
    Closed(u32, u64),
    Ready(Writer),
    Error(String),
}
struct Preparation {
    receiver: std::sync::mpsc::Receiver<PreparedEvent>,
    thread: std::thread::JoinHandle<()>,
}
/// One active muxer, one prepared successor, at most one closing job. No unbounded queue.
pub struct RecordingSink {
    writer: Option<Writer>,
    journal: Option<Journal>,
    output: PathBuf,
    cfg: VideoConfig,
    audio: bool,
    segment_ticks: i64,
    index: u32,
    base: i64,
    end: i64,
    frames: u64,
    segment_frames: u64,
    prepared: Option<Writer>,
    preparation: Option<Preparation>,
}
impl RecordingSink {
    pub fn encoder(&self) -> String {
        self.writer
            .as_ref()
            .map(|w| w.encoder.clone())
            .unwrap_or_default()
    }
    pub fn new(settings: &Settings, path: &Path, audio: bool) -> AppResult<Self> {
        let cfg = VideoConfig::from_settings(settings);
        let journal = if settings.recovery {
            Some(Journal::begin(&settings.data_dir, path)?)
        } else {
            None
        };
        let destination = journal
            .as_ref()
            .map(|j| j.segment_path(0))
            .unwrap_or_else(|| path.into());
        let writer = match Writer::new(&destination, cfg, audio) {
            Ok(w) => w,
            Err(error) => {
                if let Some(journal) = journal {
                    let _ = journal.complete();
                } else {
                    let _ = std::fs::remove_file(&destination);
                }
                return Err(error);
            }
        };
        let mut sink = Self {
            writer: Some(writer),
            journal,
            output: path.into(),
            cfg,
            audio,
            segment_ticks: settings.segment_seconds as i64 * 10_000_000,
            index: 0,
            base: 0,
            end: 0,
            frames: 0,
            segment_frames: 0,
            prepared: None,
            preparation: None,
        };
        if sink.journal.is_some() {
            sink.prepare(None)?;
        }
        Ok(sink)
    }
    fn prepare(&mut self, closed: Option<(Writer, std::path::PathBuf, u32, u64)>) -> AppResult<()> {
        if self.preparation.is_some() {
            return Err(crate::i18n::tr("Preparação de segmento ainda ativa").into());
        }
        let next_path = self.journal.as_ref().unwrap().segment_path(self.index + 1);
        let cfg = self.cfg;
        let audio = self.audio;
        let (sender, receiver) = std::sync::mpsc::sync_channel(2);
        let thread = std::thread::spawn(move || {
            let result = (|| -> AppResult<()> {
                let _runtime = crate::platform::windows::Runtime::new(true)?;
                if let Some((writer, path, count, frames)) = closed {
                    writer.finalize()?;
                    std::fs::OpenOptions::new()
                        .write(true)
                        .open(path)
                        .and_then(|f| f.sync_all())
                        .map_err(|e| e.to_string())?;
                    sender
                        .send(PreparedEvent::Closed(count, frames))
                        .map_err(|e| e.to_string())?;
                }
                let writer = Writer::new(&next_path, cfg, audio)?;
                sender
                    .send(PreparedEvent::Ready(writer))
                    .map_err(|e| e.to_string())?;
                Ok(())
            })();
            if let Err(error) = result {
                let _ = sender.send(PreparedEvent::Error(error));
            }
        });
        self.preparation = Some(Preparation { receiver, thread });
        Ok(())
    }
    fn poll_preparation(&mut self, wait: bool) -> AppResult<()> {
        loop {
            let Some(job) = self.preparation.as_ref() else {
                return Ok(());
            };
            let event = if wait {
                Some(
                    job.receiver
                        .recv_timeout(std::time::Duration::from_secs(10))
                        .map_err(|e| {
                            crate::i18n::format_text(
                                crate::i18n::tr("Preparação de segmento: {e}"),
                                &[("e", format!("{}", e))],
                            )
                        })?,
                )
            } else {
                match job.receiver.try_recv() {
                    Ok(event) => Some(event),
                    Err(std::sync::mpsc::TryRecvError::Empty) => None,
                    Err(_) => {
                        return Err(crate::i18n::tr(
                            "O worker de segmento foi encerrado sem resultado",
                        )
                        .into());
                    }
                }
            };
            match event {
                None => return Ok(()),
                Some(PreparedEvent::Closed(count, frames)) => self
                    .journal
                    .as_mut()
                    .unwrap()
                    .commit_segment(count, frames)?,
                Some(PreparedEvent::Ready(writer)) => {
                    self.prepared = Some(writer);
                    let job = self.preparation.take().unwrap();
                    let _ = job.thread.join();
                    return Ok(());
                }
                Some(PreparedEvent::Error(error)) => {
                    let job = self.preparation.take().unwrap();
                    let _ = job.thread.join();
                    return Err(error);
                }
            }
        }
    }
    pub fn video(&mut self, frame: &Frame, time: i64, duration: i64) -> AppResult<()> {
        self.poll_preparation(false)?;
        if self.journal.is_some()
            && self.segment_frames > 0
            && time - self.base >= self.segment_ticks
        {
            self.poll_preparation(true)?;
            let old = self.writer.take().ok_or(crate::i18n::tr("Muxer ausente"))?;
            let path = self.journal.as_ref().unwrap().segment_path(self.index);
            self.index += 1;
            self.writer = self.prepared.take();
            if self.writer.is_none() {
                return Err(crate::i18n::tr("O próximo segmento não ficou pronto").into());
            }
            self.prepare(Some((old, path, self.index, self.frames)))?;
            self.base = self.end;
            self.segment_frames = 0;
        }
        let relative = (time - self.base).max(0);
        // Hold the first image over any muxer-start gap instead of removing wall time.
        let first = self.segment_frames == 0;
        self.writer.as_ref().unwrap().video(
            frame,
            if first { 0 } else { relative },
            if first { relative + duration } else { duration },
        )?;
        self.end = time + duration;
        self.frames += 1;
        self.segment_frames += 1;
        Ok(())
    }
    pub fn audio(&self, samples: &[i16], time: i64) -> AppResult<()> {
        if !self.audio {
            return Ok(());
        }
        self.writer
            .as_ref()
            .unwrap()
            .audio(samples, (time - self.base).max(0))
    }
    fn close_segment(&mut self) -> AppResult<()> {
        self.poll_preparation(true)?;
        drop(self.prepared.take());
        self.writer
            .take()
            .ok_or(crate::i18n::tr("Muxer ausente"))?
            .finalize()?;
        if let Some(journal) = &mut self.journal {
            let path = journal.segment_path(self.index);
            std::fs::OpenOptions::new()
                .write(true)
                .open(&path)
                .and_then(|f| f.sync_all())
                .map_err(|e| e.to_string())?;
            self.index += 1;
            journal.commit_segment(self.index, self.frames)?;
        }
        Ok(())
    }
    pub fn bytes(&self) -> u64 {
        if let Some(j) = &self.journal {
            (0..=self.index)
                .map(|i| j.segment_path(i).metadata().map(|m| m.len()).unwrap_or(0))
                .sum()
        } else {
            self.output.metadata().map(|m| m.len()).unwrap_or(0)
        }
    }
    pub fn checkpoint(&mut self, frames: u64) -> AppResult<()> {
        self.poll_preparation(false)?;
        if let Some(journal) = self.journal.as_mut() {
            journal.checkpoint(frames)?;
        }
        Ok(())
    }
    pub fn finish(mut self) -> AppResult<f64> {
        self.close_segment()?;
        if self.journal.is_some() {
            crate::platform::windows::storage::require(
                &self.output,
                crate::core::storage::finalize_reserve(self.bytes()),
            )?;
        }
        if let Some(journal) = self.journal.take() {
            media::concatenate(&journal.segments(), &self.output)?;
            let mut reader = media::Reader::open(&self.output)?;
            if reader.frame_at(0.)?.is_none()
                || reader
                    .frame_at((reader.meta.duration - 0.1).max(0.))?
                    .is_none()
            {
                return Err(crate::i18n::tr(
                    "A montagem não passou na decodificação. Os segmentos foram preservados.",
                )
                .into());
            }
            let duration = reader.meta.duration;
            // Once the final output is valid, cleanup failure must not hide it from the user.
            let _ = journal.complete();
            Ok(duration)
        } else {
            Ok(self.end as f64 / 10_000_000.)
        }
    }
    pub fn statistics(&self) -> Option<media::WriterStatistics> {
        self.writer.as_ref()?.statistics()
    }
}
impl Drop for RecordingSink {
    fn drop(&mut self) {
        // Join the owned worker and commit closed segments even after a capture error.
        let _ = self.poll_preparation(true);
        if let Some(job) = self.preparation.take() {
            drop(job.receiver);
            let _ = job.thread.join();
        }
        drop(self.prepared.take());
        // A controlled error may still allow finalization of the current segment.
        // Commit it only after success and decoding, preserving the contiguous prefix.
        if self.segment_frames > 0
            && self
                .journal
                .as_ref()
                .is_some_and(|j| j.committed_count() == self.index)
        {
            if let Some(writer) = self.writer.take() {
                if writer.finalize().is_ok() {
                    let journal = self.journal.as_mut().unwrap();
                    let path = journal.segment_path(self.index);
                    let synced = std::fs::OpenOptions::new()
                        .write(true)
                        .open(&path)
                        .and_then(|f| f.sync_all())
                        .is_ok();
                    if synced
                        && media::Reader::open(&path)
                            .and_then(|mut r| r.frame_at(0.))
                            .is_ok_and(|f| f.is_some())
                    {
                        let _ = journal.commit_segment(self.index + 1, self.frames);
                    }
                }
            }
        }
    }
}
