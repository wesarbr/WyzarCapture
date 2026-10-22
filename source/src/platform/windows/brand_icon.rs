//! RAII ownership of the theme-coloured window/tray icon. No resource on disk is changed.
use super::graphics::Canvas;
use crate::{
    core::AppResult,
    ui::theme::{Palette, Rect},
};
use windows::Win32::{Graphics::Gdi::*, UI::WindowsAndMessaging::*};
pub(super) struct BrandIcon(pub HICON);
impl BrandIcon {
    pub(super) fn new(palette: Palette) -> AppResult<Self> {
        let mut c = Canvas::plain_image(48, 48)?;
        c.palette = palette;
        c.begin_artifact(true);
        c.logo(Rect::new(2., 2., 44., 44.));
        c.end()?;
        let frame = c.pixels()?;
        unsafe {
            let mut info = BITMAPINFO::default();
            info.bmiHeader = BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: 48,
                biHeight: -48,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            };
            let mut data = std::ptr::null_mut();
            let bitmap = CreateDIBSection(None, &info, DIB_RGB_COLORS, &mut data, None, 0)
                .map_err(|e| e.to_string())?;
            std::ptr::copy_nonoverlapping(frame.bgra.as_ptr(), data.cast::<u8>(), frame.bgra.len());
            let mask_data = [0u8; 48 * 8];
            let mask = CreateBitmap(48, 48, 1, 1, Some(mask_data.as_ptr().cast()));
            if mask.is_invalid() {
                let _ = DeleteObject(bitmap.into());
                return Err(crate::i18n::tr("Não foi possível criar a máscara do ícone").into());
            }
            let result = CreateIconIndirect(&ICONINFO {
                fIcon: true.into(),
                hbmMask: mask,
                hbmColor: bitmap,
                ..Default::default()
            })
            .map(Self)
            .map_err(|e| e.to_string());
            let _ = DeleteObject(mask.into());
            let _ = DeleteObject(bitmap.into());
            result
        }
    }
}
impl Drop for BrandIcon {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyIcon(self.0);
        }
    }
}
