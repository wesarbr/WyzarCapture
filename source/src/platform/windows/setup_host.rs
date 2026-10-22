//! Native setup window; installer work runs outside the message thread.
use super::{graphics::Canvas, pcw, wide};
use crate::{
    core::AppResult,
    installer::{
        Scope,
        view::{self, Action, Page, Wizard},
    },
};
use std::sync::atomic::Ordering;
use windows::{
    Win32::{
        Foundation::*,
        Graphics::{Dwm::*, Gdi::*},
        System::LibraryLoader::*,
        UI::{HiDpi::*, Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
    core::{BOOL, w},
};
struct Host {
    model: Wizard,
    canvas: Option<Canvas>,
    hwnd: HWND,
    busy: bool,
    pressed: Option<usize>,
    surrogate: Option<u16>,
    smoke: Option<std::path::PathBuf>,
    started: std::time::Instant,
    paints: u32,
    ticks: u32,
}
pub fn message(text: &str) {
    unsafe {
        MessageBoxW(
            None,
            pcw(&wide(text)),
            crate::platform::windows::localized("WyzarCapture — Instalador"),
            MB_OK | MB_ICONERROR,
        );
    }
}
pub fn run(model: Wizard) -> AppResult<()> {
    run_inner(model, None)
}
pub fn smoke(model: Wizard, root: std::path::PathBuf) -> AppResult<()> {
    crate::installer::guard(&root)?;
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    run_inner(model, Some(root))
}
fn run_inner(model: Wizard, smoke: Option<std::path::PathBuf>) -> AppResult<()> {
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
            lpfnWndProc: Some(callback),
            hInstance: instance.into(),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            hIcon: icon,
            hIconSm: icon,
            lpszClassName: w!("WyzarCapture.Setup"),
            ..Default::default()
        };
        if RegisterClassExW(&class) == 0 {
            return Err(windows::core::Error::from_win32().to_string());
        }
        let monitor = MonitorFromPoint(POINT::default(), MONITOR_DEFAULTTOPRIMARY);
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let _ = GetMonitorInfoW(monitor, &mut mi);
        let work = mi.rcWork;
        let width = 1280.min(work.right - work.left - 32).max(900);
        let height = 700.min(work.bottom - work.top - 32).max(490);
        let mut host = Box::new(Host {
            model,
            canvas: None,
            hwnd: HWND::default(),
            busy: false,
            pressed: None,
            surrogate: None,
            smoke,
            started: std::time::Instant::now(),
            paints: 0,
            ticks: 0,
        });
        let hwnd = CreateWindowExW(
            WS_EX_APPWINDOW,
            w!("WyzarCapture.Setup"),
            crate::platform::windows::localized("WyzarCapture — Instalador"),
            WS_POPUP | WS_THICKFRAME | WS_MINIMIZEBOX | WS_SYSMENU,
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
        let round = DWMWCP_ROUND;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            (&round as *const DWM_WINDOW_CORNER_PREFERENCE).cast(),
            4,
        );
        let dark = BOOL(1);
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            (&dark as *const BOOL).cast(),
            4,
        );
        host.canvas = Some(Canvas::window(hwnd, width as u32, height as u32)?);
        SetTimer(Some(hwnd), 1, 33, None);
        let _ = ShowWindow(hwnd, SW_SHOW);
        let observer = host.smoke.as_ref().map(|root| {
            super::accessibility::audit_own_window(hwnd, root.join("uia-observation.json"))
        });
        if host.model.resume {
            perform(&mut host, Action::Next);
        }
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        if let Some(observer) = observer {
            let _ = observer.join();
        }
        Ok(())
    }
}
unsafe fn perform(h: &mut Host, action: Action) {
    if !matches!(action, Action::LanguageMenu | Action::SelectLanguage(_)) {
        h.model.language_open = false;
    }
    let result = (|| -> AppResult<()> {
        match action {
            Action::ViewLicense => {
                h.model.license_open = true;
                h.model.license_page = 0;
            }
            Action::LicensePage(next) => {
                h.model.license_page = if next {
                    (h.model.license_page + 1).min(crate::license::page_count().saturating_sub(1))
                } else {
                    h.model.license_page.saturating_sub(1)
                };
            }
            Action::CloseLicense => {
                h.model.license_open = false;
            }
            Action::LanguageMenu => {
                h.model.language_open = !h.model.language_open;
                h.model.focus = None;
            }
            Action::SelectLanguage(index) => {
                h.model
                    .select_language(crate::i18n::Language::ALL[index.min(4)]);
                let caption = format!(
                    "WyzarCapture — {}",
                    crate::i18n::tr(if h.model.uninstall {
                        "Desinstalador"
                    } else {
                        "Instalador"
                    })
                );
                let _ = SetWindowTextW(h.hwnd, pcw(&wide(caption)));
            }
            Action::RemoveData(kind) => {
                if kind < 3 {
                    h.model.remove_data[kind] = !h.model.remove_data[kind];
                }
            }
            Action::OpenFolder | Action::Details => {
                let path = if matches!(action, Action::Details) {
                    h.model
                        .options
                        .destination
                        .join("wyzar-uninstall-preserved.txt")
                } else {
                    h.model.options.destination.clone()
                };
                h.busy = true;
                super::files::open_path(h.hwnd, &path);
                h.busy = false;
            }
            Action::Minimize => {
                h.busy = true;
                let _ = ShowWindow(h.hwnd, SW_MINIMIZE);
                h.busy = false;
            }
            Action::Close | Action::Cancel => {
                if h.model.close_prompt {
                    h.model.close_prompt = false;
                } else if h.model.page == Page::Installing {
                    if h.model.uninstall {
                        h.model.progress.operation =
                            crate::i18n::tr("Concluindo a remoção transacional antes de fechar…")
                                .into();
                    } else {
                        h.model.cancel.store(true, Ordering::Relaxed);
                        h.model.progress.operation =
                            crate::i18n::tr("Cancelando com preservação da instalação anterior…")
                                .into();
                    }
                } else {
                    let _ = DestroyWindow(h.hwnd);
                }
            }
            Action::Next | Action::CloseApp => {
                if h.model.page == Page::Welcome {
                    h.model.page = Page::Destination;
                } else if h.model.page == Page::Destination {
                    h.model.options.destination =
                        super::setup::install_folder(h.model.path_text.clone().into())?;
                    h.model.path_text = h.model.options.destination.to_string_lossy().into();
                    h.model.options.scope.ensure_available()?;
                    super::setup::validate_destination(&h.model.options)?;
                    if h.model.options.scope == Scope::Machine && !super::setup::is_admin() {
                        let mut args = vec![
                            "--continue-install".into(),
                            "--scope".into(),
                            "machine".into(),
                            "--target".into(),
                            h.model.path_text.clone(),
                        ];
                        if h.model.uninstall {
                            args.push("--uninstall".into());
                        }
                        for (on, arg) in [
                            (h.model.options.desktop, "--desktop"),
                            (h.model.options.start_menu, "--menu"),
                            (h.model.options.startup, "--startup"),
                            (h.model.options.allow_downgrade, "--allow-downgrade"),
                            (h.model.options.move_existing, "--move-existing"),
                            (matches!(action, Action::CloseApp), "--close-app"),
                            (h.model.remove_data[0], "--remove-preferences"),
                            (h.model.remove_data[1], "--remove-cache"),
                            (h.model.remove_data[2], "--remove-logs"),
                        ] {
                            if on {
                                args.push(arg.into());
                            }
                        }
                        h.busy = true;
                        let elevated = super::setup::elevate(&h.model.exe, &args);
                        h.busy = false;
                        elevated?;
                        let _ = DestroyWindow(h.hwnd);
                    } else {
                        if matches!(action, Action::CloseApp) {
                            h.model.options.close_running = true;
                            h.model.close_prompt = false;
                        }
                        h.model.begin()?;
                    }
                }
            }
            Action::Back => h.model.page = Page::Welcome,
            Action::KeepCurrent => h.model.keep_current(),
            Action::Browse => {
                h.busy = true;
                let result = super::files::choose_folder(h.hwnd, &h.model.options.destination);
                h.busy = false;
                if let Some(path) = result? {
                    let path = super::setup::install_folder(path)?;
                    if h.model.options.scope == Scope::User || super::setup::is_admin() {
                        crate::persistence::test_folder(&path)?;
                    }
                    h.model.options.destination = path.clone();
                    h.model.path_text = path.to_string_lossy().into();
                    h.model.free = super::files::free_space(&path);
                }
            }
            Action::ConfigureMedia => {
                h.model.options.media_root = if h.model.options.media_root.is_some() {
                    None
                } else {
                    Some(h.model.media_root.clone())
                };
            }
            Action::BrowseMedia => {
                h.busy = true;
                let chosen = super::files::choose_folder(h.hwnd, &h.model.media_root);
                h.busy = false;
                if let Some(path) = chosen? {
                    crate::settings::bootstrap::prepare_media(&path)?;
                    h.model.media_root = path.clone();
                    h.model.options.media_root = Some(path);
                }
            }
            Action::User | Action::Machine => {
                if matches!(action, Action::Machine) {
                    Scope::Machine.ensure_available()?;
                }
                h.model.options.scope = if matches!(action, Action::User) {
                    Scope::User
                } else {
                    Scope::Machine
                };
                h.model.options.destination =
                    super::setup::default_destination(h.model.options.scope);
                h.model.path_text = h.model.options.destination.to_string_lossy().into();
                h.model.free = super::files::free_space(&h.model.options.destination);
            }
            Action::Desktop => h.model.options.desktop = !h.model.options.desktop,
            Action::Menu => h.model.options.start_menu = !h.model.options.start_menu,
            Action::Startup => h.model.options.startup = !h.model.options.startup,
            Action::AllowDowngrade => {
                h.model.options.allow_downgrade = !h.model.options.allow_downgrade
            }
            Action::Path => {
                h.model.path_focus = true;
                h.model.path_selected = true;
            }
            Action::Launch => h.model.open_app = !h.model.open_app,
            Action::Notes => h.model.notes = !h.model.notes,
            Action::Finish => {
                if !h.model.uninstall {
                    if h.model.open_app || h.model.notes {
                        let args = if h.model.notes {
                            vec!["--news".into()]
                        } else {
                            vec![]
                        };
                        let app = crate::installer::engine::verified_owned_file(
                            &h.model.options.destination,
                            "WyzarCapture.exe",
                        )?;
                        super::setup::launch(&app, &args)?;
                    }
                }
                let _ = DestroyWindow(h.hwnd);
            }
        }
        Ok(())
    })();
    if let Err(e) = result {
        h.model.error = Some(e);
    }
    let _ = InvalidateRect(Some(h.hwnd), None, false);
}
unsafe extern "system" fn callback(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| process(hwnd, msg, w, l)))
        .unwrap_or_else(|_| DefWindowProcW(hwnd, msg, w, l))
}
unsafe fn process(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    if msg == WM_NCCREATE {
        let create = &*(l.0 as *const CREATESTRUCTW);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
    }
    if msg == WM_NCCALCSIZE {
        return LRESULT(0);
    }
    let p = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Host;
    if p.is_null() {
        return DefWindowProcW(hwnd, msg, w, l);
    }
    if (*p).busy {
        if msg == WM_PAINT {
            let _ = ValidateRect(Some(hwnd), None);
            return LRESULT(0);
        }
        return DefWindowProcW(hwnd, msg, w, l);
    }
    let h = &mut *p;
    match msg {
        WM_PAINT => {
            let mut paint = PAINTSTRUCT::default();
            BeginPaint(hwnd, &mut paint);
            if let Some(c) = &mut h.canvas {
                view::render(&mut h.model, c);
                h.paints += 1;
            }
            let _ = EndPaint(hwnd, &paint);
            LRESULT(0)
        }
        WM_SIZE => {
            if w.0 != SIZE_MINIMIZED as usize {
                let (w, hgt) = (
                    (l.0 as u32 & 65535) as u32,
                    ((l.0 as u32 >> 16) & 65535) as u32,
                );
                if w > 0 && hgt > 0 {
                    if let Some(c) = &mut h.canvas {
                        c.resize(w, hgt);
                    }
                }
            }
            LRESULT(0)
        }
        WM_TIMER => {
            h.model.tick();
            h.ticks += 1;
            if let Some(root) = h.smoke.clone() {
                let elapsed = h.started.elapsed().as_secs_f64();
                h.model.page = if elapsed < 0.5 {
                    Page::Welcome
                } else if elapsed < 1. {
                    Page::Destination
                } else if elapsed < 1.5 {
                    Page::Installing
                } else {
                    Page::Complete
                };
                if elapsed > 2.2 {
                    let pmv2 = super::accessibility::per_monitor_v2(hwnd);
                    let passed = h.paints >= 16 && h.ticks >= 20 && pmv2;
                    let report = format!(
                        "{{\"passed\":{passed},\"native_hwnd\":true,\"screens\":4,\"per_monitor_v2\":{pmv2},\"paints\":{},\"ticks\":{},\"dpi\":{},\"physical_input\":false}}",
                        h.paints,
                        h.ticks,
                        GetDpiForWindow(hwnd)
                    );
                    let _ = std::fs::write(root.join("setup-window.json"), report);
                    let _ = DestroyWindow(hwnd);
                    return LRESULT(0);
                }
            }
            let _ = InvalidateRect(Some(hwnd), None, false);
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_GETMINMAXINFO => {
            let info = &mut *(l.0 as *mut MINMAXINFO);
            let dpi = GetDpiForWindow(hwnd).max(96);
            let mut mi = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            let _ = GetMonitorInfoW(MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST), &mut mi);
            let border = GetSystemMetricsForDpi(SM_CXSIZEFRAME, dpi)
                + GetSystemMetricsForDpi(SM_CXPADDEDBORDER, dpi);
            info.ptMinTrackSize = POINT {
                x: ((900 * dpi / 96) as i32).min(mi.rcWork.right - mi.rcWork.left - 2 * border),
                y: ((490 * dpi / 96) as i32).min(mi.rcWork.bottom - mi.rcWork.top - 2 * border),
            };
            LRESULT(0)
        }
        WM_NCHITTEST => {
            let mut pt = POINT {
                x: (l.0 as u32 & 65535) as i16 as i32,
                y: ((l.0 as u32 >> 16) & 65535) as i16 as i32,
            };
            let _ = ScreenToClient(hwnd, &mut pt);
            let scale = h.canvas.as_ref().map(|c| c.scale).unwrap_or(1.);
            let (x, y) = (pt.x as f32 / scale, pt.y as f32 / scale);
            if y < 72. && !h.model.hits.iter().any(|hit| hit.rect.contains(x, y)) {
                LRESULT(HTCAPTION as isize)
            } else {
                LRESULT(HTCLIENT as isize)
            }
        }
        WM_MOUSEMOVE | WM_LBUTTONDOWN | WM_LBUTTONUP => {
            let scale = h.canvas.as_ref().map(|c| c.scale).unwrap_or(1.);
            let (x, y) = (
                ((l.0 as u32 & 65535) as i16 as f32) / scale,
                (((l.0 as u32 >> 16) & 65535) as i16 as f32) / scale,
            );
            let index = h.model.hits.iter().rposition(|hit| hit.rect.contains(x, y));
            h.model.hover = index;
            if msg == WM_LBUTTONDOWN {
                h.pressed = index;
                h.model.path_focus = false;
                h.model.focus = index;
            }
            if msg == WM_LBUTTONUP && index == h.pressed {
                if let Some(i) = index {
                    perform(h, h.model.hits[i].action);
                }
            }
            let _ = InvalidateRect(Some(hwnd), None, false);
            LRESULT(0)
        }
        WM_KEYDOWN => {
            if w.0 as u16 == VK_ESCAPE.0 && h.model.license_open {
                h.model.license_open = false;
                let _ = InvalidateRect(Some(hwnd), None, false);
                return LRESULT(0);
            }
            if w.0 as u16 == VK_ESCAPE.0 && h.model.language_open {
                h.model.language_open = false;
                h.model.focus = None;
                let _ = InvalidateRect(Some(hwnd), None, false);
                return LRESULT(0);
            }
            match w.0 as u16 {
                v if v == VK_ESCAPE.0 => perform(h, Action::Cancel),
                v if v == VK_TAB.0 => {
                    h.model.path_focus = false;
                    let len = h.model.hits.len();
                    if len > 0 {
                        let backwards = GetKeyState(VK_SHIFT.0 as i32) < 0;
                        let old = h.model.focus.unwrap_or(if backwards { 0 } else { len - 1 });
                        h.model.focus = Some(if backwards {
                            (old + len - 1) % len
                        } else {
                            (old + 1) % len
                        });
                    }
                }
                v if v == VK_RETURN.0 || v == VK_SPACE.0 && !h.model.path_focus => {
                    if let Some(i) = h.model.focus {
                        if let Some(hit) = h.model.hits.get(i) {
                            perform(h, hit.action);
                        }
                    }
                }
                v if v == VK_BACK.0 && h.model.path_focus => {
                    if h.model.path_selected {
                        h.model.path_text.clear();
                        h.model.path_selected = false;
                    } else {
                        h.model.path_text.pop();
                    }
                }
                v if v == 0x41 && h.model.path_focus && GetKeyState(VK_CONTROL.0 as i32) < 0 => {
                    h.model.path_selected = true;
                }
                v if v == 0x56 && h.model.path_focus && GetKeyState(VK_CONTROL.0 as i32) < 0 => {
                    if let Some(text) = super::files::clipboard_text(hwnd) {
                        h.model.path_text = text
                            .chars()
                            .filter(|c| !c.is_control())
                            .take(2000)
                            .collect();
                        h.model.path_selected = false;
                    }
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_CHAR => {
            if h.model.path_focus {
                let unit = w.0 as u16;
                let ch = if (0xd800..=0xdbff).contains(&unit) {
                    h.surrogate = Some(unit);
                    None
                } else if let Some(first) = h.surrogate.take() {
                    char::decode_utf16([first, unit])
                        .next()
                        .and_then(Result::ok)
                } else {
                    char::from_u32(unit as u32)
                };
                if let Some(ch) = ch.filter(|c| !c.is_control()) {
                    if h.model.path_selected {
                        h.model.path_text.clear();
                        h.model.path_selected = false;
                    }
                    if h.model.path_text.chars().count() < 2000 {
                        h.model.path_text.push(ch);
                    }
                }
            }
            LRESULT(0)
        }
        WM_DPICHANGED => {
            let mut r = *(l.0 as *const RECT);
            let mut mi = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            if GetMonitorInfoW(MonitorFromRect(&r, MONITOR_DEFAULTTONEAREST), &mut mi).as_bool() {
                let fitted = crate::core::window::fit(
                    crate::core::window::Bounds {
                        left: r.left,
                        top: r.top,
                        right: r.right,
                        bottom: r.bottom,
                    },
                    crate::core::window::Bounds {
                        left: mi.rcWork.left,
                        top: mi.rcWork.top,
                        right: mi.rcWork.right,
                        bottom: mi.rcWork.bottom,
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
        WM_CLOSE => {
            perform(h, Action::Cancel);
            LRESULT(0)
        }
        WM_DESTROY => {
            let _ = KillTimer(Some(hwnd), 1);
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, w, l),
    }
}
