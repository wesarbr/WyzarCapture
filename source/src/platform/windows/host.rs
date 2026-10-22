use super::{graphics::Canvas, wide};
use crate::{
    core::{AppResult, RecordingState},
    settings::{Settings, appearance::InterfaceMode},
    ui::{
        app::{Action, App, Field, Page, slider_action},
        views,
    },
};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::{Dwm::*, Gdi::*},
        System::{LibraryLoader::*, Registry::*},
        UI::{HiDpi::*, Input::KeyboardAndMouse::*, Shell::*, WindowsAndMessaging::*},
    },
    core::{BOOL, w},
};
const TRAY: u32 = WM_APP + 9;
pub(super) struct Host {
    pub(super) app: App,
    pub(super) canvas: Option<Canvas>,
    pub(super) hwnd: HWND,
    busy: bool,
    pressed: Option<usize>,
    closing: bool,
    hotkeys: [u32; 4],
    surrogate: Option<u16>,
    tray_added: bool,
    smoke_deadline: Option<std::time::Instant>,
    pub(super) countdown: Option<super::countdown::Overlay>,
    start_badge_until: Option<std::time::Instant>,
    pub(super) self_excluded: bool,
    ux_probe: Option<super::ux_probe::Probe>,
    ux_error: Option<String>,
    pub(super) layout_mode: InterfaceMode,
    full_placement: Option<WINDOWPLACEMENT>,
    pub(super) pinned: bool,
    brand_icon: Option<super::brand_icon::BrandIcon>,
    brand_accent: u32,
    uia: super::uia_provider::Bridge,
    uia_qa: Option<std::path::PathBuf>,
    last_paint: std::time::Instant,
    timer_ms: u32,
}
pub fn run(settings: Settings) -> AppResult<()> {
    run_mode(settings, None, None, None, None, None)
}
pub fn run_with_notice(settings: Settings, notice: Option<String>) -> AppResult<()> {
    if notice.is_none() {
        run(settings)
    } else {
        run_mode(settings, None, None, None, notice, None)
    }
}
pub fn smoke(settings: Settings, seconds: u64) -> AppResult<()> {
    run_mode(settings, Some(seconds), None, None, None, None)
}
pub fn open_news(settings: Settings, smoke: Option<u64>) -> AppResult<()> {
    run_mode(settings, smoke, None, Some(Page::News), None, None)
}
pub fn ux_native(mut settings: Settings, root: std::path::PathBuf) -> AppResult<()> {
    settings.appearance.minimize_on_record = true;
    settings.appearance.exclude_self = true;
    settings.countdown = 3;
    settings.tray = false;
    settings.width = 1280;
    settings.height = 720;
    settings.fps = 30;
    run_mode(settings, None, Some(root), None, None, None)
}
pub fn accessibility_qa(settings: Settings, root: std::path::PathBuf) -> AppResult<()> {
    run_mode(settings, None, None, None, None, Some(root))
}
fn run_mode(
    settings: Settings,
    smoke: Option<u64>,
    probe: Option<std::path::PathBuf>,
    initial_page: Option<Page>,
    initial_notice: Option<String>,
    uia_qa: Option<std::path::PathBuf>,
) -> AppResult<()> {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let instance = GetModuleHandleW(None).map_err(|e| e.to_string())?;
        let icon = LoadIconW(
            Some(instance.into()),
            windows::core::PCWSTR(1usize as *const u16),
        )
        .unwrap_or_default();
        let class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            hInstance: instance.into(),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            hIcon: icon,
            hIconSm: icon,
            lpszClassName: w!("WyzarCapture.Native"),
            ..Default::default()
        };
        if RegisterClassExW(&class) == 0 {
            return Err(windows::core::Error::from_win32().to_string());
        }
        let monitor = MonitorFromPoint(POINT::default(), MONITOR_DEFAULTTOPRIMARY);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        GetMonitorInfoW(monitor, &mut info)
            .ok()
            .map_err(|e| e.to_string())?;
        let work = info.rcWork;
        let width = (settings.window_width as i32).min(work.right - work.left - 32);
        let height = (settings.window_height as i32).min(work.bottom - work.top - 32);
        let mut host = Box::new(Host {
            app: App::new(settings),
            canvas: None,
            hwnd: HWND::default(),
            busy: false,
            pressed: None,
            closing: false,
            hotkeys: [0; 4],
            surrogate: None,
            tray_added: false,
            smoke_deadline: smoke.map(|seconds| {
                std::time::Instant::now() + std::time::Duration::from_secs(seconds.clamp(2, 10))
            }),
            countdown: None,
            start_badge_until: None,
            self_excluded: false,
            ux_probe: probe.map(super::ux_probe::Probe::new),
            ux_error: None,
            layout_mode: InterfaceMode::Standard,
            full_placement: None,
            pinned: false,
            brand_icon: None,
            brand_accent: 0,
            uia: super::uia_provider::Bridge::new(),
            uia_qa: uia_qa.clone(),
            last_paint: std::time::Instant::now(),
            timer_ms: 33,
        });
        let system_motion = super::files::animations_enabled();
        host.app.motion.ambient = system_motion;
        // Validation windows keep deterministic frames without transitions.
        host.app.motion.enabled = system_motion
            && host.smoke_deadline.is_none()
            && host.ux_probe.is_none()
            && uia_qa.is_none();
        if let Some(root) = &uia_qa {
            host.smoke_deadline =
                Some(std::time::Instant::now() + std::time::Duration::from_secs(25));
            let path = root.join("Media/calibration.mp4");
            let reader = super::media::Reader::open(&path)?;
            host.app.project.import(path, reader.meta.duration);
            host.app.project.dirty = false;
            host.app.page = Page::Editor;
        }
        if let Some(page) = initial_page {
            host.app.handle(Action::Page(page), HWND::default());
        }
        if let Some(message) = initial_notice {
            host.app.page = Page::Settings;
            host.app.category = 5;
            host.app.failure(message);
            host.app.modal = Some(crate::ui::app::Modal::ErrorDetails);
        }
        let hwnd = CreateWindowExW(
            WS_EX_APPWINDOW,
            w!("WyzarCapture.Native"),
            w!("WyzarCapture"),
            WS_POPUP | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU,
            work.left + (work.right - work.left - width) / 2,
            work.top + (work.bottom - work.top - height) / 2,
            width,
            height,
            None,
            None,
            Some(instance.into()),
            Some((&mut *host as *mut Host).cast()),
        )
        .map_err(|e| e.to_string())?;
        host.hwnd = hwnd;
        if smoke.is_none()
            && host.ux_probe.is_none()
            && initial_page.is_none()
            && host.app.settings.source == crate::core::Source::Display
        {
            host.app.handle(Action::Preview, hwnd);
        }
        let rounding = DWMWCP_ROUND;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            (&rounding as *const DWM_WINDOW_CORNER_PREFERENCE).cast(),
            4,
        );
        let dark = TRUE;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            (&dark as *const BOOL).cast(),
            4,
        );
        let mut client = RECT::default();
        GetClientRect(hwnd, &mut client).map_err(|e| e.to_string())?;
        host.canvas = Some(Canvas::window(
            hwnd,
            (client.right - client.left) as u32,
            (client.bottom - client.top) as u32,
        )?);
        host.countdown = Some(super::countdown::Overlay::new()?);
        host.uia.register(hwnd);
        register_keys(&mut host);
        SetTimer(Some(hwnd), 1, 33, None);
        let _ = ShowWindow(hwnd, SW_SHOW);
        let pmv2 = super::accessibility::per_monitor_v2(hwnd);
        let observer = if let Some(root) = &uia_qa {
            Some(super::accessibility::exercise_own_window(
                hwnd,
                root.join("uia-qa.json"),
            ))
        } else {
            smoke.map(|_| {
                super::accessibility::audit_own_window(
                    hwnd,
                    host.app
                        .settings
                        .data_dir
                        .parent()
                        .unwrap_or(&host.app.settings.data_dir)
                        .join("uia-observation.json"),
                )
            })
        };
        sync_layout(&mut host);
        let _ = UpdateWindow(hwnd);
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        if let Some(observer) = observer {
            let _ = observer.join();
        }
        if let Some(root) = &uia_qa {
            let report =
                std::fs::read_to_string(root.join("uia-qa.json")).map_err(|e| e.to_string())?;
            if !report.contains("\"passed\":true") {
                return Err("UIA acceptance failed; inspect uia-qa.json".into());
            }
        }
        if smoke.is_some() && !pmv2 {
            return Err(
                crate::i18n::tr("O HWND não confirmou Per-Monitor DPI Awareness V2").into(),
            );
        }
        if let Some(page) = initial_page.filter(|_| smoke.is_some()) {
            let path = host
                .app
                .settings
                .data_dir
                .parent()
                .unwrap_or(&host.app.settings.data_dir)
                .join("startup-page.json");
            std::fs::write(
                path,
                format!(
                    "{{\"passed\":{},\"page\":{:?},\"compact_preference\":{}}}",
                    host.app.page == page,
                    host.app.page.key(),
                    host.app.settings.appearance.compact
                ),
            )
            .map_err(|e| e.to_string())?;
        }
        if host.tray_added {
            tray(&mut host, false);
        }
        let _ = host.app.settings.save();
        let _ = host.app.library.save_index(&host.app.settings.data_dir);
        if let Some(error) = host.ux_error.take() {
            return Err(error);
        }
        Ok(())
    }
}
unsafe fn register_keys(h: &mut Host) {
    let keys = if matches!(h.app.modal, Some(crate::ui::app::Modal::Shortcut(_))) {
        [0; 4]
    } else {
        h.app.settings.shortcut_keys()
    };
    if keys == h.hotkeys {
        return;
    }
    for id in 1..=4 {
        let _ = UnregisterHotKey(Some(h.hwnd), id);
    }
    let mut registered = vec![];
    for (index, key) in keys.iter().copied().enumerate() {
        if key == 0 {
            continue;
        }
        let id = index as i32 + 1;
        if registered.contains(&key) {
            h.app.notify(
                crate::i18n::tr("Atalhos duplicados. Escolha teclas diferentes nas configurações."),
                true,
            );
            continue;
        }
        if RegisterHotKey(
            Some(h.hwnd),
            id,
            MOD_NOREPEAT | HOT_KEY_MODIFIERS(key >> 8),
            key & 255,
        )
        .is_err()
        {
            h.app.notify(
                {
                    let __wyzar_i18n_0 = crate::settings::shortcuts::label(key);
                    crate::i18n::format_text(
                        crate::i18n::tr(
                            "O atalho {} está reservado ou em uso. Escolha outra combinação.",
                        ),
                        &[("0", format!("{}", __wyzar_i18n_0))],
                    )
                },
                true,
            );
        }
        registered.push(key);
    }
    h.hotkeys = keys;
}
unsafe fn tray(h: &mut Host, add: bool) {
    let mut data = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: h.hwnd,
        uID: 1,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
        uCallbackMessage: TRAY,
        ..Default::default()
    };
    let text = wide(crate::i18n::tr("WyzarCapture · clique para abrir"));
    data.szTip[..text.len().min(128)].copy_from_slice(&text[..text.len().min(128)]);
    if let Some(icon) = &h.brand_icon {
        data.hIcon = icon.0;
    } else if let Ok(module) = GetModuleHandleW(None) {
        data.hIcon = LoadIconW(
            Some(module.into()),
            windows::core::PCWSTR(1usize as *const u16),
        )
        .unwrap_or_default();
    }
    let _ = Shell_NotifyIconW(
        if add && h.tray_added {
            NIM_MODIFY
        } else if add {
            NIM_ADD
        } else {
            NIM_DELETE
        },
        &data,
    );
    h.tray_added = add;
}
pub(super) unsafe fn dispatch(h: &mut Host, action: Action) {
    h.uia.suspend();
    let was_busy = h.busy;
    match action {
        Action::Minimize => {
            h.busy = true;
            let _ = ShowWindow(h.hwnd, SW_MINIMIZE);
            h.busy = was_busy;
        }
        Action::Maximize => {
            if h.app.interface_mode().compact() {
                h.app
                    .handle(Action::SetInterface(InterfaceMode::Standard), h.hwnd);
                sync_layout(h);
                return;
            }
            h.busy = true;
            let _ = ShowWindow(
                h.hwnd,
                if IsZoomed(h.hwnd).as_bool() {
                    SW_RESTORE
                } else {
                    SW_MAXIMIZE
                },
            );
            sync_surface(h);
            h.busy = was_busy;
        }
        Action::Close => close(h),
        _ => {
            h.busy = true;
            h.app.handle(action, h.hwnd);
            h.busy = was_busy;
        }
    }
    sync_layout(h);
    register_keys(h);
    // Input may arrive before WM_PAINT after a menu/theme change. Publish the
    // new hitboxes immediately so the next click cannot target the old page.
    if let Some(c) = &mut h.canvas {
        views::render(&mut h.app, c);
        h.uia.update(&h.app, c.scale, h.hwnd);
    }
    let _ = InvalidateRect(Some(h.hwnd), None, false);
}
// Layout changes are native window operations. Preserve the full window's placement,
// including maximized state, instead of persisting a compact size as the main size.
pub(super) unsafe fn sync_layout(h: &mut Host) {
    let appearance = &h.app.settings.appearance;
    let accent = crate::ui::theme::PRESETS
        .get(appearance.theme)
        .map(|p| p.accent)
        .unwrap_or(appearance.accent);
    if accent != h.brand_accent {
        if let Ok(icon) =
            super::brand_icon::BrandIcon::new(crate::ui::theme::Palette::new(appearance))
        {
            let busy = h.busy;
            h.busy = true;
            SendMessageW(
                h.hwnd,
                WM_SETICON,
                Some(WPARAM(ICON_BIG as usize)),
                Some(LPARAM(icon.0.0 as isize)),
            );
            SendMessageW(
                h.hwnd,
                WM_SETICON,
                Some(WPARAM(ICON_SMALL as usize)),
                Some(LPARAM(icon.0.0 as isize)),
            );
            h.brand_icon = Some(icon);
            h.brand_accent = accent;
            if h.tray_added {
                tray(h, true);
            }
            h.busy = busy;
        }
    }
    let mode = h.app.interface_mode();
    if mode != h.layout_mode {
        let busy = h.busy;
        h.busy = true;
        let old = h.layout_mode;
        if mode.compact() && !old.compact() {
            let mut placement = WINDOWPLACEMENT {
                length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
                ..Default::default()
            };
            if GetWindowPlacement(h.hwnd, &mut placement).is_ok() {
                // A minimized window must return as visible, not trap the mode switch.
                if placement.showCmd == SW_SHOWMINIMIZED.0 as u32 {
                    placement.showCmd = SW_SHOWNORMAL.0 as u32;
                }
                h.full_placement = Some(placement);
            }
        }
        h.layout_mode = mode;
        if mode.compact() {
            let _ = ShowWindow(h.hwnd, SW_RESTORE);
            let mut rect = RECT::default();
            let _ = GetWindowRect(h.hwnd, &mut rect);
            let monitor = MonitorFromWindow(h.hwnd, MONITOR_DEFAULTTONEAREST);
            let mut mi = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            if GetMonitorInfoW(monitor, &mut mi).as_bool() {
                let dpi = GetDpiForWindow(h.hwnd).max(96) as f32 / 96.;
                let design = mode.design_size();
                let scale = dpi
                    .min((mi.rcWork.right - mi.rcWork.left - 32) as f32 / design.0)
                    .min((mi.rcWork.bottom - mi.rcWork.top - 32) as f32 / design.1)
                    .max(0.5);
                let width = (design.0 * scale).round() as i32;
                let height = (design.1 * scale).round() as i32;
                let (x, y) = if old.compact() {
                    (rect.left, rect.top)
                } else {
                    (
                        (rect.left + rect.right - width) / 2,
                        (rect.top + rect.bottom - height) / 2,
                    )
                };
                let x = x.clamp(
                    mi.rcWork.left,
                    (mi.rcWork.right - width).max(mi.rcWork.left),
                );
                let y = y.clamp(
                    mi.rcWork.top,
                    (mi.rcWork.bottom - height).max(mi.rcWork.top),
                );
                let _ = SetWindowPos(
                    h.hwnd,
                    None,
                    x,
                    y,
                    width,
                    height,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        } else if old.compact() {
            if let Some(placement) = h.full_placement.take() {
                let _ = SetWindowPlacement(h.hwnd, &placement);
            }
        }
        sync_surface(h);
        // Old hitboxes have a different coordinate system. Rebuild before accepting input.
        if let Some(c) = &mut h.canvas {
            views::render(&mut h.app, c);
        }
        h.pressed = None;
        h.app.hover = None;
        h.app.focus_index = None;
        h.busy = busy;
    }
    let pinned = mode.compact() && h.app.settings.appearance.pinned;
    if pinned != h.pinned {
        let busy = h.busy;
        h.busy = true;
        let _ = SetWindowPos(
            h.hwnd,
            Some(if pinned { HWND_TOPMOST } else { HWND_NOTOPMOST }),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
        h.pinned = pinned;
        h.busy = busy;
    }
}
pub(super) unsafe fn sync_surface(h: &mut Host) {
    if IsIconic(h.hwnd).as_bool() {
        return;
    }
    let mut client = RECT::default();
    if GetClientRect(h.hwnd, &mut client).is_ok() && client.right > 0 && client.bottom > 0 {
        if let Some(canvas) = &mut h.canvas {
            canvas.resize(client.right as u32, client.bottom as u32);
        }
        h.app.maximized = IsZoomed(h.hwnd).as_bool();
        if !h.app.maximized && !h.layout_mode.compact() {
            h.app.settings.window_width = client.right as u32;
            h.app.settings.window_height = client.bottom as u32;
        }
    }
}
pub(super) unsafe fn restore_window(h: &mut Host) {
    let busy = h.busy;
    h.busy = true;
    let _ = ShowWindow(h.hwnd, SW_RESTORE);
    sync_surface(h);
    h.busy = busy;
    let _ = InvalidateRect(Some(h.hwnd), None, false);
}
pub(super) unsafe fn close(h: &mut Host) {
    if h.uia_qa.is_some() {
        h.app.project.dirty = false;
    }
    if h.app.settings.tray && !h.closing {
        if !h.tray_added {
            tray(h, true);
        }
        let _ = ShowWindow(h.hwnd, SW_HIDE);
        return;
    }
    if h.app.export_progress.is_some()
        || h.app.recovering
        || h.app.screenshot_pending
        || h.app.probing_capabilities
        || h.app.performance_running
    {
        h.closing = true;
        h.app.notify(
            crate::i18n::tr("Concluindo a tarefa antes de fechar…"),
            false,
        );
        return;
    }
    if h.app.state != RecordingState::Idle {
        h.closing = true;
        h.app.handle(Action::Stop, h.hwnd);
    } else {
        if let Err(e) = h.app.save_on_close() {
            h.app.notify(e, true);
            h.closing = false;
            return;
        }
        let _ = DestroyWindow(h.hwnd);
    }
}
unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        process(hwnd, msg, wparam, lparam)
    }));
    match result {
        Ok(r) => r,
        Err(_) => {
            let _ = std::fs::write(
                super::files::data_folder().join("panic.log"),
                crate::i18n::format_text(
                    crate::i18n::tr("Panic no callback: mensagem {msg}"),
                    &[("msg", format!("{}", msg))],
                ),
            );
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }
}
unsafe fn process(hwnd: HWND, msg: u32, wparam: WPARAM, l: LPARAM) -> LRESULT {
    // This path uses a separate registry, never a reentrant borrow of Host/App.
    if msg == WM_GETOBJECT {
        if let Some(result) = super::uia_provider::get_object(hwnd, wparam, l) {
            return result;
        }
    }
    if msg == WM_NCCREATE {
        let create = &*(l.0 as *const CREATESTRUCTW);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
    }
    if msg == WM_NCCALCSIZE {
        return LRESULT(0);
    }
    if msg == WM_ERASEBKGND {
        return LRESULT(1);
    }
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Host;
    if ptr.is_null() {
        return DefWindowProcW(hwnd, msg, wparam, l);
    }
    // Native dialogs pump a nested message loop. Never alias App's exclusive borrow during it.
    if (*ptr).busy {
        if msg == WM_PAINT {
            let _ = ValidateRect(Some(hwnd), None);
            return LRESULT(0);
        }
        return DefWindowProcW(hwnd, msg, wparam, l);
    }
    let host = &mut *ptr;
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            BeginPaint(hwnd, &mut ps);
            let mut again = false;
            if let Some(canvas) = &mut host.canvas {
                host.app.maximized = IsZoomed(hwnd).as_bool();
                again = views::render(&mut host.app, canvas);
                host.uia.update(&host.app, canvas.scale, hwnd);
            }
            host.last_paint = std::time::Instant::now();
            let _ = EndPaint(hwnd, &ps);
            if again {
                // The display device was rebuilt; draw the frame again on the new target.
                let _ = InvalidateRect(Some(hwnd), None, false);
            }
            LRESULT(0)
        }
        WM_SETTINGCHANGE => {
            let system_motion = super::files::animations_enabled();
            host.app.motion.ambient = system_motion;
            host.app.motion.enabled = system_motion
                && host.smoke_deadline.is_none()
                && host.ux_probe.is_none()
                && host.uia_qa.is_none();
            let _ = InvalidateRect(Some(hwnd), None, false);
            DefWindowProcW(hwnd, msg, wparam, l)
        }
        super::uia_provider::ACTION_MESSAGE => {
            if let Some((index, action, command)) = host.uia.take(&host.app) {
                use super::uia_provider::Command;
                host.app.focus_index = Some(index);
                match command {
                    Command::Invoke => dispatch(host, action),
                    Command::Focus => {
                        let _ = SetFocus(Some(hwnd));
                        if matches!(action, Action::Focus(_)) {
                            dispatch(host, action);
                        }
                    }
                    Command::Text(value) => {
                        if matches!(action, Action::Focus(_)) {
                            dispatch(host, action);
                            host.app.select_all = true;
                            host.app.text_input('\u{8}');
                            for c in value.chars() {
                                host.app.text_input(c);
                            }
                        }
                    }
                    Command::Number(value) => {
                        if let Action::Slider(kind, _) = action {
                            dispatch(
                                host,
                                match kind {
                                    crate::ui::app::SliderKind::Volume => {
                                        Action::SetVolume(value as f32 / 100.)
                                    }
                                    crate::ui::app::SliderKind::Speed => Action::SetSpeed(value),
                                },
                            );
                        }
                    }
                }
                let _ = InvalidateRect(Some(hwnd), None, false);
            }
            LRESULT(0)
        }
        WM_SIZE => {
            if wparam.0 == SIZE_MINIMIZED as usize {
                return LRESULT(0);
            }
            let width = (l.0 as u32 & 0xffff) as u32;
            let height = ((l.0 as u32 >> 16) & 0xffff) as u32;
            if width > 0 && height > 0 {
                if let Some(c) = &mut host.canvas {
                    c.resize(width, height);
                }
                if wparam.0 != SIZE_MAXIMIZED as usize && !host.layout_mode.compact() {
                    host.app.settings.window_width = width;
                    host.app.settings.window_height = height;
                }
            }
            LRESULT(0)
        }
        WM_SYSCOMMAND
            if wparam.0 & 0xfff0 == SC_MAXIMIZE as usize && host.app.interface_mode().compact() =>
        {
            dispatch(host, Action::SetInterface(InterfaceMode::Standard));
            if !IsZoomed(hwnd).as_bool() {
                dispatch(host, Action::Maximize);
            }
            LRESULT(0)
        }
        WM_SYSCHAR if GetKeyState(VK_CONTROL.0 as i32) < 0 && GetKeyState(VK_MENU.0 as i32) < 0 => {
            LRESULT(0)
        }
        WM_GETMINMAXINFO => {
            let info = &mut *(l.0 as *mut MINMAXINFO);
            let mode = host.app.interface_mode();
            let dpi = GetDpiForWindow(hwnd).max(96) as f32 / 96.;
            let design = mode.design_size();
            info.ptMinTrackSize = if mode.compact() {
                POINT {
                    x: (design.0 * dpi.min(1.)) as i32,
                    y: (design.1 * dpi.min(1.)) as i32,
                }
            } else {
                POINT { x: 1040, y: 650 }
            };
            let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
            let mut mi = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            if GetMonitorInfoW(monitor, &mut mi).as_bool() {
                info.ptMinTrackSize.x = info
                    .ptMinTrackSize
                    .x
                    .min(mi.rcWork.right - mi.rcWork.left - 32);
                info.ptMinTrackSize.y = info
                    .ptMinTrackSize
                    .y
                    .min(mi.rcWork.bottom - mi.rcWork.top - 32);
                info.ptMaxPosition = POINT {
                    x: mi.rcWork.left - mi.rcMonitor.left,
                    y: mi.rcWork.top - mi.rcMonitor.top,
                };
                info.ptMaxSize = POINT {
                    x: mi.rcWork.right - mi.rcWork.left,
                    y: mi.rcWork.bottom - mi.rcWork.top,
                };
            }
            LRESULT(0)
        }
        WM_NCHITTEST => {
            let mut point = POINT {
                x: (l.0 as u32 & 0xffff) as i16 as i32,
                y: ((l.0 as u32 >> 16) & 0xffff) as i16 as i32,
            };
            let _ = ScreenToClient(hwnd, &mut point);
            let mut rc = RECT::default();
            let _ = GetClientRect(hwnd, &mut rc);
            if !IsZoomed(hwnd).as_bool() {
                let edge = 6;
                let (left, right, top, bottom) = (
                    point.x < edge,
                    point.x > rc.right - edge,
                    point.y < edge,
                    point.y > rc.bottom - edge,
                );
                let hit = if top && left {
                    HTTOPLEFT
                } else if top && right {
                    HTTOPRIGHT
                } else if bottom && left {
                    HTBOTTOMLEFT
                } else if bottom && right {
                    HTBOTTOMRIGHT
                } else if left {
                    HTLEFT
                } else if right {
                    HTRIGHT
                } else if top {
                    HTTOP
                } else if bottom {
                    HTBOTTOM
                } else {
                    0
                };
                if hit != 0 {
                    return LRESULT(hit as isize);
                }
            }
            let scale = host.canvas.as_ref().map(|c| c.scale).unwrap_or(1.);
            let (x, y) = (point.x as f32 / scale, point.y as f32 / scale);
            if y < if host.app.interface_mode().compact() {
                56.
            } else {
                64.
            } && host.app.action_at(x, y).is_none()
            {
                LRESULT(HTCAPTION as isize)
            } else {
                LRESULT(HTCLIENT as isize)
            }
        }
        WM_DISPLAYCHANGE | WM_DEVICECHANGE => {
            host.app.refresh_devices();
            let _ = InvalidateRect(Some(hwnd), None, false);
            LRESULT(0)
        }
        WM_TIMER => {
            if host
                .smoke_deadline
                .is_some_and(|t| t <= std::time::Instant::now())
            {
                host.smoke_deadline = None;
                host.app.settings.tray = false;
                close(host);
                return LRESULT(0);
            }
            let previous = host.app.state;
            if host
                .app
                .countdown_deadline
                .is_some_and(|d| d <= std::time::Instant::now())
            {
                if let Some(overlay) = &mut host.countdown {
                    overlay.hide();
                }
            }
            host.app.tick();
            sync_layout(host);
            let state = host.app.state;
            if previous == RecordingState::Countdown
                && matches!(state, RecordingState::Starting | RecordingState::Recording)
                && host.app.settings.appearance.minimize_on_record
            {
                dispatch(host, Action::Minimize);
            }
            if previous != RecordingState::Recording
                && state == RecordingState::Recording
                && previous != RecordingState::Paused
            {
                host.start_badge_until =
                    Some(std::time::Instant::now() + std::time::Duration::from_millis(900));
            }
            let excluded = host.app.settings.appearance.exclude_self
                && matches!(
                    host.app.settings.source,
                    crate::core::Source::Display | crate::core::Source::Region
                )
                && (host.app.preview_enabled || state != RecordingState::Idle);
            if excluded != host.self_excluded {
                match SetWindowDisplayAffinity(
                    hwnd,
                    if excluded {
                        WDA_EXCLUDEFROMCAPTURE
                    } else {
                        WDA_NONE
                    },
                ) {
                    Ok(()) => host.self_excluded = excluded,
                    Err(_) => {
                        host.app.settings.appearance.exclude_self = false;
                        host.app.warn(crate::i18n::tr("O Windows não permitiu ocultar o app da captura. Ative minimizar ao gravar."));
                    }
                }
            }
            let capture_monitor = overlay_monitor(&host.app);
            if let Some(overlay) = &mut host.countdown {
                let started = host
                    .start_badge_until
                    .is_some_and(|until| until > std::time::Instant::now())
                    && state == RecordingState::Recording;
                let remaining = host
                    .app
                    .countdown_deadline
                    .map(|d| {
                        d.saturating_duration_since(std::time::Instant::now())
                            .as_millis()
                            .div_ceil(1000) as u32
                    })
                    .unwrap_or(0);
                if (state == RecordingState::Countdown && remaining > 0)
                    || (started && overlay.exclusion_confirmed)
                {
                    if let Some(monitor) = &capture_monitor {
                        overlay.show(
                            monitor,
                            &host.app.settings.appearance,
                            remaining,
                            host.app.settings.stop_hotkey,
                            if host.app.settings.recording_mode == 3 {
                                crate::i18n::tr("Áudio")
                            } else {
                                host.app.settings.source.label()
                            },
                            host.app.animation_started.elapsed().as_secs_f32().fract(),
                            started,
                        );
                    }
                } else {
                    overlay.hide();
                }
            }
            register_keys(host);
            if let Some(mut probe) = host.ux_probe.take() {
                host.busy = true;
                let result = probe.advance(host);
                host.busy = false;
                match result {
                    Ok(true) => {
                        host.app.settings.tray = false;
                        close(host);
                    }
                    Ok(false) => host.ux_probe = Some(probe),
                    Err(error) => {
                        probe.failure(&error);
                        host.ux_error = Some(error);
                        host.app.settings.tray = false;
                        close(host);
                    }
                }
            }
            if host.closing
                && host.app.state == RecordingState::Idle
                && host.app.export_progress.is_none()
                && !host.app.recovering
                && !host.app.screenshot_pending
                && !host.app.performance_running
                && !host.app.probing_capabilities
            {
                close(host);
            } else {
                // Repaint only what changes: transitions run at 60 Hz, previews and
                // decoration at their own pace, an idle window a few times per second.
                // Validation modes keep the original fixed cadence.
                let qa = host.ux_probe.is_some()
                    || host.uia_qa.is_some()
                    || host.smoke_deadline.is_some();
                let visible = IsWindowVisible(hwnd).as_bool() && !IsIconic(hwnd).as_bool();
                if qa || (visible && host.last_paint.elapsed() >= host.app.frame_interval()) {
                    let _ = InvalidateRect(Some(hwnd), None, false);
                }
                let wanted = if host.app.motion.busy() { 16 } else { 33 };
                if wanted != host.timer_ms {
                    SetTimer(Some(hwnd), 1, wanted, None);
                    host.timer_ms = wanted;
                }
            }
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            let scale = host.canvas.as_ref().map(|c| c.scale).unwrap_or(1.);
            let x = (l.0 as u32 & 0xffff) as i16 as f32 / scale;
            let y = ((l.0 as u32 >> 16) & 0xffff) as i16 as f32 / scale;
            let hit = host.app.action_at(x, y);
            if host.pressed.is_some() {
                if let Some(action @ Action::Slider(_, _)) = host
                    .pressed
                    .and_then(|i| host.app.hits.get(i))
                    .map(|h| h.action.clone())
                {
                    dispatch(host, slider_action(action, x));
                }
                if let Some(Action::Seek(t)) = hit
                    .and_then(|i| host.app.hits.get(i))
                    .map(|h| h.action.clone())
                {
                    host.app.handle(Action::Seek(t), hwnd);
                }
            }
            if host.app.hover != hit {
                host.app.hover = hit;
                host.app.hover_since = std::time::Instant::now();
                let rect = hit.and_then(|i| host.app.hits.get(i)).map(|h| h.rect);
                host.app.motion.set_hover(rect);
                let _ = InvalidateRect(Some(hwnd), None, false);
            }
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let scale = host.canvas.as_ref().map(|c| c.scale).unwrap_or(1.);
            let (x, y) = (
                (l.0 as u32 & 0xffff) as i16 as f32 / scale,
                ((l.0 as u32 >> 16) & 0xffff) as i16 as f32 / scale,
            );
            let hit = if let Some(c) = &mut host.canvas {
                views::pointer_hit(&mut host.app, c, x, y)
            } else {
                host.app.action_at(x, y)
            };
            if hit.is_none() {
                host.app.blur_text();
                host.app.focus_index = None;
            }
            if host.app.popup.is_some() && hit.is_none() {
                host.app.popup = None;
            } else {
                host.pressed = hit;
                host.app.focus_index = hit;
                if let Some(action @ Action::Slider(_, _)) = hit
                    .and_then(|i| host.app.hits.get(i))
                    .map(|h| h.action.clone())
                {
                    host.app.checkpoint();
                    dispatch(host, slider_action(action, x));
                }
                SetCapture(hwnd);
            }
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let _ = ReleaseCapture();
            let scale = host.canvas.as_ref().map(|c| c.scale).unwrap_or(1.);
            let (x, y) = (
                (l.0 as u32 & 0xffff) as i16 as f32 / scale,
                ((l.0 as u32 >> 16) & 0xffff) as i16 as f32 / scale,
            );
            let hit = host.app.action_at(x, y);
            if let Some(i) = host.pressed.take().filter(|p| Some(*p) == hit) {
                let action = host
                    .app
                    .hits
                    .get(i)
                    .map(|h| h.action.clone())
                    .unwrap_or(Action::None);
                if !matches!(action, Action::Focus(_) | Action::ConfirmText) {
                    host.app.blur_text();
                }
                dispatch(host, slider_action(action, x));
            }
            LRESULT(0)
        }
        WM_MOUSEWHEEL => {
            let delta = ((wparam.0 >> 16) & 0xffff) as i16 as f32;
            host.app.scroll_by(-delta);
            let _ = InvalidateRect(Some(hwnd), None, false);
            LRESULT(0)
        }
        WM_CHAR => {
            if matches!(host.app.modal, Some(crate::ui::app::Modal::Shortcut(_))) {
                return LRESULT(0);
            }
            let unit = wparam.0 as u16;
            if (0xd800..=0xdbff).contains(&unit) {
                host.surrogate = Some(unit);
            } else if let Some(high) = host.surrogate.take() {
                if let Some(c) = char::decode_utf16([high, unit]).next().and_then(Result::ok) {
                    host.app.text_input(c);
                }
            } else if let Some(c) = char::from_u32(wparam.0 as u32) {
                host.app.text_input(c);
            }
            LRESULT(0)
        }
        WM_KEYUP
            if wparam.0 == VK_SNAPSHOT.0 as usize
                && matches!(host.app.modal, Some(crate::ui::app::Modal::Shortcut(_))) =>
        {
            if let Some(crate::ui::app::Modal::Shortcut(index)) = host.app.modal {
                let modifiers = ((GetKeyState(VK_MENU.0 as i32) < 0) as u32)
                    | (((GetKeyState(VK_CONTROL.0 as i32) < 0) as u32) << 1)
                    | (((GetKeyState(VK_SHIFT.0 as i32) < 0) as u32) << 2);
                dispatch(host, Action::ApplyShortcut(index, 0x2c | (modifiers << 8)));
            }
            LRESULT(0)
        }
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            let key = wparam.0 as u16;
            let ctrl = GetKeyState(VK_CONTROL.0 as i32) < 0;
            let shift = GetKeyState(VK_SHIFT.0 as i32) < 0;
            let alt = GetKeyState(VK_MENU.0 as i32) < 0;
            if let Some(crate::ui::app::Modal::Shortcut(index)) = host.app.modal {
                if key == VK_ESCAPE.0 {
                    dispatch(host, Action::CloseModal);
                } else if let Some(value) = crate::settings::shortcuts::encode(
                    key as u32,
                    (alt as u32)
                        | ((ctrl as u32) << 1)
                        | ((shift as u32) << 2)
                        | (((GetKeyState(VK_LWIN.0 as i32) < 0 || GetKeyState(VK_RWIN.0 as i32) < 0)
                            as u32)
                            << 3),
                ) {
                    dispatch(host, Action::ApplyShortcut(index, value));
                }
            } else if let Some(mode) = InterfaceMode::shortcut(key, ctrl, alt) {
                dispatch(host, Action::SetInterface(mode));
            } else if ctrl
                && shift
                && key == 0x54
                && host.app.focused.is_none()
                && host.app.page == crate::ui::app::Page::Editor
            {
                host.app.handle(Action::AudioTracks, hwnd);
            } else if ctrl && shift && key == 0x41 && host.app.focused.is_none() {
                dispatch(host, Action::NewAudioRecording);
            } else if ctrl && key == 0x41 {
                host.app.select_all = true;
            } else if ctrl && key == 0x56 && host.app.focused.is_some() {
                if let Some(text) = super::files::clipboard_text(hwnd) {
                    for c in text.chars() {
                        host.app.text_input(c);
                    }
                }
            } else if key == VK_ESCAPE.0 {
                if host.app.state == RecordingState::Countdown {
                    dispatch(host, Action::Stop);
                }
                if host.app.popup.is_some() {
                    host.app.popup = None;
                } else {
                    dispatch(host, Action::CloseModal);
                }
                host.app.blur_text();
                host.app.focus_index = None;
            } else if key == 0x70 {
                let topic = if host.app.interface_mode().compact() {
                    12
                } else {
                    crate::ui::help::contextual(host.app.page)
                };
                host.app.help_search.clear();
                dispatch(host, Action::HelpTopic(topic));
            } else if key == VK_TAB.0 {
                host.app.blur_text();
                let len = host.app.hits.len();
                if len > 0 {
                    let n = host
                        .app
                        .focus_index
                        .unwrap_or(if shift { 0 } else { len - 1 });
                    host.app.focus_index = Some(if shift {
                        (n + len - 1) % len
                    } else {
                        (n + 1) % len
                    });
                    if let Some(Action::Focus(field)) = host
                        .app
                        .focus_index
                        .and_then(|i| host.app.hits.get(i))
                        .map(|h| h.action.clone())
                    {
                        host.app.handle(Action::Focus(field), hwnd);
                    }
                }
            } else if host.app.focused.is_none() && matches!(key, 0x21 | 0x22 | 0x24 | 0x23) {
                let distance = match key {
                    0x21 => -400.,
                    0x22 => 400.,
                    0x24 => -1_000_000.,
                    _ => 1_000_000.,
                };
                host.app.scroll_by(distance);
            } else if host.app.popup.is_some() && (key == VK_UP.0 || key == VK_DOWN.0) {
                let len = host.app.hits.len();
                if len > 0 {
                    let index = host.app.focus_index.unwrap_or(0);
                    host.app.focus_index = Some(if key == VK_UP.0 {
                        (index + len - 1) % len
                    } else {
                        (index + 1) % len
                    });
                }
            } else if key == VK_RETURN.0
                || (key == VK_SPACE.0
                    && host.app.focused.is_none()
                    && host.app.focus_index.is_some())
            {
                if matches!(
                    host.app.focused,
                    Some(Field::Text | Field::Rename | Field::Tags | Field::Appearance(_))
                ) {
                    dispatch(host, Action::ConfirmText);
                } else if let Some(i) = host.app.focus_index {
                    if let Some(hit) = host.app.hits.get(i) {
                        dispatch(host, hit.action.clone());
                    }
                }
            } else if ctrl
                && key == 0x53
                && host.app.focused.is_none()
                && host.app.page == Page::Editor
                && host.app.modal.is_none()
            {
                dispatch(host, Action::SaveProject);
            } else if ctrl
                && key == 0x5a
                && host.app.focused.is_none()
                && host.app.page == Page::Editor
                && host.app.modal.is_none()
            {
                dispatch(host, if shift { Action::Redo } else { Action::Undo });
            } else if ctrl
                && key == 0x59
                && host.app.focused.is_none()
                && host.app.page == Page::Editor
                && host.app.modal.is_none()
            {
                dispatch(host, Action::Redo);
            } else if ctrl
                && key == 0x45
                && host.app.focused.is_none()
                && host.app.page == Page::Editor
                && host.app.modal.is_none()
            {
                dispatch(host, Action::Export);
            } else if host.app.focused.is_none()
                && host.app.page == Page::Editor
                && host.app.modal.is_none()
            {
                if key == VK_SPACE.0 {
                    dispatch(host, Action::Play);
                } else if key == 0x53 {
                    dispatch(host, Action::Split);
                } else if key == VK_DELETE.0 {
                    dispatch(host, Action::RemoveClip);
                } else if key == VK_LEFT.0 || key == VK_RIGHT.0 {
                    let position =
                        host.app.project.playhead + if key == VK_LEFT.0 { -1. } else { 1. };
                    dispatch(host, Action::Seek(position));
                }
            }
            LRESULT(0)
        }
        WM_HOTKEY => {
            if matches!(host.app.modal, Some(crate::ui::app::Modal::Shortcut(_))) {
                return LRESULT(0);
            }
            dispatch(
                host,
                match wparam.0 {
                    1 => Action::StartRecording,
                    2 => Action::Pause,
                    3 => Action::Stop,
                    4 => Action::TakeScreenshot,
                    _ => Action::None,
                },
            );
            LRESULT(0)
        }
        WM_DPICHANGED => {
            let mut r = *(l.0 as *const RECT);
            let monitor = MonitorFromRect(&r, MONITOR_DEFAULTTONEAREST);
            let mut info = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            if GetMonitorInfoW(monitor, &mut info).as_bool() {
                let work = info.rcWork;
                let fitted = crate::core::window::fit(
                    crate::core::window::Bounds {
                        left: r.left,
                        top: r.top,
                        right: r.right,
                        bottom: r.bottom,
                    },
                    crate::core::window::Bounds {
                        left: work.left,
                        top: work.top,
                        right: work.right,
                        bottom: work.bottom,
                    },
                );
                r = RECT {
                    left: fitted.left,
                    top: fitted.top,
                    right: fitted.right,
                    bottom: fitted.bottom,
                };
            }
            let _ = SetWindowPos(
                hwnd,
                None,
                r.left,
                r.top,
                r.right - r.left,
                r.bottom - r.top,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
            LRESULT(0)
        }
        super::setup::CLOSE_FOR_INSTALL => {
            host.closing = true;
            close(host);
            LRESULT(0)
        }
        WM_CLOSE => {
            close(host);
            LRESULT(0)
        }
        TRAY => {
            if l.0 as u32 == WM_LBUTTONDBLCLK || l.0 as u32 == WM_LBUTTONUP {
                let _ = ShowWindow(hwnd, SW_SHOW);
                let _ = SetForegroundWindow(hwnd);
            } else if l.0 as u32 == WM_RBUTTONUP {
                if let Ok(menu) = CreatePopupMenu() {
                    let _ = AppendMenuW(
                        menu,
                        MF_STRING,
                        1,
                        crate::platform::windows::localized("Abrir WyzarCapture"),
                    );
                    let _ = AppendMenuW(
                        menu,
                        MF_STRING,
                        2,
                        crate::platform::windows::localized("Sair"),
                    );
                    let mut p = POINT::default();
                    let _ = GetCursorPos(&mut p);
                    let _ = SetForegroundWindow(hwnd);
                    let chosen = TrackPopupMenu(menu, TPM_RETURNCMD, p.x, p.y, None, hwnd, None);
                    let _ = DestroyMenu(menu);
                    if chosen.0 == 1 {
                        let _ = ShowWindow(hwnd, SW_SHOW);
                    } else if chosen.0 == 2 {
                        host.closing = true;
                        close(host);
                    }
                }
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            host.uia.unregister(hwnd);
            let _ = KillTimer(Some(hwnd), 1);
            for id in 1..=4 {
                let _ = UnregisterHotKey(Some(hwnd), id);
            }
            if host.tray_added {
                tray(host, false);
            }
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, l),
    }
}
fn overlay_monitor(app: &App) -> Option<super::devices::Monitor> {
    unsafe {
        let native = match app.settings.source {
            crate::core::Source::Window | crate::core::Source::Game => app
                .window
                .map(|w| MonitorFromWindow(HWND(w as *mut _), MONITOR_DEFAULTTONEAREST)),
            crate::core::Source::Region => app.region.map(|r| {
                MonitorFromPoint(
                    POINT {
                        x: r.x + r.width as i32 / 2,
                        y: r.y + r.height as i32 / 2,
                    },
                    MONITOR_DEFAULTTONEAREST,
                )
            }),
            _ => None,
        };
        native
            .and_then(|m| {
                app.monitors
                    .iter()
                    .find(|monitor| monitor.handle == m.0 as isize)
            })
            .or_else(|| app.monitors.get(app.settings.monitor))
            .or_else(|| app.monitors.first())
            .cloned()
    }
}
pub fn set_startup(enabled: bool) -> AppResult<()> {
    unsafe {
        let mut key = HKEY::default();
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            Some(0),
            KEY_SET_VALUE,
            &mut key,
        )
        .ok()
        .map_err(|e| e.to_string())?;
        let result = if enabled {
            let path = std::env::current_exe().map_err(|e| e.to_string())?;
            let value = wide(format!("\"{}\"", path.display()));
            let bytes = std::slice::from_raw_parts(value.as_ptr().cast::<u8>(), value.len() * 2);
            RegSetValueExW(key, w!("WyzarCapture"), Some(0), REG_SZ, Some(bytes)).ok()
        } else {
            let code = RegDeleteValueW(key, w!("WyzarCapture"));
            if code == ERROR_FILE_NOT_FOUND {
                Ok(())
            } else {
                code.ok()
            }
        };
        let _ = RegCloseKey(key);
        result.map_err(|e| e.to_string())
    }
}
