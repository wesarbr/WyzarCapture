//! Local release notes. Parsing never executes Markdown or opens a browser.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CHANGELOG: &str = include_str!("../CHANGELOG.md");
#[derive(Clone, Debug, PartialEq)]
pub struct Section {
    pub title: String,
    pub items: Vec<String>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Release {
    pub heading: String,
    pub sections: Vec<Section>,
}
pub fn parse(text: &str) -> Vec<Release> {
    let mut releases: Vec<Release> = vec![];
    for line in text.lines().map(str::trim) {
        if let Some(heading) = line.strip_prefix("## ") {
            releases.push(Release {
                heading: heading.into(),
                sections: vec![],
            });
        } else if let Some(release) = releases.last_mut() {
            if let Some(title) = line.strip_prefix("### ") {
                release.sections.push(Section {
                    title: title.into(),
                    items: vec![],
                });
            } else if let Some(item) = line.strip_prefix("- ") {
                if release.sections.is_empty() {
                    release.sections.push(Section {
                        title: "Novos recursos".into(),
                        items: vec![],
                    });
                }
                release.sections.last_mut().unwrap().items.push(item.into());
            }
        }
    }
    releases
}
pub fn should_notify(last_seen: &str, notified: &str, version: &str) -> bool {
    last_seen != version && notified != version
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_old_notes_and_sections() {
        let notes = parse(CHANGELOG);
        assert!(notes.first().unwrap().heading.starts_with(VERSION));
        assert!(notes.iter().any(|n| n.heading.starts_with("0.1.0")));
        assert_eq!(
            parse("## 1\n### Correções\n- A\n## 0\n- B")[0].sections[0].items,
            ["A"]
        );
    }
    #[test]
    fn update_notice_is_not_repeated_after_dismiss_or_restart() {
        assert!(should_notify("0.1.0", "0.1.0", "0.2.0"));
        assert!(!should_notify("0.1.0", "0.2.0", "0.2.0"));
        assert!(!should_notify("0.2.0", "", "0.2.0"));
    }
}
