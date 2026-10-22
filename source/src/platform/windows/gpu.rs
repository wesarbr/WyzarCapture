//! Optional D3D11 video resize. CPU readback and RGB32 sink input remain explicit.
use windows::{
    Win32::{
        Foundation::*,
        Graphics::{Direct3D11::*, Dxgi::Common::*},
    },
    core::{Interface, Result},
};
pub struct Scaler {
    device: ID3D11VideoDevice,
    context: ID3D11VideoContext,
    enumerator: ID3D11VideoProcessorEnumerator,
    processor: ID3D11VideoProcessor,
    output: ID3D11Texture2D,
    view: ID3D11VideoProcessorOutputView,
    pub input_size: (u32, u32),
    size: (u32, u32),
    copy_context: ID3D11DeviceContext,
    compatible_input: ID3D11Texture2D,
}
impl Scaler {
    pub fn new(
        device: &ID3D11Device,
        context: &ID3D11DeviceContext,
        input: (u32, u32),
        output: (u32, u32),
    ) -> Result<Self> {
        unsafe {
            let video_device: ID3D11VideoDevice = device.cast()?;
            let copy_context = context.clone();
            let context: ID3D11VideoContext = context.cast()?;
            let enumerator = video_device.CreateVideoProcessorEnumerator(
                &D3D11_VIDEO_PROCESSOR_CONTENT_DESC {
                    InputFrameFormat: D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE,
                    InputFrameRate: DXGI_RATIONAL {
                        Numerator: 60,
                        Denominator: 1,
                    },
                    InputWidth: input.0,
                    InputHeight: input.1,
                    OutputFrameRate: DXGI_RATIONAL {
                        Numerator: 60,
                        Denominator: 1,
                    },
                    OutputWidth: output.0,
                    OutputHeight: output.1,
                    Usage: D3D11_VIDEO_USAGE_PLAYBACK_NORMAL,
                },
            )?;
            let support = enumerator.CheckVideoProcessorFormat(DXGI_FORMAT_B8G8R8A8_UNORM)?;
            let required = D3D11_VIDEO_PROCESSOR_FORMAT_SUPPORT_INPUT.0 as u32
                | D3D11_VIDEO_PROCESSOR_FORMAT_SUPPORT_OUTPUT.0 as u32;
            if support & required != required {
                return Err(windows::core::Error::new(
                    E_NOTIMPL,
                    crate::i18n::tr("BGRA não suportado pelo processador de vídeo"),
                ));
            }
            let processor = video_device.CreateVideoProcessor(&enumerator, 0)?;
            let mut input_texture = None;
            device.CreateTexture2D(
                &D3D11_TEXTURE2D_DESC {
                    Width: input.0,
                    Height: input.1,
                    MipLevels: 1,
                    ArraySize: 1,
                    Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    SampleDesc: DXGI_SAMPLE_DESC {
                        Count: 1,
                        Quality: 0,
                    },
                    Usage: D3D11_USAGE_DEFAULT,
                    ..Default::default()
                },
                None,
                Some(&mut input_texture),
            )?;
            let mut texture = None;
            device.CreateTexture2D(
                &D3D11_TEXTURE2D_DESC {
                    Width: output.0,
                    Height: output.1,
                    MipLevels: 1,
                    ArraySize: 1,
                    Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    SampleDesc: DXGI_SAMPLE_DESC {
                        Count: 1,
                        Quality: 0,
                    },
                    Usage: D3D11_USAGE_DEFAULT,
                    BindFlags: D3D11_BIND_RENDER_TARGET.0 as u32,
                    ..Default::default()
                },
                None,
                Some(&mut texture),
            )?;
            let output_texture = texture.unwrap();
            let mut view = None;
            video_device.CreateVideoProcessorOutputView(
                &output_texture,
                &enumerator,
                &D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC {
                    ViewDimension: D3D11_VPOV_DIMENSION_TEXTURE2D,
                    Anonymous: D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC_0 {
                        Texture2D: D3D11_TEX2D_VPOV { MipSlice: 0 },
                    },
                },
                Some(&mut view),
            )?;
            context.VideoProcessorSetStreamFrameFormat(
                &processor,
                0,
                D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE,
            );
            context.VideoProcessorSetStreamAutoProcessingMode(&processor, 0, false);
            context.VideoProcessorSetOutputAlphaFillMode(
                &processor,
                D3D11_VIDEO_PROCESSOR_ALPHA_FILL_MODE_OPAQUE,
                0,
            );
            context.VideoProcessorSetOutputBackgroundColor(
                &processor,
                false,
                &D3D11_VIDEO_COLOR {
                    Anonymous: D3D11_VIDEO_COLOR_0 {
                        RGBA: D3D11_VIDEO_COLOR_RGBA {
                            R: 0.,
                            G: 0.,
                            B: 0.,
                            A: 1.,
                        },
                    },
                },
            );
            Ok(Self {
                device: video_device,
                context,
                enumerator,
                processor,
                output: output_texture,
                view: view.unwrap(),
                input_size: input,
                size: output,
                copy_context,
                compatible_input: input_texture.unwrap(),
            })
        }
    }
    pub fn process(&self, texture: &ID3D11Texture2D, source: RECT) -> Result<ID3D11Texture2D> {
        unsafe {
            let mut input = None;
            let mut description = D3D11_TEXTURE2D_DESC::default();
            texture.GetDesc(&mut description);
            let supported_flags = (D3D11_BIND_DECODER.0
                | D3D11_BIND_VIDEO_ENCODER.0
                | D3D11_BIND_RENDER_TARGET.0
                | D3D11_BIND_UNORDERED_ACCESS.0) as u32;
            let texture =
                if description.BindFlags != 0 && description.BindFlags & supported_flags == 0 {
                    self.copy_context
                        .CopyResource(&self.compatible_input, texture);
                    &self.compatible_input
                } else {
                    texture
                };
            self.device.CreateVideoProcessorInputView(
                texture,
                &self.enumerator,
                &D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC {
                    ViewDimension: D3D11_VPIV_DIMENSION_TEXTURE2D,
                    Anonymous: D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC_0 {
                        Texture2D: D3D11_TEX2D_VPIV {
                            MipSlice: 0,
                            ArraySlice: 0,
                        },
                    },
                    ..Default::default()
                },
                Some(&mut input),
            )?;
            let ratio = (self.size.0 as f64 / (source.right - source.left).max(1) as f64)
                .min(self.size.1 as f64 / (source.bottom - source.top).max(1) as f64);
            let width = ((source.right - source.left) as f64 * ratio).round() as i32;
            let height = ((source.bottom - source.top) as f64 * ratio).round() as i32;
            let left = (self.size.0 as i32 - width) / 2;
            let top = (self.size.1 as i32 - height) / 2;
            let destination = RECT {
                left,
                top,
                right: left + width,
                bottom: top + height,
            };
            let target = RECT {
                left: 0,
                top: 0,
                right: self.size.0 as i32,
                bottom: self.size.1 as i32,
            };
            self.context
                .VideoProcessorSetStreamSourceRect(&self.processor, 0, true, Some(&source));
            self.context.VideoProcessorSetStreamDestRect(
                &self.processor,
                0,
                true,
                Some(&destination),
            );
            self.context
                .VideoProcessorSetOutputTargetRect(&self.processor, true, Some(&target));
            let mut stream = D3D11_VIDEO_PROCESSOR_STREAM {
                Enable: TRUE,
                pInputSurface: std::mem::ManuallyDrop::new(input),
                ..Default::default()
            };
            let result = self.context.VideoProcessorBlt(
                &self.processor,
                &self.view,
                0,
                std::slice::from_ref(&stream),
            );
            std::mem::ManuallyDrop::drop(&mut stream.pInputSurface);
            result?;
            Ok(self.output.clone())
        }
    }
}
pub fn validate_scaler() -> crate::core::AppResult<crate::core::Frame> {
    unsafe {
        (|| -> windows::core::Result<crate::core::Frame> {
            let (mut device, mut context) = (None, None);
            windows::Win32::Graphics::Direct3D11::D3D11CreateDevice(
                None,
                windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut context),
            )?;
            let device = device.unwrap();
            let context = context.unwrap();
            let mut pixels = vec![0u8; 320 * 160 * 4];
            for y in 0..160 {
                for x in 0..320 {
                    let at = (y * 320 + x) * 4;
                    pixels[at + if x < 160 { 2 } else { 1 }] = 255;
                    pixels[at + 3] = 255;
                }
            }
            let desc = D3D11_TEXTURE2D_DESC {
                Width: 320,
                Height: 160,
                MipLevels: 1,
                ArraySize: 1,
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
                ..Default::default()
            };
            let data = D3D11_SUBRESOURCE_DATA {
                pSysMem: pixels.as_ptr().cast(),
                SysMemPitch: 320 * 4,
                ..Default::default()
            };
            let mut input = None;
            device.CreateTexture2D(&desc, Some(&data), Some(&mut input))?;
            let scaler = Scaler::new(&device, &context, (320, 160), (640, 640))?;
            let output = scaler.process(
                &input.unwrap(),
                RECT {
                    left: 0,
                    top: 0,
                    right: 320,
                    bottom: 160,
                },
            )?;
            let mut desc = D3D11_TEXTURE2D_DESC::default();
            output.GetDesc(&mut desc);
            desc.Usage = D3D11_USAGE_STAGING;
            desc.BindFlags = 0;
            desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;
            let mut staging = None;
            device.CreateTexture2D(&desc, None, Some(&mut staging))?;
            let staging = staging.unwrap();
            context.CopyResource(&staging, &output);
            let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
            context.Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))?;
            let mut frame = crate::core::Frame {
                width: 640,
                height: 640,
                bgra: vec![0; 640 * 640 * 4],
            };
            for row in 0..640usize {
                std::ptr::copy_nonoverlapping(
                    (mapped.pData as *const u8).add(row * mapped.RowPitch as usize),
                    frame.bgra.as_mut_ptr().add(row * 640 * 4),
                    640 * 4,
                );
            }
            context.Unmap(&staging, 0);
            for (x, y, rgb) in [
                (20, 20, [0, 0, 0]),
                (80, 320, [255, 0, 0]),
                (560, 320, [0, 255, 0]),
                (20, 620, [0, 0, 0]),
            ] {
                let at = (y * 640 + x) * 4;
                if (frame.bgra[at + 2] as i16 - rgb[0]).abs() > 8
                    || (frame.bgra[at + 1] as i16 - rgb[1]).abs() > 8
                    || (frame.bgra[at] as i16 - rgb[2]).abs() > 8
                {
                    return Err(windows::core::Error::new(
                        E_FAIL,
                        crate::i18n::tr("Cores/proporção da saída GPU não correspondem ao padrão"),
                    ));
                }
            }
            Ok(frame)
        })()
        .map_err(|e| {
            crate::i18n::format_text(
                crate::i18n::tr("Validação GPU: {e}"),
                &[("e", format!("{}", e))],
            )
        })
    }
}
