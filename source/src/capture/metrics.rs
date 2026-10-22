use crate::platform::windows::diagnostics::{ProcessSample, process_sample};
use std::time::Instant;
#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub video_pts_end_100ns: i64,
    pub audio_pts_end_100ns: i64,
    pub mic_device_clock_drift_ms: Option<f64>,
    pub system_device_clock_drift_ms: Option<f64>,
    pub mic_discontinuities: u64,
    pub system_discontinuities: u64,
    pub encoder: String,
    pub configured_fps: u32,
    pub active_seconds: f64,
    pub received: u64,
    pub encoded: u64,
    pub missed_deadlines: u64,
    pub processing_mean_ms: f64,
    pub processing_peak_ms: f64,
    pub encoder_mean_ms: f64,
    pub encoder_peak_ms: f64,
    pub cpu_mean: f64,
    pub cpu_peak: f64,
    pub memory_initial: u64,
    pub memory_final: u64,
    pub memory_peak: u64,
    pub audio_frames: u64,
    pub audio_padded_samples: u64,
    pub audio_dropped_samples: u64,
    pub submitted_av_difference_ms: f64,
    pub bytes: u64,
    pub free_bytes: u64,
    pub gpu_resize: bool,
    pub encoder_queue_bytes: u64,
    pub encoder_delay_ms: f64,
    pub encoder_samples_segment: u64,
}
impl Snapshot {
    pub fn effective_fps(&self) -> f64 {
        self.encoded as f64 / self.active_seconds.max(0.001)
    }
    pub fn json(&self) -> String {
        let clock = format!(
            "\"video_pts_end_100ns\":{},\"audio_pts_end_100ns\":{},\"mic_device_vs_qpc_drift_ms\":{},\"system_device_vs_qpc_drift_ms\":{},\"mic_discontinuities\":{},\"system_discontinuities\":{},",
            self.video_pts_end_100ns,
            self.audio_pts_end_100ns,
            self.mic_device_clock_drift_ms
                .map(|v| format!("{v:.6}"))
                .unwrap_or("null".into()),
            self.system_device_clock_drift_ms
                .map(|v| format!("{v:.6}"))
                .unwrap_or("null".into()),
            self.mic_discontinuities,
            self.system_discontinuities
        );
        let result=format!(
            "{{\"configured_fps\":{},\"active_seconds\":{:.6},\"frames_received\":{},\"frames_encoded\":{},\"effective_fps\":{:.3},\"missed_scheduling_deadlines\":{},\"processing_mean_ms\":{:.4},\"processing_peak_ms\":{:.4},\"encoding_submission_mean_ms\":{:.4},\"encoding_submission_peak_ms\":{:.4},\"cpu_mean_percent_machine\":{:.3},\"cpu_peak_percent_machine\":{:.3},\"memory_initial_bytes\":{},\"memory_final_bytes\":{},\"memory_peak_bytes\":{},\"audio_frames_submitted\":{},\"audio_padded_samples\":{},\"audio_dropped_samples\":{},\"submitted_av_difference_ms\":{:.6},\"bytes\":{},\"free_bytes\":{},\"gpu_utilization\":null,\"physical_av_drift_ms\":null}}",
            self.configured_fps,
            self.active_seconds,
            self.received,
            self.encoded,
            self.effective_fps(),
            self.missed_deadlines,
            self.processing_mean_ms,
            self.processing_peak_ms,
            self.encoder_mean_ms,
            self.encoder_peak_ms,
            self.cpu_mean,
            self.cpu_peak,
            self.memory_initial,
            self.memory_final,
            self.memory_peak,
            self.audio_frames,
            self.audio_padded_samples,
            self.audio_dropped_samples,
            self.submitted_av_difference_ms,
            self.bytes,
            self.free_bytes
        ).replace("\"frames_encoded\"", "\"video_samples_submitted\"").replace("\"gpu_utilization\"", &format!("\"encoder\":{:?},\"gpu_resize\":{},\"encoder_queue_bytes\":{},\"encoder_delay_ms\":{:.4},\"encoder_samples_current_segment\":{},\"wgc_dropped_frames\":null,\"gpu_utilization\"",self.encoder,self.gpu_resize,self.encoder_queue_bytes,self.encoder_delay_ms,self.encoder_samples_segment))
        ;
        result.replacen('{', &format!("{{{clock}"), 1)
    }
}
pub struct Measurements {
    pub snapshot: Snapshot,
    first: Option<ProcessSample>,
    last: Option<ProcessSample>,
    start: Instant,
    sampled: Instant,
    processing_total: f64,
    encoding_total: f64,
}
impl Measurements {
    pub fn new(fps: u32) -> Self {
        let sample = process_sample();
        Self {
            snapshot: Snapshot {
                configured_fps: fps,
                memory_initial: sample.map(|s| s.private_bytes).unwrap_or(0),
                ..Default::default()
            },
            first: sample,
            last: sample,
            start: Instant::now(),
            sampled: Instant::now(),
            processing_total: 0.,
            encoding_total: 0.,
        }
    }
    pub fn frame(&mut self, processing_ms: f64, encoding_ms: f64) {
        self.snapshot.encoded += 1;
        self.processing_total += processing_ms;
        self.encoding_total += encoding_ms;
        self.snapshot.processing_mean_ms = self.processing_total / self.snapshot.encoded as f64;
        self.snapshot.encoder_mean_ms = self.encoding_total / self.snapshot.encoded as f64;
        self.snapshot.processing_peak_ms = self.snapshot.processing_peak_ms.max(processing_ms);
        self.snapshot.encoder_peak_ms = self.snapshot.encoder_peak_ms.max(encoding_ms);
    }
    pub fn sample_process(&mut self) {
        let Some(sample) = process_sample() else {
            return;
        };
        let cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1) as f64;
        if let Some(last) = self.last {
            let cpu = sample.cpu_ticks.saturating_sub(last.cpu_ticks) as f64
                / 100_000.
                / self.sampled.elapsed().as_secs_f64().max(0.001)
                / cores;
            self.snapshot.cpu_peak = self.snapshot.cpu_peak.max(cpu);
        }
        if let Some(first) = self.first {
            self.snapshot.cpu_mean = sample.cpu_ticks.saturating_sub(first.cpu_ticks) as f64
                / 100_000.
                / self.start.elapsed().as_secs_f64().max(0.001)
                / cores;
        }
        self.snapshot.memory_final = sample.private_bytes;
        self.snapshot.memory_peak = self.snapshot.memory_peak.max(sample.private_bytes);
        let _working_set = sample.working_set;
        self.last = Some(sample);
        self.sampled = Instant::now();
    }
}
