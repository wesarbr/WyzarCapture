use super::devices::Monitor;
use crate::core::{AppResult, Frame, Region};
use windows::{
    Graphics::{
        Capture::*,
        DirectX::{Direct3D11::IDirect3DDevice, DirectXPixelFormat},
    },
    Win32::{
        Foundation::*,
        Graphics::{Direct3D::*, Direct3D11::*, Dxgi::*, Gdi::*},
        System::WinRT::{Direct3D11::*, Graphics::Capture::IGraphicsCaptureItemInterop},
    },
    core::{Interface, factory},
};

#[derive(Clone, Debug)]
pub struct Target {
    pub monitor: Monitor,
    pub window: Option<isize>,
    pub region: Option<Region>,
    pub cursor: bool,
}
/// Screen capture with Windows Graphics Capture, falling back to GDI where WGC is
/// missing or refused (older Windows 10 builds, some virtual and remote sessions).
pub struct Capture {
    backend: Backend,
    pub gpu_resize: bool,
}
enum Backend {
    Wgc(Wgc),
    Gdi(Gdi),
}
impl Capture {
    pub fn new(target: &Target) -> AppResult<Self> {
        let forced = std::env::var("WYZAR_FORCE_GDI_CAPTURE").as_deref() == Ok("1");
        let wgc = if forced {
            Err(crate::i18n::tr("Captura GDI forçada para teste").to_string())
        } else {
            Wgc::new(target)
        };
        match wgc {
            Ok(wgc) => Ok(Self {
                backend: Backend::Wgc(wgc),
                gpu_resize: false,
            }),
            Err(error) => {
                let mut gdi = Gdi::new(target).map_err(|fallback| {
                    crate::i18n::format_text(
                        crate::i18n::tr("{error}. Captura compatível: {fallback}"),
                        &[("error", error.clone()), ("fallback", fallback)],
                    )
                })?;
                gdi.warning = Some(
                    crate::i18n::tr(
                        "Windows Graphics Capture indisponível neste computador. Usando a captura compatível do Windows.",
                    )
                    .into(),
                );
                Ok(Self {
                    backend: Backend::Gdi(gdi),
                    gpu_resize: false,
                })
            }
        }
    }
    pub fn frame(&mut self) -> AppResult<Option<Frame>> {
        match &mut self.backend {
            Backend::Wgc(wgc) => {
                let frame = wgc.frame();
                self.gpu_resize = wgc.gpu_resize;
                frame
            }
            Backend::Gdi(gdi) => gdi.frame(),
        }
    }
    pub fn resize_on_gpu(&mut self, width: u32, height: u32) {
        if let Backend::Wgc(wgc) = &mut self.backend {
            wgc.resize_on_gpu(width, height);
        }
    }
    pub fn take_warning(&mut self) -> Option<String> {
        match &mut self.backend {
            Backend::Wgc(wgc) => wgc.take_warning(),
            Backend::Gdi(gdi) => gdi.warning.take(),
        }
    }
}
struct Wgc {
    device: ID3D11Device,
    direct: IDirect3DDevice,
    context: ID3D11DeviceContext,
    pool: Direct3D11CaptureFramePool,
    session: GraphicsCaptureSession,
    _item: GraphicsCaptureItem,
    pool_size: windows::Graphics::SizeInt32,
    window: Option<isize>,
    staging: Option<ID3D11Texture2D>,
    size: (u32, u32),
    crop: Option<Region>,
    origin: (i32, i32),
    output_size: Option<(u32, u32)>,
    scaler: Option<super::gpu::Scaler>,
    pub gpu_resize: bool,
    warning: Option<String>,
    closed: std::sync::Arc<std::sync::atomic::AtomicBool>,
    closed_token: i64,
}
impl Wgc {
    fn new(target: &Target) -> AppResult<Self> {
        unsafe {
            Self::create(target).map_err(|e| {
                crate::i18n::format_text(
                    crate::i18n::tr("Windows Graphics Capture: {e}"),
                    &[("e", format!("{}", e))],
                )
            })
        }
    }
    unsafe fn create(target: &Target) -> windows::core::Result<Self> {
        if !GraphicsCaptureSession::IsSupported().map_err(|e| stage("IsSupported", e))? {
            return Err(windows::core::Error::new(
                E_NOTIMPL,
                crate::i18n::tr("Windows Graphics Capture não é suportado neste sistema"),
            ));
        }
        let mut device = None;
        let mut context = None;
        let hardware = D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            None,
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            Some(&mut context),
        );
        if hardware.is_err() {
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_WARP,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut context),
            )?;
        }
        let device = device.unwrap();
        let context = context.unwrap();
        let dxgi: IDXGIDevice = device.cast()?;
        let direct: IDirect3DDevice = CreateDirect3D11DeviceFromDXGIDevice(&dxgi)?.cast()?;
        let interop: IGraphicsCaptureItemInterop =
            factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()?;
        let item: GraphicsCaptureItem = if let Some(win) = target.window {
            interop
                .CreateForWindow(HWND(win as *mut _))
                .map_err(|e| stage("CreateForWindow", e))?
        } else {
            interop
                .CreateForMonitor(HMONITOR(target.monitor.handle as *mut _))
                .map_err(|e| stage("CreateForMonitor", e))?
        };
        let size = item.Size()?;
        let closed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let closed_flag = closed.clone();
        let closed_token =
            item.Closed(&windows::Foundation::TypedEventHandler::new(move |_, _| {
                closed_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                Ok(())
            }))?;
        let pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
            &direct,
            DirectXPixelFormat::B8G8R8A8UIntNormalized,
            3,
            size,
        )
        .map_err(|e| stage("CreateFreeThreaded", e))?;
        let session = pool
            .CreateCaptureSession(&item)
            .map_err(|e| stage("CreateCaptureSession", e))?;
        // Windows 10 1903/1909 lack the cursor switch and Windows 10 lacks the
        // border switch; capture still works there with the system defaults.
        let _ = session.SetIsCursorCaptureEnabled(target.cursor);
        let _ = session.SetIsBorderRequired(false);
        session
            .StartCapture()
            .map_err(|e| stage("StartCapture", e))?;
        Ok(Self {
            device,
            direct,
            context,
            pool,
            session,
            _item: item,
            pool_size: size,
            window: target.window,
            staging: None,
            size: (0, 0),
            crop: target.region,
            origin: (target.monitor.x, target.monitor.y),
            output_size: None,
            scaler: None,
            gpu_resize: false,
            warning: None,
            closed,
            closed_token,
        })
    }
    pub fn frame(&mut self) -> AppResult<Option<Frame>> {
        unsafe {
            self.read().map_err(|e| {
                crate::i18n::format_text(
                    crate::i18n::tr("Leitura da captura: {e}"),
                    &[("e", format!("{}", e))],
                )
            })
        }
    }
    pub fn resize_on_gpu(&mut self, width: u32, height: u32) {
        self.output_size = Some((width, height));
    }
    pub fn take_warning(&mut self) -> Option<String> {
        self.warning.take()
    }
    unsafe fn read(&mut self) -> windows::core::Result<Option<Frame>> {
        if self.closed.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(windows::core::Error::new(
                E_ABORT,
                crate::i18n::tr("A fonte de captura foi encerrada ou desconectada"),
            ));
        }
        if self.window.is_some_and(|h| {
            !windows::Win32::UI::WindowsAndMessaging::IsWindow(Some(HWND(h as *mut _))).as_bool()
        }) {
            return Err(windows::core::Error::new(
                E_ABORT,
                crate::i18n::tr("A janela de captura foi fechada"),
            ));
        }
        let frame = match self.pool.TryGetNextFrame() {
            Ok(frame) => frame,
            // The WinRT method is nullable: windows-rs represents a successful null
            // interface as Error::empty (S_OK), distinct from a real failed HRESULT.
            Err(error) if error.code().is_ok() => return Ok(None),
            Err(error) => return Err(stage("TryGetNextFrame", error)),
        };
        let size = frame.ContentSize()?;
        if size.Width <= 0 || size.Height <= 0 {
            frame.Close()?;
            return Ok(None);
        }
        let surface = frame.Surface()?;
        let access: IDirect3DDxgiInterfaceAccess = surface.cast()?;
        let mut texture: ID3D11Texture2D = access.GetInterface()?;
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        texture.GetDesc(&mut desc);
        let mut w = (size.Width as u32).min(desc.Width);
        let mut h = (size.Height as u32).min(desc.Height);
        self.gpu_resize = false;
        if let Some(output) = self.output_size {
            let crop = self
                .crop
                .map(|r| Region {
                    x: (r.x - self.origin.0).max(0),
                    y: (r.y - self.origin.1).max(0),
                    width: r.width,
                    height: r.height,
                })
                .unwrap_or(Region {
                    x: 0,
                    y: 0,
                    width: w,
                    height: h,
                });
            let x = (crop.x as u32).min(w.saturating_sub(1));
            let y = (crop.y as u32).min(h.saturating_sub(1));
            let source = RECT {
                left: x as i32,
                top: y as i32,
                right: (x + crop.width.min(w - x)) as i32,
                bottom: (y + crop.height.min(h - y)) as i32,
            };
            let result = (|| -> windows::core::Result<ID3D11Texture2D> {
                if self
                    .scaler
                    .as_ref()
                    .is_none_or(|s| s.input_size != (desc.Width, desc.Height))
                {
                    self.scaler = Some(super::gpu::Scaler::new(
                        &self.device,
                        &self.context,
                        (desc.Width, desc.Height),
                        output,
                    )?);
                }
                self.scaler.as_ref().unwrap().process(&texture, source)
            })();
            match result {
                Ok(resized) => {
                    texture = resized;
                    texture.GetDesc(&mut desc);
                    w = output.0;
                    h = output.1;
                    self.gpu_resize = true;
                }
                Err(_) => {
                    self.output_size = None;
                    self.scaler = None;
                    self.warning=Some(crate::i18n::tr("Redimensionamento GPU indisponível. A captura continuará pelo caminho de CPU.").into());
                }
            }
        }
        if self.size != (desc.Width, desc.Height) {
            desc.Usage = D3D11_USAGE_STAGING;
            desc.BindFlags = 0;
            desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;
            desc.MiscFlags = 0;
            self.device
                .CreateTexture2D(&desc, None, Some(&mut self.staging))?;
            self.size = (desc.Width, desc.Height);
        }
        let staging = self.staging.as_ref().unwrap();
        self.context.CopyResource(staging, &texture);
        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        self.context
            .Map(staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))?;
        let crop = if self.gpu_resize {
            Region {
                x: 0,
                y: 0,
                width: w,
                height: h,
            }
        } else {
            self.crop
                .map(|r| Region {
                    x: (r.x - self.origin.0).max(0),
                    y: (r.y - self.origin.1).max(0),
                    width: r.width,
                    height: r.height,
                })
                .unwrap_or(Region {
                    x: 0,
                    y: 0,
                    width: w,
                    height: h,
                })
        };
        let x = (crop.x as u32).min(w.saturating_sub(1));
        let y = (crop.y as u32).min(h.saturating_sub(1));
        let width = crop.width.min(w - x);
        let height = crop.height.min(h - y);
        let mut bgra = vec![0; (width * height * 4) as usize];
        for row in 0..height {
            let src =
                (mapped.pData as *const u8).add(((row + y) * mapped.RowPitch + x * 4) as usize);
            std::ptr::copy_nonoverlapping(
                src,
                bgra.as_mut_ptr().add((row * width * 4) as usize),
                (width * 4) as usize,
            );
        }
        self.context.Unmap(staging, 0);
        frame.Close()?;
        if size != self.pool_size {
            self.pool.Recreate(
                &self.direct,
                DirectXPixelFormat::B8G8R8A8UIntNormalized,
                3,
                size,
            )?;
            self.pool_size = size;
        }
        Ok(Some(Frame {
            width,
            height,
            bgra,
        }))
    }
}
fn stage(name: &str, e: windows::core::Error) -> windows::core::Error {
    windows::core::Error::new(
        e.code(),
        crate::i18n::format_text(
            crate::i18n::tr("{name}: {e}"),
            &[("name", format!("{}", name)), ("e", format!("{}", e))],
        ),
    )
}
pub fn validate_gpu_scaler() -> AppResult<Frame> {
    super::gpu::validate_scaler()
}
impl Drop for Wgc {
    fn drop(&mut self) {
        let _ = self._item.RemoveClosed(self.closed_token);
        let _ = self.session.Close();
        let _ = self.pool.Close();
    }
}
/// GDI copy of the visible desktop: monitor, region or a window's on-screen bounds.
/// Slower than WGC and it sees overlapping windows, but it works on any Windows.
struct Gdi {
    rect: RECT,
    window: Option<isize>,
    cursor: bool,
    surface: Option<(HDC, HBITMAP, HGDIOBJ, *mut u8, i32, i32)>,
    warning: Option<String>,
}
impl Gdi {
    fn new(target: &Target) -> AppResult<Self> {
        let m = &target.monitor;
        let rect = if let Some(r) = target.region {
            RECT {
                left: r.x,
                top: r.y,
                right: r.x + r.width as i32,
                bottom: r.y + r.height as i32,
            }
        } else {
            RECT {
                left: m.x,
                top: m.y,
                right: m.x + m.width as i32,
                bottom: m.y + m.height as i32,
            }
        };
        let mut gdi = Self {
            rect,
            window: target.window,
            cursor: target.cursor,
            surface: None,
            warning: None,
        };
        // Prove the path once so a failure is reported before recording starts.
        gdi.frame()?
            .ok_or(crate::i18n::tr("A captura compatível não retornou imagem"))?;
        Ok(gdi)
    }
    fn bounds(&self) -> AppResult<RECT> {
        let Some(window) = self.window else {
            return Ok(self.rect);
        };
        let hwnd = HWND(window as *mut _);
        unsafe {
            if !windows::Win32::UI::WindowsAndMessaging::IsWindow(Some(hwnd)).as_bool() {
                return Err(crate::i18n::tr("A janela de captura foi fechada").into());
            }
            let mut r = RECT::default();
            if windows::Win32::Graphics::Dwm::DwmGetWindowAttribute(
                hwnd,
                windows::Win32::Graphics::Dwm::DWMWA_EXTENDED_FRAME_BOUNDS,
                (&mut r as *mut RECT).cast(),
                std::mem::size_of::<RECT>() as u32,
            )
            .is_err()
            {
                windows::Win32::UI::WindowsAndMessaging::GetWindowRect(hwnd, &mut r)
                    .map_err(|e| e.to_string())?;
            }
            Ok(r)
        }
    }
    fn frame(&mut self) -> AppResult<Option<Frame>> {
        let r = self.bounds()?;
        let (w, h) = (
            (r.right - r.left).max(0) & !1,
            (r.bottom - r.top).max(0) & !1,
        );
        if w < 2 || h < 2 {
            return Ok(None);
        }
        unsafe {
            if self.surface.is_some_and(|s| s.4 != w || s.5 != h) {
                self.release();
            }
            if self.surface.is_none() {
                let screen = GetDC(None);
                let memory = CreateCompatibleDC(Some(screen));
                let _ = ReleaseDC(None, screen);
                let info = BITMAPINFO {
                    bmiHeader: BITMAPINFOHEADER {
                        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                        biWidth: w,
                        biHeight: -h,
                        biPlanes: 1,
                        biBitCount: 32,
                        biCompression: BI_RGB.0,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let mut bits = std::ptr::null_mut();
                let bitmap =
                    match CreateDIBSection(Some(memory), &info, DIB_RGB_COLORS, &mut bits, None, 0)
                    {
                        Ok(b) if !bits.is_null() => b,
                        Ok(b) => {
                            let _ = DeleteObject(b.into());
                            let _ = DeleteDC(memory);
                            return Err(crate::i18n::tr("Superfície GDI sem memória").into());
                        }
                        Err(e) => {
                            let _ = DeleteDC(memory);
                            return Err(e.to_string());
                        }
                    };
                let old = SelectObject(memory, bitmap.into());
                self.surface = Some((memory, bitmap, old, bits.cast(), w, h));
            }
            let (memory, _, _, bits, _, _) = self.surface.unwrap();
            let screen = GetDC(None);
            let copied = BitBlt(
                memory,
                0,
                0,
                w,
                h,
                Some(screen),
                r.left,
                r.top,
                SRCCOPY | CAPTUREBLT,
            );
            let _ = ReleaseDC(None, screen);
            copied.map_err(|e| e.to_string())?;
            if self.cursor {
                use windows::Win32::UI::WindowsAndMessaging::*;
                let mut info = CURSORINFO {
                    cbSize: std::mem::size_of::<CURSORINFO>() as u32,
                    ..Default::default()
                };
                if GetCursorInfo(&mut info).is_ok() && info.flags.0 & CURSOR_SHOWING.0 != 0 {
                    let mut icon = ICONINFO::default();
                    let hot = if GetIconInfo(HICON(info.hCursor.0), &mut icon).is_ok() {
                        if !icon.hbmMask.is_invalid() {
                            let _ = DeleteObject(icon.hbmMask.into());
                        }
                        if !icon.hbmColor.is_invalid() {
                            let _ = DeleteObject(icon.hbmColor.into());
                        }
                        (icon.xHotspot as i32, icon.yHotspot as i32)
                    } else {
                        (0, 0)
                    };
                    let _ = DrawIconEx(
                        memory,
                        info.ptScreenPos.x - r.left - hot.0,
                        info.ptScreenPos.y - r.top - hot.1,
                        HICON(info.hCursor.0),
                        0,
                        0,
                        0,
                        None,
                        DI_NORMAL,
                    );
                }
            }
            let _ = GdiFlush();
            let pixels = std::slice::from_raw_parts(bits, (w * h * 4) as usize);
            let mut bgra = pixels.to_vec();
            for p in bgra.chunks_exact_mut(4) {
                p[3] = 255;
            }
            Ok(Some(Frame {
                width: w as u32,
                height: h as u32,
                bgra,
            }))
        }
    }
    fn release(&mut self) {
        if let Some((memory, bitmap, old, ..)) = self.surface.take() {
            unsafe {
                SelectObject(memory, old);
                let _ = DeleteObject(bitmap.into());
                let _ = DeleteDC(memory);
            }
        }
    }
}
impl Drop for Gdi {
    fn drop(&mut self) {
        self.release();
    }
}
