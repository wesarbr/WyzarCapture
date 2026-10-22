use super::{pcw, wide};
use crate::{
    core::{AppResult, Frame},
    encoder::VideoConfig,
};
use std::path::Path;
use windows::{
    Win32::{
        Media::MediaFoundation::*,
        System::Com::StructuredStorage::PropVariantToInt64,
        System::{Com::StructuredStorage::PROPVARIANT, Com::StructuredStorage::PropVariantClear},
    },
    core::{GUID, Interface},
};
const VIDEO: u32 = MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32;
const AUDIO: u32 = MF_SOURCE_READER_FIRST_AUDIO_STREAM.0 as u32;
pub unsafe fn attributes() -> windows::core::Result<IMFAttributes> {
    let mut a = None;
    MFCreateAttributes(&mut a, 8)?;
    Ok(a.unwrap())
}
unsafe fn video_type(cfg: VideoConfig, subtype: &GUID) -> windows::core::Result<IMFMediaType> {
    let t = MFCreateMediaType()?;
    t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
    t.SetGUID(&MF_MT_SUBTYPE, subtype)?;
    t.SetUINT64(
        &MF_MT_FRAME_SIZE,
        ((cfg.width as u64) << 32) | cfg.height as u64,
    )?;
    t.SetUINT64(&MF_MT_FRAME_RATE, ((cfg.fps as u64) << 32) | 1)?;
    t.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, (1u64 << 32) | 1)?;
    t.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)?;
    Ok(t)
}
unsafe fn pcm_type() -> windows::core::Result<IMFMediaType> {
    let t = MFCreateMediaType()?;
    t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio)?;
    t.SetGUID(&MF_MT_SUBTYPE, &MFAudioFormat_PCM)?;
    t.SetUINT32(&MF_MT_AUDIO_NUM_CHANNELS, 2)?;
    t.SetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND, 48000)?;
    t.SetUINT32(&MF_MT_AUDIO_BITS_PER_SAMPLE, 16)?;
    t.SetUINT32(&MF_MT_AUDIO_BLOCK_ALIGNMENT, 4)?;
    t.SetUINT32(&MF_MT_AUDIO_AVG_BYTES_PER_SECOND, 192000)?;
    t.SetUINT32(&MF_MT_ALL_SAMPLES_INDEPENDENT, 1)?;
    Ok(t)
}
unsafe fn encoding_parameters(cfg: VideoConfig) -> windows::core::Result<IMFAttributes> {
    let params = attributes()?;
    params.SetUINT32(
        &CODECAPI_AVEncCommonRateControlMode,
        eAVEncCommonRateControlMode_PeakConstrainedVBR.0 as u32,
    )?;
    params.SetUINT32(&CODECAPI_AVEncCommonMeanBitRate, cfg.bitrate)?;
    params.SetUINT32(
        &CODECAPI_AVEncCommonMaxBitRate,
        cfg.bitrate.saturating_add(cfg.bitrate / 2),
    )?;
    params.SetUINT32(&CODECAPI_AVEncMPVGOPSize, cfg.fps.max(1) * 2)?;
    params.SetUINT32(&CODECAPI_AVLowLatencyMode, 0)?;
    params.SetUINT32(&CODECAPI_AVEncCommonQualityVsSpeed, 70)?;
    Ok(params)
}
pub struct Writer {
    inner: IMFSinkWriter,
    video: u32,
    audio: Option<u32>,
    pub cfg: VideoConfig,
    pub encoder: String,
    finalized: bool,
}
impl Writer {
    pub fn new(path: &Path, mut cfg: VideoConfig, audio: bool) -> AppResult<Self> {
        let force_hardware = std::env::var("FORCE_HARDWARE_ENCODER").as_deref() == Ok("1");
        let force_software = std::env::var("FORCE_SOFTWARE_ENCODER").as_deref() == Ok("1");
        if force_hardware && force_software {
            return Err(crate::i18n::tr("Escolha somente um override de encoder").into());
        }
        if force_software {
            cfg.hardware = false;
        }
        if force_hardware {
            if !super::capabilities::inventory()?.hardware() {
                return Err(crate::i18n::tr(
                    "Nenhum MFT H.264 de hardware foi enumerado neste computador",
                )
                .into());
            }
            cfg.hardware = true;
        }
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| {
                crate::i18n::format_text(
                    crate::i18n::tr(
                        "Não foi possível reservar o arquivo de saída sem sobrescrever dados: {e}",
                    ),
                    &[("e", format!("{}", e))],
                )
            })?;
        let result = unsafe {
            Self::create(path, cfg, audio).map_err(|e| {
                crate::i18n::format_text(
                    crate::i18n::tr("Encoder H.264/AAC: {e}"),
                    &[("e", format!("{}", e))],
                )
            })
        };
        if result.is_err() {
            let _ = std::fs::remove_file(path);
        }
        result
    }
    unsafe fn create(path: &Path, cfg: VideoConfig, audio: bool) -> windows::core::Result<Self> {
        let attrs = attributes()?;
        attrs.SetUINT32(
            &MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS,
            cfg.hardware as u32,
        )?;
        attrs.SetUINT32(&MF_SINK_WRITER_DISABLE_THROTTLING, 1)?;
        let writer = MFCreateSinkWriterFromURL(pcw(&wide(path)), None, &attrs)?;
        let output = video_type(cfg, &MFVideoFormat_H264)?;
        output.SetUINT32(&MF_MT_AVG_BITRATE, cfg.bitrate)?;
        output.SetUINT32(&MF_MT_MPEG2_PROFILE, eAVEncH264VProfile_High.0 as u32)?;
        output.SetUINT32(&MF_MT_VIDEO_PRIMARIES, MFVideoPrimaries_BT709.0 as u32)?;
        output.SetUINT32(&MF_MT_TRANSFER_FUNCTION, MFVideoTransFunc_709.0 as u32)?;
        output.SetUINT32(&MF_MT_YUV_MATRIX, MFVideoTransferMatrix_BT709.0 as u32)?;
        output.SetUINT32(&MF_MT_VIDEO_NOMINAL_RANGE, MFNominalRange_16_235.0 as u32)?;
        let video = writer.AddStream(&output)?;
        let input = video_type(cfg, &MFVideoFormat_NV12)?;
        input.SetUINT32(&MF_MT_DEFAULT_STRIDE, cfg.width)?;
        input.SetUINT32(&MF_MT_YUV_MATRIX, MFVideoTransferMatrix_BT709.0 as u32)?;
        input.SetUINT32(&MF_MT_VIDEO_NOMINAL_RANGE, MFNominalRange_16_235.0 as u32)?;
        input.SetUINT32(&MF_MT_VIDEO_PRIMARIES, MFVideoPrimaries_BT709.0 as u32)?;
        input.SetUINT32(&MF_MT_TRANSFER_FUNCTION, MFVideoTransFunc_709.0 as u32)?;
        // Peak-constrained VBR spends bits on motion and few on static screens, with a
        // two-second keyframe interval for seeking. Encoders that refuse any of these
        // values are configured again with the plain average bitrate.
        let tuned = encoding_parameters(cfg)
            .ok()
            .is_some_and(|params| writer.SetInputMediaType(video, &input, &params).is_ok());
        if !tuned {
            writer.SetInputMediaType(video, &input, None)?;
        }
        let audio = if audio {
            let out = MFCreateMediaType()?;
            out.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio)?;
            out.SetGUID(&MF_MT_SUBTYPE, &MFAudioFormat_AAC)?;
            out.SetUINT32(&MF_MT_AUDIO_NUM_CHANNELS, 2)?;
            out.SetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND, 48000)?;
            out.SetUINT32(&MF_MT_AUDIO_BITS_PER_SAMPLE, 16)?;
            out.SetUINT32(&MF_MT_AUDIO_AVG_BYTES_PER_SECOND, 24000)?;
            let stream = writer.AddStream(&out)?;
            writer.SetInputMediaType(stream, &pcm_type()?, None)?;
            Some(stream)
        } else {
            None
        };
        writer.BeginWriting()?;
        let (selected, hardware) = super::capabilities::selected(&writer, video);
        if std::env::var("FORCE_HARDWARE_ENCODER").as_deref() == Ok("1") && !hardware {
            let _ = writer.Finalize();
            return Err(windows::core::Error::new(
                windows::core::HRESULT(0x80004005u32 as i32),
                crate::i18n::tr(
                    "Hardware enumerado, mas o Sink Writer não confirmou uma transformação de hardware. Teste forçado recusado.",
                ),
            ));
        }
        let mut encoder: String = if cfg.hardware {
            selected
        } else {
            crate::i18n::tr("Software · Media Foundation").into()
        };
        if tuned {
            encoder.push_str(" · VBR");
        }
        Ok(Self {
            inner: writer,
            video,
            audio,
            cfg,
            encoder,
            finalized: false,
        })
    }
    pub fn video(&self, frame: &Frame, time: i64, duration: i64) -> AppResult<()> {
        if frame.width != self.cfg.width || frame.height != self.cfg.height || !frame.valid() {
            return Err(crate::i18n::tr("Dimensão de quadro inválida").into());
        }
        let started = std::time::Instant::now();
        while self
            .statistics()
            .is_some_and(|s| s.queued_bytes > 64 * 1024 * 1024)
        {
            if started.elapsed() > std::time::Duration::from_secs(5) {
                return Err(crate::i18n::tr("O encoder não esvaziou a fila em tempo seguro. Os segmentos anteriores foram preservados.").into());
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        unsafe {
            let nv12 = crate::encoder::color::nv12(frame)?;
            self.sample(self.video, &nv12, time, duration).map_err(|e| {
                crate::i18n::format_text(
                    crate::i18n::tr("Gravação do vídeo: {e}"),
                    &[("e", format!("{}", e))],
                )
            })
        }
    }
    pub fn audio(&self, samples: &[i16], time: i64) -> AppResult<()> {
        if let Some(stream) = self.audio {
            let mut bytes = Vec::with_capacity(samples.len() * 2);
            for s in samples {
                bytes.extend(s.to_le_bytes());
            }
            unsafe {
                self.sample(
                    stream,
                    &bytes,
                    time,
                    (samples.len() as i64 / 2) * 10_000_000 / 48000,
                )
                .map_err(|e| {
                    crate::i18n::format_text(
                        crate::i18n::tr("Gravação do áudio: {e}"),
                        &[("e", format!("{}", e))],
                    )
                })?;
            }
        }
        Ok(())
    }
    unsafe fn sample(
        &self,
        stream: u32,
        data: &[u8],
        time: i64,
        duration: i64,
    ) -> windows::core::Result<()> {
        let buffer = MFCreateMemoryBuffer(data.len() as u32)?;
        let mut p = std::ptr::null_mut();
        buffer.Lock(&mut p, None, None)?;
        std::ptr::copy_nonoverlapping(data.as_ptr(), p, data.len());
        buffer.Unlock()?;
        buffer.SetCurrentLength(data.len() as u32)?;
        let sample = MFCreateSample()?;
        sample.AddBuffer(&buffer)?;
        sample.SetSampleTime(time)?;
        sample.SetSampleDuration(duration)?;
        self.inner.WriteSample(stream, &sample)
    }
    pub fn finalize(mut self) -> AppResult<()> {
        let result = unsafe {
            self.inner.Finalize().map_err(|e| {
                crate::i18n::format_text(
                    crate::i18n::tr("Finalização do MP4: {e}"),
                    &[("e", format!("{}", e))],
                )
            })
        };
        self.finalized = true;
        result
    }
    pub fn statistics(&self) -> Option<WriterStatistics> {
        unsafe {
            let mut stats = MF_SINK_WRITER_STATISTICS {
                cb: std::mem::size_of::<MF_SINK_WRITER_STATISTICS>() as u32,
                ..Default::default()
            };
            self.inner.GetStatistics(self.video, &mut stats).ok()?;
            Some(WriterStatistics {
                queued_bytes: stats.dwByteCountQueued as u64,
                encoded: stats.qwNumSamplesEncoded,
                encoding_delay_ms: stats
                    .llLastTimestampReceived
                    .saturating_sub(stats.llLastTimestampEncoded)
                    .max(0) as f64
                    / 10_000.,
            })
        }
    }
}
#[derive(Clone, Copy, Default)]
pub struct WriterStatistics {
    pub queued_bytes: u64,
    pub encoded: u64,
    pub encoding_delay_ms: f64,
}
enum EncodeCommand {
    Video(Frame, i64, i64),
    Audio(Vec<i16>, i64),
    Finalize,
}
/// COM objects never cross threads. Only owned pixels/PCM, commands and numeric
/// statistics cross a bounded queue; each encoder initializes its own MTA runtime.
pub struct EncoderWorker {
    pub encoder: String,
    sender: Option<std::sync::mpsc::SyncSender<EncodeCommand>>,
    thread: Option<std::thread::JoinHandle<AppResult<()>>>,
    stats: std::sync::Arc<std::sync::Mutex<WriterStatistics>>,
    error: std::sync::Arc<std::sync::Mutex<Option<String>>>,
}
impl EncoderWorker {
    pub fn new(path: &Path, cfg: VideoConfig, audio: bool) -> AppResult<Self> {
        let path = path.to_path_buf();
        let (sender, receiver) = std::sync::mpsc::sync_channel(4);
        let (ready, started) = std::sync::mpsc::sync_channel(1);
        let stats = std::sync::Arc::new(std::sync::Mutex::new(WriterStatistics::default()));
        let shared = stats.clone();
        let error = std::sync::Arc::new(std::sync::Mutex::new(None));
        let last_error = error.clone();
        let thread = std::thread::spawn(move || {
            let result = (|| -> AppResult<()> {
                let _runtime = super::Runtime::new(true)?;
                let writer = Writer::new(&path, cfg, audio)?;
                let _ = ready.send(Ok(writer.encoder.clone()));
                while let Ok(command) = receiver.recv() {
                    match command {
                        EncodeCommand::Video(frame, time, duration) => {
                            writer.video(&frame, time, duration)?
                        }
                        EncodeCommand::Audio(samples, time) => writer.audio(&samples, time)?,
                        EncodeCommand::Finalize => break,
                    }
                    if let Some(statistics) = writer.statistics() {
                        if let Ok(mut value) = shared.lock() {
                            *value = statistics;
                        }
                    }
                }
                writer.finalize()
            })();
            if let Err(message) = &result {
                if let Ok(mut error) = last_error.lock() {
                    *error = Some(message.clone());
                }
                let _ = ready.send(Err(message.clone()));
            }
            result
        });
        match started.recv_timeout(std::time::Duration::from_secs(10)) {
            Ok(Ok(encoder)) => Ok(Self {
                encoder,
                sender: Some(sender),
                thread: Some(thread),
                stats,
                error,
            }),
            result => {
                drop(sender);
                let _ = thread.join();
                Err(result.ok().and_then(Result::err).unwrap_or_else(|| {
                    crate::i18n::tr("O encoder não respondeu à inicialização").into()
                }))
            }
        }
    }
    fn send(&self, command: EncodeCommand) -> AppResult<()> {
        if let Some(error) = self.error.lock().map_err(|e| e.to_string())?.clone() {
            return Err(error);
        }
        self.sender
            .as_ref()
            .ok_or(crate::i18n::tr("Encoder encerrado"))?
            .send(command)
            .map_err(|_| {
                self.error
                    .lock()
                    .ok()
                    .and_then(|e| e.clone())
                    .unwrap_or_else(|| crate::i18n::tr("Encoder encerrado antes de salvar").into())
            })
    }
    pub fn video(&self, frame: &Frame, time: i64, duration: i64) -> AppResult<()> {
        self.send(EncodeCommand::Video(frame.clone(), time, duration))
    }
    pub fn audio(&self, samples: &[i16], time: i64) -> AppResult<()> {
        self.send(EncodeCommand::Audio(samples.to_vec(), time))
    }
    pub fn statistics(&self) -> Option<WriterStatistics> {
        self.stats.lock().ok().map(|s| *s)
    }
    pub fn finalize(mut self) -> AppResult<()> {
        let sent = self.send(EncodeCommand::Finalize);
        self.sender = None;
        let result = self
            .thread
            .take()
            .unwrap()
            .join()
            .map_err(|_| crate::i18n::tr("O worker de encoder foi interrompido").to_string())?;
        result.and(sent)
    }
}
impl Drop for EncoderWorker {
    fn drop(&mut self) {
        self.sender = None;
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
impl Drop for Writer {
    fn drop(&mut self) {
        if !self.finalized {
            unsafe {
                let _ = self.inner.Finalize();
            }
        }
    }
}
/// Remux closed, homogeneous Wyzar segments. No decode/re-encode and no external codec.
/// The destination is exclusively reserved; an incomplete owned output is removed on failure.
pub fn concatenate(paths: &[std::path::PathBuf], output: &Path) -> AppResult<f64> {
    if paths.is_empty() {
        return Err(crate::i18n::tr("Não há segmentos finalizados para recuperar").into());
    }
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|e| {
            crate::i18n::format_text(
                crate::i18n::tr(
                    "Não foi possível reservar a recuperação sem sobrescrever arquivos: {e}",
                ),
                &[("e", format!("{}", e))],
            )
        })?;
    let result = unsafe {
        concatenate_inner(paths, output).map_err(|e| {
            crate::i18n::format_text(
                crate::i18n::tr("Montagem dos segmentos: {e}"),
                &[("e", format!("{}", e))],
            )
        })
    };
    if result.is_err() {
        let _ = std::fs::remove_file(output);
    }
    result
}
unsafe fn concatenate_inner(
    paths: &[std::path::PathBuf],
    output: &Path,
) -> windows::core::Result<f64> {
    fn stage(name: &str, error: windows::core::Error) -> windows::core::Error {
        windows::core::Error::new(
            error.code(),
            crate::i18n::format_text(
                crate::i18n::tr("{name}: {error}"),
                &[
                    ("name", format!("{}", name)),
                    ("error", format!("{}", error)),
                ],
            ),
        )
    }
    let attrs = attributes()?;
    attrs.SetUINT32(&MF_SINK_WRITER_DISABLE_THROTTLING, 1)?;
    let writer = MFCreateSinkWriterFromURL(pcw(&wide(output)), None, &attrs)
        .map_err(|e| stage("CreateSink", e))?;
    let mut streams: Vec<(u32, u32)> = vec![];
    let mut offset = 0i64;
    let mut expected_size = 0;
    let mut expected_audio = false;
    for (index, path) in paths.iter().enumerate() {
        let reader = MFCreateSourceReaderFromURL(pcw(&wide(path)), None)
            .map_err(|e| stage("OpenSegment", e))?;
        let native = reader.GetNativeMediaType(VIDEO, 0)?;
        reader
            .SetCurrentMediaType(VIDEO, None, &native)
            .map_err(|e| stage("CompressedVideoOutput", e))?;
        let size = native.GetUINT64(&MF_MT_FRAME_SIZE)?;
        let audio_type = reader.GetNativeMediaType(AUDIO, 0).ok();
        if let Some(audio) = &audio_type {
            reader
                .SetCurrentMediaType(AUDIO, None, audio)
                .map_err(|e| stage("CompressedAudioOutput", e))?;
        }
        if native.GetGUID(&MF_MT_SUBTYPE)? != MFVideoFormat_H264
            || (index > 0 && (size != expected_size || audio_type.is_some() != expected_audio))
        {
            return Err(windows::core::Error::new(
                windows::Win32::Foundation::E_INVALIDARG,
                crate::i18n::tr("Segmentos incompatíveis"),
            ));
        }
        if index == 0 {
            expected_size = size;
            expected_audio = audio_type.is_some();
            let video_out = writer
                .AddStream(&native)
                .map_err(|e| stage("AddVideo", e))?;
            writer
                .SetInputMediaType(video_out, &native, None)
                .map_err(|e| stage("VideoInput", e))?;
            let video_in = (0..32)
                .find(|i| {
                    reader
                        .GetNativeMediaType(*i, 0)
                        .and_then(|t| t.GetGUID(&MF_MT_MAJOR_TYPE))
                        .is_ok_and(|g| g == MFMediaType_Video)
                })
                .ok_or_else(|| {
                    windows::core::Error::new(
                        windows::Win32::Foundation::E_INVALIDARG,
                        crate::i18n::tr("Trilha de vídeo ausente"),
                    )
                })?;
            streams.push((video_in, video_out));
            if let Some(audio) = &audio_type {
                let audio_out = writer.AddStream(audio).map_err(|e| stage("AddAudio", e))?;
                writer
                    .SetInputMediaType(audio_out, audio, None)
                    .map_err(|e| stage("AudioInput", e))?;
                let audio_in = (0..32)
                    .find(|i| {
                        reader
                            .GetNativeMediaType(*i, 0)
                            .and_then(|t| t.GetGUID(&MF_MT_MAJOR_TYPE))
                            .is_ok_and(|g| g == MFMediaType_Audio)
                    })
                    .ok_or_else(|| {
                        windows::core::Error::new(
                            windows::Win32::Foundation::E_INVALIDARG,
                            crate::i18n::tr("Trilha de áudio ausente"),
                        )
                    })?;
                streams.push((audio_in, audio_out));
            }
            writer.BeginWriting().map_err(|e| stage("BeginRemux", e))?;
        }
        reader.SetStreamSelection(MF_SOURCE_READER_ALL_STREAMS.0 as u32, true)?;
        let mut ended = vec![false; streams.len()];
        let mut first = vec![true; streams.len()];
        let mut end = offset;
        while ended.iter().any(|v| !v) {
            let (mut sample, mut flags, mut stream, mut time) = (None, 0, 0, 0);
            reader
                .ReadSample(
                    MF_SOURCE_READER_ANY_STREAM.0 as u32,
                    0,
                    Some(&mut stream),
                    Some(&mut flags),
                    Some(&mut time),
                    Some(&mut sample),
                )
                .map_err(|e| stage("ReadCompressed", e))?;
            let Some(position) = streams.iter().position(|(input, _)| *input == stream) else {
                continue;
            };
            if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
                ended[position] = true;
            }
            if let Some(sample) = sample {
                let time = time.max(0) + offset;
                sample.SetSampleTime(time)?;
                if let Ok(decode) = sample.GetUINT64(&MFSampleExtension_DecodeTimestamp) {
                    sample.SetUINT64(
                        &MFSampleExtension_DecodeTimestamp,
                        (decode as i64 + offset).max(0) as u64,
                    )?;
                }
                if first[position] {
                    sample.SetUINT32(&MFSampleExtension_Discontinuity, 1)?;
                    first[position] = false;
                }
                end = end.max(time + sample.GetSampleDuration().unwrap_or(1));
                writer
                    .WriteSample(streams[position].1, &sample)
                    .map_err(|e| stage(&{ let __wyzar_i18n_0 = &(streams[position].1); crate::i18n::format_text(crate::i18n::tr("WriteCompressed segment={index} stream={stream} target={} time={time}"), &[("index", format!("{}", index)), ("stream", format!("{}", stream)), ("0", format!("{}", __wyzar_i18n_0)), ("time", format!("{}", time))]) }, e))?;
            }
        }
        offset = end;
    }
    writer.Finalize().map_err(|e| stage("FinalizeRemux", e))?;
    Ok(offset as f64 / 10_000_000.)
}
#[derive(Clone, Debug)]
pub struct Metadata {
    pub duration: f64,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub audio: bool,
}
#[derive(Clone, Debug, Default)]
pub struct StreamTiming {
    pub video_samples: u64,
    pub audio_samples: u64,
    pub video_end: f64,
    pub audio_end: f64,
}
pub fn stream_timing(path: &Path) -> AppResult<StreamTiming> {
    unsafe {
        (|| -> windows::core::Result<StreamTiming> {
            let reader = MFCreateSourceReaderFromURL(pcw(&wide(path)), None)?;
            let mut kinds = std::collections::BTreeMap::new();
            reader.SetStreamSelection(MF_SOURCE_READER_ALL_STREAMS.0 as u32, false)?;
            for index in 0..32 {
                if let Ok(media_type) = reader.GetNativeMediaType(index, 0) {
                    let major = media_type.GetGUID(&MF_MT_MAJOR_TYPE)?;
                    if major == MFMediaType_Video || major == MFMediaType_Audio {
                        reader.SetCurrentMediaType(index, None, &media_type)?;
                        reader.SetStreamSelection(index, true)?;
                        kinds.insert(index, major == MFMediaType_Video);
                    }
                }
            }
            let mut ended = std::collections::BTreeSet::new();
            let mut timing = StreamTiming::default();
            while ended.len() < kinds.len() {
                let (mut sample, mut flags, mut stream, mut time) = (None, 0, 0, 0);
                reader.ReadSample(
                    MF_SOURCE_READER_ANY_STREAM.0 as u32,
                    0,
                    Some(&mut stream),
                    Some(&mut flags),
                    Some(&mut time),
                    Some(&mut sample),
                )?;
                if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
                    ended.insert(stream);
                }
                if let Some(sample) = sample {
                    let end = (time + sample.GetSampleDuration().unwrap_or(0)) as f64 / 10_000_000.;
                    if kinds.get(&stream) == Some(&true) {
                        timing.video_samples += 1;
                        timing.video_end = timing.video_end.max(end);
                    } else if kinds.contains_key(&stream) {
                        timing.audio_samples += 1;
                        timing.audio_end = timing.audio_end.max(end);
                    }
                }
            }
            Ok(timing)
        })()
        .map_err(|e| {
            crate::i18n::format_text(
                crate::i18n::tr("Inspeção dos timestamps: {e}"),
                &[("e", format!("{}", e))],
            )
        })
    }
}
pub struct Reader {
    reader: IMFSourceReader,
    pub meta: Metadata,
    stride: i32,
}
impl Reader {
    pub fn open(path: &Path) -> AppResult<Self> {
        unsafe {
            Self::create_with_processing(path, true)
                .or_else(|_| Self::create_with_processing(path, false))
                .map_err(|e| {
                    let __wyzar_i18n_0 = &(path.display());
                    crate::i18n::format_text(
                        crate::i18n::tr("Leitura de {}: {e}"),
                        &[
                            ("0", format!("{}", __wyzar_i18n_0)),
                            ("e", format!("{}", e)),
                        ],
                    )
                })
        }
    }
    unsafe fn create_with_processing(path: &Path, advanced: bool) -> windows::core::Result<Self> {
        let a = attributes()?;
        a.SetUINT32(
            if advanced {
                &MF_SOURCE_READER_ENABLE_ADVANCED_VIDEO_PROCESSING
            } else {
                &MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING
            },
            1,
        )?;
        let reader = MFCreateSourceReaderFromURL(pcw(&wide(path)), &a)?;
        reader.SetStreamSelection(MF_SOURCE_READER_ALL_STREAMS.0 as u32, false)?;
        reader.SetStreamSelection(VIDEO, true)?;
        let t = MFCreateMediaType()?;
        t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
        t.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32)?;
        reader.SetCurrentMediaType(VIDEO, None, &t)?;
        let t = reader.GetCurrentMediaType(VIDEO)?;
        let size = t.GetUINT64(&MF_MT_FRAME_SIZE)?;
        let ratio = t.GetUINT64(&MF_MT_FRAME_RATE).unwrap_or(30u64 << 32 | 1);
        let mut duration = reader
            .GetPresentationAttribute(MF_SOURCE_READER_MEDIASOURCE.0 as u32, &MF_PD_DURATION)?;
        let ticks = PropVariantToInt64(&duration).unwrap_or(0);
        let _ = PropVariantClear(&mut duration);
        let audio = reader.GetNativeMediaType(AUDIO, 0).is_ok();
        let w = (size >> 32) as u32;
        let h = size as u32;
        if w == 0 || h == 0 || w > 16384 || h > 16384 {
            return Err(windows::core::Error::new(
                windows::Win32::Foundation::E_INVALIDARG,
                crate::i18n::tr("Dimensões de vídeo inválidas"),
            ));
        }
        let stride = t.GetUINT32(&MF_MT_DEFAULT_STRIDE).unwrap_or(w * 4) as i32;
        Ok(Self {
            reader,
            meta: Metadata {
                duration: ticks as f64 / 10_000_000.,
                width: w,
                height: h,
                fps: (ratio >> 32) as f64 / (ratio as u32).max(1) as f64,
                audio,
            },
            stride,
        })
    }
    pub fn seek(&self, seconds: f64) -> AppResult<()> {
        unsafe {
            let pos = PROPVARIANT::from((seconds.max(0.) * 10_000_000.) as i64);
            self.reader
                .SetCurrentPosition(&GUID::zeroed(), &pos)
                .map_err(|e| e.to_string())
        }
    }
    pub fn next(&mut self) -> AppResult<Option<(Frame, f64)>> {
        unsafe { self.read().map_err(|e| e.to_string()) }
    }
    unsafe fn read(&mut self) -> windows::core::Result<Option<(Frame, f64)>> {
        for _ in 0..20 {
            let (mut sample, mut flags, mut time) = (None, 0, 0);
            self.reader.ReadSample(
                VIDEO,
                0,
                None,
                Some(&mut flags),
                Some(&mut time),
                Some(&mut sample),
            )?;
            if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
                return Ok(None);
            }
            let Some(sample) = sample else {
                continue;
            };
            let buffer = sample.ConvertToContiguousBuffer()?;
            let (mut data, mut len) = (std::ptr::null_mut(), 0);
            buffer.Lock(&mut data, None, Some(&mut len))?;
            let (w, h) = (self.meta.width, self.meta.height);
            let stride = self.stride.unsigned_abs() as usize;
            let mut bgra = vec![0; (w * h * 4) as usize];
            if len as usize >= stride * h as usize {
                for y in 0..h as usize {
                    let src_y = if self.stride < 0 {
                        h as usize - 1 - y
                    } else {
                        y
                    };
                    std::ptr::copy_nonoverlapping(
                        data.add(src_y * stride),
                        bgra.as_mut_ptr().add(y * w as usize * 4),
                        w as usize * 4,
                    );
                }
            }
            buffer.Unlock()?;
            return Ok(Some((
                Frame {
                    width: w,
                    height: h,
                    bgra,
                },
                time as f64 / 10_000_000.,
            )));
        }
        Ok(None)
    }
    pub fn frame_at(&mut self, seconds: f64) -> AppResult<Option<Frame>> {
        self.seek(seconds)?;
        let mut last = None;
        for _ in 0..250 {
            match self.next()? {
                Some((f, t)) => {
                    last = Some(f);
                    if t + 0.002 >= seconds {
                        break;
                    }
                }
                None => break,
            }
        }
        Ok(last)
    }
}
pub struct AudioReader {
    reader: IMFSourceReader,
    pending: std::collections::VecDeque<f32>,
}
impl AudioReader {
    pub fn open(path: &Path, seconds: f64) -> AppResult<Self> {
        unsafe {
            let reader =
                MFCreateSourceReaderFromURL(pcw(&wide(path)), None).map_err(|e| e.to_string())?;
            reader
                .SetStreamSelection(MF_SOURCE_READER_ALL_STREAMS.0 as u32, false)
                .map_err(|e| e.to_string())?;
            reader
                .SetStreamSelection(AUDIO, true)
                .map_err(|e| e.to_string())?;
            reader
                .SetCurrentMediaType(AUDIO, None, &pcm_type().map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            let pos = PROPVARIANT::from((seconds * 10_000_000.) as i64);
            reader
                .SetCurrentPosition(&GUID::zeroed(), &pos)
                .map_err(|e| e.to_string())?;
            Ok(Self {
                reader,
                pending: Default::default(),
            })
        }
    }
    pub fn take(&mut self, frames: usize, speed: f64, volume: f32) -> Vec<i16> {
        let n = (frames as f64 * speed).ceil() as usize * 2;
        unsafe {
            while self.pending.len() < n {
                let (mut sample, mut flags) = (None, 0);
                if self
                    .reader
                    .ReadSample(AUDIO, 0, None, Some(&mut flags), None, Some(&mut sample))
                    .is_err()
                    || flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0
                {
                    break;
                }
                let Some(sample) = sample else {
                    break;
                };
                let Ok(buf) = sample.ConvertToContiguousBuffer() else {
                    break;
                };
                let (mut p, mut len) = (std::ptr::null_mut(), 0);
                if buf.Lock(&mut p, None, Some(&mut len)).is_err() {
                    break;
                }
                let bytes = std::slice::from_raw_parts(p, len as usize);
                self.pending.extend(
                    bytes
                        .chunks_exact(2)
                        .map(|s| i16::from_le_bytes([s[0], s[1]]) as f32 / 32768.),
                );
                let _ = buf.Unlock();
            }
        }
        let input: Vec<_> = (0..n)
            .map(|_| self.pending.pop_front().unwrap_or(0.))
            .collect();
        (0..frames * 2)
            .map(|i| {
                let index = ((i / 2) as f64 * speed) as usize * 2 + i % 2;
                (input.get(index).copied().unwrap_or(0.) * volume * 32767.).clamp(-32767., 32767.)
                    as i16
            })
            .collect()
    }
}
pub struct Webcam {
    reader: IMFSourceReader,
    source: IMFMediaSource,
    width: u32,
    height: u32,
    stride: i32,
}
impl Webcam {
    pub fn open(id: &str) -> AppResult<Self> {
        unsafe {
            let acts = super::devices::camera_activations().map_err(|e| e.to_string())?;
            let a = acts
                .iter()
                .find(|a| {
                    id.is_empty()
                        || super::devices::attr_string(
                            a,
                            &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK,
                        ) == id
                })
                .ok_or(crate::i18n::tr("Nenhuma webcam disponível"))?;
            let source: IMFMediaSource = a.ActivateObject().map_err(|e| crate::i18n::format_text(crate::i18n::tr("Webcam · abrir dispositivo: {e}. Confira as permissões de câmera do Windows e se outro aplicativo está usando o dispositivo."), &[("e", format!("{}", e))]))?;
            let configured = (|| -> AppResult<Self> {
                let mut last = String::new();
                for advanced in [true, false] {
                    match Self::reader(&source, advanced) {
                        Ok((reader, width, height, stride)) => {
                            return Ok(Self {
                                reader,
                                source: source.clone(),
                                width,
                                height,
                                stride,
                            });
                        }
                        Err(error) => last = error.to_string(),
                    }
                }
                Err(crate::i18n::format_text(
                    crate::i18n::tr(
                        "Webcam · negociar formato RGB32: {last}. O dispositivo pode estar ocupado, sem permissão ou usar um formato indisponível neste Windows.",
                    ),
                    &[("last", format!("{}", last))],
                ))
            })();
            if configured.is_err() {
                let _ = source.Shutdown();
            }
            configured
        }
    }
    unsafe fn reader(
        source: &IMFMediaSource,
        advanced: bool,
    ) -> windows::core::Result<(IMFSourceReader, u32, u32, i32)> {
        let attrs = attributes()?;
        // Failed format negotiation drops this reader before the legacy retry.
        // Keep ownership of the shared source here; Webcam shuts it down once.
        attrs.SetUINT32(&MF_SOURCE_READER_DISCONNECT_MEDIASOURCE_ON_SHUTDOWN, 1)?;
        attrs.SetUINT32(
            if advanced {
                &MF_SOURCE_READER_ENABLE_ADVANCED_VIDEO_PROCESSING
            } else {
                &MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING
            },
            1,
        )?;
        let reader = MFCreateSourceReaderFromMediaSource(source, &attrs)?;
        reader.SetStreamSelection(MF_SOURCE_READER_ALL_STREAMS.0 as u32, false)?;
        reader.SetStreamSelection(VIDEO, true)?;
        let output = MFCreateMediaType()?;
        output.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
        output.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32)?;
        // The driver default is often 640×480. Prefer the sharpest mode up to 720p
        // that still delivers at least 24 FPS, then let the reader convert it.
        let mut modes = vec![];
        for index in 0..128 {
            let Ok(native) = reader.GetNativeMediaType(VIDEO, index) else {
                break;
            };
            let size = native.GetUINT64(&MF_MT_FRAME_SIZE).unwrap_or(0);
            let (w, h) = ((size >> 32) as u32, size as u32);
            if w == 0 || h == 0 || w > 1920 || h > 1080 {
                continue;
            }
            let rate = native.GetUINT64(&MF_MT_FRAME_RATE).unwrap_or(0);
            let fps = if rate as u32 == 0 {
                0
            } else {
                ((rate >> 32) / (rate as u32 as u64).max(1)) as u32
            };
            let pixels = w as u64 * h as u64;
            modes.push((
                (fps >= 24, pixels.min(1280 * 720), fps.min(30), pixels),
                native,
            ));
        }
        modes.sort_by(|a, b| b.0.cmp(&a.0));
        let preferred = modes.iter().any(|(_, native)| {
            reader.SetCurrentMediaType(VIDEO, None, native).is_ok()
                && reader.SetCurrentMediaType(VIDEO, None, &output).is_ok()
        });
        if preferred {
            // Negotiated below from the current type.
        } else if let Err(original) = reader.SetCurrentMediaType(VIDEO, None, &output) {
            let mut formats = vec![];
            for index in 0..64 {
                let Ok(native) = reader.GetNativeMediaType(VIDEO, index) else {
                    break;
                };
                if let Ok(size) = native.GetUINT64(&MF_MT_FRAME_SIZE) {
                    let (w, h) = ((size >> 32) as u32, size as u32);
                    if w > 0 && h > 0 && w <= 1920 && h <= 1080 {
                        formats.push((w as u64 * h as u64, native));
                    }
                }
            }
            // Prefer a bounded native mode; fall back without assuming a driver
            // exports RGB directly (many laptop cameras expose YUY2/MJPEG/NV12).
            formats.sort_by_key(|(pixels, _)| {
                (
                    if *pixels <= 1280 * 720 { 0 } else { 1 },
                    std::cmp::Reverse(*pixels),
                )
            });
            let accepted = formats.iter().any(|(_, native)| {
                reader.SetCurrentMediaType(VIDEO, None, native).is_ok()
                    && reader.SetCurrentMediaType(VIDEO, None, &output).is_ok()
            });
            if !accepted {
                return Err(original);
            }
        }
        let current = reader.GetCurrentMediaType(VIDEO)?;
        let size = current.GetUINT64(&MF_MT_FRAME_SIZE)?;
        let (w, h) = ((size >> 32) as u32, size as u32);
        if w == 0 || h == 0 || w > 4096 || h > 4096 {
            return Err(windows::core::Error::new(
                windows::Win32::Foundation::E_INVALIDARG,
                crate::i18n::tr("Dimensões da câmera inválidas"),
            ));
        }
        let stride = current.GetUINT32(&MF_MT_DEFAULT_STRIDE).unwrap_or(w * 4) as i32;
        Ok((reader, w, h, stride))
    }
    pub fn next(&mut self) -> AppResult<Option<Frame>> {
        unsafe {
            let mut sample = None;
            let mut flags = 0;
            self.reader
                .ReadSample(VIDEO, 0, None, Some(&mut flags), None, Some(&mut sample))
                .map_err(|e| {
                    crate::i18n::format_text(
                        crate::i18n::tr("Webcam · ler quadro: {e}"),
                        &[("e", format!("{}", e))],
                    )
                })?;
            if flags & MF_SOURCE_READERF_CURRENTMEDIATYPECHANGED.0 as u32 != 0 {
                let current = self
                    .reader
                    .GetCurrentMediaType(VIDEO)
                    .map_err(|e| e.to_string())?;
                let size = current
                    .GetUINT64(&MF_MT_FRAME_SIZE)
                    .map_err(|e| e.to_string())?;
                self.width = (size >> 32) as u32;
                self.height = size as u32;
                self.stride = current
                    .GetUINT32(&MF_MT_DEFAULT_STRIDE)
                    .unwrap_or(self.width.saturating_mul(4)) as i32;
            }
            let Some(sample) = sample else {
                return Ok(None);
            };
            let b = sample
                .ConvertToContiguousBuffer()
                .map_err(|e| e.to_string())?;
            let (mut p, mut n) = (std::ptr::null_mut(), 0);
            b.Lock(&mut p, None, Some(&mut n))
                .map_err(|e| e.to_string())?;
            let result = if p.is_null() || n == 0 {
                Err(crate::i18n::tr("Webcam retornou um buffer vazio.").into())
            } else {
                Frame::from_strided_rgb32(
                    std::slice::from_raw_parts(p, n as usize),
                    self.width,
                    self.height,
                    self.stride,
                )
                .map(Some)
            };
            let _ = b.Unlock();
            result
        }
    }
}
impl Drop for Webcam {
    fn drop(&mut self) {
        unsafe {
            let _ = self.source.Shutdown();
        }
    }
}

/// Exercise the camera reader's shared-source retry contract with a local video
/// source. This validates COM/MF ownership without claiming a physical camera.
pub fn validate_webcam_retry_lifetime(path: &Path) -> AppResult<()> {
    unsafe {
        let resolver = MFCreateSourceResolver().map_err(|e| e.to_string())?;
        let mut kind = MF_OBJECT_TYPE::default();
        let mut object = None;
        resolver
            .CreateObjectFromURL(
                pcw(&wide(path)),
                MF_RESOLUTION_MEDIASOURCE.0 as u32,
                None,
                &mut kind,
                &mut object,
            )
            .map_err(|e| e.to_string())?;
        let source: IMFMediaSource = object
            .ok_or(crate::i18n::tr(
                "Source resolver não retornou fonte de mídia",
            ))?
            .cast()
            .map_err(|e| e.to_string())?;
        let result = (|| -> windows::core::Result<()> {
            let (first, _, _, _) = Webcam::reader(&source, true)?;
            let invalid = MFCreateMediaType()?;
            invalid.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio)?;
            let rejected = first.SetCurrentMediaType(VIDEO, None, &invalid).is_err();
            drop(first);
            if !rejected {
                return Err(windows::core::Error::new(
                    windows::Win32::Foundation::E_FAIL,
                    crate::i18n::tr("Fonte de vídeo aceitou negociação incompatível no teste"),
                ));
            }
            // Without DISCONNECT_MEDIASOURCE_ON_SHUTDOWN, dropping `first`
            // shuts down the source and this call/legacy retry must fail.
            source.GetCharacteristics()?;
            let (second, _, _, _) = Webcam::reader(&source, false)?;
            let mut sample = None;
            let mut flags = 0;
            second.ReadSample(VIDEO, 0, None, Some(&mut flags), None, Some(&mut sample))?;
            if sample.is_none() {
                return Err(windows::core::Error::new(
                    windows::Win32::Foundation::E_FAIL,
                    crate::i18n::tr("Segunda tentativa não leu o vídeo da fonte preservada"),
                ));
            }
            Ok(())
        })();
        let _ = source.Shutdown();
        result.map_err(|e| {
            crate::i18n::format_text(
                crate::i18n::tr("Regressão de ciclo de vida do leitor da webcam: {e}"),
                &[("e", format!("{}", e))],
            )
        })
    }
}
