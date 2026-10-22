//! Embedded UI translations. Language changes never alter paths or user media.
use std::{
    collections::{BTreeSet, HashMap},
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum Language {
    #[default]
    PtBr,
    En,
    Es,
    ZhHans,
    Ja,
}
impl Language {
    pub const ALL: [Self; 5] = [Self::PtBr, Self::En, Self::Es, Self::ZhHans, Self::Ja];
    pub fn code(self) -> &'static str {
        ["pt-BR", "en", "es", "zh-Hans", "ja"][self as usize]
    }
    pub fn locale(self) -> &'static str {
        ["pt-BR", "en-US", "es-ES", "zh-CN", "ja-JP"][self as usize]
    }
    pub fn name(self) -> &'static str {
        [
            "Português (Brasil)",
            "English",
            "Español",
            "简体中文",
            "日本語",
        ][self as usize]
    }
    pub fn font(self) -> &'static str {
        match self {
            Self::ZhHans => "Microsoft YaHei UI",
            Self::Ja => "Yu Gothic UI",
            _ => "Segoe UI",
        }
    }
    pub fn parse(code: &str) -> Option<Self> {
        match code.to_ascii_lowercase().replace('_', "-").as_str() {
            "pt" | "pt-br" => Some(Self::PtBr),
            "en" | "en-us" | "en-gb" => Some(Self::En),
            "es" | "es-es" | "es-mx" => Some(Self::Es),
            "zh" | "zh-cn" | "zh-hans" | "zh-sg" => Some(Self::ZhHans),
            "ja" | "ja-jp" => Some(Self::Ja),
            _ => None,
        }
    }
}
static ACTIVE: AtomicU8 = AtomicU8::new(0);
static FORCED: AtomicU8 = AtomicU8::new(255);
static INDEX: OnceLock<HashMap<&'static str, usize>> = OnceLock::new();
static MISSING: OnceLock<Mutex<BTreeSet<String>>> = OnceLock::new();
static AUDIT_RENDER: AtomicBool = AtomicBool::new(false);
mod generated;

pub fn current() -> Language {
    Language::ALL[ACTIVE.load(Ordering::Relaxed).min(4) as usize]
}
pub fn set(language: Language) {
    ACTIVE.store(language as u8, Ordering::Relaxed);
}
pub fn forced() -> Option<Language> {
    Language::ALL
        .get(FORCED.load(Ordering::Relaxed) as usize)
        .copied()
}
pub fn configure_cli(arguments: &[String]) -> Result<(), String> {
    if let Some(index) = arguments.iter().position(|s| s == "--language") {
        let language = arguments
            .get(index + 1)
            .and_then(|s| Language::parse(s))
            .ok_or_else(|| {
                format!(
                    "{}: --language (pt-BR, en, es, zh-Hans, ja). {}",
                    tr("Idioma"),
                    tr("O valor do controle é inválido.")
                )
            })?;
        FORCED.store(language as u8, Ordering::Relaxed);
        set(language);
    }
    Ok(())
}
pub fn translate(language: Language, source: &str) -> &str {
    let index = INDEX.get_or_init(|| {
        generated::ENTRIES
            .iter()
            .enumerate()
            .map(|(i, e)| (e.0, i))
            .collect()
    });
    if let Some(&i) = index.get(source) {
        generated::ENTRIES[i].1[language as usize]
    } else {
        MISSING
            .get_or_init(Mutex::default)
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(source.into());
        source
    }
}
pub fn tr(source: &str) -> &str {
    translate(current(), source)
}
pub fn missing() -> Vec<String> {
    MISSING
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .cloned()
        .collect()
}
pub fn clear_missing() {
    MISSING
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
}
pub fn audit_rendering(enabled: bool) {
    AUDIT_RENDER.store(enabled, Ordering::Relaxed);
}
pub fn observe_rendered_text(text: &str) {
    if Language::ALL.iter().any(|language| language.name() == text) {
        return;
    }
    if !AUDIT_RENDER.load(Ordering::Relaxed) || current() == Language::PtBr {
        return;
    }
    if let Some(index) = INDEX.get().and_then(|keys| keys.get(text)) {
        if generated::ENTRIES[*index].1[current() as usize] != text {
            MISSING
                .get_or_init(Mutex::default)
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .insert(format!("Rendered source without translation: {text}"));
        }
    }
}
pub fn entry_count() -> usize {
    generated::ENTRIES.len()
}
pub fn corpus(language: Language) -> impl Iterator<Item = &'static str> {
    generated::ENTRIES
        .iter()
        .map(move |e| e.1[language as usize])
        .chain(std::iter::once(crate::license::text(language)))
}

/// Format pre-rendered Rust arguments without interpreting paths/user content.
/// Argument values are appended as data and are never parsed a second time.
pub fn format_text(template: &str, arguments: &[(&str, String)]) -> String {
    let mut output = String::with_capacity(template.len() + 32);
    let mut chars = template.chars().peekable();
    let mut implicit = 0;
    while let Some(ch) = chars.next() {
        match ch {
            '{' if chars.peek() == Some(&'{') => {
                chars.next();
                output.push('{');
            }
            '}' if chars.peek() == Some(&'}') => {
                chars.next();
                output.push('}');
            }
            '{' => {
                let mut token = String::new();
                for ch in chars.by_ref() {
                    if ch == '}' {
                        break;
                    }
                    token.push(ch);
                }
                let (name, spec) = token
                    .split_once(':')
                    .map_or((token.as_str(), None), |(a, b)| (a, Some(b)));
                let name = if name.is_empty() {
                    let n = implicit.to_string();
                    implicit += 1;
                    n
                } else {
                    name.to_owned()
                };
                let key = if let Some(spec) = spec {
                    format!("{name}:{spec}")
                } else {
                    name
                };
                if let Some((_, value)) = arguments.iter().find(|(k, _)| *k == key) {
                    output.push_str(value);
                } else {
                    output.push('{');
                    output.push_str(&token);
                    output.push('}');
                }
            }
            _ => output.push(ch),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn codes_are_bounded_and_autonyms_are_distinct() {
        assert_eq!(Language::parse("ZH_cn"), Some(Language::ZhHans));
        assert_eq!(Language::parse("unknown"), None);
        let names: BTreeSet<_> = Language::ALL.iter().map(|l| l.name()).collect();
        assert_eq!(names.len(), 5);
    }
    #[test]
    fn formatting_reorders_arguments_without_parsing_private_values() {
        let args = [
            ("0", "C:\\Vídeos\\{name} 日本語".into()),
            ("1", "42".into()),
        ];
        assert_eq!(
            format_text("{1}: {0}; {{literal}}", &args),
            "42: C:\\Vídeos\\{name} 日本語; {literal}"
        );
        assert_eq!(
            format_text("{} / {}", &args),
            "C:\\Vídeos\\{name} 日本語 / 42"
        );
    }
    #[test]
    fn every_embedded_entry_has_five_explicit_translations() {
        assert!(!generated::ENTRIES.is_empty());
        let mut keys = BTreeSet::new();
        for (key, values) in generated::ENTRIES {
            assert!(keys.insert(key), "duplicate translation key");
            for language in Language::ALL {
                assert!(
                    !values[language as usize].trim().is_empty(),
                    "missing {}: {key}",
                    language.code()
                );
            }
        }
    }
}
