use super::*;
use crate::{
    platform::windows::{graphics::Canvas, setup as os},
    ui::theme::*,
};
use std::{
    sync::{
        Arc,
        atomic::AtomicBool,
        mpsc::{self, Receiver},
    },
    thread,
};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Page {
    Welcome,
    Destination,
    Installing,
    Complete,
}
#[derive(Clone, Copy, Debug)]
pub enum Action {
    ViewLicense,
    LicensePage(bool),
    CloseLicense,
    LanguageMenu,
    SelectLanguage(usize),
    Next,
    Back,
    Cancel,
    Browse,
    BrowseMedia,
    ConfigureMedia,
    User,
    Machine,
    Desktop,
    Menu,
    Startup,
    AllowDowngrade,
    Path,
    Launch,
    Notes,
    Finish,
    CloseApp,
    Minimize,
    Close,
    RemoveData(usize),
    OpenFolder,
    Details,
    KeepCurrent,
}
pub struct Hit {
    pub rect: Rect,
    pub action: Action,
}
pub enum Event {
    Progress(Progress),
    Finished(AppResult<()>),
}
pub struct Wizard {
    pub language: crate::i18n::Language,
    pub license_open: bool,
    pub license_page: usize,
    pub language_open: bool,
    pub remove_data: [bool; 3],
    pub page: Page,
    pub options: Options,
    pub manifest: Option<payload::Manifest>,
    pub package: PathBuf,
    pub exe: PathBuf,
    pub progress: Progress,
    pub error: Option<String>,
    pub uninstall: bool,
    pub close_prompt: bool,
    pub open_app: bool,
    pub notes: bool,
    pub path_focus: bool,
    pub path_selected: bool,
    pub path_text: String,
    pub media_root: PathBuf,
    pub hits: Vec<Hit>,
    pub focus: Option<usize>,
    pub hover: Option<usize>,
    pub cancel: Arc<AtomicBool>,
    pub events: Option<Receiver<Event>>,
    pub free: Option<(u64, u64)>,
    pub resume: bool,
    /// A working installation registered in another folder than the chosen one.
    pub relocation: Option<engine::Registered>,
    checked_path: Option<String>,
    pub started: std::time::Instant,
    pub page_since: std::time::Instant,
    shown_page: Page,
    /// Follows the Windows animation preference; off means static frames.
    pub motion: bool,
}
impl Wizard {
    pub fn new(
        package: PathBuf,
        exe: PathBuf,
        manifest: Option<payload::Manifest>,
        mut options: Options,
        uninstall: bool,
        resume: bool,
    ) -> AppResult<Self> {
        options.scope.ensure_available()?;
        if !uninstall && !resume {
            if let Ok(ctx) = engine::Context::real(options.scope) {
                // Default to the folder of a working installation. A stale entry
                // (folder removed or unrecognized) keeps the normal default.
                if let Some(registered) =
                    engine::registered(options.scope, &ctx)?.filter(|r| r.version.is_some())
                {
                    options.destination = registered.path;
                }
                if let Ok(Some(v)) = engine::read_state(&options.destination) {
                    options.desktop = v.get("desktop").is_some_and(|s| s == "true");
                    options.start_menu = v.get("start_menu").is_some_and(|s| s == "true");
                    options.startup = v.get("startup").is_some_and(|s| s == "true");
                }
            }
        }
        // Setup/uninstall must remain available even if preferences belong to
        // a future version. This read only seeds the display; it never saves.
        let settings =
            crate::settings::Settings::load(crate::platform::windows::files::data_folder());
        let language = options.language.unwrap_or(settings.language);
        options.language = Some(language);
        crate::i18n::set(language);
        let media_root = options
            .media_root
            .clone()
            .unwrap_or_else(|| settings.videos.clone());
        let free = crate::platform::windows::files::free_space(&options.destination);
        let path_text = options.destination.to_string_lossy().into();
        let page = if uninstall || resume {
            Page::Destination
        } else {
            Page::Welcome
        };
        Ok(Self {
            language,
            license_open: false,
            license_page: 0,
            language_open: false,
            remove_data: [false; 3],
            page,
            options,
            manifest,
            package,
            exe,
            progress: Progress::default(),
            error: None,
            uninstall,
            close_prompt: false,
            open_app: !uninstall,
            notes: false,
            path_focus: false,
            path_selected: false,
            path_text,
            media_root,
            hits: vec![],
            focus: None,
            hover: None,
            cancel: Arc::new(AtomicBool::new(false)),
            events: None,
            free,
            resume,
            relocation: None,
            checked_path: None,
            started: std::time::Instant::now(),
            page_since: std::time::Instant::now(),
            shown_page: page,
            motion: crate::platform::windows::files::animations_enabled(),
        })
    }
    /// Re-evaluate the typed or browsed folder against the registered installation.
    /// Choosing another folder while one is installed means moving it; the page says so.
    pub fn refresh_target(&mut self) {
        if self.checked_path.as_deref() == Some(self.path_text.as_str()) {
            return;
        }
        self.checked_path = Some(self.path_text.clone());
        self.relocation = None;
        if !self.uninstall {
            if let (Ok(target), Ok(ctx)) = (
                os::install_folder(PathBuf::from(&self.path_text)),
                engine::Context::real(self.options.scope),
            ) {
                self.relocation = engine::installed_elsewhere(self.options.scope, &ctx, &target)
                    .ok()
                    .flatten();
            }
        }
        self.options.move_existing = self.relocation.is_some();
    }
    pub fn keep_current(&mut self) {
        if let Some(current) = self.relocation.take() {
            self.options.destination = current.path.clone();
            self.path_text = current.path.to_string_lossy().into();
            self.free = crate::platform::windows::files::free_space(&current.path);
            self.options.move_existing = false;
            self.checked_path = None;
            self.error = None;
        }
    }
    pub fn begin(&mut self) -> AppResult<()> {
        self.options.destination = os::install_folder(PathBuf::from(&self.path_text))?;
        self.path_text = self.options.destination.to_string_lossy().into();
        guard(&self.options.destination)?;
        self.refresh_target();
        let previous = self.relocation.as_ref().map(|r| r.path.clone());
        let running = !os::running(&self.options.destination).is_empty()
            || previous.is_some_and(|p| !os::running(&p).is_empty());
        if running && !self.options.close_running {
            self.close_prompt = true;
            return Ok(());
        }
        let options = self.options.clone();
        let manifest = self.manifest.clone();
        let ctx = engine::Context::real(options.scope)?;
        let package = self.package.clone();
        let exe = self.exe.clone();
        let cancel = self.cancel.clone();
        let uninstall = self.uninstall;
        let remove_data = self.remove_data;
        let (tx, rx) = mpsc::channel();
        self.events = Some(rx);
        self.page = Page::Installing;
        self.error = None;
        thread::spawn(move || {
            let result = (|| {
                let _com = os::Com::new()?;
                if uninstall {
                    engine::uninstall(&options.destination, &ctx, options.close_running, |p| {
                        let _ = tx.send(Event::Progress(p));
                    })?;
                    let data = crate::platform::windows::files::data_folder();
                    let settings = crate::settings::Settings::load(data.clone());
                    let mut cleanup_error = false;
                    for (kind, on) in remove_data.iter().enumerate() {
                        if !on {
                            continue;
                        }
                        let result = (|| -> AppResult<()> {
                            let plan = match kind {
                                0 => crate::persistence::maintenance::Plan::preferences(&data)?,
                                1 => {
                                    crate::persistence::maintenance::Plan::cache(&settings, false)?
                                }
                                _ => crate::persistence::maintenance::Plan::logs(&data, false)?,
                            };
                            plan.execute(true)
                        })();
                        if let Err(e) = result {
                            cleanup_error = true;
                            let _ = tx.send(Event::Progress(Progress {
                                completed: 1,
                                total: 1,
                                operation: crate::i18n::format_text(
                                    crate::i18n::tr(
                                        "Aplicativo removido; limpeza opcional pendente: {e}",
                                    ),
                                    &[("e", format!("{}", e))],
                                ),
                                file: String::new(),
                            }));
                        }
                    }
                    if remove_data.iter().any(|on| *on) && !cleanup_error {
                        let _=tx.send(Event::Progress(Progress{completed:1,total:1,operation:crate::i18n::tr("Aplicativo removido e limpeza local solicitada concluída. Biblioteca, mídias e projetos preservados.").into(),file:String::new()}));
                    }
                    Ok(())
                } else {
                    engine::install(
                        manifest
                            .as_ref()
                            .ok_or(crate::i18n::tr("Payload ausente"))?,
                        &package,
                        &exe,
                        &options,
                        &ctx,
                        &cancel,
                        |p| {
                            let _ = tx.send(Event::Progress(p));
                        },
                    )
                }
            })();
            let _ = tx.send(Event::Finished(result));
        });
        Ok(())
    }
    pub fn select_language(&mut self, language: crate::i18n::Language) {
        self.language = language;
        self.options.language = Some(language);
        self.language_open = false;
        self.focus = None;
        self.hover = None;
        crate::i18n::set(language);
    }
    pub fn tick(&mut self) {
        let events: Vec<_> = self
            .events
            .as_ref()
            .map(|rx| rx.try_iter().collect())
            .unwrap_or_default();
        for event in events {
            match event {
                Event::Progress(p) => self.progress = p,
                Event::Finished(result) => {
                    self.events = None;
                    match result {
                        Ok(()) => self.page = Page::Complete,
                        Err(e) => {
                            self.page = Page::Destination;
                            self.error = Some(e);
                            self.close_prompt = false;
                            self.cancel
                                .store(false, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                }
            }
        }
    }
    pub fn hit(&mut self, r: Rect, a: Action) {
        self.hits.push(Hit { rect: r, action: a });
    }
    fn elapsed(&self) -> f32 {
        if self.motion {
            self.started.elapsed().as_secs_f32()
        } else {
            0.
        }
    }
    /// 0 → 1 over a short interval after the page changed.
    fn entrance(&self) -> f32 {
        if !self.motion {
            return 1.;
        }
        let t = (self.page_since.elapsed().as_secs_f32() / 0.22).clamp(0., 1.);
        1. - (1. - t).powi(3)
    }
}
fn text(c: &mut Canvas, s: &str, r: Rect, size: f32, color: Color, bold: bool) {
    c.text(s, r, size, color, bold);
}
fn button(
    m: &mut Wizard,
    c: &mut Canvas,
    r: Rect,
    label: &str,
    action: Action,
    primary: bool,
    enabled: bool,
) {
    let hover = m.hover == Some(m.hits.len());
    if !enabled {
        c.round(r, 8., PANEL, Some(BORDER.alpha(0.6)));
    } else if primary {
        c.round(r, 8., if hover { Color::rgb(0xffe268) } else { GOLD }, None);
    } else {
        c.round(
            r,
            8.,
            if hover { HOVER } else { RAISED },
            Some(if hover { DIM } else { BORDER }),
        );
    }
    c.center(
        label,
        r,
        14.,
        if !enabled {
            DIM
        } else if primary {
            c.palette.ink()
        } else {
            TEXT
        },
        true,
    );
    if enabled {
        if m.focus == Some(m.hits.len()) {
            c.outline(r.inset(-3.), 10., GOLD, 1.5);
        }
        m.hit(r, action);
    }
}
fn checkbox(m: &mut Wizard, c: &mut Canvas, r: Rect, label: &str, checked: bool, action: Action) {
    let hover = m.hover == Some(m.hits.len());
    let bx = Rect::new(r.x, r.y + (r.h - 20.) / 2., 20., 20.);
    c.round(
        bx,
        5.,
        if checked { GOLD } else { BG },
        Some(if checked {
            GOLD
        } else if hover {
            MUTED
        } else {
            DIM
        }),
    );
    if checked {
        c.icon(
            "check",
            Rect::new(bx.x + 3., bx.y + 3., 14., 14.),
            c.palette.ink(),
        );
    }
    text(
        c,
        label,
        Rect::new(r.x + 32., r.y, r.w - 32., r.h),
        14.,
        TEXT,
        false,
    );
    if m.focus == Some(m.hits.len()) {
        c.outline(r.inset(-3.), 6., GOLD, 1.5);
    }
    m.hit(r, action);
}
/// Product illustration drawn with the same vectors as the app: a capture window,
/// a moving pointer, a recording badge, live audio bars and a camera bubble.
fn illustration(c: &mut Canvas, r: Rect, t: f32) {
    c.round(r, 14., PANEL, Some(BORDER));
    c.logo(Rect::new(r.x + 16., r.y + 13., 18., 18.));
    for (i, w) in [64., 38., 52.].iter().enumerate() {
        c.round(
            Rect::new(r.x + 46. + i as f32 * 74., r.y + 18., *w, 8.),
            4.,
            BORDER,
            None,
        );
    }
    c.line(r.x, r.y + 44., r.right(), r.y + 44., BORDER, 1.);
    let screen = Rect::new(r.x + 16., r.y + 58., r.w - 32., r.h - 132.);
    c.round(screen, 9., BG, Some(BORDER.alpha(0.7)));
    c.clip(screen);
    c.wallpaper(screen, 0.8);
    for (i, (x, y, w, h)) in [(0.07, 0.12, 0.46, 0.5), (0.4, 0.3, 0.5, 0.56)]
        .iter()
        .enumerate()
    {
        let win = Rect::new(
            screen.x + screen.w * x,
            screen.y + screen.h * y,
            screen.w * w,
            screen.h * h,
        );
        c.round(win, 7., RAISED, Some(BORDER));
        c.line(win.x, win.y + 18., win.right(), win.y + 18., BORDER, 1.);
        for row in 0..3 {
            c.round(
                Rect::new(
                    win.x + 12.,
                    win.y + 30. + row as f32 * 16.,
                    win.w * (0.7 - row as f32 * 0.16 - i as f32 * 0.05),
                    7.,
                ),
                3.5,
                HOVER,
                None,
            );
        }
    }
    let px = screen.x + screen.w * (0.5 + 0.28 * (t * 0.45).sin());
    let py = screen.y + screen.h * (0.52 + 0.22 * (t * 0.62).sin());
    let ripple = (t * 0.7).fract();
    c.ring(
        px,
        py,
        6. + ripple * 18.,
        GOLD.alpha(0.5 * (1. - ripple)),
        1.5,
    );
    c.icon("cursor", Rect::new(px - 2., py - 2., 20., 20.), TEXT);
    c.unclip();
    let badge = Rect::new(screen.x + 12., screen.y + 12., 92., 26.);
    c.round(badge, 13., BG.alpha(0.85), Some(BORDER));
    let pulse = 0.55 + 0.45 * (t * 3.1).sin().abs();
    c.circle(badge.x + 15., badge.y + 13., 5., RED.alpha(pulse));
    let seconds = t as u32;
    c.text(
        &format!("{:02}:{:02}", seconds / 60 % 60, seconds % 60),
        Rect::new(badge.x + 28., badge.y, 60., 26.),
        12.,
        TEXT,
        true,
    );
    let cx = screen.right() - 44.;
    let cy = screen.bottom() - 42.;
    c.circle(cx, cy, 30., RAISED);
    c.ring(cx, cy, 30., GOLD, 2.);
    c.circle(cx, cy - 7., 9., MUTED);
    c.arc(cx, cy + 22., 16., -70., 140., MUTED, 7.);
    let bars = Rect::new(r.x + 16., r.bottom() - 58., r.w - 32., 42.);
    c.round(bars, 9., RAISED, Some(BORDER.alpha(0.7)));
    c.icon(
        "microphone",
        Rect::new(bars.x + 12., bars.y + 11., 20., 20.),
        GOLD,
    );
    let count = ((bars.w - 64.) / 9.).max(1.) as usize;
    for i in 0..count {
        let k = i as f32;
        let level = 0.25
            + 0.75
                * ((t * 4.3 + k * 0.55).sin() * 0.5 + 0.5)
                * ((t * 1.7 + k * 0.23).cos() * 0.35 + 0.65);
        let hgt = 4. + level * 20.;
        c.round(
            Rect::new(bars.x + 44. + k * 9., bars.y + (bars.h - hgt) / 2., 5., hgt),
            2.5,
            GOLD.alpha(0.35 + level * 0.5),
            None,
        );
    }
}
fn status_card(c: &mut Canvas, r: Rect, icon: &str, title: &str, value: &str, accent: bool) {
    c.round(r, 10., PANEL, Some(BORDER));
    c.icon(
        icon,
        Rect::new(r.x + 14., r.y + (r.h - 22.) / 2., 22., 22.),
        MUTED,
    );
    text(
        c,
        title,
        Rect::new(r.x + 48., r.y + 8., r.w - 58., 20.),
        12.,
        MUTED,
        false,
    );
    text(
        c,
        value,
        Rect::new(r.x + 48., r.y + 27., r.w - 58., 24.),
        16.,
        if accent { GOLD } else { TEXT },
        true,
    );
}
pub fn render(m: &mut Wizard, c: &mut Canvas) {
    c.palette = Palette::default();
    c.animation_time = m.elapsed();
    m.refresh_target();
    if m.shown_page != m.page {
        m.shown_page = m.page;
        m.page_since = std::time::Instant::now();
    }
    let t = m.elapsed();
    let (w, h) = c.begin_layout(1280., 700.);
    m.hits.clear();
    c.fill(Rect::new(0., 0., w, 64.), SIDEBAR);
    c.line(0., 64., w, 64., BORDER, 1.);
    c.logo(Rect::new(24., 16., 32., 32.));
    let brand = c.text_width("Wyzar", 21., true);
    text(
        c,
        "Wyzar",
        Rect::new(68., 14., brand + 4., 36.),
        21.,
        GOLD,
        true,
    );
    let capture = c.text_width("Capture", 21., true);
    text(
        c,
        "Capture",
        Rect::new(68. + brand + 1., 14., capture + 4., 36.),
        21.,
        TEXT,
        true,
    );
    let divider = 68. + brand + capture + 22.;
    c.line(divider, 22., divider, 42., BORDER, 1.);
    text(
        c,
        if m.uninstall {
            crate::i18n::tr("Desinstalador")
        } else {
            crate::i18n::tr("Instalador")
        },
        Rect::new(divider + 18., 16., 220., 32.),
        15.,
        MUTED,
        false,
    );
    for (i, a) in [Action::Minimize, Action::Close].into_iter().enumerate() {
        let r = Rect::new(w - 100. + i as f32 * 48., 12., 42., 40.);
        if m.hover == Some(m.hits.len()) {
            c.round(r, 7., if i == 1 { RED } else { HOVER }, None);
        }
        c.icon(
            if i == 0 { "minimize" } else { "close" },
            r.inset(12.),
            TEXT,
        );
        m.hit(r, a);
    }
    let interactive = m.page != Page::Installing && !m.close_prompt;
    button(
        m,
        c,
        Rect::new(w - 318., 14., 196., 36.),
        m.language.name(),
        Action::LanguageMenu,
        false,
        interactive,
    );
    button(
        m,
        c,
        Rect::new(w - 434., 14., 104., 36.),
        crate::i18n::tr("Licença"),
        Action::ViewLicense,
        false,
        m.page != Page::Installing,
    );
    let rail = 280.;
    c.fill(Rect::new(0., 65., rail, h - 141.), SIDEBAR);
    c.line(rail, 64., rail, h - 76., BORDER, 1.);
    let active = match m.page {
        Page::Welcome => 0,
        Page::Destination => 1,
        Page::Installing => 2,
        Page::Complete => 3,
    };
    for (i, (name, detail)) in [
        (
            crate::i18n::tr("Bem-vindo"),
            crate::i18n::tr("Início da instalação"),
        ),
        (
            crate::i18n::tr("Destino"),
            crate::i18n::tr("Escolha o local"),
        ),
        (
            crate::i18n::tr("Instalação"),
            crate::i18n::tr("Arquivos verificados"),
        ),
        (
            crate::i18n::tr("Conclusão"),
            crate::i18n::tr("Pronto para usar"),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let cy = 124. + i as f32 * 86.;
        if i < 3 {
            c.line(
                44.,
                cy + 19.,
                44.,
                cy + 67.,
                if i < active { GOLD.alpha(0.7) } else { BORDER },
                2.,
            );
        }
        if i < active {
            c.circle(44., cy, 15., GOLD);
            c.icon("check", Rect::new(36., cy - 8., 16., 16.), c.palette.ink());
        } else if i == active {
            let breathe = if m.motion {
                0.5 + 0.5 * (t * 2.2).sin()
            } else {
                1.
            };
            c.glow(44., cy, 26., GOLD.alpha(0.10 + 0.08 * breathe));
            c.circle(44., cy, 15., GOLD_DARK);
            c.ring(44., cy, 15., GOLD, 2.);
            c.center(
                &(i + 1).to_string(),
                Rect::new(29., cy - 15., 30., 30.),
                14.,
                GOLD,
                true,
            );
        } else {
            c.ring(44., cy, 15., DIM, 1.5);
            c.center(
                &(i + 1).to_string(),
                Rect::new(29., cy - 15., 30., 30.),
                14.,
                DIM,
                true,
            );
        }
        text(
            c,
            name,
            Rect::new(74., cy - 21., rail - 90., 24.),
            15.,
            if i == active { TEXT } else { MUTED },
            true,
        );
        text(
            c,
            detail,
            Rect::new(74., cy + 2., rail - 90., 20.),
            12.5,
            if i == active { MUTED } else { DIM },
            false,
        );
    }
    text(
        c,
        crate::i18n::tr("Para este usuário · sem administrador"),
        Rect::new(24., h - 140., rail - 40., 22.),
        12.,
        DIM,
        false,
    );
    let x = rail + 48.;
    let rw = w - x - 40.;
    let left = 440f32.min(rw - 460.).max(380.);
    let enter = m.entrance();
    c.push_opacity(enter);
    c.translate(0., (1. - enter) * 10.);
    match m.page {
        Page::Welcome => {
            text(
                c,
                crate::i18n::tr("Bem-vindo ao instalador do"),
                Rect::new(x, 102., left, 26.),
                15.,
                MUTED,
                false,
            );
            text(
                c,
                "WyzarCapture",
                Rect::new(x, 126., left, 56.),
                40.,
                TEXT,
                true,
            );
            c.paragraph(
                crate::i18n::tr("Gravador de tela, áudio e câmera para Windows 10 e 11."),
                Rect::new(x, 188., left, 50.),
                16.,
                MUTED,
                false,
            );
            for (i, (icon, line)) in [
                (
                    "record",
                    crate::i18n::tr(
                        "Grave a tela inteira, uma janela ou uma região, com microfone, som do sistema e webcam.",
                    ),
                ),
                (
                    "folder",
                    crate::i18n::tr(
                        "Arquivos salvos no seu computador, nas pastas que você escolher.",
                    ),
                ),
                (
                    "shield",
                    crate::i18n::tr(
                        "Instalação para este usuário, sem administrador e sem downloads.",
                    ),
                ),
            ]
            .into_iter()
            .enumerate()
            {
                let y = 262. + i as f32 * 74.;
                let tile = Rect::new(x, y, 40., 40.);
                c.round(tile, 10., RAISED, Some(BORDER));
                c.icon(icon, tile.inset(10.), GOLD);
                c.paragraph(
                    line,
                    Rect::new(x + 56., y + 1., left - 56., 60.),
                    14.,
                    TEXT,
                    false,
                );
            }
            illustration(c, Rect::new(x + left + 36., 100., rw - left - 36., 440.), t);
        }
        Page::Destination if m.uninstall => {
            text(
                c,
                crate::i18n::tr("Remover o WyzarCapture"),
                Rect::new(x, 98., rw, 40.),
                26.,
                TEXT,
                true,
            );
            c.paragraph(crate::i18n::tr("Por padrão, somente os arquivos da aplicação e os atalhos desta instalação serão removidos. Vídeos, áudios, capturas, projetos e biblioteca serão preservados. Limpeza local é opcional abaixo."),Rect::new(x,144.,rw,70.),14.5,MUTED,false);
            let field = Rect::new(x, 222., rw, 44.);
            c.round(field, 8., PANEL, Some(BORDER));
            c.icon(
                "folder",
                Rect::new(field.x + 14., field.y + 12., 20., 20.),
                MUTED,
            );
            text(
                c,
                &m.path_text,
                Rect::new(field.x + 46., field.y, field.w - 60., field.h),
                14.,
                TEXT,
                false,
            );
            c.round(Rect::new(x, 286., rw, 58.), 10., PANEL, Some(BORDER));
            c.icon("shield", Rect::new(x + 16., 301., 28., 28.), GOLD);
            c.paragraph(crate::i18n::tr("Vídeos, capturas, projetos e biblioteca serão preservados. As opções abaixo afetam somente dados locais deste usuário."),Rect::new(x+58.,292.,rw-72.,48.),13.5,TEXT,false);
            for (kind, name) in [
                crate::i18n::tr("Remover preferências (settings.ini)"),
                crate::i18n::tr("Remover cache temporário reconhecido"),
                crate::i18n::tr("Remover logs locais"),
            ]
            .iter()
            .enumerate()
            {
                checkbox(
                    m,
                    c,
                    Rect::new(x, 360. + kind as f32 * 38., rw, 32.),
                    name,
                    m.remove_data[kind],
                    Action::RemoveData(kind),
                );
            }
            text(
                c,
                &{
                    let __wyzar_i18n_0 = &(crate::platform::windows::files::data_folder()
                        .display()
                        .to_string());
                    crate::i18n::format_text(
                        crate::i18n::tr("Dados: {}"),
                        &[("0", format!("{}", __wyzar_i18n_0))],
                    )
                },
                Rect::new(x, 474., rw, 22.),
                12.,
                MUTED,
                false,
            );
        }
        Page::Destination => {
            text(
                c,
                crate::i18n::tr("Escolha onde instalar o WyzarCapture"),
                Rect::new(x, 98., rw, 40.),
                26.,
                TEXT,
                true,
            );
            c.paragraph(
                crate::i18n::tr(
                    "Escolha uma pasta e as opções desejadas. Atualizações e reparos preservam seus dados pessoais.",
                ),
                Rect::new(x, 142., rw, 46.),
                14.5,
                MUTED,
                false,
            );
            let field = Rect::new(x, 192., rw - 152., 44.);
            c.round(
                field,
                8.,
                PANEL,
                Some(if m.path_focus { GOLD } else { BORDER }),
            );
            c.icon(
                "folder",
                Rect::new(field.x + 14., field.y + 12., 20., 20.),
                MUTED,
            );
            if m.path_focus && m.path_selected {
                c.round(
                    Rect::new(field.x + 42., field.y + 8., field.w - 54., 28.),
                    4.,
                    GOLD.alpha(0.16),
                    None,
                );
            }
            text(
                c,
                &m.path_text,
                Rect::new(field.x + 46., field.y, field.w - 60., field.h),
                14.,
                TEXT,
                false,
            );
            if m.path_focus && !m.path_selected && (!m.motion || (t * 1.6).fract() < 0.6) {
                let caret = (field.x + 47. + c.text_width(&m.path_text, 14., false))
                    .min(field.right() - 12.);
                c.line(caret, field.y + 12., caret, field.bottom() - 12., GOLD, 1.);
            }
            m.hit(field, Action::Path);
            button(
                m,
                c,
                Rect::new(x + rw - 140., 192., 140., 44.),
                crate::i18n::tr("Procurar"),
                Action::Browse,
                false,
                true,
            );
            let mut y = 250.;
            if let Some(current) = m.relocation.clone() {
                let note = Rect::new(x, y, rw, 62.);
                c.round(note, 10., GOLD_DARK, Some(GOLD.alpha(0.45)));
                c.icon(
                    "info",
                    Rect::new(note.x + 14., note.y + 19., 24., 24.),
                    GOLD,
                );
                text(
                    c,
                    &crate::i18n::format_text(
                        crate::i18n::tr("Instalação atual: {path} · versão {version}"),
                        &[
                            ("path", current.path.to_string_lossy().into_owned()),
                            ("version", current.version.clone().unwrap_or_default()),
                        ],
                    ),
                    Rect::new(note.x + 50., note.y + 8., note.w - 230., 24.),
                    13.,
                    TEXT,
                    true,
                );
                text(
                    c,
                    crate::i18n::tr(
                        "Instalar nesta pasta move o programa. Vídeos, projetos e preferências continuam onde estão.",
                    ),
                    Rect::new(note.x + 50., note.y + 31., note.w - 230., 22.),
                    12.5,
                    MUTED,
                    false,
                );
                button(
                    m,
                    c,
                    Rect::new(note.right() - 168., note.y + 13., 154., 36.),
                    crate::i18n::tr("Usar pasta atual"),
                    Action::KeepCurrent,
                    false,
                    true,
                );
                y += 74.;
            } else {
                y += 4.;
            }
            let necessary = m
                .manifest
                .as_ref()
                .map(|v| engine::required_bytes(v, &m.exe).unwrap_or(v.size()))
                .unwrap_or(0);
            let third = (rw - 24.) / 3.;
            status_card(
                c,
                Rect::new(x, y, third, 58.),
                "storage",
                crate::i18n::tr("Espaço necessário"),
                &crate::core::size_label(necessary),
                false,
            );
            status_card(
                c,
                Rect::new(x + third + 12., y, third, 58.),
                "storage",
                crate::i18n::tr("Espaço disponível"),
                &m.free
                    .map(|v| crate::core::size_label(v.0))
                    .unwrap_or_else(|| crate::i18n::tr("Não disponível").into()),
                false,
            );
            let op = if m.relocation.is_some() {
                crate::i18n::tr("Mudança de pasta")
            } else {
                engine::operation(&PathBuf::from(&m.path_text), crate::updates::VERSION)
                    .unwrap_or(crate::i18n::tr("Instalação nova"))
            };
            status_card(
                c,
                Rect::new(x + (third + 12.) * 2., y, third, 58.),
                "download",
                &{
                    let __wyzar_i18n_0 = &(crate::updates::VERSION);
                    crate::i18n::format_text(
                        crate::i18n::tr("Versão {} · x64"),
                        &[("0", format!("{}", __wyzar_i18n_0))],
                    )
                },
                op,
                true,
            );
            y += 76.;
            let column = (rw - 32.) / 2.;
            checkbox(
                m,
                c,
                Rect::new(x, y, column, 32.),
                crate::i18n::tr("Criar atalho no Menu Iniciar"),
                m.options.start_menu,
                Action::Menu,
            );
            checkbox(
                m,
                c,
                Rect::new(x, y + 38., column, 32.),
                crate::i18n::tr("Criar atalho na Área de Trabalho"),
                m.options.desktop,
                Action::Desktop,
            );
            checkbox(
                m,
                c,
                Rect::new(x, y + 76., column, 32.),
                crate::i18n::tr("Iniciar com o Windows"),
                m.options.startup,
                Action::Startup,
            );
            let mx = x + column + 32.;
            checkbox(
                m,
                c,
                Rect::new(mx, y, column, 32.),
                crate::i18n::tr("Usar uma pasta personalizada para todas as mídias"),
                m.options.media_root.is_some(),
                Action::ConfigureMedia,
            );
            c.paragraph(
                &if m.options.media_root.is_some() {
                    m.media_root.to_string_lossy().into_owned()
                } else {
                    crate::i18n::tr("Padrão: Vídeos, Imagens, Música e Documentos do Windows")
                        .into()
                },
                Rect::new(mx + 32., y + 36., column - 32., 36.),
                12.5,
                MUTED,
                false,
            );
            button(
                m,
                c,
                Rect::new(mx + 32., y + 76., 170., 32.),
                crate::i18n::tr("Escolher mídias"),
                Action::BrowseMedia,
                false,
                true,
            );
            if op.starts_with(crate::i18n::tr("Versão anterior")) {
                checkbox(
                    m,
                    c,
                    Rect::new(x, y + 116., rw, 32.),
                    crate::i18n::tr("Confirmo instalar esta versão anterior"),
                    m.options.allow_downgrade,
                    Action::AllowDowngrade,
                );
            }
        }
        Page::Installing => {
            text(
                c,
                if m.uninstall {
                    crate::i18n::tr("Removendo o WyzarCapture")
                } else {
                    crate::i18n::tr("Instalando o WyzarCapture")
                },
                Rect::new(x, 98., left, 40.),
                26.,
                TEXT,
                true,
            );
            c.paragraph(crate::i18n::tr("Aguarde enquanto os arquivos são verificados e as operações solicitadas são concluídas. O progresso abaixo corresponde ao trabalho realmente executado."),Rect::new(x,144.,left,90.),14.5,MUTED,false);
            let percent = m.progress.percent();
            text(
                c,
                &format!("{percent}%"),
                Rect::new(x, 246., left, 44.),
                34.,
                TEXT,
                true,
            );
            let bar = Rect::new(x, 300., left, 10.);
            c.round(bar, 5., RAISED, Some(BORDER));
            if percent > 0 {
                let fill = Rect::new(bar.x, bar.y, bar.w * percent as f32 / 100., bar.h);
                c.round(fill, 5., GOLD, None);
                if m.motion && percent < 100 {
                    c.clip(fill);
                    let sweep = fill.x + (t * 0.8).fract() * (fill.w + 120.) - 60.;
                    c.glow(
                        sweep,
                        fill.y + 5.,
                        50.,
                        Color::literal(0xffffff).alpha(0.35),
                    );
                    c.unclip();
                }
            }
            c.paragraph(
                &m.progress.operation,
                Rect::new(x, 324., left, 46.),
                14.5,
                TEXT,
                false,
            );
            let panel = Rect::new(x, 384., left, 84.);
            c.round(panel, 10., PANEL, Some(BORDER));
            c.icon(
                "shield",
                Rect::new(panel.x + 16., panel.y + 16., 22., 22.),
                GOLD,
            );
            text(
                c,
                crate::i18n::tr("Operação atual"),
                Rect::new(panel.x + 50., panel.y + 12., panel.w - 64., 24.),
                12.5,
                MUTED,
                false,
            );
            text(
                c,
                &m.progress.file,
                Rect::new(panel.x + 50., panel.y + 40., panel.w - 64., 28.),
                13.5,
                TEXT,
                false,
            );
            illustration(c, Rect::new(x + left + 36., 100., rw - left - 36., 440.), t);
        }
        Page::Complete => {
            let pop = m.entrance();
            c.glow(x + 30., 128., 60., GOLD.alpha(0.18 * pop));
            c.circle(x + 30., 128., 26. * (0.7 + 0.3 * pop), GOLD);
            c.icon("check", Rect::new(x + 18., 116., 24., 24.), c.palette.ink());
            text(
                c,
                if m.uninstall {
                    crate::i18n::tr("Remoção concluída")
                } else {
                    crate::i18n::tr("Instalação concluída")
                },
                Rect::new(x, 172., left, 44.),
                28.,
                TEXT,
                true,
            );
            c.paragraph(if m.uninstall{crate::i18n::tr("O aplicativo foi removido. Vídeos, áudios, capturas, biblioteca e projetos foram preservados. Confira abaixo o resultado das opções de limpeza.")}else{crate::i18n::tr("O WyzarCapture está pronto. Seus dados continuam no local escolhido por você.")},Rect::new(x,222.,left,70.),15.,MUTED,false);
            let note = m.progress.operation.clone();
            let plain = note.is_empty()
                || note == crate::i18n::tr("Instalação concluída")
                || note == crate::i18n::tr("Remoção concluída. Arquivos pessoais preservados.");
            let mut y = 300.;
            if !plain {
                let height = c.paragraph_height(&note, left - 56., 13., false) + 20.;
                let r = Rect::new(x, y, left, height.max(46.));
                c.round(r, 10., PANEL, Some(BORDER));
                c.icon("info", Rect::new(r.x + 14., r.y + 12., 20., 20.), GOLD);
                c.paragraph(
                    &note,
                    Rect::new(r.x + 44., r.y + 10., r.w - 56., height),
                    13.,
                    TEXT,
                    false,
                );
                y += r.h + 18.;
            }
            if !m.uninstall {
                checkbox(
                    m,
                    c,
                    Rect::new(x, y, left, 32.),
                    crate::i18n::tr("Abrir o WyzarCapture agora"),
                    m.open_app,
                    Action::Launch,
                );
                checkbox(
                    m,
                    c,
                    Rect::new(x, y + 38., left, 32.),
                    crate::i18n::tr("Abrir Novidades no WyzarCapture"),
                    m.notes,
                    Action::Notes,
                );
                illustration(c, Rect::new(x + left + 36., 100., rw - left - 36., 440.), t);
            } else {
                button(
                    m,
                    c,
                    Rect::new(x, y, 180., 40.),
                    crate::i18n::tr("Abrir pasta"),
                    Action::OpenFolder,
                    false,
                    true,
                );
                button(
                    m,
                    c,
                    Rect::new(x + 192., y, 180., 40.),
                    crate::i18n::tr("Ver detalhes"),
                    Action::Details,
                    false,
                    m.options
                        .destination
                        .join("wyzar-uninstall-preserved.txt")
                        .exists(),
                );
            }
        }
    }
    c.reset_transform();
    c.pop_opacity();
    c.fill(Rect::new(0., h - 76., w, 76.), SIDEBAR);
    c.line(0., h - 76., w, h - 76., BORDER, 1.);
    text(
        c,
        &{
            let __wyzar_i18n_0 = &(crate::updates::VERSION);
            crate::i18n::format_text(
                crate::i18n::tr("Versão {} · x64"),
                &[("0", format!("{}", __wyzar_i18n_0))],
            )
        },
        Rect::new(24., h - 50., 240., 24.),
        12.5,
        DIM,
        false,
    );
    if m.page != Page::Installing {
        text(
            c,
            crate::i18n::tr("Arquivos verificados · operação offline"),
            Rect::new(x, h - 50., 360., 24.),
            12.5,
            MUTED,
            false,
        );
    }
    if let Some(error) = m.error.clone() {
        let height = (c.paragraph_height(&error, rw - 62., 13., false) + 18.).clamp(42., 92.);
        let r = Rect::new(x, h - 88. - height, rw, height);
        c.round(r, 10., Color::rgb(0x261317), Some(RED.alpha(0.7)));
        c.icon(
            "warning",
            Rect::new(r.x + 14., r.y + (r.h - 20.) / 2., 20., 20.),
            RED,
        );
        c.paragraph(
            &error,
            Rect::new(r.x + 46., r.y + 9., r.w - 60., r.h - 12.),
            13.,
            TEXT,
            false,
        );
    }
    let by = h - 59.;
    button(
        m,
        c,
        Rect::new(w - 452., by, 120., 42.),
        crate::i18n::tr("Voltar"),
        Action::Back,
        false,
        m.page == Page::Destination && !m.uninstall,
    );
    if m.page == Page::Complete {
        button(
            m,
            c,
            Rect::new(w - 320., by, 136., 42.),
            crate::i18n::tr("Fechar"),
            Action::Close,
            false,
            true,
        );
        button(
            m,
            c,
            Rect::new(w - 172., by, 132., 42.),
            if m.uninstall {
                crate::i18n::tr("Concluir")
            } else {
                crate::i18n::tr("Iniciar")
            },
            Action::Finish,
            true,
            true,
        );
    } else {
        button(
            m,
            c,
            Rect::new(w - 320., by, 136., 42.),
            if m.page == Page::Destination {
                if m.uninstall {
                    crate::i18n::tr("Desinstalar")
                } else {
                    crate::i18n::tr("Instalar")
                }
            } else {
                crate::i18n::tr("Próximo")
            },
            Action::Next,
            true,
            m.page != Page::Installing,
        );
        button(
            m,
            c,
            Rect::new(w - 172., by, 132., 42.),
            crate::i18n::tr("Cancelar"),
            Action::Cancel,
            false,
            !m.uninstall || m.page != Page::Installing,
        );
    }
    if m.close_prompt {
        c.fill(Rect::new(0., 65., w, h - 65.), BG.alpha(0.88));
        m.hits.clear();
        let r = Rect::new((w - 600.) / 2., (h - 240.) / 2., 600., 240.);
        c.round(r, 14., PANEL, Some(BORDER));
        c.icon("warning", Rect::new(r.x + 24., r.y + 26., 24., 24.), GOLD);
        text(
            c,
            crate::i18n::tr("O WyzarCapture precisa ser fechado para continuar."),
            Rect::new(r.x + 60., r.y + 18., r.w - 84., 40.),
            19.,
            TEXT,
            true,
        );
        c.paragraph(crate::i18n::tr("A gravação será finalizada e o aplicativo receberá uma solicitação de fechamento limpo. Nenhum processo será morto."),Rect::new(r.x+60.,r.y+66.,r.w-84.,70.),14.,MUTED,false);
        button(
            m,
            c,
            Rect::new(r.x + 24., r.bottom() - 66., 360., 42.),
            crate::i18n::tr("Fechar WyzarCapture e continuar"),
            Action::CloseApp,
            true,
            true,
        );
        button(
            m,
            c,
            Rect::new(r.right() - 168., r.bottom() - 66., 144., 42.),
            crate::i18n::tr("Cancelar"),
            Action::Cancel,
            false,
            true,
        );
    }
    if m.license_open {
        m.hits.clear();
        let layout = crate::ui::license_view::render(c, w, h, m.license_page);
        button(
            m,
            c,
            layout.previous,
            crate::i18n::tr("Voltar"),
            Action::LicensePage(false),
            false,
            true,
        );
        button(
            m,
            c,
            layout.next,
            crate::i18n::tr("Próximo"),
            Action::LicensePage(true),
            false,
            true,
        );
        button(
            m,
            c,
            layout.close,
            crate::i18n::tr("Fechar"),
            Action::CloseLicense,
            true,
            true,
        );
    }
    if m.language_open && m.page != Page::Installing && !m.close_prompt {
        let r = Rect::new(w - 318., 56., 196., 12. + 5. * 34.);
        c.round(
            Rect::new(r.x + 2., r.y + 6., r.w, r.h),
            10.,
            BG.alpha(0.7),
            None,
        );
        c.round(r, 10., RAISED, Some(BORDER));
        m.hits.clear();
        m.hit(Rect::new(w - 318., 14., 196., 36.), Action::LanguageMenu);
        for (index, language) in crate::i18n::Language::ALL.iter().enumerate() {
            let row = Rect::new(r.x + 6., r.y + 6. + index as f32 * 34., r.w - 12., 34.);
            if *language == m.language || m.hover == Some(m.hits.len()) {
                c.round(
                    row,
                    6.,
                    if *language == m.language {
                        GOLD_DARK
                    } else {
                        HOVER
                    },
                    None,
                );
            }
            c.text(
                language.name(),
                Rect::new(row.x + 12., row.y, row.w - 40., row.h),
                13.5,
                TEXT,
                *language == m.language,
            );
            if *language == m.language {
                c.icon(
                    "check",
                    Rect::new(row.right() - 26., row.y + 10., 14., 14.),
                    GOLD,
                );
            }
            m.hit(row, Action::SelectLanguage(index));
        }
    }
    let _ = c.present();
}
pub fn render_matrix(
    root: PathBuf,
    package: &Path,
    embedded: &str,
    trace: Option<PathBuf>,
) -> AppResult<()> {
    guard(&root)?;
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let manifest = payload::Manifest::trusted(package, embedded)?;
    let options = Options {
        language: crate::i18n::forced(),
        media_root: None,
        destination: os::default_destination(Scope::User),
        scope: Scope::User,
        desktop: false,
        start_menu: false,
        startup: false,
        allow_downgrade: false,
        close_running: false,
        move_existing: false,
    };
    let mut wizard = Wizard::new(
        package.into(),
        std::env::current_exe().map_err(|e| e.to_string())?,
        Some(manifest),
        options,
        false,
        false,
    )?;
    // Renders are still frames: no entrance fade, no time-dependent motion.
    wizard.motion = false;
    if let Some(p) = trace {
        let v = crate::persistence::read(&p)?;
        wizard.progress = Progress {
            completed: v.get("completed").and_then(|s| s.parse().ok()).unwrap_or(0),
            total: v.get("total").and_then(|s| s.parse().ok()).unwrap_or(0),
            operation: v.get("operation").cloned().unwrap_or_default(),
            file: v.get("file").cloned().unwrap_or_default(),
        };
    }
    for dpi in [100, 125, 150, 175, 200] {
        let folder = root.join(dpi.to_string());
        std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
        let mut c = Canvas::image(1280 * dpi / 100, 700 * dpi / 100)?;
        for (i, page) in [
            Page::Welcome,
            Page::Destination,
            Page::Installing,
            Page::Complete,
        ]
        .into_iter()
        .enumerate()
        {
            wizard.page = page;
            render(&mut wizard, &mut c);
            let name = [
                "01-welcome.png",
                "02-destination.png",
                "03-installing.png",
                "04-complete.png",
            ][i];
            c.save(&folder.join(name))?;
            if dpi == 100 {
                c.save(&root.join(name))?;
            }
        }
        // Explicit render fixtures only. These receipts do not claim an update
        // was executed; installation acceptance is recorded by installer::qa.
        let fixture = root.join("render-fixture");
        std::fs::create_dir_all(&fixture).map_err(|e| e.to_string())?;
        wizard.options.destination = fixture.clone();
        wizard.path_text = fixture.to_string_lossy().into();
        wizard.page = Page::Destination;
        wizard.uninstall = false;
        for (name, version) in [
            ("05-update.png", "0.1.0"),
            ("06-repair.png", crate::updates::VERSION),
        ] {
            crate::persistence::write(
                &fixture.join("install-state.ini"),
                &std::collections::BTreeMap::from([
                    ("product".into(), "WyzarCapture".into()),
                    ("schema".into(), "1".into()),
                    ("version".into(), version.into()),
                ]),
            )?;
            render(&mut wizard, &mut c);
            c.save(&folder.join(name))?;
            if dpi == 100 {
                c.save(&root.join(name))?;
            }
        }
        // Moving a registered installation; render fixture only, no registry change.
        wizard.relocation = Some(engine::Registered {
            path: os::default_destination(Scope::User),
            version: Some("0.6.3".into()),
        });
        wizard.checked_path = Some(wizard.path_text.clone());
        render(&mut wizard, &mut c);
        c.save(&folder.join("08-move.png"))?;
        if dpi == 100 {
            c.save(&root.join("08-move.png"))?;
        }
        wizard.relocation = None;
        wizard.checked_path = None;
        wizard.uninstall = true;
        render(&mut wizard, &mut c);
        c.save(&folder.join("07-uninstall.png"))?;
        if dpi == 100 {
            c.save(&root.join("07-uninstall.png"))?;
        }
        std::fs::remove_file(fixture.join("install-state.ini")).map_err(|e| e.to_string())?;
        std::fs::remove_dir(fixture).map_err(|e| e.to_string())?;
        wizard.uninstall = false;
        wizard.options.destination = os::default_destination(Scope::User);
        wizard.path_text = wizard.options.destination.to_string_lossy().into();
    }
    std::fs::write(root.join("render-report.json"),"{\"passed\":true,\"renderer_scales\":[100,125,150,175,200],\"screens\":8,\"update_repair_render_fixtures\":true,\"physical_dpi_test\":false}").map_err(|e|e.to_string())
}
