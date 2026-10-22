#[derive(Clone, Copy, Debug)]
pub struct Color(pub u32, pub f32);
impl Color {
    pub const fn rgb(hex: u32) -> Self {
        Self(hex, 1.)
    }
    pub fn alpha(self, a: f32) -> Self {
        Self(self.0, a)
    }
    pub const fn literal(hex: u32) -> Self {
        Self(hex | 0x80000000, 1.)
    }
}
pub const BG: Color = Color::rgb(0x090c0f);
pub const SIDEBAR: Color = Color::rgb(0x0b0e11);
pub const PANEL: Color = Color::rgb(0x101418);
pub const RAISED: Color = Color::rgb(0x161b20);
pub const HOVER: Color = Color::rgb(0x20262c);
pub const BORDER: Color = Color::rgb(0x2a3036);
pub const TEXT: Color = Color::rgb(0xf3f5f7);
pub const MUTED: Color = Color::rgb(0x9ca5af);
pub const DIM: Color = Color::rgb(0x626e79);
pub const GOLD: Color = Color::rgb(0xffd21a);
pub const GOLD_DARK: Color = Color::rgb(0x332b0b);
pub const GREEN: Color = Color::rgb(0x21cf7a);
pub const RED: Color = Color::rgb(0xff3b49);
pub const BLUE: Color = Color::rgb(0x398ef5);
pub const PURPLE: Color = Color::rgb(0xa66bf3);
pub const CONTROL_FONT: f32 = 14.;
pub const COMPACT_FONT: f32 = 12.;
pub const BUTTON_RADIUS: f32 = 8.;
pub const PANEL_RADIUS: f32 = 11.;
pub const PAGE_TITLE_FONT: f32 = 28.;

#[derive(Clone, Copy, Debug)]
pub struct Preset {
    pub name: &'static str,
    pub detail: &'static str,
    pub accent: u32,
    pub background: u32,
    pub surface: u32,
    pub icon_style: usize,
}
pub const PRESETS: [Preset; 8] = [
    Preset {
        name: "Wyzar Gold",
        detail: "Preto e dourado · varredura de luz",
        accent: 0xffd21a,
        background: 0x090c0f,
        surface: 0x101418,
        icon_style: 1,
    },
    Preset {
        name: "Safira",
        detail: "Azul profundo · luzes suaves",
        accent: 0x75b8ff,
        background: 0x090f1a,
        surface: 0x111e30,
        icon_style: 1,
    },
    Preset {
        name: "Esmeralda",
        detail: "Verde mineral · linhas de contorno",
        accent: 0x5edcb0,
        background: 0x091411,
        surface: 0x11241e,
        icon_style: 2,
    },
    Preset {
        name: "Ametista",
        detail: "Violeta · nebulosa",
        accent: 0xc7a0ff,
        background: 0x110c1a,
        surface: 0x20172e,
        icon_style: 2,
    },
    Preset {
        name: "Coral",
        detail: "Coral quente · ondas",
        accent: 0xffac97,
        background: 0x160e0e,
        surface: 0x291b1b,
        icon_style: 1,
    },
    Preset {
        name: "Cobre",
        detail: "Âmbar e café · brasas",
        accent: 0xeeb57c,
        background: 0x15100c,
        surface: 0x292018,
        icon_style: 3,
    },
    Preset {
        name: "Ártico",
        detail: "Prata · neve fina",
        accent: 0xc6d8e9,
        background: 0x0d1218,
        surface: 0x1b242e,
        icon_style: 3,
    },
    Preset {
        name: "Aurora",
        detail: "Turquesa · aurora",
        accent: 0x6ee4e9,
        background: 0x0a111a,
        surface: 0x132432,
        icon_style: 2,
    },
];
#[derive(Clone, Debug)]
pub struct Palette {
    pub appearance: crate::settings::appearance::Appearance,
    pub accent: u32,
    pub background: u32,
    pub surface: u32,
    pub text: u32,
    pub style: usize,
}
impl Default for Palette {
    fn default() -> Self {
        Self::new(&Default::default())
    }
}
pub fn mix(a: u32, b: u32, t: f32) -> u32 {
    let channel = |shift: u32| {
        let a = ((a >> shift) & 255u32) as f32;
        let b = ((b >> shift) & 255u32) as f32;
        ((a + (b - a) * t.clamp(0., 1.)).round() as u32) << shift
    };
    channel(16) | channel(8) | channel(0)
}
impl Palette {
    pub fn new(a: &crate::settings::appearance::Appearance) -> Self {
        let (accent, background, surface, text, linked) = if let Some(p) = PRESETS.get(a.theme) {
            (p.accent, p.background, p.surface, TEXT.0, p.icon_style)
        } else {
            (a.accent, a.background, a.surface, a.text, 1)
        };
        Self {
            appearance: a.clone(),
            accent,
            background,
            surface,
            text,
            style: if a.icon_style == 0 {
                linked
            } else {
                a.icon_style
            },
        }
    }
    pub fn resolve(&self, c: Color) -> Color {
        if c.0 & 0x80000000 != 0 {
            return Color(c.0 & 0xffffff, c.1);
        }
        let hex = match c.0 {
            0x090c0f => self.background,
            0x0b0e11 => mix(self.background, self.surface, 0.27),
            0x101418 => self.surface,
            0x161b20 | 0x242b31 => mix(self.surface, self.text, 0.035),
            0x20262c => mix(self.surface, self.text, 0.08),
            0x2a3036 => mix(self.surface, self.text, 0.14),
            0xf3f5f7 => self.text,
            0x9ca5af => mix(self.surface, self.text, 0.67),
            0x626e79 => mix(self.surface, self.text, 0.49),
            0xffd21a => self.accent,
            0x332b0b | 0x28220e => mix(self.background, self.accent, 0.14),
            0xffe268 => mix(self.accent, 0xffffff, 0.27),
            0xffdc32 => mix(self.accent, 0xffffff, 0.1),
            0xffe66c => mix(self.accent, 0xffffff, 0.2),
            0x5c4b0d => mix(self.accent, self.surface, 0.25),
            0x10241b => mix(self.surface, GREEN.0, 0.10),
            0x2b141a | 0x261317 => mix(self.surface, RED.0, 0.10),
            0x52202b => mix(self.surface, RED.0, 0.20),
            0x574a11 => mix(self.surface, self.accent, 0.25),
            _ => c.0,
        };
        Color(hex, c.1)
    }
    pub fn ink(&self) -> Color {
        use crate::settings::appearance::contrast;
        Color::literal(
            if contrast(self.accent, 0x090c0f) >= contrast(self.accent, 0xffffff) {
                0x090c0f
            } else {
                0xffffff
            },
        )
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }
    pub fn contains(self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.w && y >= self.y && y <= self.y + self.h
    }
    pub fn inset(self, n: f32) -> Self {
        Self::new(
            self.x + n,
            self.y + n,
            (self.w - 2. * n).max(0.),
            (self.h - 2. * n).max(0.),
        )
    }
    pub fn right(self) -> f32 {
        self.x + self.w
    }
    pub fn bottom(self) -> f32 {
        self.y + self.h
    }
    pub fn intersection(self, other: Self) -> Option<Self> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        (right > x && bottom > y).then(|| Self::new(x, y, right - x, bottom - y))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_themes_have_legible_text_and_controls() {
        use crate::settings::appearance::contrast;
        for preset in PRESETS {
            assert!(contrast(preset.surface, TEXT.0) > 4.5, "{}", preset.name);
            assert!(
                contrast(preset.surface, preset.accent) > 3.,
                "{}",
                preset.name
            );
        }
    }
    #[test]
    fn clipped_hitboxes_never_extend_beyond_the_scrolling_panel() {
        let panel = Rect::new(10., 20., 200., 100.);
        let clipped = Rect::new(0., 90., 100., 100.).intersection(panel).unwrap();
        assert_eq!(clipped, Rect::new(10., 90., 90., 30.));
        assert!(!clipped.contains(50., 130.));
        assert!(Rect::new(0., 200., 10., 10.).intersection(panel).is_none());
    }
}
