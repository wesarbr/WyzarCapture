use super::theme::Rect;
use crate::{
    capture::{Event, Session},
    core::{AppResult, Frame, RecordingState, Region, Source},
    editor::Project,
    library::Library,
    platform::windows::{
        self as win,
        capture::Target,
        devices::{Device, Monitor},
    },
    settings::{Settings, appearance::InterfaceMode},
};
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use windows::Win32::Foundation::HWND;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Home,
    Library,
    Settings,
    Editor,
    Components,
    Help,
    About,
    News,
    Recovery,
    Diagnostics,
    Capabilities,
}
impl Page {
    pub fn key(self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::Library => "library",
            Self::Settings => "settings",
            Self::Editor => "editor",
            Self::Components => "components",
            Self::Help => "help",
            Self::About => "about",
            Self::News => "news",
            Self::Recovery => "recovery",
            Self::Diagnostics => "diagnostics",
            Self::Capabilities => "capabilities",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Search,
    Naming,
    Rename,
    Tags,
    Text,
    ExportName,
    ProjectName,
    Appearance(usize),
    HelpSearch,
    IconSearch,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChoiceKind {
    Language,
    RecordingMode,
    MicGain,
    SystemGain,
    Interface,
    CaptureSource,
    Theme,
    Resolution,
    Fps,
    Bitrate,
    Monitor,
    Microphone,
    Output,
    Webcam,
    Countdown,
    Naming,
    Subfolders,
    Encoder,
    Window,
    WebcamPosition,
    WebcamSize,
    RecordKey,
    PauseKey,
    StopKey,
    ScreenshotKey,
    ScreenshotSource,
    ScreenshotDelay,
    SpaceAlert,
    StorageLimit,
    Profile,
    IconStyle,
    IconWeight,
    WebcamShape,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toggle {
    AudioLimiter,
    AudioSources,
    AutoFallback,
    Microphone,
    SystemAudio,
    Webcam,
    Cursor,
    Clicks,
    Recovery,
    Notifications,
    Tray,
    Startup,
    GpuResize,
    Animated,
    MinimizeOnRecord,
    ExcludeSelf,
    ScreenshotClipboard,
    ScreenshotCursor,
    WebcamMirror,
    VoiceFilter,
}
#[derive(Clone, Debug)]
pub enum SliderKind {
    Volume,
    Speed,
}
#[derive(Clone, Debug)]
pub enum Action {
    ShowLicense,
    LicensePage(bool),
    None,
    SetInterface(InterfaceMode),
    Pin,
    Page(Page),
    NewRecording,
    NewAudioRecording,
    CloseModal,
    StartRecording,
    Pause,
    Stop,
    Source(Source),
    Preset(usize),
    Toggle(Toggle),
    ChooseFolder(usize),
    OpenChoice(ChoiceKind, Rect),
    SelectChoice(ChoiceKind, usize),
    SettingsCategory(usize),
    Focus(Field),
    Preview,
    OpenVideos,
    OpenSelectedFolder,
    Screenshot,
    TakeScreenshot,
    OpenFolders,
    OpenFolder(usize),
    ChooseMediaRoot,
    RestoreMediaDefaults,
    CameraView,
    ApplyShortcut(usize, u32),
    ResetShortcuts,
    SaveProfile,
    Select(usize),
    PlaySelected,
    EditSelected,
    Rename,
    Tags,
    Favorite(usize),
    Delete,
    ConfirmDelete,
    Import,
    Reindex,
    Relink,
    LibraryTab(usize),
    Sort,
    ListView,
    SaveProject,
    OpenProject,
    Split,
    Trim,
    RemoveClip,
    Speed,
    Volume,
    Slider(SliderKind, Rect),
    SetVolume(f32),
    SetSpeed(f64),
    Text,
    Arrow,
    Blur,
    Fade,
    Crop,
    Play,
    Seek(f64),
    SelectClip(usize),
    Undo,
    Redo,
    Export,
    BeginExport,
    ConfirmText,
    Minimize,
    Maximize,
    Close,
    RefreshDevices,
    DismissToast,
    Recover(usize),
    RecoveryFolder(usize),
    RecoveryDetails(usize),
    DiscardRecovery(usize),
    ConfirmDiscardRecovery(usize),
    ShowErrorDetails,
    OpenLogs,
    CopyDiagnostics,
    ProbeCapabilities,
    PerformanceTest,
    Cleanup(usize),
    ConfirmCleanup,
    ImportExistingData,
    ConfirmImportData,
    ViewExistingData,
    AudioTracks,
    TrackEdit(usize, usize),
    SelectTrack(usize),
    ToggleTrackMix,
    SetTheme(usize),
    EditAppearance(usize),
    ResetAppearance,
    OpenSettings(usize),
    HelpTopic(usize),
    ResetHelpSearch,
    ComponentsTab(usize),
    SelectIcon(String),
    ImportIcon,
    ResetIcon,
}
#[derive(Clone, Debug)]
pub struct Hit {
    pub rect: Rect,
    pub action: Action,
    pub label: String,
}
pub fn webcam_shapes() -> [&'static str; 3] {
    [
        crate::i18n::tr("Retângulo"),
        crate::i18n::tr("Cantos arredondados"),
        crate::i18n::tr("Círculo"),
    ]
}
pub fn slider_action(action: Action, x: f32) -> Action {
    if let Action::Slider(kind, r) = action {
        let v = ((x - r.x) / r.w).clamp(0., 1.);
        match kind {
            SliderKind::Volume => Action::SetVolume(v * 1.5),
            SliderKind::Speed => Action::SetSpeed(0.25 + v as f64 * 1.75),
        }
    } else {
        action
    }
}
#[derive(Clone, Debug)]
pub struct Popup {
    pub rect: Rect,
    pub kind: ChoiceKind,
    pub values: Vec<String>,
    pub first_visible: usize,
}
#[derive(Clone, Debug)]
pub enum Modal {
    Shortcut(usize),
    Screenshot,
    Folders,
    Camera,
    License,
    ExistingData(crate::persistence::migration::Import),
    Cleanup(crate::persistence::maintenance::Plan),
    AudioTracks,
    Recording,
    Export,
    Text { field: Field, title: String },
    Delete,
    RecoveryDetails(usize),
    DiscardRecovery(usize),
    ErrorDetails,
}
#[derive(Clone, Debug)]
pub struct Toast {
    pub text: String,
    pub error: bool,
    pub warning: bool,
    pub until: Instant,
    pub action: Option<Action>,
    pub since: Instant,
}
/// Short interface transitions. Durations are fixed and nothing moves when the
/// Windows animation preference is off; recording and export freeze decoration.
#[derive(Clone, Debug)]
pub struct Motion {
    /// Interface transitions (hover, pages, switches, dialogs).
    pub enabled: bool,
    /// Windows allows decorative motion; the theme effect also needs the user's choice.
    pub ambient: bool,
    pub hover: Option<Rect>,
    pub hover_at: Instant,
    pub previous: Option<Rect>,
    pub previous_at: Instant,
    pub page: Option<(Page, usize, bool)>,
    pub page_at: Instant,
    pub nav_from: f32,
    pub nav_to: f32,
    pub nav_at: Instant,
    pub modal_open: bool,
    pub modal_at: Instant,
    pub popup_open: bool,
    pub popup_at: Instant,
    pub toggled: Option<(Toggle, Instant)>,
}
pub fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0., 1.);
    1. - (1. - t).powi(3)
}
impl Motion {
    pub fn new(enabled: bool, ambient: bool) -> Self {
        let past = Instant::now() - Duration::from_secs(5);
        Self {
            enabled,
            ambient,
            hover: None,
            hover_at: past,
            previous: None,
            previous_at: past,
            page: None,
            page_at: past,
            nav_from: -1.,
            nav_to: -1.,
            nav_at: past,
            modal_open: false,
            modal_at: past,
            popup_open: false,
            popup_at: past,
            toggled: None,
        }
    }
    /// Eased progress of a transition that started at `at`.
    pub fn progress(&self, at: Instant, seconds: f32) -> f32 {
        if !self.enabled {
            return 1.;
        }
        ease_out(at.elapsed().as_secs_f32() / seconds)
    }
    /// Hover highlight for a control, fading in and out.
    pub fn hover_amount(&self, r: Rect, hovered: bool) -> f32 {
        if hovered {
            if self.hover == Some(r) {
                self.progress(self.hover_at, 0.12)
            } else {
                1.
            }
        } else if self.previous == Some(r) {
            1. - self.progress(self.previous_at, 0.16)
        } else {
            0.
        }
    }
    pub fn set_hover(&mut self, rect: Option<Rect>) {
        if rect == self.hover {
            return;
        }
        self.previous = self.hover;
        self.previous_at = Instant::now();
        self.hover = rect;
        self.hover_at = Instant::now();
    }
    pub fn nav_y(&mut self, target: f32) -> f32 {
        if self.nav_to < 0. || !self.enabled {
            self.nav_from = target;
            self.nav_to = target;
            return target;
        }
        if (self.nav_to - target).abs() > 0.5 {
            let current =
                self.nav_from + (self.nav_to - self.nav_from) * self.progress(self.nav_at, 0.24);
            self.nav_from = current;
            self.nav_to = target;
            self.nav_at = Instant::now();
        }
        self.nav_from + (self.nav_to - self.nav_from) * self.progress(self.nav_at, 0.24)
    }
    pub fn busy(&self) -> bool {
        if !self.enabled {
            return false;
        }
        let active = |at: Instant, seconds: f32| at.elapsed().as_secs_f32() < seconds;
        active(self.hover_at, 0.14)
            || active(self.previous_at, 0.18)
            || active(self.page_at, 0.26)
            || active(self.nav_at, 0.26)
            || active(self.modal_at, 0.2)
            || active(self.popup_at, 0.16)
            || self.toggled.is_some_and(|(_, at)| active(at, 0.18))
    }
}
pub enum Background {
    Notice(String, bool),
    Metadata(PathBuf, AppResult<(win::media::Metadata, Option<Frame>)>),
    Decoded(u64, Option<Frame>),
    Export(f32),
    ExportDone(AppResult<PathBuf>),
    ScreenshotDone(AppResult<PathBuf>),
    ScreenshotImage(PathBuf, Frame),
    Recovered(crate::recovery::Candidate, AppResult<PathBuf>),
    Capabilities(AppResult<Vec<crate::encoder::capabilities::Profile>>),
    Performance(AppResult<crate::encoder::benchmark::Report>),
}
pub struct PreviewWorker {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}
impl Drop for PreviewWorker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
pub struct App {
    pub license_page: usize,
    pub inventory: crate::encoder::capabilities::Inventory,
    pub performance_running: bool,
    pub performance_result: Option<crate::encoder::benchmark::Report>,
    pub maximized: bool,
    pub animation_started: Instant,
    pub help_topic: usize,
    pub help_search: String,
    pub components_tab: usize,
    pub icon_search: String,
    pub selected_icon: String,
    pub settings: Settings,
    pub page: Page,
    pub category: usize,
    pub library: Library,
    pub project: Project,
    pub waveforms: std::collections::HashMap<PathBuf, Vec<f32>>,
    pub undo: Vec<Project>,
    pub redo: Vec<Project>,
    pub modal: Option<Modal>,
    pub popup: Option<Popup>,
    pub hits: Vec<Hit>,
    pub hover: Option<usize>,
    pub hover_since: Instant,
    pub focus_index: Option<usize>,
    pub focused: Option<Field>,
    pub select_all: bool,
    pub toast: Option<Toast>,
    pub state: RecordingState,
    pub seconds: f64,
    pub frames: u64,
    pub mic_level: f32,
    pub system_level: f32,
    pub session: Option<Session>,
    pub preview: Arc<Mutex<Option<Frame>>>,
    pub preview_worker: Option<PreviewWorker>,
    pub preview_enabled: bool,
    pub webcam_frame: Arc<Mutex<Option<Frame>>>,
    pub webcam_worker: Option<PreviewWorker>,
    pub monitors: Vec<Monitor>,
    pub microphones: Vec<Device>,
    pub outputs: Vec<Device>,
    pub webcams: Vec<Device>,
    pub window: Option<isize>,
    pub window_name: String,
    pub region: Option<Region>,
    pub window_options: Vec<win::devices::Window>,
    pub search: String,
    pub selected: Option<usize>,
    pub library_tab: usize,
    pub oldest: bool,
    pub list_view: bool,
    pub scroll: f32,
    pub text_buffer: String,
    pub export_name: String,
    pub export_progress: Option<f32>,
    pub editor_frame: Option<Frame>,
    pub playing: bool,
    pub audio_playback: Option<win::playback::Playback>,
    pub last_tick: Instant,
    pub background_tx: Sender<Background>,
    pub background_rx: Receiver<Background>,
    pub decode_generation: u64,
    pub decode_workers_started: u64,
    pub decoding: bool,
    pub countdown_deadline: Option<Instant>,
    pub pending_record: Option<(Option<Target>, PathBuf)>,
    pub profile_paths: Vec<PathBuf>,
    pub project_file: Option<PathBuf>,
    pub last_autosave: Instant,
    pub pending_recovery: usize,
    pub scroll_max: f32,
    pub recovery_items: Vec<crate::recovery::Candidate>,
    pub recovering: bool,
    pub metrics: Option<crate::capture::metrics::Snapshot>,
    pub last_error: Option<String>,
    pub capabilities: Vec<crate::encoder::capabilities::Profile>,
    pub probing_capabilities: bool,
    pub viewport: (f32, f32),
    pub hit_clip: Option<Rect>,
    pub screenshot_pending: bool,
    pub last_screenshot: Option<PathBuf>,
    pub motion: Motion,
    /// Few cores or little memory: decoration refreshes less often.
    pub low_power: bool,
}
impl App {
    pub fn new(settings: Settings) -> Self {
        crate::i18n::set(settings.language);
        let (tx, rx) = mpsc::channel();
        let mut library = Library::default();
        library.restore_index(&settings.data_dir);
        library.scan(&settings.videos);
        library.scan(&settings.audios);
        let pending = crate::recovery::pending(&settings.data_dir).len();
        let recovery_items = crate::recovery::candidates(&settings.data_dir);
        let mut app = Self {
            inventory: win::capabilities::inventory().unwrap_or_default(),
            performance_running: false,
            performance_result: None,
            maximized: false,
            animation_started: Instant::now(),
            help_topic: 0,
            help_search: String::new(),
            components_tab: 0,
            icon_search: String::new(),
            selected_icon: "settings".into(),
            settings,
            page: Page::Home,
            category: 0,
            library,
            project: Project::default(),
            waveforms: std::collections::HashMap::new(),
            undo: vec![],
            redo: vec![],
            modal: None,
            popup: None,
            hits: vec![],
            hover: None,
            license_page: 0,
            hover_since: Instant::now(),
            focus_index: None,
            focused: None,
            select_all: false,
            toast: None,
            state: RecordingState::Idle,
            seconds: 0.,
            frames: 0,
            mic_level: 0.,
            system_level: 0.,
            session: None,
            preview: Arc::new(Mutex::new(None)),
            preview_worker: None,
            preview_enabled: false,
            webcam_frame: Arc::new(Mutex::new(None)),
            webcam_worker: None,
            monitors: win::devices::monitors(),
            microphones: win::devices::audio_devices(true),
            outputs: win::devices::audio_devices(false),
            webcams: win::devices::cameras(),
            window: None,
            window_name: String::new(),
            region: None,
            window_options: vec![],
            search: String::new(),
            selected: None,
            library_tab: 0,
            oldest: false,
            list_view: false,
            scroll: 0.,
            text_buffer: String::new(),
            export_name: crate::i18n::tr("Meu vídeo").into(),
            export_progress: None,
            editor_frame: None,
            playing: false,
            audio_playback: None,
            last_tick: Instant::now(),
            background_tx: tx,
            background_rx: rx,
            decode_generation: 0,
            decode_workers_started: 0,
            decoding: false,
            countdown_deadline: None,
            pending_record: None,
            profile_paths: vec![],
            project_file: None,
            last_autosave: Instant::now(),
            pending_recovery: pending,
            scroll_max: 0.,
            recovery_items,
            recovering: false,
            metrics: None,
            last_error: None,
            capabilities: vec![],
            probing_capabilities: false,
            viewport: (1600., 900.),
            hit_clip: None,
            screenshot_pending: false,
            last_screenshot: None,
            // Off-screen renders and checks are still frames; the window host
            // enables transitions from the Windows animation preference.
            motion: Motion::new(false, true),
            low_power: false,
        };
        app.low_power = app.inventory.logical_cpus <= 2
            || (app.inventory.memory_bytes > 0 && app.inventory.memory_bytes < 6 * 1_073_741_824);
        app.reconcile_devices();
        app.refresh_metadata();
        if pending > 0 {
            app.page = Page::Recovery;
            app.notify(
                crate::i18n::format_text(
                    crate::i18n::tr(
                        "{pending} sessão(ões) interrompida(s). Consulte Configurações › Avançado.",
                    ),
                    &[("pending", format!("{}", pending))],
                ),
                true,
            );
        }
        if pending == 0
            && crate::updates::should_notify(
                &app.settings.last_seen_version,
                &app.settings.notified_version,
                crate::updates::VERSION,
            )
        {
            app.notify(
                {
                    let __wyzar_i18n_0 = &(crate::updates::VERSION);
                    crate::i18n::format_text(
                        crate::i18n::tr("WyzarCapture foi atualizado para {}."),
                        &[("0", format!("{}", __wyzar_i18n_0))],
                    )
                },
                false,
            );
            if let Some(toast) = app.toast.as_mut() {
                toast.action = Some(Action::Page(Page::News));
            }
            app.settings.notified_version = crate::updates::VERSION.into();
            app.save_settings();
        }
        if let Some(notice) = app.settings.storage_notice.take() {
            app.warn(notice);
            if let Some(t) = app.toast.as_mut() {
                t.action = Some(Action::OpenFolders);
            }
        }
        app
    }
    pub fn notify(&mut self, text: impl Into<String>, error: bool) {
        if !error && !self.settings.notifications {
            return;
        }
        let since = self
            .toast
            .as_ref()
            .filter(|t| t.until > Instant::now())
            .map(|t| t.since)
            .unwrap_or_else(Instant::now);
        self.toast = Some(Toast {
            text: text.into(),
            error,
            warning: false,
            until: Instant::now() + Duration::from_secs(if error { 9 } else { 5 }),
            action: None,
            since,
        });
    }
    pub fn hit(&mut self, rect: Rect, action: Action, label: impl Into<String>) {
        let rect = if let Some(clip) = self.hit_clip {
            let Some(rect) = rect.intersection(clip) else {
                return;
            };
            rect
        } else {
            rect
        };
        self.hits.push(Hit {
            rect,
            action,
            label: label.into(),
        });
    }
    pub fn warn(&mut self, text: impl Into<String>) {
        self.notify(text, true);
        if let Some(toast) = self.toast.as_mut() {
            toast.error = false;
            toast.warning = true;
        }
    }
    pub fn action_at(&self, x: f32, y: f32) -> Option<usize> {
        self.hits.iter().rposition(|h| h.rect.contains(x, y))
    }
    pub fn scroll_by(&mut self, delta: f32) {
        if let Some(popup) = self.popup.as_mut() {
            let rows = ((popup.rect.h - 12.) / 35.).floor() as usize;
            popup.first_visible = (popup.first_visible as i64 + (delta / 40.) as i64)
                .clamp(0, popup.values.len().saturating_sub(rows) as i64)
                as usize;
            self.focus_index = None;
        } else {
            self.scroll = (self.scroll + delta).clamp(0., self.scroll_max);
        }
    }
    pub fn save_settings(&mut self) {
        if let Err(e) = self.settings.save() {
            self.notify(e, true);
        }
    }
    pub fn save_on_close(&mut self) -> AppResult<()> {
        self.settings.save()?;
        self.library.save_index(&self.settings.data_dir)?;
        if self.project.dirty {
            let path = self
                .project_file
                .clone()
                .unwrap_or_else(|| self.settings.data_dir.join("Recovery/autosave.wyzproj"));
            self.project.save(&path)?;
        }
        Ok(())
    }
    fn target(&self) -> AppResult<Target> {
        let mut monitor =
            self.monitors
                .get(self.settings.monitor)
                .cloned()
                .ok_or(crate::i18n::tr(
                    "O monitor selecionado está indisponível. Atualize os dispositivos.",
                ))?;
        let window = match self.settings.source {
            Source::Window | Source::Game => Some(
                self.window
                    .ok_or(crate::i18n::tr("Selecione a janela que deseja gravar"))?,
            ),
            _ => None,
        };
        let region = if self.settings.source == Source::Region {
            Some(
                self.region
                    .ok_or(crate::i18n::tr("Selecione a região da tela"))?,
            )
        } else {
            None
        };
        if let Some(r) = region {
            monitor = self
                .monitors
                .iter()
                .find(|m| {
                    r.x >= m.x
                        && r.y >= m.y
                        && (r.x as i64 + r.width as i64) <= m.x as i64 + m.width as i64
                        && (r.y as i64 + r.height as i64) <= m.y as i64 + m.height as i64
                })
                .cloned()
                .ok_or(crate::i18n::tr(
                    "Selecione uma região inteiramente dentro de um único monitor",
                ))?;
        }
        Ok(Target {
            monitor,
            window,
            region,
            cursor: self.settings.cursor,
        })
    }
    /// Decorative theme motion runs only while idle; recording keeps every cycle.
    pub fn effects_running(&self) -> bool {
        self.settings.appearance.animated
            && self.motion.ambient
            && self.state == RecordingState::Idle
            && self.export_progress.is_none()
    }
    /// How soon the window needs another frame. Input repaints immediately.
    pub fn frame_interval(&self) -> Duration {
        let toast_moving = self
            .toast
            .as_ref()
            .is_some_and(|t| t.since.elapsed() < Duration::from_millis(260));
        let ms = if self.motion.busy() || (self.motion.enabled && toast_moving) {
            16
        } else if self.state == RecordingState::Countdown {
            33
        } else if self.state != RecordingState::Idle {
            if self.preview_enabled || self.motion.enabled {
                66
            } else {
                125
            }
        } else if self.playing
            || self.export_progress.is_some()
            || self.decoding
            || self.performance_running
        {
            33
        } else if self.preview_enabled || self.webcam_worker.is_some() {
            66
        } else if self.effects_running() {
            if self.low_power { 100 } else { 40 }
        } else if self.hover.is_some() && self.hover_since.elapsed() < Duration::from_millis(900) {
            100
        } else {
            250
        };
        Duration::from_millis(ms)
    }
    pub fn interface_mode(&self) -> InterfaceMode {
        let mode = self.settings.appearance.mode();
        if mode.compact() && (self.page != Page::Home || self.modal.is_some()) {
            InterfaceMode::Standard
        } else {
            mode
        }
    }
    pub fn handle(&mut self, action: Action, owner: HWND) {
        if let Err(e) = self.perform(action, owner) {
            self.failure(e);
        }
    }
    pub fn failure(&mut self, error: String) {
        crate::diagnostics::event("operation_failed", crate::diagnostics::code(&error));
        let message = if error.contains("0x80070005") {
            crate::i18n::tr(
                "O Windows negou acesso à captura ou ao destino. Verifique as permissões.",
            )
        } else if error.contains(crate::i18n::tr("WASAPI")) {
            crate::i18n::tr(
                "Não foi possível iniciar o áudio. Escolha outro dispositivo em Configurações › Áudio.",
            )
        } else if error.contains(crate::i18n::tr("Encoder")) || error.contains("0xC00D") {
            crate::i18n::tr("O perfil de vídeo não pôde ser usado. Tente outra resolução ou FPS.")
        } else if error.contains("0x") {
            crate::i18n::tr(
                "A operação não pôde ser concluída. Abra os detalhes para revisar a causa.",
            )
        } else {
            &error
        };
        self.notify(message, true);
        self.last_error = Some(error);
        if let Some(toast) = self.toast.as_mut() {
            toast.action = Some(Action::ShowErrorDetails);
        }
    }
    fn perform(&mut self, action: Action, owner: HWND) -> AppResult<()> {
        if matches!(
            action,
            Action::Page(_)
                | Action::OpenSettings(_)
                | Action::SettingsCategory(_)
                | Action::SetTheme(_)
                | Action::SelectChoice(_, _)
                | Action::HelpTopic(_)
                | Action::ComponentsTab(_)
        ) {
            self.hover = None;
            self.focus_index = None;
        }
        if self.project.clips.is_empty()
            && matches!(
                action,
                Action::Split
                    | Action::Trim
                    | Action::RemoveClip
                    | Action::Speed
                    | Action::Volume
                    | Action::SetSpeed(_)
                    | Action::SetVolume(_)
                    | Action::Text
                    | Action::Arrow
                    | Action::Blur
                    | Action::Fade
                    | Action::Crop
            )
        {
            return Err(crate::i18n::tr("Importe um vídeo antes de editar.").into());
        }
        if matches!(action,Action::SetSpeed(v) if !v.is_finite())
            || matches!(action,Action::SetVolume(v) if !v.is_finite())
        {
            return Err(crate::i18n::tr("O valor do controle é inválido.").into());
        }
        if matches!(
            action,
            Action::Split
                | Action::Trim
                | Action::RemoveClip
                | Action::Speed
                | Action::Volume
                | Action::SetSpeed(_)
                | Action::SetVolume(_)
                | Action::Text
                | Action::Arrow
                | Action::Blur
                | Action::Fade
                | Action::Crop
                | Action::Undo
                | Action::Redo
                | Action::OpenProject
                | Action::Import
                | Action::EditSelected
        ) {
            self.playing = false;
            self.audio_playback = None;
        }
        match action {
            Action::SetInterface(mode) => {
                if self.modal.is_some() {
                    self.webcam_worker = None;
                }
                self.settings.appearance.set_mode(mode);
                self.page = Page::Home;
                self.modal = None;
                self.popup = None;
                self.focused = None;
                self.focus_index = None;
                self.hover = None;
                self.playing = false;
                self.audio_playback = None;
                self.scroll = 0.;
                self.save_settings();
            }
            Action::Pin => {
                self.settings.appearance.pinned = !self.settings.appearance.pinned;
                self.save_settings();
            }
            Action::SetTheme(index) => {
                self.settings.appearance.theme = index.min(8);
                self.save_settings();
            }
            Action::EditAppearance(index) => {
                let a = &self.settings.appearance;
                let (title, value) = match index {
                    0 => (
                        crate::i18n::tr("Cor de destaque · #RRGGBB"),
                        format!("#{:06X}", a.accent),
                    ),
                    1 => (
                        crate::i18n::tr("Cor do fundo · #RRGGBB"),
                        format!("#{:06X}", a.background),
                    ),
                    2 => (
                        crate::i18n::tr("Cor dos painéis · #RRGGBB"),
                        format!("#{:06X}", a.surface),
                    ),
                    3 => (
                        crate::i18n::tr("Cor do texto · #RRGGBB"),
                        format!("#{:06X}", a.text),
                    ),
                    5 => (
                        crate::i18n::tr("Paleta completa · quatro cores"),
                        format!(
                            "#{:06X}, #{:06X}, #{:06X}, #{:06X}",
                            a.accent, a.background, a.surface, a.text
                        ),
                    ),
                    _ => (
                        crate::i18n::tr("Cor dos ícones · #RRGGBB ou auto"),
                        a.icon_tint
                            .map(|v| format!("#{v:06X}"))
                            .unwrap_or("auto".into()),
                    ),
                };
                self.text_buffer = value;
                self.focused = Some(Field::Appearance(index));
                self.select_all = true;
                self.modal = Some(Modal::Text {
                    field: Field::Appearance(index),
                    title: title.into(),
                });
            }
            Action::ResetAppearance => {
                self.settings.appearance = Default::default();
                self.save_settings();
                self.notify(crate::i18n::tr("Aparência original restaurada"), false);
            }
            Action::OpenSettings(index) => {
                self.playing = false;
                self.audio_playback = None;
                self.modal = None;
                self.popup = None;
                self.page = Page::Settings;
                self.category = index.min(9);
                self.scroll = 0.;
                self.focused = None;
            }
            Action::HelpTopic(index) => {
                self.playing = false;
                self.audio_playback = None;
                self.modal = None;
                self.popup = None;
                self.page = Page::Help;
                self.help_topic = index;
                self.scroll = 0.;
                self.focused = None;
            }
            Action::ResetHelpSearch => {
                self.help_search.clear();
                self.scroll = 0.;
                self.focused = Some(Field::HelpSearch);
            }
            Action::ComponentsTab(tab) => {
                self.components_tab = tab.min(1);
                self.scroll = 0.;
                self.focused = None;
            }
            Action::SelectIcon(name) => {
                if super::icons::NAMES.contains(&name.as_str()) {
                    self.selected_icon = name;
                }
            }
            Action::ImportIcon => {
                if let Some(path) = win::files::choose_svg(owner)? {
                    let vector = super::icons::import_svg(&path)?;
                    self.settings
                        .appearance
                        .icon_overrides
                        .insert(self.selected_icon.clone(), vector);
                    self.save_settings();
                    self.notify(
                        crate::i18n::tr("Ícone personalizado salvo. As cores acompanham o tema."),
                        false,
                    );
                }
            }
            Action::ResetIcon => {
                self.settings
                    .appearance
                    .icon_overrides
                    .remove(&self.selected_icon);
                self.save_settings();
            }
            Action::None | Action::Minimize | Action::Maximize | Action::Close => {}
            Action::Page(page) => {
                if page == Page::Help && self.page != Page::Help {
                    self.help_topic = crate::ui::help::contextual(self.page);
                    self.help_search.clear();
                }
                if page != Page::Editor {
                    self.playing = false;
                    self.audio_playback = None;
                }
                if page == Page::News {
                    self.settings.last_seen_version = crate::updates::VERSION.into();
                    self.settings.notified_version = crate::updates::VERSION.into();
                    self.toast = None;
                    self.save_settings();
                }
                self.page = page;
                self.settings.last_page = page.key().into();
                self.scroll = 0.;
                self.popup = None;
                self.focused = None;
                self.save_settings();
            }
            Action::DismissToast => self.toast = None,
            Action::ImportExistingData => {
                if self.state != RecordingState::Idle
                    || self.export_progress.is_some()
                    || self.performance_running
                {
                    return Err(
                        crate::i18n::tr("Aguarde a tarefa atual antes de importar dados").into(),
                    );
                }
                if let Some(source) = win::files::choose_folder(owner, &self.settings.data_dir)? {
                    self.modal = Some(Modal::ExistingData(
                        crate::persistence::migration::Import::inspect(source)?,
                    ));
                }
            }
            Action::ViewExistingData => {
                if let Some(Modal::ExistingData(data)) = &self.modal {
                    win::files::open_path(owner, &data.source);
                }
            }
            Action::ConfirmImportData => {
                self.preview_worker = None;
                self.preview_enabled = false;
                self.webcam_worker = None;
                self.audio_playback = None;
                self.playing = false;
                let Some(Modal::ExistingData(data)) = self.modal.clone() else {
                    return Err(crate::i18n::tr("Selecione e revise os dados existentes").into());
                };
                let _backup = data.apply(&self.settings.data_dir)?;
                self.settings = Settings::load_checked(self.settings.data_dir.clone())?;
                self.library = Library::default();
                self.library.restore_index(&self.settings.data_dir);
                self.library.scan(&self.settings.videos);
                self.library.scan(&self.settings.audios);
                self.refresh_metadata();
                self.modal = None;
                self.notify(crate::i18n::tr("Preferências, perfis e índice importados. Backup guardado em Migrations; mídias preservadas nos locais originais."),false);
            }
            Action::Cleanup(kind) => {
                if self.state != RecordingState::Idle
                    || self.export_progress.is_some()
                    || self.performance_running
                {
                    return Err(crate::i18n::tr(
                        "Aguarde as tarefas antes de limpar arquivos temporários",
                    )
                    .into());
                }
                let plan = crate::persistence::maintenance::for_kind(&self.settings, kind)?;
                self.modal = Some(Modal::Cleanup(plan));
            }
            Action::ConfirmCleanup => {
                let Some(Modal::Cleanup(plan)) = self.modal.clone() else {
                    return Err(crate::i18n::tr("Abra a confirmação de limpeza").into());
                };
                plan.execute(true)?;
                self.modal = None;
                self.notify(
                    {
                        let __wyzar_i18n_0 = &(crate::core::size_label(plan.bytes));
                        crate::i18n::format_text(
                            crate::i18n::tr("Limpeza concluída: {}. Gravações preservadas."),
                            &[("0", format!("{}", __wyzar_i18n_0))],
                        )
                    },
                    false,
                );
            }
            Action::AudioTracks => {
                self.modal = Some(Modal::AudioTracks);
                self.scroll = 0.;
            }
            Action::SelectTrack(i) => self.project.selected_track = Some(i),
            Action::TrackEdit(i, command) => {
                let mut project = self.project.clone();
                project.edit_track(i, command)?;
                self.checkpoint();
                self.project = project;
                self.playing = false;
                self.audio_playback = None;
            }
            Action::ToggleTrackMix => {
                if self.project.audio_tracks.is_empty() {
                    return Err(crate::i18n::tr("Não há WAVs associados. Importe um vídeo com seus arquivos .mic.wav e .system.wav ou importe um WAV no editor.").into());
                }
                self.checkpoint();
                self.project.tracks_active = !self.project.tracks_active;
                self.project.dirty = true;
                self.playing = false;
                self.audio_playback = None;
            }
            Action::ProbeCapabilities => {
                self.inventory = win::capabilities::inventory()?;
                if self.state != RecordingState::Idle
                    || self.probing_capabilities
                    || self.recovering
                    || self.export_progress.is_some()
                {
                    return Err(crate::i18n::tr(
                        "Aguarde a tarefa atual antes de testar os perfis.",
                    )
                    .into());
                }
                self.probing_capabilities = true;
                let tx = self.background_tx.clone();
                let cache = self.settings.cache.clone();
                let hardware = self.settings.hardware;
                thread::spawn(move || {
                    let result = crate::encoder::capabilities::probe(&cache, hardware);
                    let _ = tx.send(Background::Capabilities(result));
                });
            }
            Action::PerformanceTest => {
                if self.performance_running
                    || self.state != RecordingState::Idle
                    || self.export_progress.is_some()
                    || self.probing_capabilities
                {
                    return Err(
                        crate::i18n::tr("Aguarde a tarefa atual antes do diagnóstico").into(),
                    );
                }
                self.performance_running = true;
                let settings = self.settings.clone();
                let tx = self.background_tx.clone();
                self.notify(crate::i18n::tr("Teste de 5 segundos: captura controlada de tela, sem áudio. A mídia será removida após conferir o arquivo."),false);
                thread::spawn(move || {
                    let result = crate::encoder::benchmark::run(settings);
                    let _ = tx.send(Background::Performance(result));
                });
            }
            Action::ShowErrorDetails => {
                self.modal = Some(Modal::ErrorDetails);
                self.toast = None;
            }
            Action::OpenLogs => {
                let path = self.settings.data_dir.join("Logs");
                std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
                win::files::open_path(owner, &path);
            }
            Action::CopyDiagnostics => {
                win::files::set_clipboard_text(
                    owner,
                    &crate::diagnostics::summary(self.metrics.as_ref(), self.last_error.as_deref()),
                )?;
                self.notify(
                    crate::i18n::tr("Diagnóstico copiado, sem caminhos ou nomes privados."),
                    false,
                );
            }
            Action::Recover(index) => {
                if self.recovering || self.state != RecordingState::Idle {
                    return Err(
                        crate::i18n::tr("Aguarde a tarefa atual antes de recuperar.").into(),
                    );
                }
                let candidate = self
                    .recovery_items
                    .get(index)
                    .cloned()
                    .ok_or(crate::i18n::tr("Sessão não encontrada"))?;
                crate::persistence::test_folder(&self.settings.videos)?;
                let path = self.settings.videos.join(format!(
                    "Recuperado_{}.mp4",
                    crate::settings::unique_stamp()
                ));
                self.recovering = true;
                let tx = self.background_tx.clone();
                thread::spawn(move || {
                    let result = crate::recovery::recover(&candidate, &path);
                    let _ = tx.send(Background::Recovered(candidate, result));
                });
            }
            Action::RecoveryFolder(index) => {
                if let Some(candidate) = self.recovery_items.get(index) {
                    win::files::open_path(
                        owner,
                        candidate
                            .bundle
                            .as_deref()
                            .or(candidate.output.parent())
                            .ok_or(crate::i18n::tr("Pasta indisponível"))?,
                    );
                }
            }
            Action::RecoveryDetails(index) => self.modal = Some(Modal::RecoveryDetails(index)),
            Action::DiscardRecovery(index) => {
                if self.recovering {
                    return Err(crate::i18n::tr("Aguarde a recuperação em andamento.").into());
                }
                self.modal = Some(Modal::DiscardRecovery(index));
            }
            Action::ConfirmDiscardRecovery(index) => {
                if self.recovering {
                    return Err(crate::i18n::tr("Aguarde a recuperação em andamento.").into());
                }
                let candidate = self
                    .recovery_items
                    .get(index)
                    .cloned()
                    .ok_or(crate::i18n::tr("Sessão não encontrada"))?;
                win::files::recycle_many(owner, &crate::recovery::discard_paths(&candidate))?;
                self.recovery_items = crate::recovery::candidates(&self.settings.data_dir);
                self.pending_recovery = crate::recovery::pending(&self.settings.data_dir).len();
                self.modal = None;
                self.notify(
                    crate::i18n::tr("Arquivos temporários enviados para a Lixeira."),
                    false,
                );
            }
            Action::NewAudioRecording => {
                if self.state == RecordingState::Idle {
                    self.stop_preview();
                    self.webcam_worker = None;
                    self.settings.recording_mode = 3;
                    self.save_settings();
                    self.handle(Action::NewRecording, owner);
                }
            }
            Action::NewRecording => {
                if self.recovering {
                    return Err(crate::i18n::tr(
                        "Aguarde a recuperação antes de iniciar uma gravação.",
                    )
                    .into());
                }
                if self.state == RecordingState::Idle {
                    self.modal = Some(Modal::Recording);
                    self.popup = None;
                    self.refresh_devices();
                    if self.settings.webcam {
                        self.start_webcam_preview();
                    }
                }
            }
            Action::CloseModal => {
                self.webcam_worker = None;
                self.modal = None;
                self.popup = None;
                self.focused = None;
                if self.settings.webcam
                    && self.state == RecordingState::Idle
                    && !self.webcams.is_empty()
                {
                    self.start_webcam_preview();
                } else if let Ok(mut frame) = self.webcam_frame.lock() {
                    *frame = None;
                }
            }
            Action::ApplyShortcut(index, value) => {
                let mut keys = self.settings.shortcut_keys();
                if index >= keys.len() {
                    return Ok(());
                }
                keys[index] = value;
                crate::settings::shortcuts::validate(&keys)?;
                let mut next = self.settings.clone();
                [
                    next.record_hotkey,
                    next.pause_hotkey,
                    next.stop_hotkey,
                    next.screenshot_hotkey,
                ] = keys;
                next.save()?;
                self.settings = next;
                self.modal = None;
            }
            Action::ResetShortcuts => {
                [
                    self.settings.record_hotkey,
                    self.settings.pause_hotkey,
                    self.settings.stop_hotkey,
                    self.settings.screenshot_hotkey,
                ] = crate::settings::shortcuts::DEFAULTS;
                self.save_settings();
            }
            Action::CameraView => {
                self.refresh_devices();
                if self.webcams.is_empty() {
                    return Err(crate::i18n::tr(
                        "Nenhuma webcam disponível. Conecte uma câmera e atualize os dispositivos.",
                    )
                    .into());
                }
                if self.state != RecordingState::Idle {
                    return Err(
                        crate::i18n::tr("Finalize a gravação antes de alterar as fontes").into(),
                    );
                }
                self.modal = Some(Modal::Camera);
                self.start_webcam_preview();
            }
            Action::OpenFolders => self.modal = Some(Modal::Folders),
            Action::OpenFolder(index) => {
                let path = match index {
                    0 => &self.settings.videos,
                    1 => &self.settings.screenshots,
                    2 => &self.settings.cache,
                    3 => &self.settings.exports,
                    5 => &self.settings.audios,
                    _ => &self.settings.projects,
                };
                crate::persistence::test_folder(path)?;
                win::files::open_path(owner, path);
            }
            Action::RestoreMediaDefaults => {
                if self.state != RecordingState::Idle {
                    return Err(
                        crate::i18n::tr("Finalize a gravação antes de alterar as pastas").into(),
                    );
                }
                let mut next = self.settings.clone();
                crate::settings::bootstrap::restore_windows_media(&mut next)?;
                next.save()?;
                self.settings = next;
                self.last_error = None;
                self.modal = Some(Modal::Folders);
                self.library.scan(&self.settings.videos);
                self.library.scan(&self.settings.audios);
                self.notify(crate::i18n::tr("Pasta validada e salva"), false);
            }
            Action::ChooseMediaRoot => {
                if self.state != RecordingState::Idle {
                    return Err(
                        crate::i18n::tr("Finalize a gravação antes de alterar as pastas").into(),
                    );
                }
                let root = win::files::choose_folder(owner, &self.settings.videos)?;
                if let Some(root) = root {
                    let mut next = self.settings.clone();
                    crate::settings::bootstrap::set_media(&mut next, &root)?;
                    next.prepare()?;
                    next.save()?;
                    self.settings = next;
                    self.last_error = None;
                    self.modal = Some(Modal::Folders);
                    self.library.scan(&self.settings.videos);
                    self.library.scan(&self.settings.audios);
                    self.notify(crate::i18n::tr("Pasta validada e salva"), false);
                }
            }
            Action::ShowLicense => {
                self.license_page = 0;
                self.modal = Some(Modal::License);
                self.popup = None;
            }
            Action::LicensePage(next) => {
                self.license_page = if next {
                    (self.license_page + 1).min(crate::license::page_count().saturating_sub(1))
                } else {
                    self.license_page.saturating_sub(1)
                };
            }
            Action::Source(source) => {
                if self.state == RecordingState::Idle {
                    self.settings.source = source;
                    self.stop_preview();
                    self.save_settings();
                }
            }
            Action::Preset(i) => {
                let (w, h, bitrate) = [
                    (1920, 1080, 20_000_000),
                    (2560, 1440, 40_000_000),
                    (3840, 2160, 60_000_000),
                ][i.min(2)];
                self.settings.width = w;
                self.settings.height = h;
                self.settings.bitrate = bitrate;
                self.settings.fps = 60;
                self.save_settings();
            }
            Action::Toggle(toggle) => {
                self.motion.toggled = Some((toggle, Instant::now()));
                if self.state != RecordingState::Idle {
                    return Err(
                        crate::i18n::tr("Finalize a gravação antes de alterar as fontes").into(),
                    );
                }
                match toggle {
                    Toggle::ScreenshotClipboard => {
                        self.settings.screenshot_clipboard = !self.settings.screenshot_clipboard
                    }
                    Toggle::ScreenshotCursor => {
                        self.settings.screenshot_cursor = !self.settings.screenshot_cursor
                    }
                    Toggle::WebcamMirror => {
                        self.settings.webcam_mirror = !self.settings.webcam_mirror
                    }
                    Toggle::VoiceFilter => self.settings.voice_filter = !self.settings.voice_filter,
                    Toggle::AudioSources => {
                        self.settings.audio_sources = !self.settings.audio_sources
                    }
                    Toggle::AudioLimiter => {
                        self.settings.audio_limiter = !self.settings.audio_limiter
                    }
                    Toggle::AutoFallback => {
                        self.settings.auto_fallback = !self.settings.auto_fallback
                    }
                    Toggle::Microphone => self.settings.microphone = !self.settings.microphone,
                    Toggle::SystemAudio => self.settings.system_audio = !self.settings.system_audio,
                    Toggle::Webcam => {
                        self.refresh_devices();
                        if self.webcams.is_empty() && !self.settings.webcam {
                            return Err(crate::i18n::tr("Nenhuma webcam disponível. Conecte uma câmera e atualize os dispositivos.").into());
                        }
                        self.settings.webcam = !self.settings.webcam;
                        if self.settings.webcam {
                            self.start_webcam_preview();
                        } else {
                            self.webcam_worker = None;
                            if let Ok(mut frame) = self.webcam_frame.lock() {
                                *frame = None;
                            }
                        }
                    }
                    Toggle::Cursor => self.settings.cursor = !self.settings.cursor,
                    Toggle::Clicks => self.settings.click_effects = !self.settings.click_effects,
                    Toggle::Recovery => self.settings.recovery = !self.settings.recovery,
                    Toggle::GpuResize => self.settings.gpu_resize = !self.settings.gpu_resize,
                    Toggle::Animated => {
                        self.settings.appearance.animated = !self.settings.appearance.animated
                    }
                    Toggle::MinimizeOnRecord => {
                        self.settings.appearance.minimize_on_record =
                            !self.settings.appearance.minimize_on_record
                    }
                    Toggle::ExcludeSelf => {
                        self.settings.appearance.exclude_self =
                            !self.settings.appearance.exclude_self
                    }
                    Toggle::Notifications => {
                        self.settings.notifications = !self.settings.notifications
                    }
                    Toggle::Tray => {
                        self.settings.tray = !self.settings.tray;
                    }
                    Toggle::Startup => {
                        let next = !self.settings.startup;
                        win::host::set_startup(next)?;
                        self.settings.startup = next;
                    }
                }
                self.save_settings();
            }
            Action::ChooseFolder(index) => {
                if self.state != RecordingState::Idle {
                    return Err(
                        crate::i18n::tr("Finalize a gravação antes de alterar as pastas").into(),
                    );
                }
                let current = match index {
                    0 => &self.settings.videos,
                    1 => &self.settings.screenshots,
                    2 => &self.settings.cache,
                    3 => &self.settings.exports,
                    5 => &self.settings.audios,
                    _ => &self.settings.projects,
                };
                if let Some(path) = win::files::choose_folder(owner, current)? {
                    crate::persistence::test_folder(&path)?;
                    match index {
                        0 => self.settings.videos = path,
                        1 => self.settings.screenshots = path,
                        2 => self.settings.cache = path,
                        3 => self.settings.exports = path,
                        5 => self.settings.audios = path,
                        _ => self.settings.projects = path,
                    };
                    self.save_settings();
                    if index == 0 {
                        self.library.scan(&self.settings.videos);
                        self.library.scan(&self.settings.audios);
                        self.refresh_metadata();
                    }
                    self.notify(crate::i18n::tr("Pasta validada e salva"), false);
                    if matches!(self.modal, Some(Modal::ErrorDetails)) {
                        self.modal = None;
                        self.last_error = None;
                    }
                }
            }
            Action::OpenChoice(kind, anchor) => {
                if self.state != RecordingState::Idle
                    && !matches!(
                        kind,
                        ChoiceKind::Interface | ChoiceKind::Theme | ChoiceKind::Language
                    )
                {
                    return Err(crate::i18n::tr(
                        "Finalize a gravação antes de alterar a configuração",
                    )
                    .into());
                }
                self.open_choice(kind, anchor, owner)?;
            }
            Action::SelectChoice(kind, index) => {
                if self.state != RecordingState::Idle
                    && !matches!(
                        kind,
                        ChoiceKind::Interface | ChoiceKind::Theme | ChoiceKind::Language
                    )
                {
                    return Err(crate::i18n::tr(
                        "Finalize a gravação antes de alterar a configuração",
                    )
                    .into());
                }
                self.apply_choice(kind, index)?;
                self.popup = None;
                self.save_settings();
            }
            Action::SettingsCategory(i) => {
                self.category = i;
                self.scroll = 0.;
                self.focused = None;
                self.popup = None;
            }
            Action::Focus(field) => {
                self.focused = Some(field);
                self.select_all = false;
            }
            Action::Preview => {
                if self.preview_enabled {
                    self.stop_preview();
                } else {
                    let target = self.target()?;
                    let stop = Arc::new(AtomicBool::new(false));
                    let s = stop.clone();
                    let preview = self.preview.clone();
                    let tx = self.background_tx.clone();
                    let worker = thread::spawn(move || {
                        let result = (|| -> AppResult<()> {
                            let _runtime = win::Runtime::new(true)?;
                            let mut capture = win::capture::Capture::new(&target)?;
                            while !s.load(Ordering::Relaxed) {
                                if let Some(f) = capture.frame()? {
                                    let width = 960.min(f.width);
                                    let height = (width * f.height / f.width).max(1);
                                    if let Ok(mut p) = preview.lock() {
                                        *p = Some(f.resize(width, height));
                                    }
                                }
                                thread::sleep(Duration::from_millis(100));
                            }
                            Ok(())
                        })();
                        if let Err(e) = result {
                            let _ = tx.send(Background::Notice(
                                crate::i18n::format_text(
                                    crate::i18n::tr("Prévia: {e}"),
                                    &[("e", format!("{}", e))],
                                ),
                                true,
                            ));
                        }
                    });
                    self.preview_worker = Some(PreviewWorker {
                        stop,
                        thread: Some(worker),
                    });
                    self.preview_enabled = true;
                }
            }
            Action::OpenVideos => win::files::open_path(owner, &self.settings.videos),
            Action::OpenSelectedFolder => {
                if let Some(r) = self.selected.and_then(|i| self.library.items.get(i)) {
                    win::files::reveal(owner, &r.path);
                }
            }
            Action::Screenshot => {
                self.modal = Some(Modal::Screenshot);
                self.popup = None;
            }
            Action::TakeScreenshot => {
                if self.screenshot_pending {
                    return Err(crate::i18n::tr("Aguarde a captura de tela em andamento.").into());
                }
                if self.state == RecordingState::Idle {
                    self.refresh_devices();
                }
                let original = self.settings.source;
                self.settings.source = match self.settings.screenshot_mode {
                    0 => Source::Display,
                    1 => Source::Window,
                    2 => Source::Region,
                    _ => original,
                };
                let target = self.target();
                self.settings.source = original;
                let mut target = target?;
                target.cursor = self.settings.screenshot_cursor;
                let path = self
                    .settings
                    .output_path(&self.settings.screenshots, "png")?;
                let tx = self.background_tx.clone();
                let delay = self.settings.screenshot_delay;
                let clipboard = self.settings.screenshot_clipboard;
                self.modal = None;
                self.popup = None;
                self.screenshot_pending = true;
                thread::spawn(move || {
                    let mut copied_image = None;
                    let result = (|| {
                        // Lets the dialog close before acquiring the first frame.
                        thread::sleep(
                            Duration::from_millis(250) + Duration::from_secs(delay as u64),
                        );
                        let _r = win::Runtime::new(true)?;
                        let mut c = win::capture::Capture::new(&target)?;
                        for _ in 0..100 {
                            if let Some(f) = c.frame()? {
                                win::graphics::save_frame(&f, &path)?;
                                if clipboard {
                                    copied_image = Some((path.clone(), f));
                                }
                                return Ok(path);
                            }
                            thread::sleep(Duration::from_millis(20));
                        }
                        Err(crate::i18n::tr("A captura não forneceu uma imagem").into())
                    })();
                    let _ = tx.send(Background::ScreenshotDone(result));
                    if let Some((path, frame)) = copied_image {
                        let _ = tx.send(Background::ScreenshotImage(path, frame));
                    }
                });
                self.notify(crate::i18n::tr("Capturando imagem…"), false);
            }
            Action::SaveProfile => {
                let p = self.settings.save_profile()?;
                self.notify(
                    {
                        let __wyzar_i18n_0 = &(p.file_name().unwrap_or_default().to_string_lossy());
                        crate::i18n::format_text(
                            crate::i18n::tr("Perfil salvo: {}"),
                            &[("0", format!("{}", __wyzar_i18n_0))],
                        )
                    },
                    false,
                );
            }
            Action::StartRecording => {
                if self.state == RecordingState::Idle {
                    self.refresh_devices();
                }
                if self.performance_running {
                    return Err(
                        crate::i18n::tr("Aguarde o teste de desempenho de 5 segundos").into(),
                    );
                }
                let audio_only = self.settings.recording_mode == 3;
                if audio_only && !self.settings.microphone && !self.settings.system_audio {
                    return Err(crate::i18n::tr("Ative pelo menos uma fonte de áudio.").into());
                }
                if !audio_only && self.probing_capabilities {
                    return Err(crate::i18n::tr(
                        "Aguarde a verificação de formatos antes de gravar.",
                    )
                    .into());
                }
                if !audio_only
                    && !self.settings.auto_fallback
                    && crate::encoder::capabilities::label(
                        &self.capabilities,
                        self.settings.width,
                        self.settings.height,
                        self.settings.fps,
                    ) == crate::i18n::tr("Indisponível")
                {
                    return Err(crate::i18n::tr("O encoder não aceitou este perfil neste dispositivo. Escolha outra resolução ou FPS.").into());
                }
                if self.recovering {
                    return Err(crate::i18n::tr("Aguarde a recuperação em andamento.").into());
                }
                if self.state != RecordingState::Idle {
                    return Ok(());
                }
                let target = if audio_only {
                    None
                } else {
                    Some(self.target()?)
                };
                let folder = if audio_only {
                    self.settings.audios.clone()
                } else {
                    self.settings.videos.clone()
                };
                crate::persistence::test_folder(&folder)?;
                if self.settings.storage_limit_gb > 0 {
                    let used: u64 = self
                        .library
                        .items
                        .iter()
                        .filter(|r| !r.missing)
                        .map(|r| r.bytes)
                        .sum();
                    if used >= self.settings.storage_limit_gb as u64 * 1_073_741_824 {
                        return Err(crate::i18n::tr("O limite da biblioteca foi atingido. Ajuste o limite em Armazenamento.").into());
                    }
                }
                if let Some((free, _)) = win::files::free_space(&folder) {
                    if free < 256 * 1024 * 1024 {
                        return Err(
                            crate::i18n::tr("Espaço insuficiente para iniciar a gravação").into(),
                        );
                    }
                    if free < (self.settings.low_space_gb as u64) * 1_073_741_824 {
                        self.notify(
                            crate::i18n::tr("Pouco espaço livre na pasta de destino"),
                            true,
                        );
                    }
                }
                let path = self.settings.output_path(
                    &folder,
                    if audio_only {
                        "wav"
                    } else {
                        crate::container::VIDEO_EXTENSION
                    },
                )?;
                self.stop_preview();
                self.webcam_worker = None;
                self.modal = None;
                self.pending_record = Some((target, path));
                self.seconds = 0.;
                self.frames = 0;
                self.state = RecordingState::Countdown;
                self.countdown_deadline =
                    Some(Instant::now() + Duration::from_secs(self.settings.countdown as u64));
            }
            Action::Pause => {
                if let Some(session) = &self.session {
                    let pause = self.state == RecordingState::Recording;
                    session.pause(pause);
                    self.state = if pause {
                        RecordingState::Paused
                    } else {
                        RecordingState::Recording
                    };
                }
            }
            Action::Stop => {
                if let Some(session) = &self.session {
                    session.stop();
                    self.state = RecordingState::Finalizing;
                } else if self.state == RecordingState::Countdown {
                    self.pending_record = None;
                    self.countdown_deadline = None;
                    self.state = RecordingState::Idle;
                }
            }
            Action::Select(i) => {
                self.selected = Some(i);
            }
            Action::PlaySelected => {
                if let Some(r) = self.selected.and_then(|i| self.library.items.get(i)) {
                    win::files::open_path(owner, &r.path);
                }
            }
            Action::EditSelected => {
                if let Some(r) = self
                    .selected
                    .and_then(|i| self.library.items.get(i))
                    .cloned()
                {
                    if r.path
                        .extension()
                        .is_some_and(|e| e.eq_ignore_ascii_case("wav"))
                    {
                        return Err(crate::i18n::tr("O editor atual trabalha com vídeo. Reproduza, renomeie ou organize este WAV pela biblioteca.").into());
                    }
                    self.checkpoint();
                    self.project.import(r.path.clone(), r.duration.max(1.));
                    self.page = Page::Editor;
                    self.project_file = None;
                    self.request_frame();
                }
            }
            Action::Favorite(i) => {
                if let Some(r) = self.library.items.get_mut(i) {
                    r.favorite = !r.favorite;
                    Library::persist_record(r)?;
                }
            }
            Action::Rename => {
                let r = self
                    .selected
                    .and_then(|i| self.library.items.get(i))
                    .ok_or(crate::i18n::tr("Selecione uma gravação"))?;
                self.text_buffer = r.title.clone();
                self.modal = Some(Modal::Text {
                    field: Field::Rename,
                    title: crate::i18n::tr("Renomear gravação").into(),
                });
                self.focused = Some(Field::Rename);
            }
            Action::Tags => {
                let r = self
                    .selected
                    .and_then(|i| self.library.items.get(i))
                    .ok_or(crate::i18n::tr("Selecione uma gravação"))?;
                self.text_buffer = r.tags.join(", ");
                self.modal = Some(Modal::Text {
                    field: Field::Tags,
                    title: crate::i18n::tr("Tags separadas por vírgula").into(),
                });
                self.focused = Some(Field::Tags);
            }
            Action::Delete => {
                if self.selected.is_some() {
                    self.modal = Some(Modal::Delete);
                }
            }
            Action::ConfirmDelete => {
                if let Some(i) = self.selected {
                    if let Some(r) = self.library.items.get(i) {
                        win::files::recycle_many(owner, &crate::library::bundle::files(&r.path))?;
                    }
                    self.library.items.remove(i);
                    self.selected = None;
                    self.library.save_index(&self.settings.data_dir)?;
                }
                self.modal = None;
            }
            Action::Import => {
                if let Some(path) = win::files::choose_file(owner, false)? {
                    self.import_path(path)?;
                }
            }
            Action::Reindex => {
                self.library.scan(&self.settings.videos);
                self.library.scan(&self.settings.audios);
                self.refresh_metadata();
                self.library.save_index(&self.settings.data_dir)?;
                self.notify(crate::i18n::tr("Biblioteca reindexada"), false);
            }
            Action::Relink => {
                if let Some(i) = self.selected {
                    if let Some(path) = win::files::choose_file(owner, false)? {
                        self.library.relink(i, path)?;
                        self.library.save_index(&self.settings.data_dir)?;
                        self.refresh_metadata();
                    }
                }
            }
            Action::LibraryTab(i) => {
                self.library_tab = i;
                self.scroll = 0.;
            }
            Action::Sort => self.oldest = !self.oldest,
            Action::ListView => {
                self.list_view = !self.list_view;
                self.scroll = 0.;
            }
            Action::SaveProject => {
                if self.project.clips.is_empty() {
                    return Err(crate::i18n::tr("Importe um vídeo para criar seu projeto").into());
                }
                crate::persistence::test_folder(&self.settings.projects)?;
                let path = self.project_file.clone().unwrap_or_else(|| {
                    self.settings.projects.join(format!(
                        "Projeto_{}.wyzproj",
                        crate::settings::unique_stamp()
                    ))
                });
                self.project.save(&path)?;
                self.project.dirty = false;
                self.project_file = Some(path);
                self.notify(crate::i18n::tr("Projeto salvo"), false);
            }
            Action::OpenProject => {
                if let Some(path) = win::files::choose_file(owner, true)? {
                    self.open_project_path(path)?;
                }
            }
            Action::Split => {
                let mut project = self.project.clone();
                project.split()?;
                self.checkpoint();
                self.project = project;
                self.request_frame();
            }
            Action::Trim => {
                let mut project = self.project.clone();
                project.trim_start()?;
                self.checkpoint();
                self.project = project;
                self.request_frame();
            }
            Action::RemoveClip => {
                let mut project = self.project.clone();
                project.remove_selected()?;
                self.checkpoint();
                self.project = project;
                self.request_frame();
            }
            Action::Speed => {
                self.checkpoint();
                if let Some(c) = self.project.clips.get_mut(self.project.selected) {
                    c.speed = if c.speed >= 2. { 0.5 } else { c.speed + 0.5 };
                    self.project.dirty = true;
                }
                self.project.playhead = self.project.playhead.min(self.project.duration());
                self.request_frame();
            }
            Action::Volume => {
                self.checkpoint();
                if let Some(c) = self.project.clips.get_mut(self.project.selected) {
                    c.volume = if c.volume >= 1.5 { 0. } else { c.volume + 0.25 };
                    self.project.dirty = true;
                }
            }
            Action::Slider(_, _) => {}
            Action::SetVolume(value) => {
                if let Some(c) = self.project.clips.get_mut(self.project.selected) {
                    c.volume = value.clamp(0., 1.5);
                    self.project.dirty = true;
                }
            }
            Action::SetSpeed(value) => {
                if let Some(c) = self.project.clips.get_mut(self.project.selected) {
                    c.speed = value.clamp(0.25, 2.);
                    self.project.dirty = true;
                }
                self.project.playhead = self.project.playhead.min(self.project.duration());
                self.request_frame();
            }
            Action::Text => {
                self.text_buffer = String::new();
                self.modal = Some(Modal::Text {
                    field: Field::Text,
                    title: crate::i18n::tr("Adicionar texto ao vídeo").into(),
                });
                self.focused = Some(Field::Text);
            }
            Action::Arrow => {
                self.checkpoint();
                self.project.arrow = !self.project.arrow;
                self.project.dirty = true;
                self.request_frame();
            }
            Action::Blur => {
                self.checkpoint();
                self.project.blur = !self.project.blur;
                self.project.dirty = true;
                self.request_frame();
            }
            Action::Fade => {
                self.checkpoint();
                self.project.fade = !self.project.fade;
                self.project.dirty = true;
                self.request_frame();
            }
            Action::Crop => {
                self.checkpoint();
                self.project.crop = !self.project.crop;
                self.project.dirty = true;
                self.request_frame();
            }
            Action::Play => {
                if !self.project.clips.is_empty() {
                    if self.project.playhead >= self.project.duration() {
                        self.project.playhead = 0.;
                    }
                    self.playing = !self.playing;
                    self.audio_playback = if self.playing {
                        Some(win::playback::Playback::start(
                            self.project.clone(),
                            self.project.playhead,
                        ))
                    } else {
                        None
                    };
                    self.last_tick = Instant::now();
                }
            }
            Action::Seek(t) => {
                self.project.playhead = t.clamp(0., self.project.duration().max(0.));
                self.request_frame();
                if self.playing {
                    self.audio_playback = Some(win::playback::Playback::start(
                        self.project.clone(),
                        self.project.playhead,
                    ));
                }
            }
            Action::SelectClip(i) => {
                if i < self.project.clips.len() {
                    self.project.selected = i;
                }
            }
            Action::Undo => {
                if let Some(p) = self.undo.pop() {
                    self.redo.push(self.project.clone());
                    self.project = p;
                    self.project.dirty = true;
                    self.request_frame();
                }
            }
            Action::Redo => {
                if let Some(p) = self.redo.pop() {
                    self.undo.push(self.project.clone());
                    self.project = p;
                    self.project.dirty = true;
                    self.request_frame();
                }
            }
            Action::Export => {
                if self.export_progress.is_some() {
                    return Err(crate::i18n::tr("Uma exportação já está em andamento").into());
                }
                if self.page != Page::Editor {
                    if let Some(r) = self
                        .selected
                        .and_then(|i| self.library.items.get(i))
                        .cloned()
                    {
                        self.project = Project::default();
                        self.project.import(r.path, r.duration.max(0.1));
                    } else {
                        return Err(crate::i18n::tr("Selecione uma gravação para exportar").into());
                    }
                }
                if self.project.clips.is_empty() {
                    return Err(crate::i18n::tr("Importe um vídeo para exportar").into());
                }
                self.export_name = self.project.name.clone();
                self.modal = Some(Modal::Export);
            }
            Action::BeginExport => {
                if self.export_progress.is_some() {
                    return Err(crate::i18n::tr("Uma exportação já está em andamento").into());
                }
                if !crate::settings::valid_pattern(&self.export_name) {
                    return Err(crate::i18n::tr("Informe um nome de arquivo válido").into());
                }
                crate::persistence::test_folder(&self.settings.exports)?;
                let path = self
                    .settings
                    .exports
                    .join(format!("{}.mp4", self.export_name));
                if path.exists() {
                    return Err(crate::i18n::tr(
                        "Já existe um arquivo com esse nome. Escolha outro nome.",
                    )
                    .into());
                }
                let p = self.project.clone();
                let cfg = crate::encoder::VideoConfig::from_settings(&self.settings);
                let tx = self.background_tx.clone();
                self.export_progress = Some(0.);
                self.modal = None;
                thread::spawn(move || {
                    let result = crate::editor::export::export(&p, cfg, &path, |v| {
                        let _ = tx.send(Background::Export(v));
                    })
                    .map(|_| path);
                    let _ = tx.send(Background::ExportDone(result));
                });
            }
            Action::ConfirmText => {
                let field = self.focused;
                match field {
                    Some(Field::Appearance(index)) => {
                        let mut a = self.settings.appearance.clone();
                        if index == 5 {
                            let colors = self
                                .text_buffer
                                .split(',')
                                .map(crate::settings::appearance::hex)
                                .collect::<Result<Vec<_>, _>>()?;
                            if colors.len() != 4 {
                                return Err(crate::i18n::tr(
                                    "Informe quatro cores: destaque, fundo, painéis e texto.",
                                )
                                .into());
                            }
                            a.accent = colors[0];
                            a.background = colors[1];
                            a.surface = colors[2];
                            a.text = colors[3];
                            a.theme = 8;
                        } else if index == 4 && self.text_buffer.trim().eq_ignore_ascii_case("auto")
                        {
                            a.icon_tint = None;
                        } else {
                            let color = crate::settings::appearance::hex(&self.text_buffer)?;
                            match index {
                                0 => a.accent = color,
                                1 => a.background = color,
                                2 => a.surface = color,
                                3 => a.text = color,
                                _ => a.icon_tint = Some(color),
                            }
                        }
                        a.validate()?;
                        if index < 4 {
                            a.theme = 8;
                        }
                        self.settings.appearance = a;
                        self.save_settings();
                    }
                    Some(Field::Rename) => {
                        if !crate::settings::valid_pattern(&self.text_buffer) {
                            return Err(crate::i18n::tr("Nome de arquivo inválido").into());
                        }
                        if let Some(i) = self.selected {
                            self.library.rename(i, &self.text_buffer)?;
                            self.library.save_index(&self.settings.data_dir)?;
                        }
                    }
                    Some(Field::Tags) => {
                        if let Some(r) = self.selected.and_then(|i| self.library.items.get_mut(i)) {
                            r.tags = self
                                .text_buffer
                                .split(',')
                                .map(str::trim)
                                .filter(|s| !s.is_empty())
                                .take(20)
                                .map(str::to_owned)
                                .collect();
                            Library::persist_record(r)?;
                        }
                    }
                    Some(Field::Text) => {
                        if !self.text_buffer.is_empty() {
                            self.checkpoint();
                            self.project.text.push(crate::editor::TextOverlay {
                                text: self.text_buffer.clone(),
                                start: self.project.playhead,
                                end: (self.project.playhead + 5.).min(self.project.duration()),
                            });
                            self.project.dirty = true;
                            self.request_frame();
                        }
                    }
                    _ => {}
                }
                self.modal = None;
                self.focused = None;
            }
            Action::RefreshDevices => {
                self.refresh_devices();
                self.notify(crate::i18n::tr("Dispositivos atualizados"), false);
            }
        }
        Ok(())
    }
    pub fn open_project_path(&mut self, path: PathBuf) -> AppResult<()> {
        let project = Project::load(&path)?;
        self.playing = false;
        self.audio_playback = None;
        self.project = project;
        self.project_file = Some(path);
        self.undo.clear();
        self.redo.clear();
        self.page = Page::Editor;
        self.focused = None;
        self.request_frame();
        Ok(())
    }
    pub fn import_path(&mut self, path: PathBuf) -> AppResult<()> {
        if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("wav"))
        {
            crate::audio::wav::duration(&path)?;
            if self.page == Page::Editor {
                let end = crate::audio::wav::duration(&path)?;
                if self.project.clips.is_empty() {
                    return Err(crate::i18n::tr(
                        "Importe primeiro o vídeo e depois suas trilhas WAV.",
                    )
                    .into());
                }
                self.checkpoint();
                self.project
                    .audio_tracks
                    .push(crate::editor::tracks::Track {
                        source: path,
                        kind: 2,
                        offset: self.project.playhead,
                        start: 0.,
                        end,
                        speed: 1.,
                        gain: 1.,
                        mute: false,
                        solo: false,
                        fade_in: 0.,
                        fade_out: 0.,
                    });
                self.project.tracks_active = true;
                self.project.dirty = true;
                self.modal = Some(Modal::AudioTracks);
                return Ok(());
            }
            self.library.import(&path)?;
            self.refresh_metadata();
            return self.library.save_index(&self.settings.data_dir);
        }
        let metadata = win::media::Reader::open(&path)?.meta;
        self.library.import(&path)?;
        self.refresh_metadata();
        if self.page == Page::Editor {
            self.playing = false;
            self.audio_playback = None;
            self.checkpoint();
            self.project.import(path, metadata.duration);
            self.request_frame();
        }
        self.library.save_index(&self.settings.data_dir)?;
        Ok(())
    }
    pub fn refresh_devices(&mut self) {
        if self.state != RecordingState::Idle {
            return;
        }
        self.monitors = win::devices::monitors();
        self.microphones = win::devices::audio_devices(true);
        self.outputs = win::devices::audio_devices(false);
        self.webcams = win::devices::cameras();
        if let Ok(inventory) = win::capabilities::inventory() {
            if inventory.adapters != self.inventory.adapters
                || inventory.hardware() != self.inventory.hardware()
            {
                self.capabilities.clear();
            }
            self.inventory = inventory;
        }
        self.reconcile_devices();
    }
    fn reconcile_devices(&mut self) {
        if self.settings.monitor >= self.monitors.len() {
            self.settings.monitor = 0;
        }
        for (selected, devices) in [
            (&mut self.settings.microphone_id, &self.microphones),
            (&mut self.settings.output_id, &self.outputs),
        ] {
            if !selected.is_empty() && !devices.iter().any(|d| d.id == *selected) {
                selected.clear();
            }
        }
        if !self.settings.webcam_id.is_empty()
            && !self.webcams.iter().any(|d| d.id == self.settings.webcam_id)
        {
            self.settings.webcam_id = self
                .webcams
                .first()
                .map(|d| d.id.clone())
                .unwrap_or_default();
        }
    }
    fn open_choice(&mut self, kind: ChoiceKind, anchor: Rect, owner: HWND) -> AppResult<()> {
        use ChoiceKind::*;
        let values = match kind {
            Language => crate::i18n::Language::ALL
                .iter()
                .map(|l| l.name().to_owned())
                .collect(),
            RecordingMode => crate::audio::MODE_LABELS
                .iter()
                .map(|s| crate::i18n::tr(s).to_string())
                .collect(),
            MicGain | SystemGain => [0, 25, 50, 75, 100, 125, 150, 200]
                .iter()
                .map(|g| format!("{g}%"))
                .collect(),
            Interface => InterfaceMode::ALL
                .iter()
                .enumerate()
                .map(|(i, m)| {
                    let __wyzar_i18n_0 = &(m.label());
                    let __wyzar_i18n_1 = &(i + 1);
                    crate::i18n::format_text(
                        crate::i18n::tr("{}    Ctrl+Alt+{}"),
                        &[
                            ("0", format!("{}", __wyzar_i18n_0)),
                            ("1", format!("{}", __wyzar_i18n_1)),
                        ],
                    )
                })
                .collect(),
            CaptureSource => Source::ALL.iter().map(|s| s.label().to_owned()).collect(),
            Theme => crate::ui::theme::PRESETS
                .iter()
                .map(|p| crate::i18n::tr(p.name).to_owned())
                .chain(std::iter::once(crate::i18n::tr("Personalizado").into()))
                .collect(),
            IconStyle => [
                crate::i18n::tr("Vinculado ao tema"),
                crate::i18n::tr("Contorno suave"),
                crate::i18n::tr("Duo com destaque"),
                crate::i18n::tr("Técnico"),
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            IconWeight => [
                crate::i18n::tr("Fino"),
                crate::i18n::tr("Equilibrado"),
                crate::i18n::tr("Forte"),
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            Resolution => vec![
                crate::i18n::tr("1280 × 720 · HD"),
                crate::i18n::tr("1920 × 1080 · Full HD"),
                crate::i18n::tr("2560 × 1440 · QHD"),
                crate::i18n::tr("3840 × 2160 · 4K"),
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            Fps => [24, 30, 60, 120]
                .iter()
                .map(|n| {
                    let __wyzar_i18n_0 = &(crate::encoder::capabilities::label(
                        &self.capabilities,
                        self.settings.width,
                        self.settings.height,
                        *n,
                    ));
                    crate::i18n::format_text(
                        crate::i18n::tr("{n} FPS · {}"),
                        &[
                            ("n", format!("{}", n)),
                            ("0", format!("{}", __wyzar_i18n_0)),
                        ],
                    )
                })
                .collect(),
            Bitrate => [5, 10, 20, 40, 60]
                .iter()
                .map(|n| {
                    crate::i18n::format_text(
                        crate::i18n::tr("{n} Mbps"),
                        &[("n", format!("{}", n))],
                    )
                })
                .collect(),
            Monitor => self.monitors.iter().map(|d| d.name.clone()).collect(),
            Microphone => std::iter::once(crate::i18n::tr("Padrão do Windows").into())
                .chain(self.microphones.iter().map(|d| d.name.clone()))
                .collect(),
            Output => std::iter::once(crate::i18n::tr("Padrão do Windows").into())
                .chain(self.outputs.iter().map(|d| d.name.clone()))
                .collect(),
            Webcam => self.webcams.iter().map(|d| d.name.clone()).collect(),
            Countdown => [0, 3, 5, 10]
                .iter()
                .map(|n| {
                    if *n == 0 {
                        crate::i18n::tr("Sem contagem").into()
                    } else {
                        crate::i18n::format_text(
                            crate::i18n::tr("{n} segundos"),
                            &[("n", format!("{}", n))],
                        )
                    }
                })
                .collect(),
            Naming => vec![
                "Wyzar_{data}_{hora}".into(),
                "Captura_{contador}".into(),
                "MeuVideo_{data}_{contador}".into(),
            ],
            Subfolders => vec![
                crate::i18n::tr("Sem subpastas").into(),
                crate::i18n::tr("Por data").into(),
                crate::i18n::tr("Por tipo").into(),
                crate::i18n::tr("Por projeto").into(),
            ],
            Encoder => vec![
                crate::i18n::tr("Automático · hardware preferido").into(),
                crate::i18n::tr("Software · Media Foundation").into(),
            ],
            Window => {
                if (matches!(self.modal, Some(Modal::Screenshot))
                    && (self.settings.screenshot_mode == 2
                        || (self.settings.screenshot_mode == 3
                            && self.settings.source == Source::Region)))
                    || (!matches!(self.modal, Some(Modal::Screenshot))
                        && self.settings.source == Source::Region)
                {
                    if let Some(r) = win::selection::select_region(owner)? {
                        self.region = Some(r);
                    }
                    return Ok(());
                }
                self.window_options = win::devices::windows(owner.0 as isize);
                self.window_options
                    .iter()
                    .take(14)
                    .map(|w| w.name.clone())
                    .collect()
            }
            WebcamPosition => vec![
                crate::i18n::tr("Superior esquerda"),
                crate::i18n::tr("Superior direita"),
                crate::i18n::tr("Inferior esquerda"),
                crate::i18n::tr("Inferior direita"),
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            WebcamShape => webcam_shapes().into_iter().map(str::to_owned).collect(),
            WebcamSize => [15, 22, 30, 40]
                .iter()
                .map(|n| {
                    crate::i18n::format_text(
                        crate::i18n::tr("{n}% da largura"),
                        &[("n", format!("{}", n))],
                    )
                })
                .collect(),
            RecordKey | PauseKey | StopKey => {
                self.focused = None;
                self.modal = Some(Modal::Shortcut(match kind {
                    RecordKey => 0,
                    PauseKey => 1,
                    _ => 2,
                }));
                self.popup = None;
                return Ok(());
            }
            ScreenshotKey => {
                self.focused = None;
                self.modal = Some(Modal::Shortcut(3));
                self.popup = None;
                return Ok(());
            }
            ScreenshotSource => ["Tela inteira", "Janela", "Região", "Usar fonte da gravação"]
                .into_iter()
                .map(|s| crate::i18n::tr(s).into())
                .collect(),
            ScreenshotDelay => [0, 3, 5, 10].iter().map(|n| format!("{n} s")).collect(),
            SpaceAlert => [1, 2, 5, 10, 20]
                .iter()
                .map(|n| {
                    crate::i18n::format_text(crate::i18n::tr("{n} GB"), &[("n", format!("{}", n))])
                })
                .collect(),
            StorageLimit => [0, 10, 50, 100, 250, 500]
                .iter()
                .map(|n| {
                    if *n == 0 {
                        crate::i18n::tr("Sem limite").into()
                    } else {
                        crate::i18n::format_text(
                            crate::i18n::tr("{n} GB"),
                            &[("n", format!("{}", n))],
                        )
                    }
                })
                .collect(),
            Profile => {
                self.profile_paths = std::fs::read_dir(self.settings.data_dir.join("Profiles"))
                    .into_iter()
                    .flatten()
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| p.extension().is_some_and(|s| s == "ini"))
                    .collect();
                self.profile_paths
                    .iter()
                    .map(|p| p.file_stem().unwrap_or_default().to_string_lossy().into())
                    .collect()
            }
        };
        if values.is_empty() {
            return Err(crate::i18n::tr(
                "Nenhuma opção disponível. Atualize os dispositivos ou salve um perfil.",
            )
            .into());
        }
        let height = (values.len() as f32 * 35. + 12.)
            .min(505.)
            .min((self.viewport.1 - 74.).max(82.));
        let y = if anchor.bottom() + height > self.viewport.1 - 10. {
            anchor.y - height - 5.
        } else {
            anchor.bottom() + 5.
        };
        self.popup = Some(Popup {
            rect: Rect::new(
                anchor.x,
                y.max(58.),
                anchor
                    .w
                    .max(if matches!(kind, ChoiceKind::Fps | ChoiceKind::Interface) {
                        302.
                    } else {
                        220.
                    }),
                height,
            ),
            kind,
            values,
            first_visible: 0,
        });
        if let Some(popup) = self.popup.as_mut() {
            popup.rect.x = popup
                .rect
                .x
                .min(self.viewport.0 - popup.rect.w - 12.)
                .max(12.);
            popup.rect.y = popup
                .rect
                .y
                .min(self.viewport.1 - popup.rect.h - 12.)
                .max(58.);
        }
        self.focus_index = Some(0);
        Ok(())
    }
    fn apply_choice(&mut self, kind: ChoiceKind, i: usize) -> AppResult<()> {
        use ChoiceKind::*;
        match kind {
            Language => {
                self.settings.language = crate::i18n::Language::ALL[i.min(4)];
                crate::i18n::set(self.settings.language);
            }
            RecordingMode => {
                self.settings.recording_mode = i.min(3);
                self.stop_preview();
                self.webcam_worker = None;
            }
            MicGain => self.settings.mic_gain = [0, 25, 50, 75, 100, 125, 150, 200][i.min(7)],
            SystemGain => self.settings.system_gain = [0, 25, 50, 75, 100, 125, 150, 200][i.min(7)],
            Interface => {
                self.perform(
                    Action::SetInterface(InterfaceMode::ALL[i.min(3)]),
                    HWND::default(),
                )?;
            }
            CaptureSource => {
                if self.state != RecordingState::Idle {
                    return Err(
                        crate::i18n::tr("Finalize a gravação antes de alterar a fonte.").into(),
                    );
                }
                self.settings.source = Source::ALL[i.min(3)];
                self.stop_preview();
            }
            Theme => self.settings.appearance.theme = i.min(8),
            IconStyle => self.settings.appearance.icon_style = i.min(3),
            IconWeight => self.settings.appearance.icon_weight = i.min(2),
            Resolution => {
                let (w, h) = [(1280, 720), (1920, 1080), (2560, 1440), (3840, 2160)][i.min(3)];
                self.settings.width = w;
                self.settings.height = h;
                self.settings.bitrate =
                    crate::encoder::recommended_bitrate(w, h, self.settings.fps);
            }
            Fps => {
                let fps = [24, 30, 60, 120][i.min(3)];
                if crate::encoder::capabilities::label(
                    &self.capabilities,
                    self.settings.width,
                    self.settings.height,
                    fps,
                ) == crate::i18n::tr("Indisponível")
                {
                    return Err(crate::i18n::tr(
                        "Este FPS não foi aceito pelo encoder neste perfil.",
                    )
                    .into());
                }
                self.settings.fps = fps;
                self.settings.bitrate = crate::encoder::recommended_bitrate(
                    self.settings.width,
                    self.settings.height,
                    fps,
                );
            }
            Bitrate => self.settings.bitrate = [5, 10, 20, 40, 60][i.min(4)] * 1_000_000,
            Monitor => self.settings.monitor = i,
            Microphone => {
                self.settings.microphone_id = if i == 0 {
                    String::new()
                } else {
                    self.microphones
                        .get(i - 1)
                        .map(|d| d.id.clone())
                        .unwrap_or_default()
                }
            }
            Output => {
                self.settings.output_id = if i == 0 {
                    String::new()
                } else {
                    self.outputs
                        .get(i - 1)
                        .map(|d| d.id.clone())
                        .unwrap_or_default()
                }
            }
            Webcam => {
                self.webcam_worker = None;
                if let Ok(mut f) = self.webcam_frame.lock() {
                    *f = None;
                }
                self.settings.webcam_id = self
                    .webcams
                    .get(i)
                    .map(|d| d.id.clone())
                    .unwrap_or_default();
                if self.settings.webcam || matches!(self.modal, Some(Modal::Camera)) {
                    self.start_webcam_preview();
                }
            }
            Countdown => self.settings.countdown = [0, 3, 5, 10][i.min(3)],
            Naming => {
                self.settings.naming = [
                    "Wyzar_{data}_{hora}",
                    "Captura_{contador}",
                    "MeuVideo_{data}_{contador}",
                ][i.min(2)]
                .into()
            }
            Subfolders => {
                self.settings.subfolders = ["none", "date", "type", "project"][i.min(3)].into()
            }
            Encoder => {
                self.settings.hardware = i == 0;
                self.capabilities.clear();
            }
            Window => {
                if let Some(w) = self.window_options.get(i) {
                    self.window = Some(w.handle);
                    self.window_name = w.name.clone();
                }
            }
            WebcamPosition => self.settings.webcam_position = i.min(3),
            WebcamSize => self.settings.webcam_size = [15, 22, 30, 40][i.min(3)],
            WebcamShape => self.settings.webcam_shape = i.min(2),
            RecordKey | PauseKey | StopKey | ScreenshotKey => {}
            ScreenshotSource => self.settings.screenshot_mode = i.min(3),
            ScreenshotDelay => self.settings.screenshot_delay = [0, 3, 5, 10][i.min(3)],
            SpaceAlert => self.settings.low_space_gb = [1, 2, 5, 10, 20][i.min(4)],
            StorageLimit => self.settings.storage_limit_gb = [0, 10, 50, 100, 250, 500][i.min(5)],
            Profile => {
                if let Some(path) = self.profile_paths.get(i) {
                    let values = crate::persistence::read(path)?;
                    let mut cfg = Settings::defaults(self.settings.data_dir.clone());
                    cfg.apply_profile(&values);
                    cfg.last_seen_version = self.settings.last_seen_version.clone();
                    cfg.notified_version = self.settings.notified_version.clone();
                    cfg.appearance = self.settings.appearance.clone();
                    cfg.language = self.settings.language;
                    cfg.install_language_request = self.settings.install_language_request.clone();
                    cfg.install_storage_request = self.settings.install_storage_request.clone();
                    self.settings = cfg;
                    self.capabilities.clear();
                    self.notify(crate::i18n::tr("Perfil carregado"), false);
                }
            }
        }
        Ok(())
    }
    pub fn stop_preview(&mut self) {
        self.preview_worker = None;
        self.preview_enabled = false;
    }
    fn start_webcam_preview(&mut self) {
        self.webcam_worker = None;
        if self.webcams.is_empty() || self.state != RecordingState::Idle {
            return;
        }
        let stop = Arc::new(AtomicBool::new(false));
        let s = stop.clone();
        let shared = self.webcam_frame.clone();
        let id = self.settings.webcam_id.clone();
        let tx = self.background_tx.clone();
        let worker = thread::spawn(move || {
            let result = (|| -> AppResult<()> {
                let _r = win::Runtime::new(true)?;
                let mut cam = win::media::Webcam::open(&id)?;
                while !s.load(Ordering::Relaxed) {
                    if let Some(frame) = cam.next()? {
                        let width = frame.width.min(1280);
                        let h = (width * frame.height / frame.width.max(1)).max(1);
                        if let Ok(mut output) = shared.lock() {
                            *output = Some(frame.resize(width, h));
                        }
                    }
                    thread::sleep(Duration::from_millis(12));
                }
                Ok(())
            })();
            if let Err(e) = result {
                let _ = tx.send(Background::Notice(
                    crate::i18n::format_text(
                        crate::i18n::tr("Webcam: {e}"),
                        &[("e", format!("{}", e))],
                    ),
                    true,
                ));
            }
        });
        self.webcam_worker = Some(PreviewWorker {
            stop,
            thread: Some(worker),
        });
    }
    pub fn refresh_metadata(&self) {
        let paths: Vec<_> = self
            .library
            .items
            .iter()
            .filter(|r| !r.missing && (r.duration == 0. || r.thumbnail.is_none()))
            .map(|r| r.path.clone())
            .collect();
        let tx = self.background_tx.clone();
        thread::spawn(move || {
            let Ok(_runtime) = win::Runtime::new(true) else {
                return;
            };
            for path in paths {
                let result = if path
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("wav"))
                {
                    crate::audio::wav::duration(&path).and_then(|duration| {
                        Ok((
                            win::media::Metadata {
                                duration,
                                width: 0,
                                height: 0,
                                fps: 0.,
                                audio: true,
                            },
                            Some(crate::audio::thumbnail(&path)?),
                        ))
                    })
                } else {
                    win::media::Reader::open(&path).and_then(|mut reader| {
                        let meta = reader.meta.clone();
                        let frame = reader
                            .frame_at((meta.duration * 0.1).min(2.))?
                            .map(|f| f.resize(480, 270));
                        Ok((meta, frame))
                    })
                };
                let _ = tx.send(Background::Metadata(path, result));
            }
        });
    }
    pub fn checkpoint(&mut self) {
        self.undo.push(self.project.clone());
        if self.undo.len() > 50 {
            self.undo.remove(0);
        }
        self.redo.clear();
    }
    pub fn request_frame(&mut self) {
        self.decode_generation += 1;
        if self.decoding {
            return;
        }
        self.decode_workers_started += 1;
        let generation = self.decode_generation;
        self.decoding = true;
        let project = self.project.clone();
        let tx = self.background_tx.clone();
        thread::spawn(move || {
            let frame = (|| {
                let _r = win::Runtime::new(true).ok()?;
                let (i, t) =
                    project.locate(project.playhead.min((project.duration() - 0.001).max(0.)))?;
                let mut reader = win::media::Reader::open(&project.clips[i].source).ok()?;
                let mut f = reader.frame_at(t).ok()??;
                if f.width > 960 {
                    f = f.resize(960, (960 * f.height / f.width).max(1));
                }
                crate::editor::effects::apply(&mut f, &project, project.playhead);
                Some(f)
            })();
            let _ = tx.send(Background::Decoded(generation, frame));
        });
    }
    pub fn tick(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f64();
        self.last_tick = now;
        if self.toast.as_ref().is_some_and(|t| t.until < now) {
            self.toast = None;
        }
        if self.state == RecordingState::Countdown
            && self.countdown_deadline.is_some_and(|d| d <= now)
        {
            if let Some((target, path)) = self.pending_record.take() {
                self.state = RecordingState::Starting;
                self.session = Some(if let Some(target) = target {
                    Session::start(self.settings.clone(), target, path, self.preview.clone())
                } else {
                    Session::start_audio(self.settings.clone(), path)
                });
            }
            self.countdown_deadline = None;
        }
        let events: Vec<_> = self
            .session
            .as_ref()
            .map(|s| s.events.try_iter().collect())
            .unwrap_or_default();
        let mut finished = false;
        for event in events {
            match event {
                Event::Metrics(metrics) => self.metrics = Some(metrics),
                Event::Warning(message) => self.warn(message),
                Event::Started => self.state = RecordingState::Recording,
                Event::Progress {
                    seconds,
                    frames,
                    mic,
                    system,
                } => {
                    self.seconds = seconds;
                    self.frames = frames;
                    self.mic_level = mic;
                    self.system_level = system;
                }
                Event::Saved(info) => {
                    self.state = RecordingState::Idle;
                    self.seconds = info.duration.as_secs_f64();
                    self.frames = info.frames;
                    self.library.scan(&self.settings.videos);
                    self.library.scan(&self.settings.audios);
                    let _ = self.library.import(&info.path);
                    if let Some(r) = self.library.items.iter_mut().find(|r| r.path == info.path) {
                        r.duration = info.duration.as_secs_f64();
                        r.width = info.width;
                        r.height = info.height;
                        r.fps = if info.frames == 0 {
                            0.
                        } else {
                            self.settings.fps as f64
                        };
                        r.tags = vec![if info.frames == 0 {
                            crate::i18n::tr("Áudio").into()
                        } else {
                            self.settings.source.label().into()
                        }];
                        let _ = Library::persist_record(r);
                    }
                    let _ = self.library.save_index(&self.settings.data_dir);
                    self.refresh_metadata();
                    if self.toast.as_ref().is_some_and(|t| t.warning) {
                        self.warn(crate::i18n::tr(
                            "Gravação salva. Revise os avisos das fontes ou do armazenamento.",
                        ));
                    } else {
                        self.notify(
                            crate::i18n::tr("Gravação salva com sucesso na biblioteca"),
                            false,
                        );
                    }
                    finished = true;
                }
                Event::Error(e) => {
                    self.state = RecordingState::Idle;
                    self.recovery_items = crate::recovery::candidates(&self.settings.data_dir);
                    self.pending_recovery = crate::recovery::pending(&self.settings.data_dir).len();
                    if !self.recovery_items.is_empty() {
                        self.page = Page::Recovery;
                    }
                    self.library.scan(&self.settings.videos);
                    self.library.scan(&self.settings.audios);
                    self.refresh_metadata();
                    self.failure(e);
                    finished = true;
                }
            }
        }
        if finished {
            self.session = None;
        }
        let messages: Vec<_> = self.background_rx.try_iter().collect();
        for message in messages {
            match message {
                Background::Notice(text, error) => self.notify(text, error),
                Background::Metadata(path, Ok((meta, frame))) => {
                    if let Some(r) = self.library.items.iter_mut().find(|r| r.path == path) {
                        r.duration = meta.duration;
                        r.width = meta.width;
                        r.height = meta.height;
                        r.fps = meta.fps;
                        r.thumbnail = frame;
                        r.problem = None;
                        let _ = Library::persist_record(r);
                    }
                }
                Background::Metadata(path, Err(error)) => {
                    if let Some(r) = self.library.items.iter_mut().find(|r| r.path == path) {
                        r.problem = Some(error);
                    }
                }
                Background::Decoded(generation, frame) => {
                    self.decoding = false;
                    if generation == self.decode_generation {
                        self.editor_frame = frame;
                    } else {
                        self.request_frame();
                    }
                }
                Background::Export(p) => self.export_progress = Some(p),
                Background::Capabilities(result) => {
                    self.probing_capabilities = false;
                    match result {
                        Ok(profiles) => {
                            self.capabilities = profiles;
                            self.notify(
                                crate::i18n::tr("Formatos conferidos. O teste não certifica desempenho sustentado."),
                                false,
                            );
                        }
                        Err(error) => self.failure(error),
                    }
                }
                Background::Performance(result) => {
                    self.performance_running = false;
                    match result {
                        Ok(report) => {
                            self.metrics = Some(report.metrics.clone());
                            self.notify(&report.recommendation, false);
                            self.performance_result = Some(report);
                        }
                        Err(e) => self.failure(e),
                    }
                }
                Background::Recovered(candidate, result) => {
                    self.recovering = false;
                    match result {
                        Ok(path) => {
                            let _ = self.library.import(&path);
                            let _ = self.library.save_index(&self.settings.data_dir);
                            self.refresh_metadata();
                            let cleanup = crate::recovery::acknowledge(&candidate, &path);
                            self.recovery_items =
                                crate::recovery::candidates(&self.settings.data_dir);
                            self.pending_recovery =
                                crate::recovery::pending(&self.settings.data_dir).len();
                            self.notify(if cleanup.is_ok() { crate::i18n::tr("Gravação recuperada e adicionada à biblioteca.") } else { crate::i18n::tr("Gravação recuperada. A limpeza dos temporários precisa ser revisada.") }, cleanup.is_err());
                        }
                        Err(error) => self.notify(error, true),
                    }
                }
                Background::ExportDone(result) => {
                    self.export_progress = None;
                    match result {
                        Ok(p) => self.notify(
                            {
                                let __wyzar_i18n_0 =
                                    &(p.file_name().unwrap_or_default().to_string_lossy());
                                crate::i18n::format_text(
                                    crate::i18n::tr("Arquivo salvo: {}"),
                                    &[("0", format!("{}", __wyzar_i18n_0))],
                                )
                            },
                            false,
                        ),
                        Err(e) => self.notify(e, true),
                    }
                }
                Background::ScreenshotImage(path, frame) => {
                    if let Err(error) = win::files::set_clipboard_frame(HWND::default(), &frame) {
                        self.warn(crate::i18n::format_text(
                            crate::i18n::tr("PNG salvo; não foi possível copiar a imagem: {error}"),
                            &[("error", error)],
                        ));
                    }
                    self.last_screenshot = Some(path);
                }
                Background::ScreenshotDone(result) => {
                    self.screenshot_pending = false;
                    match result {
                        Ok(path) => {
                            self.last_screenshot = Some(path);
                            self.notify(
                                crate::i18n::tr("Captura PNG salva na sua pasta de capturas."),
                                false,
                            );
                            if let Some(t) = self.toast.as_mut() {
                                t.action = Some(Action::OpenFolder(1));
                            }
                        }
                        Err(error) => self.failure(error),
                    }
                }
            }
        }
        if self.playing {
            if let Some(error) = self.audio_playback.as_ref().and_then(|a| a.error()) {
                self.notify(
                    crate::i18n::format_text(
                        crate::i18n::tr("Áudio da prévia: {error}"),
                        &[("error", format!("{}", error))],
                    ),
                    true,
                );
                self.audio_playback = None;
            }
            self.project.playhead = (self.project.playhead + dt).min(self.project.duration());
            if self.project.playhead >= self.project.duration() {
                self.playing = false;
                self.audio_playback = None;
            }
            if !self.decoding {
                self.request_frame();
            }
        }
        if self.project.dirty && self.last_autosave.elapsed() > Duration::from_secs(30) {
            let path = self.settings.data_dir.join("Recovery/autosave.wyzproj");
            if self.project.save(&path).is_ok() {
                self.last_autosave = now;
            }
        }
    }
    pub fn text_input(&mut self, c: char) {
        let Some(field) = self.focused else {
            return;
        };
        let buffer = match field {
            Field::HelpSearch => &mut self.help_search,
            Field::IconSearch => &mut self.icon_search,
            Field::Search => &mut self.search,
            Field::Naming => &mut self.settings.naming,
            Field::ProjectName => &mut self.settings.project_name,
            Field::ExportName => &mut self.export_name,
            _ => &mut self.text_buffer,
        };
        if c == '\u{8}' {
            if self.select_all {
                buffer.clear();
            } else {
                buffer.pop();
            }
            self.select_all = false;
        } else if !c.is_control() && buffer.len() < 500 {
            if self.select_all {
                buffer.clear();
                self.select_all = false;
            }
            buffer.push(c);
        }
        if field == Field::Search {
            self.scroll = 0.;
        }
    }
    pub fn blur_text(&mut self) {
        if matches!(self.focused, Some(Field::Naming | Field::ProjectName)) {
            let value = if self.focused == Some(Field::Naming) {
                &self.settings.naming
            } else {
                &self.settings.project_name
            };
            if crate::settings::valid_pattern(value) {
                self.save_settings();
            } else {
                self.notify(crate::i18n::tr("Regra de nomeação inválida"), true);
            }
        }
        self.focused = None;
        self.select_all = false;
    }
    pub fn device_name(&self, t: Toggle) -> String {
        let (id, list) = match t {
            Toggle::Microphone => (&self.settings.microphone_id, &self.microphones),
            Toggle::SystemAudio => (&self.settings.output_id, &self.outputs),
            _ => (&self.settings.webcam_id, &self.webcams),
        };
        if id.is_empty() && t != Toggle::Webcam {
            return crate::i18n::tr("Padrão do Windows").into();
        }
        list.iter()
            .find(|d| d.id == *id)
            .or_else(|| if id.is_empty() { list.first() } else { None })
            .map(|d| d.name.clone())
            .unwrap_or_else(|| {
                if t == Toggle::Webcam {
                    crate::i18n::tr("Nenhuma câmera").into()
                } else {
                    crate::i18n::tr("Dispositivo indisponível").into()
                }
            })
    }
}
