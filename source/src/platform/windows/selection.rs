use super::capture::Target;
use crate::core::{AppResult, Frame, Region};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
    core::w,
};
struct Selection {
    origin: POINT,
    start: Option<POINT>,
    end: POINT,
    result: Option<Region>,
    done: bool,
}
pub fn select_region(owner: HWND) -> AppResult<Option<Region>> {
    unsafe {
        let instance = GetModuleHandleW(None).map_err(|e| e.to_string())?;
        let class = WNDCLASSW {
            lpfnWndProc: Some(proc),
            hInstance: instance.into(),
            lpszClassName: w!("WyzarRegionSelector"),
            hCursor: LoadCursorW(None, IDC_CROSS).unwrap_or_default(),
            ..Default::default()
        };
        RegisterClassW(&class);
        let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let height = GetSystemMetrics(SM_CYVIRTUALSCREEN);
        let mut state = Box::new(Selection {
            origin: POINT { x, y },
            start: None,
            end: POINT::default(),
            result: None,
            done: false,
        });
        let window = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TOOLWINDOW,
            w!("WyzarRegionSelector"),
            crate::platform::windows::localized("Selecionar região · WyzarCapture"),
            WS_POPUP,
            x,
            y,
            width,
            height,
            Some(owner),
            None,
            Some(instance.into()),
            Some((&mut *state as *mut Selection).cast()),
        )
        .map_err(|e| e.to_string())?;
        SetLayeredWindowAttributes(window, COLORREF(0), 110, LWA_ALPHA)
            .map_err(|e| e.to_string())?;
        let _ = ShowWindow(window, SW_SHOW);
        let _ = SetForegroundWindow(window);
        let _ = SetFocus(Some(window));
        let mut msg = MSG::default();
        while !state.done && GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        if IsWindow(Some(window)).as_bool() {
            let _ = DestroyWindow(window);
        }
        Ok(state.result)
    }
}
unsafe extern "system" fn proc(h: HWND, m: u32, wparam: WPARAM, l: LPARAM) -> LRESULT {
    if m == WM_NCCREATE {
        let create = &*(l.0 as *const CREATESTRUCTW);
        SetWindowLongPtrW(h, GWLP_USERDATA, create.lpCreateParams as isize);
    }
    let ptr = GetWindowLongPtrW(h, GWLP_USERDATA) as *mut Selection;
    if ptr.is_null() {
        return DefWindowProcW(h, m, wparam, l);
    }
    let s = &mut *ptr;
    let point = || POINT {
        x: (l.0 as u32 & 0xffff) as i16 as i32,
        y: ((l.0 as u32 >> 16) & 0xffff) as i16 as i32,
    };
    match m {
        WM_LBUTTONDOWN => {
            let p = point();
            s.start = Some(p);
            s.end = p;
            SetCapture(h);
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            s.end = point();
            let _ = InvalidateRect(Some(h), None, false);
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let _ = ReleaseCapture();
            if let Some(start) = s.start {
                let p = point();
                let r = Region {
                    x: start.x.min(p.x) + s.origin.x,
                    y: start.y.min(p.y) + s.origin.y,
                    width: (start.x - p.x).unsigned_abs() & !1,
                    height: (start.y - p.y).unsigned_abs() & !1,
                };
                if r.valid() {
                    s.result = Some(r);
                }
            }
            s.done = true;
            let _ = DestroyWindow(h);
            LRESULT(0)
        }
        WM_KEYDOWN if wparam.0 == VK_ESCAPE.0 as usize => {
            s.done = true;
            let _ = DestroyWindow(h);
            LRESULT(0)
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let dc = BeginPaint(h, &mut ps);
            let mut client = RECT::default();
            let _ = GetClientRect(h, &mut client);
            FillRect(dc, &client, HBRUSH(GetStockObject(BLACK_BRUSH).0));
            SetBkMode(dc, TRANSPARENT);
            SetTextColor(dc, COLORREF(0xffffff));
            let text = crate::i18n::tr("Arraste para selecionar a região · Esc para cancelar")
                .encode_utf16()
                .collect::<Vec<_>>();
            let _ = TextOutW(dc, 36, 35, &text);
            if let Some(start) = s.start {
                let pen = CreatePen(PS_SOLID, 3, COLORREF(0x001ad2ff));
                let old = SelectObject(dc, pen.into());
                let brush = SelectObject(dc, GetStockObject(NULL_BRUSH));
                let _ = Rectangle(
                    dc,
                    start.x.min(s.end.x),
                    start.y.min(s.end.y),
                    start.x.max(s.end.x),
                    start.y.max(s.end.y),
                );
                SelectObject(dc, old);
                SelectObject(dc, brush);
                let _ = DeleteObject(pen.into());
            }
            let _ = EndPaint(h, &ps);
            LRESULT(0)
        }
        WM_DESTROY => {
            s.done = true;
            LRESULT(0)
        }
        _ => DefWindowProcW(h, m, wparam, l),
    }
}
pub fn click_overlay(frame: &mut Frame, target: &Target) {
    unsafe {
        if GetAsyncKeyState(VK_LBUTTON.0 as i32) >= 0 {
            return;
        }
        let mut p = POINT::default();
        if GetCursorPos(&mut p).is_err() {
            return;
        }
        let (ox, oy, sw, sh) = if let Some(r) = target.region {
            (r.x, r.y, r.width, r.height)
        } else if let Some(h) = target.window {
            let mut r = RECT::default();
            if GetWindowRect(HWND(h as *mut _), &mut r).is_err() {
                return;
            }
            (
                r.left,
                r.top,
                (r.right - r.left) as u32,
                (r.bottom - r.top) as u32,
            )
        } else {
            (
                target.monitor.x,
                target.monitor.y,
                target.monitor.width,
                target.monitor.height,
            )
        };
        let x = (p.x - ox) as f32 * frame.width as f32 / sw.max(1) as f32;
        let y = (p.y - oy) as f32 * frame.height as f32 / sh.max(1) as f32;
        for yy in (y as i32 - 22)..=(y as i32 + 22) {
            for xx in (x as i32 - 22)..=(x as i32 + 22) {
                let distance = ((xx as f32 - x).powi(2) + (yy as f32 - y).powi(2)).sqrt();
                if (15.0..=20.0).contains(&distance)
                    && xx >= 0
                    && yy >= 0
                    && xx < frame.width as i32
                    && yy < frame.height as i32
                {
                    let i = ((yy as u32 * frame.width + xx as u32) * 4) as usize;
                    frame.bgra[i..i + 4].copy_from_slice(&[26, 210, 255, 255]);
                }
            }
        }
    }
}
