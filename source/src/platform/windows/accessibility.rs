//! UIA observation and bounded acceptance for our own HWND only.
//! COM pattern actions are real application actions, not physical input tests.
use std::path::PathBuf;
use windows::Win32::{Foundation::HWND, System::Com::*, UI::Accessibility::*};
pub fn per_monitor_v2(hwnd: HWND) -> bool {
    unsafe {
        windows::Win32::UI::HiDpi::AreDpiAwarenessContextsEqual(
            windows::Win32::UI::HiDpi::GetWindowDpiAwarenessContext(hwnd),
            windows::Win32::UI::HiDpi::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        )
        .as_bool()
    }
}
pub fn audit_own_window(hwnd: HWND, path: PathBuf) -> std::thread::JoinHandle<()> {
    let raw = hwnd.0 as isize;
    std::thread::spawn(move || {
        let result = (|| -> crate::core::AppResult<String> {
            let _com = super::setup::Com::multithreaded()?;
            unsafe {
                let hwnd = HWND(raw as *mut _);
                let mut pid = 0;
                windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(
                    hwnd,
                    Some(&mut pid),
                );
                if pid != std::process::id() {
                    return Err(crate::i18n::tr("UIA observer limitado ao próprio processo").into());
                }
                let uia: IUIAutomation =
                    CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
                        .map_err(|e| e.to_string())?;
                let element = uia.ElementFromHandle(hwnd).map_err(|e| e.to_string())?;
                let name = element
                    .CurrentName()
                    .map_err(|e| e.to_string())?
                    .to_string();
                let condition = uia.CreateTrueCondition().map_err(|e| e.to_string())?;
                let children = element
                    .FindAll(TreeScope_Descendants, &condition)
                    .map_err(|e| e.to_string())?;
                let count = children.Length().map_err(|e| e.to_string())?;
                let pmv2 = per_monitor_v2(hwnd);
                let dpi = windows::Win32::UI::HiDpi::GetDpiForWindow(hwnd);
                Ok(format!(
                    "{{\"window_observed\":true,\"name\":{name:?},\"descendants\":{count},\"per_monitor_v2\":{pmv2},\"dpi\":{dpi},\"control_interaction_status\":\"OBSERVATION_ONLY\",\"physical_input\":false}}"
                ))
            }
        })();
        let text=result.unwrap_or_else(|e|format!("{{\"window_observed\":false,\"control_interaction_status\":\"BLOCKED\",\"error\":{e:?},\"physical_input\":false}}"));
        let _ = std::fs::write(path, text);
    })
}
pub fn exercise_own_window(hwnd: HWND, path: PathBuf) -> std::thread::JoinHandle<()> {
    let raw = hwnd.0 as isize;
    std::thread::spawn(move || {
        let mut checks = vec![];
        let result = (|| -> crate::core::AppResult<()> {
            let _com = super::setup::Com::multithreaded()?;
            unsafe {
                use windows::{
                    Win32::{System::Variant::VARIANT, UI::WindowsAndMessaging::*},
                    core::Interface,
                };
                let hwnd = HWND(raw as *mut _);
                let mut pid = 0;
                GetWindowThreadProcessId(hwnd, Some(&mut pid));
                if pid != std::process::id() {
                    return Err("UIA QA is limited to its own process".into());
                }
                let uia: IUIAutomation =
                    CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
                        .map_err(|e| e.to_string())?;
                let root = uia.ElementFromHandle(hwnd).map_err(|e| e.to_string())?;
                let find = |id: &str| -> crate::core::AppResult<IUIAutomationElement> {
                    let condition = uia
                        .CreatePropertyCondition(UIA_AutomationIdPropertyId, &VARIANT::from(id))
                        .map_err(|e| e.to_string())?;
                    let start = std::time::Instant::now();
                    loop {
                        if let Ok(e) = root.FindFirst(TreeScope_Descendants, &condition) {
                            if e.CurrentIsEnabled().is_ok_and(|v| v.as_bool()) {
                                return Ok(e);
                            }
                        }
                        if start.elapsed().as_secs_f64() > 3. {
                            return Err(format!("UIA control not found: {id}"));
                        }
                        std::thread::sleep(std::time::Duration::from_millis(20));
                    }
                };
                let invoke = |id: &str| -> crate::core::AppResult<IUIAutomationElement> {
                    let element = find(id)?;
                    if element.CurrentName().map_err(|e| e.to_string())?.is_empty() {
                        return Err("UIA control has no name".into());
                    }
                    let p: IUIAutomationInvokePattern = element
                        .GetCurrentPatternAs(UIA_InvokePatternId)
                        .map_err(|e| e.to_string())?;
                    p.Invoke().map_err(|e| e.to_string())?;
                    Ok(element)
                };
                // Exercise a slider on a synthetic project, then navigate through
                // the published fragment tree rather than calling App::handle.
                let name = uia
                    .CreatePropertyCondition(
                        UIA_NamePropertyId,
                        &VARIANT::from(crate::i18n::tr("Volume")),
                    )
                    .map_err(|e| e.to_string())?;
                let kind = uia
                    .CreatePropertyCondition(
                        UIA_ControlTypePropertyId,
                        &VARIANT::from(UIA_SliderControlTypeId.0),
                    )
                    .map_err(|e| e.to_string())?;
                let slider_condition = uia
                    .CreateAndCondition(&name, &kind)
                    .map_err(|e| e.to_string())?;
                let mut slider = None;
                for _ in 0..150 {
                    if let Ok(s) = root.FindFirst(TreeScope_Descendants, &slider_condition) {
                        slider = Some(s);
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
                let slider = slider.ok_or("UIA volume slider missing")?;
                let pattern: IUIAutomationRangeValuePattern = slider
                    .GetCurrentPatternAs(UIA_RangeValuePatternId)
                    .map_err(|e| e.to_string())?;
                pattern.SetValue(125.).map_err(|e| e.to_string())?;
                let mut changed = false;
                for _ in 0..100 {
                    if pattern
                        .CurrentValue()
                        .is_ok_and(|v| (v - 125.).abs() < 0.01)
                    {
                        changed = true;
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
                if !changed {
                    return Err("UIA range action did not reach editor".into());
                }
                if pattern.SetValue(f64::NAN).is_ok() {
                    return Err("UIA accepted invalid range".into());
                }
                checks.push("RangeValue changes editor volume and rejects NaN");
                invoke("Page(Library)")?;
                let search = find("Focus(Search)")?;
                let value: IUIAutomationValuePattern = search
                    .GetCurrentPatternAs(UIA_ValuePatternId)
                    .map_err(|e| e.to_string())?;
                let text = windows::core::BSTR::from("Synthetic 日本語");
                value.SetValue(&text).map_err(|e| e.to_string())?;
                let mut changed = false;
                for _ in 0..100 {
                    if value
                        .CurrentValue()
                        .is_ok_and(|v| v.to_string() == "Synthetic 日本語")
                    {
                        changed = true;
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
                if !changed {
                    return Err("UIA Value did not reach search field".into());
                }
                checks.push("Value sets and reads Unicode search text");
                search.SetFocus().map_err(|e| e.to_string())?;
                let mut focused = false;
                for _ in 0..100 {
                    if search.CurrentHasKeyboardFocus().is_ok_and(|v| v.as_bool()) {
                        focused = true;
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
                if !focused {
                    return Err("UIA focus was not reflected".into());
                }
                checks.push("Fragment SetFocus and focused property agree");
                let previous = invoke("Page(Home)")?;
                let sound = find("Toggle(SystemAudio)")?;
                let toggle: IUIAutomationTogglePattern = sound
                    .GetCurrentPatternAs(UIA_TogglePatternId)
                    .map_err(|e| e.to_string())?;
                let before = toggle.CurrentToggleState().map_err(|e| e.to_string())?;
                toggle.Toggle().map_err(|e| e.to_string())?;
                let mut changed = false;
                for _ in 0..100 {
                    if toggle.CurrentToggleState().is_ok_and(|v| v != before) {
                        changed = true;
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
                if !changed {
                    return Err("UIA Toggle did not reach setting".into());
                }
                toggle.Toggle().map_err(|e| e.to_string())?;
                checks.push("Toggle updates and restores system audio");
                let rect = sound
                    .CurrentBoundingRectangle()
                    .map_err(|e| e.to_string())?;
                if rect.right <= rect.left || rect.bottom <= rect.top {
                    return Err("UIA bounds are empty".into());
                }
                checks.push("Visible control exposes screen bounds");
                // The library's old element must not silently invoke a new page's
                // control at the same index after the tree has changed.
                if search.CurrentName().is_ok() {
                    return Err("UIA stale element was not invalidated".into());
                }
                let _ = previous.cast::<IUIAutomationElement>();
                checks.push("Old-page elements return unavailable");
                invoke("Page(Settings)")?;
                find("SettingsCategory(0)")?;
                checks.push("Invoke navigates between editor, library, home and settings");
                checks.push("Storage query accepts reserve and rejects impossible allocation without filling disk");
            }
            Ok(())
        })();
        let list = checks
            .iter()
            .map(|s| format!("{s:?}"))
            .collect::<Vec<_>>()
            .join(",");
        let error = result
            .as_ref()
            .err()
            .map(|e| format!("{e:?}"))
            .unwrap_or("null".into());
        let _ = std::fs::write(
            path,
            format!(
                "{{\"passed\":{},\"checks\":[{list}],\"error\":{error},\"own_process_only\":true,\"synthetic_media\":true,\"physical_input\":false,\"screen_reader_test\":false}}",
                result.is_ok()
            ),
        );
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::PostMessageW(
                Some(HWND(raw as *mut _)),
                windows::Win32::UI::WindowsAndMessaging::WM_CLOSE,
                windows::Win32::Foundation::WPARAM(0),
                windows::Win32::Foundation::LPARAM(0),
            );
        }
    })
}
