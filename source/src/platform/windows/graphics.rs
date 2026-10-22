use super::{pcw, wide};
use crate::{
    core::{AppResult, Frame},
    ui::{
        icons::{self, Command},
        theme::{Color, Palette, Rect},
    },
};
use std::{collections::HashMap, path::Path};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::{Direct2D::Common::*, Direct2D::*, DirectWrite::*, Dxgi::Common::*, Imaging::*},
        System::Com::*,
    },
    core::Interface,
};
fn color(c: Color) -> D2D1_COLOR_F {
    D2D1_COLOR_F {
        r: ((c.0 >> 16) & 255) as f32 / 255.,
        g: ((c.0 >> 8) & 255) as f32 / 255.,
        b: (c.0 & 255) as f32 / 255.,
        a: c.1,
    }
}
fn rect(r: Rect) -> D2D_RECT_F {
    D2D_RECT_F {
        left: r.x,
        top: r.y,
        right: r.right(),
        bottom: r.bottom(),
    }
}
fn point(x: f32, y: f32) -> windows_numerics::Vector2 {
    windows_numerics::Vector2 { X: x, Y: y }
}
pub struct Canvas {
    pub palette: Palette,
    pub animation_time: f32,
    pub target: ID2D1RenderTarget,
    hwnd_target: Option<ID2D1HwndRenderTarget>,
    wic_bitmap: Option<IWICBitmap>,
    factory: ID2D1Factory,
    write: IDWriteFactory,
    wic: IWICImagingFactory,
    brush: ID2D1SolidColorBrush,
    rounded_stroke: ID2D1StrokeStyle,
    formats: HashMap<(crate::i18n::Language, u32, bool), IDWriteTextFormat>,
    pub width: f32,
    pub height: f32,
    pub scale: f32,
    hwnd: Option<HWND>,
    variable_font: Option<bool>,
    layers: usize,
}
impl Canvas {
    pub fn window(hwnd: HWND, width: u32, height: u32) -> AppResult<Self> {
        unsafe {
            let factory: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)
                .map_err(|e| e.to_string())?;
            let rt = factory
                .CreateHwndRenderTarget(
                    &D2D1_RENDER_TARGET_PROPERTIES {
                        r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                        pixelFormat: D2D1_PIXEL_FORMAT {
                            format: DXGI_FORMAT_B8G8R8A8_UNORM,
                            alphaMode: D2D1_ALPHA_MODE_IGNORE,
                        },
                        dpiX: 96.,
                        dpiY: 96.,
                        ..Default::default()
                    },
                    &D2D1_HWND_RENDER_TARGET_PROPERTIES {
                        hwnd,
                        pixelSize: D2D_SIZE_U { width, height },
                        presentOptions: D2D1_PRESENT_OPTIONS_NONE,
                    },
                )
                .map_err(|e| e.to_string())?;
            let mut canvas = Self::base(
                factory,
                rt.cast().map_err(|e| e.to_string())?,
                Some(rt),
                None,
                width,
                height,
                true,
            )?;
            canvas.hwnd = Some(hwnd);
            Ok(canvas)
        }
    }
    /// A lost display device (driver update, sleep, remote session) invalidates every
    /// device resource. Rebuild the window target instead of leaving a frozen window.
    fn recreate_window_target(&mut self) -> AppResult<()> {
        let Some(hwnd) = self.hwnd else {
            return Ok(());
        };
        unsafe {
            let rt = self
                .factory
                .CreateHwndRenderTarget(
                    &D2D1_RENDER_TARGET_PROPERTIES {
                        r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                        pixelFormat: D2D1_PIXEL_FORMAT {
                            format: DXGI_FORMAT_B8G8R8A8_UNORM,
                            alphaMode: D2D1_ALPHA_MODE_IGNORE,
                        },
                        dpiX: 96.,
                        dpiY: 96.,
                        ..Default::default()
                    },
                    &D2D1_HWND_RENDER_TARGET_PROPERTIES {
                        hwnd,
                        pixelSize: D2D_SIZE_U {
                            width: self.width.max(1.) as u32,
                            height: self.height.max(1.) as u32,
                        },
                        presentOptions: D2D1_PRESENT_OPTIONS_NONE,
                    },
                )
                .map_err(|e| e.to_string())?;
            self.target = rt.cast().map_err(|e| e.to_string())?;
            self.hwnd_target = Some(rt);
            self.brush = self
                .target
                .CreateSolidColorBrush(&color(Color::rgb(0xffffff)), None)
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    pub fn image(width: u32, height: u32) -> AppResult<Self> {
        Self::surface(width, height, true)
    }
    pub fn plain_image(width: u32, height: u32) -> AppResult<Self> {
        Self::surface(width, height, false)
    }
    fn surface(width: u32, height: u32, load_assets: bool) -> AppResult<Self> {
        unsafe {
            let factory: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)
                .map_err(|e| e.to_string())?;
            let wic: IWICImagingFactory =
                CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)
                    .map_err(|e| e.to_string())?;
            let bitmap = wic
                .CreateBitmap(
                    width,
                    height,
                    &GUID_WICPixelFormat32bppPBGRA,
                    WICBitmapCacheOnLoad,
                )
                .map_err(|e| e.to_string())?;
            let rt = factory
                .CreateWicBitmapRenderTarget(
                    &bitmap,
                    &D2D1_RENDER_TARGET_PROPERTIES {
                        r#type: D2D1_RENDER_TARGET_TYPE_SOFTWARE,
                        pixelFormat: D2D1_PIXEL_FORMAT {
                            format: DXGI_FORMAT_B8G8R8A8_UNORM,
                            alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                        },
                        dpiX: 96.,
                        dpiY: 96.,
                        ..Default::default()
                    },
                )
                .map_err(|e| e.to_string())?;
            Self::base(factory, rt, None, Some(bitmap), width, height, load_assets)
        }
    }
    unsafe fn base(
        factory: ID2D1Factory,
        target: ID2D1RenderTarget,
        hwnd_target: Option<ID2D1HwndRenderTarget>,
        wic_bitmap: Option<IWICBitmap>,
        width: u32,
        height: u32,
        // Kept for the existing constructors; all artwork is vector now.
        _load_assets: bool,
    ) -> AppResult<Self> {
        let write: IDWriteFactory =
            DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).map_err(|e| e.to_string())?;
        let wic: IWICImagingFactory =
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)
                .map_err(|e| e.to_string())?;
        let brush = target
            .CreateSolidColorBrush(&color(Color::rgb(0xffffff)), None)
            .map_err(|e| e.to_string())?;
        let c = Self {
            palette: Palette::default(),
            animation_time: 0.,
            target,
            hwnd_target,
            wic_bitmap,
            factory: factory.clone(),
            write,
            wic,
            brush,
            rounded_stroke: factory
                .CreateStrokeStyle(
                    &D2D1_STROKE_STYLE_PROPERTIES {
                        startCap: D2D1_CAP_STYLE_ROUND,
                        endCap: D2D1_CAP_STYLE_ROUND,
                        dashCap: D2D1_CAP_STYLE_ROUND,
                        lineJoin: D2D1_LINE_JOIN_ROUND,
                        miterLimit: 4.,
                        dashStyle: D2D1_DASH_STYLE_SOLID,
                        dashOffset: 0.,
                    },
                    None,
                )
                .map_err(|e| e.to_string())?,
            formats: HashMap::new(),
            width: width as f32,
            height: height as f32,
            scale: 1.,
            hwnd: None,
            variable_font: None,
            layers: 0,
        };
        Ok(c)
    }
    pub fn resize(&mut self, w: u32, h: u32) {
        self.width = w as f32;
        self.height = h as f32;
        if let Some(rt) = &self.hwnd_target {
            unsafe {
                let _ = rt.Resize(&D2D_SIZE_U {
                    width: w,
                    height: h,
                });
            }
        }
    }
    pub fn begin(&mut self) -> (f32, f32) {
        self.begin_layout(1600., 900.)
    }
    pub fn begin_layout(&mut self, width: f32, height: f32) -> (f32, f32) {
        self.scale = (self.width / width).min(self.height / height).max(0.1);
        unsafe {
            self.target.BeginDraw();
            self.target.SetDpi(96. * self.scale, 96. * self.scale);
            self.target
                .Clear(Some(&color(self.palette.resolve(crate::ui::theme::BG))));
            self.target
                .SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_CLEARTYPE);
        }
        (self.width / self.scale, self.height / self.scale)
    }
    pub fn begin_artifact(&self, transparent: bool) {
        unsafe {
            self.target.BeginDraw();
            self.target.SetDpi(96., 96.);
            self.target
                .Clear(Some(&color(Color(0, if transparent { 0. } else { 1. }))));
            self.target
                .SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
        }
    }
    pub fn end(&self) -> AppResult<()> {
        unsafe { self.target.EndDraw(None, None).map_err(|e| e.to_string()) }
    }
    /// Window frames end here. Returns true when the device was rebuilt and the
    /// window must be painted again; any other failure is reported as an error.
    pub fn present(&mut self) -> AppResult<bool> {
        while self.layers > 0 {
            self.pop_opacity();
        }
        match unsafe { self.target.EndDraw(None, None) } {
            Ok(()) => Ok(false),
            Err(e) if e.code() == windows::core::HRESULT(0x8899000Cu32 as i32) => {
                self.recreate_window_target()?;
                Ok(true)
            }
            Err(e) => Err(e.to_string()),
        }
    }
    /// Text width in layout units, used to place labels next to each other.
    pub fn text_width(&mut self, s: &str, size: f32, bold: bool) -> f32 {
        unsafe {
            if let Some(format) = self.text_format(size, bold) {
                let _ = format.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP);
                let units: Vec<u16> = s.encode_utf16().collect();
                if let Ok(layout) =
                    self.write
                        .CreateTextLayout(&units, &format, 100_000., size * 2.)
                {
                    let mut metrics = DWRITE_TEXT_METRICS::default();
                    if layout.GetMetrics(&mut metrics).is_ok() {
                        return metrics.widthIncludingTrailingWhitespace.ceil();
                    }
                }
            }
        }
        s.chars().count() as f32 * size * 0.55
    }
    /// Groups drawn between push/pop fade together, used by short transitions.
    pub fn push_opacity(&mut self, opacity: f32) {
        unsafe {
            if let Ok(layer) = self.target.CreateLayer(None) {
                self.target.PushLayer(
                    &D2D1_LAYER_PARAMETERS {
                        contentBounds: D2D_RECT_F {
                            left: -100_000.,
                            top: -100_000.,
                            right: 100_000.,
                            bottom: 100_000.,
                        },
                        maskTransform: windows_numerics::Matrix3x2::identity(),
                        opacity: opacity.clamp(0., 1.),
                        ..Default::default()
                    },
                    &layer,
                );
                self.layers += 1;
            }
        }
    }
    pub fn pop_opacity(&mut self) {
        if self.layers > 0 {
            unsafe { self.target.PopLayer() };
            self.layers -= 1;
        }
    }
    /// Offset following drawing calls; hit areas are unaffected and must not use it.
    pub fn translate(&self, x: f32, y: f32) {
        unsafe {
            self.target
                .SetTransform(&windows_numerics::Matrix3x2::translation(x, y));
        }
    }
    pub fn reset_transform(&self) {
        unsafe {
            self.target
                .SetTransform(&windows_numerics::Matrix3x2::identity());
        }
    }
    /// Soft radial light, transparent at the edge.
    pub fn glow(&self, x: f32, y: f32, radius: f32, c: Color) {
        unsafe {
            let center = self.palette.resolve(c);
            let stops = [
                D2D1_GRADIENT_STOP {
                    position: 0.,
                    color: color(center),
                },
                D2D1_GRADIENT_STOP {
                    position: 1.,
                    color: color(Color(center.0, 0.)),
                },
            ];
            if let Ok(stops) = self.target.CreateGradientStopCollection(
                &stops,
                D2D1_GAMMA_2_2,
                D2D1_EXTEND_MODE_CLAMP,
            ) {
                if let Ok(brush) = self.target.CreateRadialGradientBrush(
                    &D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES {
                        center: point(x, y),
                        gradientOriginOffset: point(0., 0.),
                        radiusX: radius,
                        radiusY: radius,
                    },
                    None,
                    &stops,
                ) {
                    self.target.FillEllipse(
                        &D2D1_ELLIPSE {
                            point: point(x, y),
                            radiusX: radius,
                            radiusY: radius,
                        },
                        &brush,
                    );
                }
            }
        }
    }
    /// Arc of a circle, in degrees clockwise from 12 o'clock.
    pub fn arc(&self, x: f32, y: f32, r: f32, from: f32, sweep: f32, c: Color, width: f32) {
        if sweep.abs() < 0.5 {
            return;
        }
        let at = |deg: f32| {
            let a = (deg - 90.).to_radians();
            (x + r * a.cos(), y + r * a.sin())
        };
        let steps = ((sweep.abs() / 6.).ceil() as usize).max(2);
        let mut last = at(from);
        for i in 1..=steps {
            let next = at(from + sweep * i as f32 / steps as f32);
            unsafe {
                self.target.DrawLine(
                    point(last.0, last.1),
                    point(next.0, next.1),
                    self.brush(c),
                    width,
                    &self.rounded_stroke,
                );
            }
            last = next;
        }
    }
    fn brush(&self, c: Color) -> &ID2D1SolidColorBrush {
        unsafe {
            self.brush.SetColor(&color(self.palette.resolve(c)));
        }
        &self.brush
    }
    /// Theme-resolved mix of two tokens, for hover and toggle transitions.
    pub fn blend(&self, a: Color, b: Color, t: f32) -> Color {
        let (a, b) = (self.palette.resolve(a), self.palette.resolve(b));
        let t = t.clamp(0., 1.);
        Color(
            crate::ui::theme::mix(a.0, b.0, t) | 0x8000_0000,
            a.1 + (b.1 - a.1) * t,
        )
    }
    pub fn fill(&self, r: Rect, c: Color) {
        unsafe {
            self.target.FillRectangle(&rect(r), self.brush(c));
        }
    }
    pub fn round(&self, r: Rect, radius: f32, fill: Color, border: Option<Color>) {
        let rr = D2D1_ROUNDED_RECT {
            rect: rect(r),
            radiusX: radius,
            radiusY: radius,
        };
        unsafe {
            self.target.FillRoundedRectangle(&rr, self.brush(fill));
            if let Some(c) = border {
                self.target
                    .DrawRoundedRectangle(&rr, self.brush(c), 1., None);
            }
        }
    }
    pub fn outline(&self, r: Rect, radius: f32, c: Color, stroke: f32) {
        unsafe {
            self.target.DrawRoundedRectangle(
                &D2D1_ROUNDED_RECT {
                    rect: rect(r),
                    radiusX: radius,
                    radiusY: radius,
                },
                self.brush(c),
                stroke,
                None,
            );
        }
    }
    pub fn line(&self, x1: f32, y1: f32, x2: f32, y2: f32, c: Color, width: f32) {
        unsafe {
            self.target
                .DrawLine(point(x1, y1), point(x2, y2), self.brush(c), width, None);
        }
    }
    pub fn circle(&self, x: f32, y: f32, r: f32, c: Color) {
        unsafe {
            self.target.FillEllipse(
                &D2D1_ELLIPSE {
                    point: point(x, y),
                    radiusX: r,
                    radiusY: r,
                },
                self.brush(c),
            );
        }
    }
    pub fn ring(&self, x: f32, y: f32, r: f32, c: Color, width: f32) {
        unsafe {
            self.target.DrawEllipse(
                &D2D1_ELLIPSE {
                    point: point(x, y),
                    radiusX: r,
                    radiusY: r,
                },
                self.brush(c),
                width,
                None,
            );
        }
    }
    pub fn gradient(&self, r: Rect, a: Color, b: Color, horizontal: bool, radius: f32) {
        unsafe {
            let stops = [
                D2D1_GRADIENT_STOP {
                    position: 0.,
                    color: color(self.palette.resolve(a)),
                },
                D2D1_GRADIENT_STOP {
                    position: 1.,
                    color: color(self.palette.resolve(b)),
                },
            ];
            if let Ok(stops) = self.target.CreateGradientStopCollection(
                &stops,
                D2D1_GAMMA_2_2,
                D2D1_EXTEND_MODE_CLAMP,
            ) {
                if let Ok(brush) = self.target.CreateLinearGradientBrush(
                    &D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                        startPoint: point(r.x, r.y),
                        endPoint: if horizontal {
                            point(r.right(), r.y)
                        } else {
                            point(r.x, r.bottom())
                        },
                    },
                    None,
                    &stops,
                ) {
                    self.target.FillRoundedRectangle(
                        &D2D1_ROUNDED_RECT {
                            rect: rect(r),
                            radiusX: radius,
                            radiusY: radius,
                        },
                        &brush,
                    );
                }
            }
        }
    }
    pub fn text(&mut self, s: &str, r: Rect, size: f32, c: Color, bold: bool) {
        self.text_align(s, r, size, c, bold, false, false)
    }
    pub fn center(&mut self, s: &str, r: Rect, size: f32, c: Color, bold: bool) {
        self.text_align(s, r, size, c, bold, true, false)
    }
    pub fn paragraph(&mut self, s: &str, r: Rect, size: f32, c: Color, bold: bool) {
        self.text_align(s, r, size, c, bold, false, true)
    }
    pub fn paragraph_height(&mut self, text: &str, width: f32, size: f32, bold: bool) -> f32 {
        unsafe {
            if let Some(format) = self.text_format(size, bold) {
                let _ = format.SetWordWrapping(DWRITE_WORD_WRAPPING_WRAP);
                let _ = format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_NEAR);
                let _ = format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
                let units: Vec<u16> = text.encode_utf16().collect();
                if let Ok(layout) =
                    self.write
                        .CreateTextLayout(&units, &format, width.max(1.), 100_000.)
                {
                    let mut metrics = DWRITE_TEXT_METRICS::default();
                    if layout.GetMetrics(&mut metrics).is_ok() {
                        return metrics.height.max(size * 1.3).ceil();
                    }
                }
            }
        }
        size * 1.5
    }
    /// Check the actual installed CJK faces used by this renderer.
    pub fn validate_language_glyphs(
        &self,
        language: crate::i18n::Language,
    ) -> AppResult<(usize, usize)> {
        use std::collections::BTreeSet;
        let codepoints: Vec<u32> = crate::i18n::corpus(language)
            .flat_map(str::chars)
            .map(u32::from)
            .filter(|c| (0x3000..=0x9fff).contains(c) || (0xff01..=0xffef).contains(c))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let mut fallback_count = 0;
        unsafe {
            let mut fonts = None;
            self.write
                .GetSystemFontCollection(&mut fonts, false)
                .map_err(|e| e.to_string())?;
            let fonts = fonts.ok_or("Missing system font collection")?;
            let mut index = 0;
            let mut exists = windows::core::BOOL::default();
            fonts
                .FindFamilyName(pcw(&wide(language.font())), &mut index, &mut exists)
                .map_err(|e| e.to_string())?;
            if !exists.as_bool() {
                return Err(format!("Missing font family: {}", language.font()));
            }
            let family = fonts.GetFontFamily(index).map_err(|e| e.to_string())?;
            if matches!(
                language,
                crate::i18n::Language::ZhHans | crate::i18n::Language::Ja
            ) {
                for weight in [DWRITE_FONT_WEIGHT_NORMAL, DWRITE_FONT_WEIGHT_SEMI_BOLD] {
                    let font = family
                        .GetFirstMatchingFont(
                            weight,
                            DWRITE_FONT_STRETCH_NORMAL,
                            DWRITE_FONT_STYLE_NORMAL,
                        )
                        .map_err(|e| e.to_string())?;
                    let face = font.CreateFontFace().map_err(|e| e.to_string())?;
                    let mut glyphs = vec![0u16; codepoints.len()];
                    face.GetGlyphIndices(
                        codepoints.as_ptr(),
                        codepoints.len() as u32,
                        glyphs.as_mut_ptr(),
                    )
                    .map_err(|e| e.to_string())?;
                    // Mixed-language autonyms use DirectWrite system fallback.
                    let missing: Vec<usize> = glyphs
                        .iter()
                        .enumerate()
                        .filter_map(|(i, g)| (*g == 0).then_some(i))
                        .collect();
                    fallback_count = fallback_count.max(missing.len());
                    if !missing.is_empty() {
                        let mut fallback_index = 0;
                        let mut fallback_exists = windows::core::BOOL::default();
                        fonts
                            .FindFamilyName(
                                pcw(&wide("Microsoft YaHei UI")),
                                &mut fallback_index,
                                &mut fallback_exists,
                            )
                            .map_err(|e| e.to_string())?;
                        if fallback_exists.as_bool() {
                            let font = fonts
                                .GetFontFamily(fallback_index)
                                .and_then(|f| {
                                    f.GetFirstMatchingFont(
                                        weight,
                                        DWRITE_FONT_STRETCH_NORMAL,
                                        DWRITE_FONT_STYLE_NORMAL,
                                    )
                                })
                                .map_err(|e| e.to_string())?;
                            let face = font.CreateFontFace().map_err(|e| e.to_string())?;
                            let mut alternate = vec![0u16; codepoints.len()];
                            face.GetGlyphIndices(
                                codepoints.as_ptr(),
                                codepoints.len() as u32,
                                alternate.as_mut_ptr(),
                            )
                            .map_err(|e| e.to_string())?;
                            for i in missing {
                                glyphs[i] = alternate[i];
                            }
                        }
                    }
                    if let Some(i) = glyphs.iter().position(|g| *g == 0) {
                        return Err(format!(
                            "Missing glyph U+{:04X} in {}",
                            codepoints[i],
                            language.font()
                        ));
                    }
                }
            }
        }
        Ok((codepoints.len(), fallback_count))
    }
    /// Windows 11 ships Segoe UI Variable with optical sizes for captions, text and
    /// display. Windows 10 keeps Segoe UI. CJK languages keep their own UI faces.
    fn family(&mut self, size: f32) -> &'static str {
        let language = crate::i18n::current();
        if language.font() != "Segoe UI" {
            return language.font();
        }
        let available = *self.variable_font.get_or_insert_with(|| unsafe {
            let mut fonts = None;
            if self
                .write
                .GetSystemFontCollection(&mut fonts, false)
                .is_err()
            {
                return false;
            }
            let Some(fonts) = fonts else {
                return false;
            };
            let mut index = 0;
            let mut exists = windows::core::BOOL::default();
            fonts
                .FindFamilyName(
                    pcw(&wide("Segoe UI Variable Text")),
                    &mut index,
                    &mut exists,
                )
                .is_ok()
                && exists.as_bool()
        });
        if !available {
            "Segoe UI"
        } else if size >= 21. {
            "Segoe UI Variable Display"
        } else if size <= 11.5 {
            "Segoe UI Variable Small"
        } else {
            "Segoe UI Variable Text"
        }
    }
    fn text_format(&mut self, size: f32, bold: bool) -> Option<IDWriteTextFormat> {
        unsafe {
            let language = crate::i18n::current();
            let key = (language, (size * 10.) as u32, bold);
            if !self.formats.contains_key(&key) {
                let family = self.family(size);
                if let Ok(f) = self.write.CreateTextFormat(
                    pcw(&wide(family)),
                    None,
                    if bold {
                        DWRITE_FONT_WEIGHT_SEMI_BOLD
                    } else {
                        DWRITE_FONT_WEIGHT_NORMAL
                    },
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    size,
                    pcw(&wide(language.locale())),
                ) {
                    let _ = f.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP);
                    let _ = f.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);
                    if let Ok(sign) = self.write.CreateEllipsisTrimmingSign(&f) {
                        let _ = f.SetTrimming(
                            &DWRITE_TRIMMING {
                                granularity: DWRITE_TRIMMING_GRANULARITY_CHARACTER,
                                ..Default::default()
                            },
                            &sign,
                        );
                    }
                    self.formats.insert(key, f);
                }
            }
            self.formats.get(&key).cloned()
        }
    }
    fn text_align(
        &mut self,
        s: &str,
        r: Rect,
        size: f32,
        c: Color,
        bold: bool,
        center: bool,
        wrap: bool,
    ) {
        crate::i18n::observe_rendered_text(s);
        unsafe {
            if let Some(f) = self.text_format(size, bold) {
                let _ = f.SetWordWrapping(if wrap {
                    DWRITE_WORD_WRAPPING_WRAP
                } else {
                    DWRITE_WORD_WRAPPING_NO_WRAP
                });
                let _ = f.SetParagraphAlignment(if wrap {
                    DWRITE_PARAGRAPH_ALIGNMENT_NEAR
                } else {
                    DWRITE_PARAGRAPH_ALIGNMENT_CENTER
                });
                let _ = f.SetTextAlignment(if center {
                    DWRITE_TEXT_ALIGNMENT_CENTER
                } else {
                    DWRITE_TEXT_ALIGNMENT_LEADING
                });
                let text: Vec<_> = s.encode_utf16().collect();
                self.target.DrawText(
                    &text,
                    &f,
                    &rect(r),
                    self.brush(c),
                    D2D1_DRAW_TEXT_OPTIONS_CLIP,
                    DWRITE_MEASURING_MODE_NATURAL,
                );
            }
        }
    }
    pub fn clip(&self, r: Rect) {
        unsafe {
            self.target
                .PushAxisAlignedClip(&rect(r), D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
        }
    }
    pub fn unclip(&self) {
        unsafe {
            self.target.PopAxisAlignedClip();
        }
    }
    pub fn icon(&self, name: &str, r: Rect, c: Color) {
        let control = ["minimize", "maximize", "restore", "close"].contains(&name);
        let a = &self.palette.appearance;
        let path = a
            .icon_overrides
            .get(name)
            .map(String::as_str)
            .unwrap_or_else(|| icons::path(name));
        let tint = if !control && c.0 == crate::ui::theme::TEXT.0 {
            a.icon_tint.map(Color::literal).unwrap_or(c)
        } else {
            c
        };
        if self.palette.style == 2 && !control && r.w >= 19. {
            self.round(r.inset(-2.), 5., crate::ui::theme::GOLD.alpha(0.09), None);
        }
        let weight = if control {
            1.45
        } else {
            [1.35, 1.75, 2.15][a.icon_weight.min(2)]
        };
        self.path(&icons::parse(path), r, tint, weight, false);
    }
    pub fn path(&self, commands: &[Command], r: Rect, c: Color, stroke: f32, fill: bool) {
        unsafe {
            let Ok(path) = self.factory.CreatePathGeometry() else {
                return;
            };
            let Ok(sink) = path.Open() else {
                return;
            };
            let scale = r.w / 24.;
            let p = |x, y| point(r.x + x * scale, r.y + y * scale);
            let mut open = false;
            for cmd in commands {
                match *cmd {
                    Command::Move(x, y) => {
                        if open {
                            sink.EndFigure(D2D1_FIGURE_END_OPEN);
                        }
                        sink.BeginFigure(
                            p(x, y),
                            if fill {
                                D2D1_FIGURE_BEGIN_FILLED
                            } else {
                                D2D1_FIGURE_BEGIN_HOLLOW
                            },
                        );
                        open = true;
                    }
                    Command::Line(x, y) => sink.AddLine(p(x, y)),
                    Command::Cubic(a, b, d, e, x, y) => sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                        point1: p(a, b),
                        point2: p(d, e),
                        point3: p(x, y),
                    }),
                    Command::Quad(a, b, x, y) => {
                        sink.AddQuadraticBezier(&D2D1_QUADRATIC_BEZIER_SEGMENT {
                            point1: p(a, b),
                            point2: p(x, y),
                        })
                    }
                    Command::Arc(rx, ry, angle, large, sweep, x, y) => {
                        sink.AddArc(&D2D1_ARC_SEGMENT {
                            point: p(x, y),
                            size: D2D_SIZE_F {
                                width: rx * scale,
                                height: ry * scale,
                            },
                            rotationAngle: angle,
                            sweepDirection: if sweep {
                                D2D1_SWEEP_DIRECTION_CLOCKWISE
                            } else {
                                D2D1_SWEEP_DIRECTION_COUNTER_CLOCKWISE
                            },
                            arcSize: if large {
                                D2D1_ARC_SIZE_LARGE
                            } else {
                                D2D1_ARC_SIZE_SMALL
                            },
                        })
                    }
                    Command::Close => {
                        sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                        open = false;
                    }
                }
            }
            if open {
                sink.EndFigure(D2D1_FIGURE_END_OPEN);
            }
            if sink.Close().is_ok() {
                if fill {
                    self.target.FillGeometry(&path, self.brush(c), None);
                } else {
                    self.target.DrawGeometry(
                        &path,
                        self.brush(c),
                        stroke * scale,
                        if self.palette.style == 3 {
                            None
                        } else {
                            Some(&self.rounded_stroke)
                        },
                    );
                }
            }
        }
    }
    /// Decorative backdrop for headers and empty previews. Each theme has its own
    /// vector motif; `animation_time` is zero when motion is off or while recording.
    pub fn wallpaper(&self, r: Rect, opacity: f32) {
        use crate::ui::theme::{BG, GOLD, PANEL};
        if r.w < 2. || r.h < 2. {
            return;
        }
        self.clip(r);
        self.gradient(r, PANEL, BG, false, 0.);
        let t = self.animation_time;
        let o = opacity.clamp(0., 1.);
        // Stable pseudo-random placement, independent of frame rate.
        let rnd = |n: u32| {
            let mut x = n.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
            x ^= x >> 15;
            x = x.wrapping_mul(0x2C1B_3C6D);
            x ^= x >> 12;
            (x & 0xFFFF) as f32 / 65535.
        };
        let accent = GOLD;
        let small = r.h < 160.;
        match self.palette.appearance.theme {
            // Safira: soft out-of-focus lights drifting sideways.
            1 => {
                for i in 0..if small { 4 } else { 7 } {
                    let k = i as u32;
                    let x =
                        r.x + ((rnd(k) * r.w + t * (6. + rnd(k + 9) * 10.)) % (r.w + 160.)) - 80.;
                    let y = r.y + rnd(k + 3) * r.h + (t * 0.4 + k as f32).sin() * 8.;
                    let size = r.h.min(260.) * (0.22 + rnd(k + 5) * 0.35);
                    self.glow(x, y, size, accent.alpha(o * (0.07 + rnd(k + 7) * 0.08)));
                }
            }
            // Esmeralda: topographic contour lines that flow slowly.
            2 => {
                let lines = if small { 5 } else { 9 };
                for i in 0..lines {
                    let k = i as f32;
                    let base = r.y + r.h * (0.15 + k / lines as f32 * 0.8);
                    let mut last = (r.x, base);
                    for step in 1..=40 {
                        let x = r.x + r.w * step as f32 / 40.;
                        let phase = x / r.w * 6.283 + t * 0.25 + k * 0.7;
                        let y =
                            base + phase.sin() * r.h * 0.06 + (phase * 0.5 + k).cos() * r.h * 0.03;
                        self.line(
                            last.0,
                            last.1,
                            x,
                            y,
                            accent.alpha(o * (0.05 + k * 0.012)),
                            1.2,
                        );
                        last = (x, y);
                    }
                }
            }
            // Ametista: two slow nebula orbs breathing.
            3 => {
                for i in 0..3 {
                    let k = i as f32;
                    let x = r.x + r.w * (0.35 + k * 0.25) + (t * 0.15 + k * 2.).sin() * r.w * 0.06;
                    let y = r.y + r.h * (0.3 + k * 0.2) + (t * 0.21 + k).cos() * r.h * 0.08;
                    let pulse = 0.85 + (t * 0.5 + k * 1.7).sin() * 0.15;
                    self.glow(x, y, r.h.min(300.) * 0.75 * pulse, accent.alpha(o * 0.1));
                }
            }
            // Coral: layered warm waves near the bottom edge.
            4 => {
                for layer in 0..4 {
                    let k = layer as f32;
                    let base = r.bottom() - r.h * (0.12 + k * 0.1);
                    let mut last = (r.x, base);
                    for step in 1..=36 {
                        let x = r.x + r.w * step as f32 / 36.;
                        let y = base
                            + ((x - r.x) / r.w * 9. + t * (0.3 + k * 0.08) + k).sin() * r.h * 0.04;
                        self.line(
                            last.0,
                            last.1,
                            x,
                            y,
                            accent.alpha(o * (0.16 - k * 0.03)),
                            2.2 - k * 0.3,
                        );
                        last = (x, y);
                    }
                }
            }
            // Cobre: embers rising and fading.
            5 => {
                for i in 0..if small { 10 } else { 22 } {
                    let k = i as u32;
                    let speed = 8. + rnd(k + 1) * 14.;
                    let travel = (rnd(k + 2) * r.h + t * speed) % r.h;
                    let x = r.x + rnd(k) * r.w + (t * 0.8 + k as f32).sin() * 6.;
                    let y = r.bottom() - travel;
                    let fade = 1. - travel / r.h;
                    self.circle(x, y, 1.2 + rnd(k + 4) * 1.8, accent.alpha(o * 0.55 * fade));
                }
                self.glow(
                    r.x + r.w * 0.7,
                    r.bottom(),
                    r.h * 0.9,
                    accent.alpha(o * 0.08),
                );
            }
            // Ártico: fine snow drifting down.
            6 => {
                for i in 0..if small { 14 } else { 34 } {
                    let k = i as u32;
                    let speed = 6. + rnd(k + 1) * 10.;
                    let y = r.y + (rnd(k + 2) * r.h + t * speed) % r.h;
                    let x = r.x + rnd(k) * r.w + (t * 0.5 + k as f32 * 1.3).sin() * 10.;
                    self.circle(
                        x,
                        y,
                        0.8 + rnd(k + 4) * 1.4,
                        accent.alpha(o * (0.25 + rnd(k + 6) * 0.3)),
                    );
                }
            }
            // Aurora: wide translucent ribbons waving.
            7 => {
                for ribbon in 0..3 {
                    let k = ribbon as f32;
                    let mut last: Option<(f32, f32)> = None;
                    for step in 0..=48 {
                        let x = r.x + r.w * step as f32 / 48.;
                        let wave = (x / r.w * 4.2 + t * (0.18 + k * 0.05) + k * 1.9).sin();
                        let y = r.y + r.h * (0.35 + k * 0.12) + wave * r.h * 0.12;
                        if let Some((lx, ly)) = last {
                            for band in 0..5 {
                                let off = band as f32 * 5.;
                                self.line(
                                    lx,
                                    ly + off,
                                    x,
                                    y + off,
                                    accent.alpha(o * (0.07 - band as f32 * 0.012)),
                                    5.,
                                );
                            }
                        }
                        last = Some((x, y));
                    }
                }
            }
            // Wyzar Gold and custom palettes: a slow light sweep over fine hairlines.
            _ => {
                for i in 0..if small { 6 } else { 12 } {
                    let k = i as f32;
                    let x = r.x + r.w * (k + 0.5) / if small { 6. } else { 12. };
                    self.line(
                        x,
                        r.y,
                        x - r.h * 0.6,
                        r.bottom(),
                        accent.alpha(o * 0.035),
                        1.,
                    );
                }
                let span = r.w + r.h * 2.;
                let x = r.x - r.h + (t * 26. + span * 0.62) % span;
                self.glow(
                    x,
                    r.y + r.h * 0.45,
                    r.h.min(280.) * 0.9,
                    accent.alpha(o * 0.12),
                );
                self.glow(
                    r.right() - r.w * 0.18,
                    r.y + r.h * 0.25,
                    r.h.min(280.) * 0.7,
                    accent.alpha(o * 0.07),
                );
            }
        }
        self.unclip();
    }
    pub fn fitted_frame(&self, frame: &Frame, r: Rect) {
        self.fill(r, Color::literal(0x050608));
        if !frame.valid() {
            return;
        }
        let scale = (r.w / frame.width as f32).min(r.h / frame.height as f32);
        let (w, h) = (frame.width as f32 * scale, frame.height as f32 * scale);
        self.frame(
            frame,
            Rect::new(r.x + (r.w - w) / 2., r.y + (r.h - h) / 2., w, h),
        );
    }
    pub fn logo(&self, r: Rect) {
        // Brand geometry stays vector-based and follows the active theme, including custom palettes.
        let unit = r.w.min(r.h) / 64.;
        self.gradient(
            r,
            Color::rgb(0xffdc32),
            crate::ui::theme::GOLD,
            false,
            12. * unit,
        );
        let commands = icons::parse(
            "M6.375 10.5 L9 15 L12 11.25 L14.25 15 L18.375 8.625 M12.375 8.25 L16.125 6.375",
        );
        self.path(&commands, r, self.palette.ink(), 2.25, false);
    }
    pub fn frame(&self, frame: &Frame, r: Rect) {
        if !frame.valid() {
            return;
        }
        unsafe {
            let props = D2D1_BITMAP_PROPERTIES {
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_IGNORE,
                },
                dpiX: 96.,
                dpiY: 96.,
            };
            if let Ok(b) = self.target.CreateBitmap(
                D2D_SIZE_U {
                    width: frame.width,
                    height: frame.height,
                },
                Some(frame.bgra.as_ptr().cast()),
                frame.width * 4,
                &props,
            ) {
                self.target.DrawBitmap(
                    &b,
                    Some(&rect(r)),
                    1.,
                    D2D1_BITMAP_INTERPOLATION_MODE_LINEAR,
                    None,
                );
            }
        }
    }
    pub fn save(&self, path: &Path) -> AppResult<()> {
        let b = self.wic_bitmap.as_ref().ok_or(crate::i18n::tr(
            "Somente superfícies de imagem podem ser exportadas",
        ))?;
        unsafe {
            let stream = self.wic.CreateStream().map_err(|e| e.to_string())?;
            stream
                .InitializeFromFilename(pcw(&wide(path)), GENERIC_WRITE.0)
                .map_err(|e| e.to_string())?;
            let encoder = self
                .wic
                .CreateEncoder(&GUID_ContainerFormatPng, std::ptr::null())
                .map_err(|e| e.to_string())?;
            encoder
                .Initialize(&stream, WICBitmapEncoderNoCache)
                .map_err(|e| e.to_string())?;
            let (mut frame, mut props) = (None, None);
            encoder
                .CreateNewFrame(&mut frame, &mut props)
                .map_err(|e| e.to_string())?;
            let frame = frame.unwrap();
            frame
                .Initialize(props.as_ref())
                .map_err(|e| e.to_string())?;
            frame
                .WriteSource(b, std::ptr::null())
                .map_err(|e| e.to_string())?;
            frame
                .Commit()
                .and_then(|_| encoder.Commit())
                .map_err(|e| e.to_string())
        }
    }
    pub fn pixels(&self) -> AppResult<Frame> {
        let b = self
            .wic_bitmap
            .as_ref()
            .ok_or(crate::i18n::tr("Superfície inválida"))?;
        unsafe {
            let (mut w, mut h) = (0, 0);
            b.GetSize(&mut w, &mut h).map_err(|e| e.to_string())?;
            let mut bgra = vec![0; (w * h * 4) as usize];
            b.CopyPixels(std::ptr::null(), w * 4, &mut bgra)
                .map_err(|e| e.to_string())?;
            Ok(Frame {
                width: w,
                height: h,
                bgra,
            })
        }
    }
}
pub fn save_frame(frame: &Frame, path: &Path) -> AppResult<()> {
    let c = Canvas::plain_image(frame.width, frame.height)?;
    unsafe {
        c.target.BeginDraw();
    }
    c.frame(
        frame,
        Rect::new(0., 0., frame.width as f32, frame.height as f32),
    );
    c.end()?;
    c.save(path)
}
pub fn text_overlay(frame: &Frame, texts: &[String]) -> AppResult<Frame> {
    let mut c = Canvas::plain_image(frame.width, frame.height)?;
    unsafe {
        c.target.BeginDraw();
    }
    c.frame(
        frame,
        Rect::new(0., 0., frame.width as f32, frame.height as f32),
    );
    for (i, text) in texts.iter().enumerate() {
        let y = frame.height as f32 * 0.8 - i as f32 * 52.;
        let r = Rect::new(frame.width as f32 * 0.1, y, frame.width as f32 * 0.8, 44.);
        c.round(r, 8., Color::rgb(0x090c0f).alpha(0.8), None);
        c.center(
            text,
            r,
            frame.width as f32 / 40.,
            Color::rgb(0xffffff),
            true,
        );
    }
    c.end()?;
    c.pixels()
}
