//! Enumerate actual adapters and registered H.264 MFTs, never infer a vendor codec.
use crate::{
    core::AppResult,
    encoder::capabilities::{Inventory, Transform},
};
use windows::{
    Win32::{Graphics::Dxgi::*, Media::MediaFoundation::*, System::Com::CoTaskMemFree},
    core::Interface,
};
pub fn inventory() -> AppResult<Inventory> {
    unsafe {
        let mut result = Inventory::default();
        result.logical_cpus = std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1);
        let mut memory = windows::Win32::System::SystemInformation::MEMORYSTATUSEX {
            dwLength: std::mem::size_of::<windows::Win32::System::SystemInformation::MEMORYSTATUSEX>(
            ) as u32,
            ..Default::default()
        };
        if windows::Win32::System::SystemInformation::GlobalMemoryStatusEx(&mut memory).is_ok() {
            result.memory_bytes = memory.ullTotalPhys;
        }
        let dc = windows::Win32::Graphics::Gdi::GetDC(None);
        if !dc.is_invalid() {
            result.primary_refresh_hz = windows::Win32::Graphics::Gdi::GetDeviceCaps(
                Some(dc),
                windows::Win32::Graphics::Gdi::VREFRESH,
            ) as u32;
            let _ = windows::Win32::Graphics::Gdi::ReleaseDC(None, dc);
        }
        result.monitors = super::devices::monitors_checked()
            .unwrap_or_else(|e| {
                result.errors.push(e);
                vec![]
            })
            .iter()
            .map(|m| (m.width, m.height))
            .collect();
        result.inputs = super::devices::audio_devices_checked(true)
            .unwrap_or_else(|e| {
                result.errors.push(e);
                vec![]
            })
            .into_iter()
            .map(|d| d.name)
            .collect();
        result.outputs = super::devices::audio_devices_checked(false)
            .unwrap_or_else(|e| {
                result.errors.push(e);
                vec![]
            })
            .into_iter()
            .map(|d| d.name)
            .collect();
        result.cameras = super::devices::cameras_checked()
            .unwrap_or_else(|e| {
                result.errors.push(e);
                vec![]
            })
            .into_iter()
            .map(|d| d.name)
            .collect();
        if let Ok(factory) = CreateDXGIFactory1::<IDXGIFactory1>() {
            for index in 0..32 {
                let Ok(adapter) = factory.EnumAdapters1(index) else {
                    break;
                };
                if let Ok(desc) = adapter.GetDesc1() {
                    if desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 == 0 {
                        result.dedicated_video_bytes =
                            result.dedicated_video_bytes.max(desc.DedicatedVideoMemory);
                    }
                    let length = desc
                        .Description
                        .iter()
                        .position(|v| *v == 0)
                        .unwrap_or(desc.Description.len());
                    result.adapters.push({
                        let __wyzar_i18n_0 =
                            &(String::from_utf16_lossy(&desc.Description[..length]));
                        let __wyzar_i18n_1 = &(desc.DedicatedVideoMemory / 1048576);
                        let __wyzar_i18n_2 =
                            &(if desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0 {
                                crate::i18n::tr(" · software")
                            } else {
                                ""
                            });
                        crate::i18n::format_text(
                            crate::i18n::tr("{} · {} MiB{}"),
                            &[
                                ("0", format!("{}", __wyzar_i18n_0)),
                                ("1", format!("{}", __wyzar_i18n_1)),
                                ("2", format!("{}", __wyzar_i18n_2)),
                            ],
                        )
                    });
                }
            }
        } else {
            result.errors.push(
                crate::i18n::tr(
                    "Não foi possível consultar DXGI; as outras detecções foram preservadas.",
                )
                .into(),
            );
        }
        for hardware in [true, false] {
            let mut raw = std::ptr::null_mut();
            let mut count = 0;
            let flags = MFT_ENUM_FLAG(
                if hardware {
                    MFT_ENUM_FLAG_HARDWARE.0
                } else {
                    MFT_ENUM_FLAG_SYNCMFT.0 | MFT_ENUM_FLAG_ASYNCMFT.0 | MFT_ENUM_FLAG_LOCALMFT.0
                } | MFT_ENUM_FLAG_SORTANDFILTER.0,
            );
            let output = MFT_REGISTER_TYPE_INFO {
                guidMajorType: MFMediaType_Video,
                guidSubtype: MFVideoFormat_H264,
            };
            if let Err(e) = MFTEnumEx(
                MFT_CATEGORY_VIDEO_ENCODER,
                flags,
                None,
                Some(&output),
                &mut raw,
                &mut count,
            ) {
                result.errors.push(e.to_string());
                continue;
            }
            for i in 0..count as usize {
                if let Some(activate) = (*raw.add(i)).take() {
                    result.encoders.push(Transform {
                        name: super::devices::attr_string(&activate, &MFT_FRIENDLY_NAME_Attribute),
                        clsid: activate
                            .GetGUID(&MFT_TRANSFORM_CLSID_Attribute)
                            .map(|id| format!("{id:?}"))
                            .unwrap_or_default(),
                        hardware,
                    });
                }
            }
            CoTaskMemFree(Some(raw.cast()));
        }
        Ok(result)
    }
}
pub fn selected(writer: &IMFSinkWriter, stream: u32) -> (String, bool) {
    unsafe {
        if let Ok(ex) = writer.cast::<IMFSinkWriterEx>() {
            for index in 0..16 {
                let mut transform = None;
                let mut category = windows::core::GUID::zeroed();
                if ex
                    .GetTransformForStream(stream, index, Some(&mut category), &mut transform)
                    .is_err()
                {
                    break;
                }
                if category != MFT_CATEGORY_VIDEO_ENCODER {
                    continue;
                }
                if let Some(t) = transform {
                    if let Ok(a) = t.GetAttributes() {
                        let mut length = 0;
                        let hardware = a
                            .GetStringLength(&MFT_ENUM_HARDWARE_URL_Attribute)
                            .is_ok_and(|n| {
                                length = n;
                                n > 0
                            });
                        let _ = length;
                        let name = a
                            .cast::<IMFActivate>()
                            .ok()
                            .map(|a| super::devices::attr_string(&a, &MFT_FRIENDLY_NAME_Attribute))
                            .unwrap_or_default();
                        return (
                            if hardware {
                                crate::i18n::tr("Hardware automático").into()
                            } else if !name.is_empty() {
                                name
                            } else {
                                crate::i18n::tr("Media Foundation · encoder selecionado").into()
                            },
                            hardware,
                        );
                    }
                }
            }
        }
    }
    (
        crate::i18n::tr("Media Foundation · seleção automática").into(),
        false,
    )
}
