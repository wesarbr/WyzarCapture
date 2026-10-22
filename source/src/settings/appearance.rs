//! Local, versioned appearance preferences. No executable or remote theme content.
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterfaceMode {
    Standard,
    Focus,
    Compact,
    CompactFocus,
}
impl InterfaceMode {
    pub const ALL: [Self; 4] = [
        Self::Standard,
        Self::Focus,
        Self::Compact,
        Self::CompactFocus,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Standard => crate::i18n::tr("Interface padrão"),
            Self::Focus => crate::i18n::tr("Modo foco"),
            Self::Compact => crate::i18n::tr("Modo compacto"),
            Self::CompactFocus => crate::i18n::tr("Compacto · foco"),
        }
    }
    pub fn compact(self) -> bool {
        matches!(self, Self::Compact | Self::CompactFocus)
    }
    pub fn shortcut(key: u16, ctrl: bool, alt: bool) -> Option<Self> {
        if ctrl && alt && (0x31..=0x34).contains(&key) {
            Some(Self::ALL[(key - 0x31) as usize])
        } else {
            None
        }
    }
    pub fn design_size(self) -> (f32, f32) {
        match self {
            Self::Compact => (1120., 408.),
            Self::CompactFocus => (760., 236.),
            _ => (1600., 900.),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Appearance {
    pub theme: usize,
    pub accent: u32,
    pub background: u32,
    pub surface: u32,
    pub text: u32,
    pub icon_style: usize,
    pub icon_tint: Option<u32>,
    pub icon_weight: usize,
    pub animated: bool,
    pub simple: bool,
    pub compact: bool,
    pub compact_focus: bool,
    pub pinned: bool,
    pub minimize_on_record: bool,
    pub exclude_self: bool,
    pub icon_overrides: BTreeMap<String, String>,
}
impl Default for Appearance {
    fn default() -> Self {
        Self {
            theme: 0,
            accent: 0xffd21a,
            background: 0x090c0f,
            surface: 0x101418,
            text: 0xf3f5f7,
            icon_style: 0,
            icon_tint: None,
            icon_weight: 1,
            animated: true,
            simple: false,
            compact: false,
            compact_focus: false,
            pinned: false,
            minimize_on_record: true,
            exclude_self: true,
            icon_overrides: BTreeMap::new(),
        }
    }
}
pub fn hex(value: &str) -> Result<u32, String> {
    let value = value.trim().trim_start_matches('#');
    if value.len() != 6 || !value.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(
            crate::i18n::tr("Use uma cor hexadecimal com seis dígitos, como #FFD21A.").into(),
        );
    }
    u32::from_str_radix(value, 16).map_err(|e| e.to_string())
}
pub fn luminance(rgb: u32) -> f32 {
    let channel = |n: u32| {
        let c = n as f32 / 255.;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    channel((rgb >> 16) & 255) * 0.2126
        + channel((rgb >> 8) & 255) * 0.7152
        + channel(rgb & 255) * 0.0722
}
pub fn contrast(a: u32, b: u32) -> f32 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}
impl Appearance {
    pub fn mode(&self) -> InterfaceMode {
        if self.compact {
            if self.compact_focus {
                InterfaceMode::CompactFocus
            } else {
                InterfaceMode::Compact
            }
        } else if self.simple {
            InterfaceMode::Focus
        } else {
            InterfaceMode::Standard
        }
    }
    pub fn set_mode(&mut self, mode: InterfaceMode) {
        self.compact = mode.compact();
        self.compact_focus = mode == InterfaceMode::CompactFocus;
        self.simple = mode == InterfaceMode::Focus;
    }
    pub fn validate(&self) -> Result<(), String> {
        if contrast(self.text, self.background) < 4.5 || contrast(self.text, self.surface) < 4.5 {
            return Err(crate::i18n::tr(
                "O texto precisa de mais contraste com o fundo e os painéis (mínimo 4,5:1).",
            )
            .into());
        }
        if contrast(self.accent, self.background) < 3. || contrast(self.accent, self.surface) < 3. {
            return Err(crate::i18n::tr(
                "Escolha um destaque com mais contraste com o fundo e os painéis (mínimo 3:1).",
            )
            .into());
        }
        if self
            .icon_tint
            .is_some_and(|c| contrast(c, self.surface) < 3.)
        {
            return Err(crate::i18n::tr(
                "A cor dos ícones precisa de contraste mínimo de 3:1 com o painel.",
            )
            .into());
        }
        Ok(())
    }
    pub fn load(values: &BTreeMap<String, String>) -> Self {
        let mut a = Self::default();
        for (key, dst) in [
            ("theme", &mut a.theme),
            ("icon_style", &mut a.icon_style),
            ("icon_weight", &mut a.icon_weight),
        ] {
            if let Some(n) = values.get(key).and_then(|v| v.parse().ok()) {
                *dst = n;
            }
        }
        a.theme = a.theme.min(8);
        a.icon_style = a.icon_style.min(3);
        a.icon_weight = a.icon_weight.min(2);
        for (key, dst) in [
            ("custom_accent", &mut a.accent),
            ("custom_background", &mut a.background),
            ("custom_surface", &mut a.surface),
            ("custom_text", &mut a.text),
        ] {
            if let Some(c) = values.get(key).and_then(|v| hex(v).ok()) {
                *dst = c;
            }
        }
        a.icon_tint = values.get("icon_tint").and_then(|v| hex(v).ok());
        // 0.7 theme effects are on by default, independent of the old background flag.
        a.animated = values.get("theme_effects").is_none_or(|v| v == "true");
        a.simple = values.get("simple_mode").is_some_and(|v| v == "true");
        a.compact = values.get("compact_mode").is_some_and(|v| v == "true");
        a.compact_focus = values.get("compact_focus").is_some_and(|v| v == "true");
        a.pinned = values.get("compact_pinned").is_some_and(|v| v == "true");
        if let Some(v) = values.get("minimize_on_record") {
            a.minimize_on_record = v == "true";
        }
        if let Some(v) = values.get("exclude_self") {
            a.exclude_self = v == "true";
        }
        if a.validate().is_err() {
            let defaults = Self::default();
            a.accent = defaults.accent;
            a.background = defaults.background;
            a.surface = defaults.surface;
            a.text = defaults.text;
            a.icon_tint = None;
        }
        for (key, value) in values.range("icon_override.".to_owned()..) {
            let Some(name) = key.strip_prefix("icon_override.") else {
                break;
            };
            if crate::ui::icons::NAMES.contains(&name)
                && crate::ui::icons::validate_path(value).is_ok()
            {
                a.icon_overrides.insert(name.into(), value.clone());
            }
        }
        a
    }
    pub fn store(&self, values: &mut BTreeMap<String, String>) {
        for (k, v) in [
            ("theme", self.theme.to_string()),
            ("icon_style", self.icon_style.to_string()),
            ("icon_weight", self.icon_weight.to_string()),
            ("custom_accent", format!("{:06X}", self.accent)),
            ("custom_background", format!("{:06X}", self.background)),
            ("custom_surface", format!("{:06X}", self.surface)),
            ("custom_text", format!("{:06X}", self.text)),
            (
                "icon_tint",
                self.icon_tint
                    .map(|c| format!("{c:06X}"))
                    .unwrap_or_default(),
            ),
            ("animated_background", self.animated.to_string()),
            ("theme_effects", self.animated.to_string()),
            ("simple_mode", self.simple.to_string()),
            ("compact_mode", self.compact.to_string()),
            ("compact_focus", self.compact_focus.to_string()),
            ("compact_pinned", self.pinned.to_string()),
            ("minimize_on_record", self.minimize_on_record.to_string()),
            ("exclude_self", self.exclude_self.to_string()),
        ] {
            values.insert(k.into(), v);
        }
        for (name, path) in &self.icon_overrides {
            values.insert(format!("icon_override.{name}"), path.clone());
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interface_modes_migrate_persist_and_have_scoped_shortcuts() {
        let mut a = Appearance::load(&BTreeMap::from([("simple_mode".into(), "true".into())]));
        assert_eq!(a.mode(), InterfaceMode::Focus);
        for (i, mode) in InterfaceMode::ALL.into_iter().enumerate() {
            a.set_mode(mode);
            a.pinned = true;
            let mut values = BTreeMap::new();
            a.store(&mut values);
            assert_eq!(Appearance::load(&values), a);
            assert_eq!(Appearance::load(&values).mode(), mode);
            assert_eq!(
                InterfaceMode::shortcut(0x31 + i as u16, true, true),
                Some(mode)
            );
            assert_eq!(InterfaceMode::shortcut(0x31 + i as u16, false, true), None);
            assert_eq!(InterfaceMode::shortcut(0x31 + i as u16, true, false), None);
        }
        assert_eq!(InterfaceMode::shortcut(0x35, true, true), None);
    }
    #[test]
    fn colors_are_bounded_and_readable() {
        assert!(hex("#aB90F1").is_ok());
        for value in ["#FFF", "FFFFFFFF", "1e+100", "url(x)", "#12345G"] {
            assert!(hex(value).is_err());
        }
        let mut a = Appearance::default();
        a.text = a.surface;
        assert!(a.validate().is_err());
    }
    #[test]
    fn appearance_roundtrips_and_legacy_uses_defaults() {
        assert_eq!(Appearance::load(&BTreeMap::new()), Appearance::default());
        let a = Appearance {
            theme: 5,
            animated: true,
            simple: true,
            icon_style: 2,
            ..Default::default()
        };
        let mut v = BTreeMap::new();
        a.store(&mut v);
        assert_eq!(Appearance::load(&v), a);
    }
}
