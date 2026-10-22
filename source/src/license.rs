//! Versioned own terms; original third-party notices are retained verbatim.
use crate::i18n::Language;
pub fn text(language: Language) -> &'static str {
    match language {
        Language::PtBr => include_str!("../localization/legal/pt-BR.txt"),
        Language::En => include_str!("../localization/legal/en.txt"),
        Language::Es => include_str!("../localization/legal/es.txt"),
        Language::ZhHans => include_str!("../localization/legal/zh-Hans.txt"),
        Language::Ja => include_str!("../localization/legal/ja.txt"),
    }
}
pub fn page_count() -> usize {
    text(crate::i18n::current())
        .split("\n\n")
        .count()
        .saturating_sub(1)
}
