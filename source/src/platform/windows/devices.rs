use super::{pcw, wide};
use crate::core::AppResult;
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        Media::{Audio::*, MediaFoundation::*},
        System::{
            Com::StructuredStorage::{PropVariantClear, PropVariantToStringAlloc},
            Com::*,
        },
        UI::WindowsAndMessaging::*,
    },
    core::{BOOL, GUID, PWSTR},
};
#[derive(Clone, Debug)]
pub struct Device {
    pub id: String,
    pub name: String,
}
#[derive(Clone, Debug)]
pub struct Monitor {
    pub handle: isize,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
#[derive(Clone, Debug)]
pub struct Window {
    pub handle: isize,
    pub name: String,
}
pub fn window_for_process(process_id: u32) -> Option<Window> {
    windows(0).into_iter().find(|window| unsafe {
        let mut actual = 0;
        GetWindowThreadProcessId(HWND(window.handle as *mut _), Some(&mut actual));
        actual == process_id
    })
}
pub fn audio_devices(input: bool) -> Vec<Device> {
    audio_devices_checked(input).unwrap_or_default()
}
pub fn audio_devices_checked(input: bool) -> AppResult<Vec<Device>> {
    unsafe {
        audio_list(input).map_err(|e| {
            crate::i18n::format_text(
                crate::i18n::tr("Enumeração de áudio: {e}"),
                &[("e", format!("{}", e))],
            )
        })
    }
}
unsafe fn audio_list(input: bool) -> windows::core::Result<Vec<Device>> {
    let e: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER)?;
    let collection =
        e.EnumAudioEndpoints(if input { eCapture } else { eRender }, DEVICE_STATE_ACTIVE)?;
    let mut out = vec![];
    let friendly = PROPERTYKEY {
        fmtid: GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
        pid: 14,
    };
    for i in 0..collection.GetCount()? {
        let Ok(d) = collection.Item(i) else {
            continue;
        };
        let Ok(id) = d.GetId() else {
            continue;
        };
        let id_string = id.to_string().unwrap_or_default();
        CoTaskMemFree(Some(id.0.cast()));
        let name = if let Ok(mut value) = d
            .OpenPropertyStore(STGM_READ)
            .and_then(|store| store.GetValue(&friendly))
        {
            let name = PropVariantToStringAlloc(&value)
                .ok()
                .and_then(|p| {
                    let s = p.to_string().ok();
                    CoTaskMemFree(Some(p.0.cast()));
                    s
                })
                .unwrap_or_else(|| {
                    let __wyzar_i18n_0 = &(i + 1);
                    crate::i18n::format_text(
                        crate::i18n::tr("Dispositivo {}"),
                        &[("0", format!("{}", __wyzar_i18n_0))],
                    )
                });
            let _ = PropVariantClear(&mut value);
            name
        } else {
            {
                let __wyzar_i18n_0 = &(i + 1);
                crate::i18n::format_text(
                    crate::i18n::tr("Dispositivo {}"),
                    &[("0", format!("{}", __wyzar_i18n_0))],
                )
            }
        };
        out.push(Device {
            id: id_string,
            name,
        });
    }
    Ok(out)
}
pub unsafe fn camera_activations() -> windows::core::Result<Vec<IMFActivate>> {
    let mut a = None;
    MFCreateAttributes(&mut a, 2)?;
    let a = a.unwrap();
    a.SetGUID(
        &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
        &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
    )?;
    let mut raw = std::ptr::null_mut();
    let mut count = 0;
    MFEnumDeviceSources(&a, &mut raw, &mut count)?;
    let mut out = vec![];
    for i in 0..count as usize {
        if let Some(v) = (*raw.add(i)).take() {
            out.push(v);
        }
    }
    CoTaskMemFree(Some(raw.cast()));
    Ok(out)
}
pub unsafe fn attr_string(a: &IMFActivate, key: &GUID) -> String {
    let (mut p, mut n) = (PWSTR::null(), 0);
    if a.GetAllocatedString(key, &mut p, &mut n).is_err() {
        return String::new();
    }
    let s = p.to_string().unwrap_or_default();
    CoTaskMemFree(Some(p.0.cast()));
    s
}
pub fn cameras() -> Vec<Device> {
    cameras_checked().unwrap_or_default()
}
pub fn cameras_checked() -> AppResult<Vec<Device>> {
    unsafe {
        Ok(camera_activations()
            .map_err(|e| {
                crate::i18n::format_text(
                    crate::i18n::tr("Enumeração de câmeras: {e}"),
                    &[("e", format!("{}", e))],
                )
            })?
            .iter()
            .map(|a| Device {
                id: attr_string(a, &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK),
                name: attr_string(a, &MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME),
            })
            .collect())
    }
}
pub fn monitors() -> Vec<Monitor> {
    monitors_checked().unwrap_or_default()
}
pub fn monitors_checked() -> AppResult<Vec<Monitor>> {
    unsafe {
        let mut out = vec![];
        EnumDisplayMonitors(
            None,
            None,
            Some(monitor_proc),
            LPARAM(&mut out as *mut Vec<Monitor> as isize),
        )
        .ok()
        .map_err(|e| {
            crate::i18n::format_text(
                crate::i18n::tr("Enumeração de monitores: {e}"),
                &[("e", format!("{}", e))],
            )
        })?;
        Ok(out)
    }
}
unsafe extern "system" fn monitor_proc(m: HMONITOR, _: HDC, _: *mut RECT, data: LPARAM) -> BOOL {
    let out = &mut *(data.0 as *mut Vec<Monitor>);
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if GetMonitorInfoW(m, &mut info).as_bool() {
        out.push(Monitor {
            handle: m.0 as isize,
            name: {
                let __wyzar_i18n_0 = &(out.len() + 1);
                let __wyzar_i18n_1 = &(info.rcMonitor.right - info.rcMonitor.left);
                let __wyzar_i18n_2 = &(info.rcMonitor.bottom - info.rcMonitor.top);
                crate::i18n::format_text(
                    crate::i18n::tr("Monitor {} · {} × {}"),
                    &[
                        ("0", format!("{}", __wyzar_i18n_0)),
                        ("1", format!("{}", __wyzar_i18n_1)),
                        ("2", format!("{}", __wyzar_i18n_2)),
                    ],
                )
            },
            x: info.rcMonitor.left,
            y: info.rcMonitor.top,
            width: (info.rcMonitor.right - info.rcMonitor.left) as u32,
            height: (info.rcMonitor.bottom - info.rcMonitor.top) as u32,
        });
    }
    TRUE
}
pub fn windows(own: isize) -> Vec<Window> {
    unsafe {
        let mut out = (own, vec![]);
        let _ = EnumWindows(
            Some(window_proc),
            LPARAM(&mut out as *mut (isize, Vec<Window>) as isize),
        );
        out.1
    }
}
unsafe extern "system" fn window_proc(h: HWND, data: LPARAM) -> BOOL {
    let (own, out) = &mut *(data.0 as *mut (isize, Vec<Window>));
    if h.0 as isize != *own && IsWindowVisible(h).as_bool() && GetWindow(h, GW_OWNER).is_err() {
        let mut title = [0u16; 512];
        let n = GetWindowTextW(h, &mut title);
        if n > 0 && !IsIconic(h).as_bool() {
            out.push(Window {
                handle: h.0 as isize,
                name: String::from_utf16_lossy(&title[..n as usize]),
            });
        }
    }
    TRUE
}
pub fn default_target(monitor: usize, cursor: bool) -> AppResult<super::capture::Target> {
    let list = monitors();
    let mon = list
        .get(monitor)
        .or_else(|| list.first())
        .cloned()
        .ok_or(crate::i18n::tr("Nenhum monitor disponível"))?;
    Ok(super::capture::Target {
        monitor: mon,
        window: None,
        region: None,
        cursor,
    })
}
pub fn audio_endpoint(input: bool, id: &str) -> windows::core::Result<IMMDevice> {
    unsafe {
        let e: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER)?;
        if id.is_empty() {
            e.GetDefaultAudioEndpoint(if input { eCapture } else { eRender }, eConsole)
        } else {
            e.GetDevice(pcw(&wide(id)))
        }
    }
}
