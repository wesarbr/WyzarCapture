//! Desktop countdown owned by this process. It never takes focus or enters recordings.
use super::{devices::Monitor, graphics::Canvas};
use crate::{core::AppResult, settings::appearance::Appearance, ui::theme::*};
use windows::{
    Win32::{
        Foundation::*, Graphics::Gdi::*, System::LibraryLoader::*, UI::WindowsAndMessaging::*,
    },
    core::w,
};

pub fn paint(
    canvas: &mut Canvas,
    appearance: &Appearance,
    remaining: u32,
    stop_key: u32,
    source: &str,
    phase: f32,
    started: bool,
) {
    canvas.palette = Palette::new(appearance);
    let (w, h) = canvas.begin();
    canvas.fill(Rect::new(0., 0., w, h), Color::literal(0x05070a));
    let r = Rect::new((w - 456.) / 2., (h - 402.) / 2., 456., 402.);
    canvas.round(r, 24., PANEL, Some(BORDER));
    canvas.center(
        if started {
            crate::i18n::tr("SUA CAPTURA COMEÇOU")
        } else {
            crate::i18n::tr("PREPARE SUA CAPTURA")
        },
        Rect::new(r.x + 24., r.y + 25., r.w - 48., 31.),
        15.,
        MUTED,
        true,
    );
    let cy = r.y + 163.;
    canvas.ring(w / 2., cy, 72. + phase * 5., GOLD.alpha(0.08), 8.);
    canvas.ring(w / 2., cy, 68., if started { GREEN } else { GOLD }, 3.);
    if started {
        canvas.icon("check", Rect::new(w / 2. - 32., cy - 32., 64., 64.), GREEN);
    } else {
        canvas.center(
            &remaining.to_string(),
            Rect::new(w / 2. - 60., cy - 63., 120., 126.),
            88.,
            GOLD,
            true,
        );
    }
    canvas.center(
        if started {
            crate::i18n::tr("Gravando")
        } else {
            crate::i18n::tr("A gravação vai começar")
        },
        Rect::new(r.x + 20., r.y + 265., r.w - 40., 37.),
        23.,
        TEXT,
        true,
    );
    canvas.center(
        source,
        Rect::new(r.x + 20., r.y + 311., r.w - 40., 29.),
        14.,
        MUTED,
        false,
    );
    canvas.center(
        &{
            let __wyzar_i18n_0 = crate::settings::shortcuts::label(stop_key);
            let __wyzar_i18n_1 = &(if started {
                crate::i18n::tr("parar e salvar")
            } else {
                "cancelar"
            });
            crate::i18n::format_text(
                crate::i18n::tr("{} para {}"),
                &[
                    ("0", format!("{}", __wyzar_i18n_0)),
                    ("1", format!("{}", __wyzar_i18n_1)),
                ],
            )
        },
        Rect::new(r.x + 20., r.y + 358., r.w - 40., 24.),
        13.,
        DIM,
        false,
    );
    let _ = canvas.end();
}
struct State {
    canvas: Option<Canvas>,
    appearance: Appearance,
    remaining: u32,
    stop_key: u32,
    source: String,
    phase: f32,
    started: bool,
}
pub struct Overlay {
    hwnd: HWND,
    state: Box<State>,
    pub visible: bool,
    pub exclusion_confirmed: bool,
}
impl Overlay {
    pub fn verify_monitor(&self, monitor: Option<&Monitor>) -> AppResult<bool> {
        unsafe {
            let Some(monitor) = monitor else {
                return Err(crate::i18n::tr("Monitor não disponível").into());
            };
            let mut r = RECT::default();
            GetWindowRect(self.hwnd, &mut r).map_err(|e| e.to_string())?;
            let style = GetWindowLongPtrW(self.hwnd, GWL_EXSTYLE) as u32;
            Ok(IsWindowVisible(self.hwnd).as_bool()
                && GetForegroundWindow() != self.hwnd
                && (style & WS_EX_TOPMOST.0) != 0
                && r.left == monitor.x
                && r.top == monitor.y
                && r.right - r.left == monitor.width as i32
                && r.bottom - r.top == monitor.height as i32)
        }
    }
    pub fn new() -> AppResult<Self> {
        unsafe {
            let instance = GetModuleHandleW(None).map_err(|e| e.to_string())?;
            let class = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                lpfnWndProc: Some(proc),
                hInstance: instance.into(),
                lpszClassName: w!("WyzarCapture.Countdown"),
                ..Default::default()
            };
            RegisterClassExW(&class);
            let mut state = Box::new(State {
                canvas: None,
                appearance: Default::default(),
                remaining: 3,
                stop_key: 0x79,
                source: String::new(),
                phase: 0.,
                started: false,
            });
            let hwnd = CreateWindowExW(
                WS_EX_TOPMOST
                    | WS_EX_TOOLWINDOW
                    | WS_EX_NOACTIVATE
                    | WS_EX_LAYERED
                    | WS_EX_TRANSPARENT,
                w!("WyzarCapture.Countdown"),
                crate::platform::windows::localized("WyzarCapture · Contagem regressiva"),
                WS_POPUP,
                0,
                0,
                320,
                240,
                None,
                None,
                Some(instance.into()),
                Some((&mut *state as *mut State).cast()),
            )
            .map_err(|e| e.to_string())?;
            let mut overlay = Self {
                hwnd,
                state,
                visible: false,
                exclusion_confirmed: false,
            };
            SetLayeredWindowAttributes(hwnd, COLORREF(0), 235, LWA_ALPHA)
                .map_err(|e| e.to_string())?;
            let set = SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE).is_ok();
            let mut affinity = 0;
            overlay.exclusion_confirmed = set
                && GetWindowDisplayAffinity(hwnd, &mut affinity).is_ok()
                && affinity == WDA_EXCLUDEFROMCAPTURE.0;
            overlay.state.canvas = Some(Canvas::window(hwnd, 320, 240)?);
            Ok(overlay)
        }
    }
    pub fn show(
        &mut self,
        monitor: &Monitor,
        appearance: &Appearance,
        remaining: u32,
        stop_key: u32,
        source: &str,
        phase: f32,
        started: bool,
    ) {
        self.state.appearance = appearance.clone();
        self.state.remaining = remaining;
        self.state.stop_key = stop_key;
        self.state.source = source.into();
        self.state.phase = phase;
        self.state.started = started;
        if let Some(c) = &mut self.state.canvas {
            if c.width != monitor.width as f32 || c.height != monitor.height as f32 {
                c.resize(monitor.width, monitor.height);
            }
        }
        unsafe {
            if !self.visible {
                let _ = SetWindowPos(
                    self.hwnd,
                    Some(HWND_TOPMOST),
                    monitor.x,
                    monitor.y,
                    monitor.width as i32,
                    monitor.height as i32,
                    SWP_NOACTIVATE | SWP_SHOWWINDOW,
                );
                let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
                self.visible = true;
            }
            let _ = InvalidateRect(Some(self.hwnd), None, false);
        }
    }
    pub fn hide(&mut self) {
        if self.visible {
            unsafe {
                let _ = ShowWindow(self.hwnd, SW_HIDE);
            }
            self.visible = false;
        }
    }
}
impl Drop for Overlay {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}
unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_NCCREATE {
        let create = &*(lparam.0 as *const CREATESTRUCTW);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
    }
    if msg == WM_PAINT {
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut State;
        let mut ps = PAINTSTRUCT::default();
        BeginPaint(hwnd, &mut ps);
        if let Some(state) = ptr.as_mut() {
            if let Some(c) = &mut state.canvas {
                paint(
                    c,
                    &state.appearance,
                    state.remaining,
                    state.stop_key,
                    &state.source,
                    state.phase,
                    state.started,
                );
            }
        }
        let _ = EndPaint(hwnd, &ps);
        return LRESULT(0);
    }
    if msg == WM_ERASEBKGND {
        return LRESULT(1);
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}
