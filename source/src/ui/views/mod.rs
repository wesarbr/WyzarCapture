mod appearance_screen;
mod capabilities_screen;
mod compact_screen;
mod compact_window;
mod components_screen;
mod dashboard;
mod diagnostics_screen;
mod editor_screen;
mod help_screen;
mod icon_catalog;
mod information;
mod library_screen;
mod modals;
mod news;
mod recovery_screen;
mod settings_screen;
mod usability_dialogs;
use components_screen::components;
use dashboard::home;
use editor_screen::editor;
use help_screen::help;
use information::about;
use library_screen::library;
use modals::{delete_modal, export_modal, record_modal, text_modal};
use settings_screen::settings;

use super::{app::*, theme::*};
use crate::{
    core::{RecordingState, Source, duration_label, size_label},
    platform::windows::{files, graphics::Canvas},
    settings::{Settings, appearance::InterfaceMode},
};
#[derive(Clone, Copy)]
enum Style {
    Primary,
    Secondary,
    Ghost,
    Selected,
    Danger,
}
fn panel(c: &Canvas, r: Rect) {
    c.round(r, PANEL_RADIUS, PANEL, Some(BORDER));
}
fn label(c: &mut Canvas, text: &str, x: f32, y: f32, w: f32, size: f32, color: Color, bold: bool) {
    c.text(text, Rect::new(x, y, w, size + 10.), size, color, bold);
}
fn button(
    app: &mut App,
    c: &mut Canvas,
    r: Rect,
    title: &str,
    icon: &str,
    action: Action,
    style: Style,
) {
    let no_media = app.project.clips.is_empty();
    let disabled = (app.page == Page::Editor
        && no_media
        && matches!(
            action,
            Action::SaveProject
                | Action::Export
                | Action::Play
                | Action::Split
                | Action::Trim
                | Action::RemoveClip
                | Action::Speed
                | Action::Volume
                | Action::Text
                | Action::Arrow
                | Action::Blur
                | Action::Fade
                | Action::Crop
        ))
        || (matches!(action, Action::Undo) && app.undo.is_empty())
        || (matches!(action, Action::Redo) && app.redo.is_empty())
        || (app.state != RecordingState::Idle
            && matches!(
                action,
                Action::Toggle(_) | Action::ChooseFolder(_) | Action::NewRecording
            ));
    if disabled {
        if !matches!(style, Style::Ghost) {
            c.round(r, BUTTON_RADIUS, PANEL, Some(BORDER.alpha(0.6)));
        }
        if !icon.is_empty() {
            c.icon(
                icon,
                Rect::new(
                    if title.is_empty() {
                        r.x + (r.w - 20.) / 2.
                    } else {
                        r.x + 14.
                    },
                    r.y + (r.h - 20.) / 2.,
                    20.,
                    20.,
                ),
                DIM,
            );
        }
        if !title.is_empty() {
            if icon.is_empty() {
                c.center(title, r, CONTROL_FONT, DIM, true);
            } else {
                c.text(
                    title,
                    Rect::new(r.x + 44., r.y, r.w - 51., r.h),
                    CONTROL_FONT,
                    DIM,
                    false,
                );
            }
        }
        return;
    }
    let index = app.hits.len();
    let amount = app.motion.hover_amount(r, app.hover == Some(index));
    match style {
        Style::Primary => {
            c.round(
                r,
                BUTTON_RADIUS,
                c.blend(GOLD, Color::rgb(0xffe268), amount * 0.7),
                None,
            );
        }
        Style::Selected => {
            c.round(
                r,
                BUTTON_RADIUS,
                GOLD.alpha(0.11 + amount * 0.05),
                Some(GOLD.alpha(0.22)),
            );
        }
        Style::Danger => c.round(
            r,
            BUTTON_RADIUS,
            c.blend(Color::rgb(0x261317), Color::rgb(0x52202b), amount),
            Some(RED.alpha(0.5)),
        ),
        Style::Ghost => {
            if amount > 0.01 {
                c.round(r, BUTTON_RADIUS, HOVER.alpha(amount), None);
            }
        }
        Style::Secondary => c.round(
            r,
            BUTTON_RADIUS,
            c.blend(RAISED, HOVER, amount),
            Some(c.blend(BORDER, DIM, amount)),
        ),
    };
    let color = if matches!(style, Style::Primary) {
        c.palette.ink()
    } else if matches!(style, Style::Danger) {
        RED
    } else {
        TEXT
    };
    if !icon.is_empty() {
        let x = if title.is_empty() {
            r.x + (r.w - 20.) / 2.
        } else {
            r.x + 14.
        };
        c.icon(
            icon,
            Rect::new(x, r.y + (r.h - 20.) / 2., 20., 20.),
            if matches!(style, Style::Selected) {
                GOLD
            } else {
                color
            },
        );
    }
    if !title.is_empty() {
        let font = if r.w < 165. && title.chars().count() > 12 {
            COMPACT_FONT
        } else {
            CONTROL_FONT
        };
        // Navigation and quiet actions read better in the regular weight.
        let strong = !matches!(style, Style::Ghost);
        if icon.is_empty() {
            c.center(title, r, font, color, strong);
        } else {
            c.text(
                title,
                Rect::new(r.x + 44., r.y, r.w - 51., r.h),
                font,
                color,
                strong,
            );
        }
    }
    if app.focus_index == Some(index) {
        c.outline(r.inset(-3.), 10., GOLD, 1.5);
    }
    app.hit(
        r,
        action,
        if title.is_empty() {
            crate::ui::icons::label(icon)
        } else {
            title
        },
    );
}
fn disabled(c: &mut Canvas, r: Rect, title: &str, icon: &str) {
    c.round(r, 8., PANEL, Some(BORDER));
    c.icon(icon, Rect::new(r.x + 14., r.y + 12., 21., 21.), DIM);
    c.text(
        title,
        Rect::new(r.x + 44., r.y, r.w - 48., r.h),
        13.,
        DIM,
        false,
    );
}
fn dropdown(app: &mut App, c: &mut Canvas, r: Rect, text: &str, kind: ChoiceKind) {
    if app.state != RecordingState::Idle
        && !matches!(
            kind,
            ChoiceKind::Interface | ChoiceKind::Theme | ChoiceKind::Language
        )
    {
        c.round(r, 7., PANEL, Some(BORDER));
        c.text(
            text,
            Rect::new(r.x + 12., r.y, r.w - 25., r.h),
            13.,
            DIM,
            false,
        );
        return;
    }
    let amount = app
        .motion
        .hover_amount(r, app.hover == Some(app.hits.len()));
    let open = app.popup.as_ref().is_some_and(|p| p.kind == kind);
    c.round(
        r,
        7.,
        c.blend(RAISED, HOVER, amount),
        Some(if open {
            GOLD.alpha(0.7)
        } else {
            c.blend(BORDER, DIM, amount)
        }),
    );
    c.text(
        text,
        Rect::new(r.x + 12., r.y, r.w - 43., r.h),
        13.,
        TEXT,
        false,
    );
    c.icon(
        "chevron",
        Rect::new(r.right() - 26., r.y + (r.h - 14.) / 2., 14., 14.),
        if open { GOLD } else { MUTED },
    );
    app.hit(r, Action::OpenChoice(kind, r), text);
}
fn toggle(app: &mut App, c: &mut Canvas, r: Rect, on: bool, which: Toggle) {
    let knob = r.h / 2. - 3.;
    if app.state != RecordingState::Idle {
        c.round(r, r.h / 2., BORDER, None);
        c.circle(
            if on {
                r.right() - r.h / 2.
            } else {
                r.x + r.h / 2.
            },
            r.y + r.h / 2.,
            knob,
            DIM,
        );
        return;
    }
    // A switch that was just flipped slides from its previous side.
    let t = match app.motion.toggled {
        Some((t, at)) if t == which => app.motion.progress(at, 0.16),
        _ => 1.,
    };
    let position = if on { t } else { 1. - t };
    let off = Color::rgb(0x343c44);
    c.round(r, r.h / 2., c.blend(off, GOLD, position), None);
    let hover = app.hover == Some(app.hits.len());
    let x = r.x + r.h / 2. + (r.w - r.h) * position;
    if hover {
        c.circle(x, r.y + r.h / 2., knob + 4., TEXT.alpha(0.08));
    }
    c.circle(
        x,
        r.y + r.h / 2.,
        knob,
        c.blend(Color::rgb(0xd9e1e8), BG, position),
    );
    app.hit(
        r.inset(-7.),
        Action::Toggle(which),
        if on {
            crate::i18n::tr("Desativar")
        } else {
            crate::i18n::tr("Ativar")
        },
    );
}
fn field(app: &mut App, c: &mut Canvas, r: Rect, text: &str, kind: Field, placeholder: &str) {
    let active = app.focused == Some(kind);
    c.round(r, 7., BG, Some(if active { GOLD } else { BORDER }));
    if active && app.select_all {
        c.round(r.inset(4.), 3., Color::rgb(0x574a11), None);
    }
    let display = if text.is_empty() { placeholder } else { text };
    c.text(
        display,
        Rect::new(r.x + 12., r.y, r.w - 24., r.h),
        14.,
        if text.is_empty() { DIM } else { TEXT },
        false,
    );
    let blink =
        !app.motion.enabled || (app.animation_started.elapsed().as_secs_f32() * 1.6).fract() < 0.6;
    if active && !app.select_all && blink {
        let x = (r.x + 13. + c.text_width(text, 14., false)).min(r.right() - 10.);
        c.line(x, r.y + 10., x, r.bottom() - 10., GOLD, 1.);
    }
    app.hit(r, Action::Focus(kind), placeholder);
}
fn progress(c: &Canvas, r: Rect, value: f32, color: Color) {
    c.round(r, r.h / 2., BORDER, None);
    if value > 0. {
        c.round(
            Rect::new(r.x, r.y, (r.w * value.clamp(0., 1.)).max(r.h), r.h),
            r.h / 2.,
            color,
            None,
        );
    }
}
fn chip(c: &mut Canvas, r: Rect, text: &str, color: Color) {
    c.round(r, r.h / 2., color.alpha(0.12), Some(color.alpha(0.22)));
    c.center(text, r, 11., color, true);
}
fn section(c: &mut Canvas, r: Rect, title: &str, subtitle: &str, icon: &str) {
    panel(c, r);
    c.icon(icon, Rect::new(r.x + 20., r.y + 19., 21., 21.), GOLD);
    label(c, title, r.x + 53., r.y + 13., r.w - 80., 17., TEXT, true);
    if !subtitle.is_empty() {
        label(
            c,
            subtitle,
            r.x + 53.,
            r.y + 38.,
            r.w - 72.,
            12.,
            MUTED,
            false,
        );
    }
}
fn row_toggle(
    app: &mut App,
    c: &mut Canvas,
    x: f32,
    y: f32,
    w: f32,
    title: &str,
    subtitle: &str,
    on: bool,
    t: Toggle,
) {
    label(c, title, x, y, w - 70., 14., TEXT, false);
    if !subtitle.is_empty() {
        label(c, subtitle, x, y + 23., w - 70., 12., MUTED, false);
    }
    toggle(app, c, Rect::new(x + w - 45., y + 5., 42., 23.), on, t);
}
fn row_choice(
    app: &mut App,
    c: &mut Canvas,
    x: f32,
    y: f32,
    w: f32,
    title: &str,
    value: &str,
    k: ChoiceKind,
) {
    label(c, title, x, y + 2., w * 0.45, 13., TEXT, false);
    dropdown(app, c, Rect::new(x + w * 0.46, y, w * 0.54, 33.), value, k);
}
fn resolution(s: &Settings) -> String {
    format!("{} × {}", s.width, s.height)
}
/// An outside click both dismisses a choice and targets the visible control.
/// Rebuild before hit testing so the old popup cannot consume the first click.
pub fn pointer_hit(app: &mut App, c: &mut Canvas, x: f32, y: f32) -> Option<usize> {
    if app.popup.as_ref().is_some_and(|p| !p.rect.contains(x, y)) {
        app.popup = None;
        app.hover = None;
        app.focus_index = None;
        render(app, c);
    }
    app.action_at(x, y)
}
/// Returns true when the display device was rebuilt and the frame must be drawn again.
pub fn render(app: &mut App, c: &mut Canvas) -> bool {
    c.palette = Palette::new(&app.settings.appearance);
    c.animation_time = if app.effects_running() {
        app.animation_started.elapsed().as_secs_f32()
    } else {
        0.
    };
    let mode = app.interface_mode();
    let design = mode.design_size();
    let (w, h) = c.begin_layout(design.0, design.1);
    app.viewport = (w, h);
    app.hit_clip = None;
    app.scroll_max = 0.;
    app.hits.clear();
    // A different page, settings category or modal starts a short entrance.
    let screen = (app.page, app.category, mode.compact());
    if app.motion.page != Some(screen) {
        if app.motion.page.is_some() {
            app.motion.page_at = std::time::Instant::now();
        }
        app.motion.page = Some(screen);
    }
    if mode.compact() {
        compact_window::render(app, c, w, h, mode);
    } else {
        shell(app, c, w, h);
        let enter = app.motion.progress(app.motion.page_at, 0.22);
        let fading = enter < 0.999;
        if fading {
            c.push_opacity(0.25 + 0.75 * enter);
        }
        match app.page {
            Page::Home => {
                if app.settings.appearance.simple {
                    compact_screen::compact(app, c, w, h)
                } else {
                    home(app, c, w, h)
                }
            }
            Page::Settings => settings(app, c, w, h),
            Page::Library => library(app, c, w, h),
            Page::Editor => editor(app, c, w, h),
            Page::Components => components(app, c, w, h),
            Page::Help => help(app, c, w, h),
            Page::About => about(app, c, w, h),
            Page::News => news::news(app, c, w, h),
            Page::Recovery => recovery_screen::recovery(app, c, w, h),
            Page::Diagnostics => diagnostics_screen::diagnostics(app, c, w, h),
            Page::Capabilities => capabilities_screen::capabilities(app, c, w, h),
        }
        if fading {
            c.pop_opacity();
        }
    }
    if app.modal.is_some() != app.motion.modal_open {
        app.motion.modal_open = app.modal.is_some();
        app.motion.modal_at = std::time::Instant::now();
    }
    if let Some(modal) = app.modal.clone() {
        app.hits.clear();
        let enter = app.motion.progress(app.motion.modal_at, 0.18);
        c.fill(Rect::new(0., 64., w, h - 64.), BG.alpha(0.78 * enter));
        c.push_opacity(enter);
        match modal {
            Modal::Shortcut(index) => usability_dialogs::shortcut(app, c, w, h, index),
            Modal::Folders => usability_dialogs::folders(app, c, w, h),
            Modal::Screenshot => usability_dialogs::screenshot(app, c, w, h),
            Modal::Camera => usability_dialogs::camera(app, c, w, h),
            Modal::License => {
                let layout = crate::ui::license_view::render(c, w, h, app.license_page);
                button(
                    app,
                    c,
                    layout.previous,
                    crate::i18n::tr("Voltar"),
                    "",
                    Action::LicensePage(false),
                    Style::Secondary,
                );
                button(
                    app,
                    c,
                    layout.next,
                    crate::i18n::tr("Próximo"),
                    "",
                    Action::LicensePage(true),
                    Style::Secondary,
                );
                button(
                    app,
                    c,
                    layout.close,
                    crate::i18n::tr("Fechar"),
                    "",
                    Action::CloseModal,
                    Style::Primary,
                );
            }
            Modal::Recording => record_modal(app, c, w, h),
            Modal::Export => export_modal(app, c, w, h),
            Modal::Text { field, title } => text_modal(app, c, w, h, field, &title),
            Modal::Delete => delete_modal(app, c, w, h),
            Modal::RecoveryDetails(index) => recovery_screen::details(app, c, w, h, index, false),
            Modal::DiscardRecovery(index) => recovery_screen::details(app, c, w, h, index, true),
            Modal::ErrorDetails => diagnostics_screen::error_details(app, c, w, h),
            Modal::AudioTracks => modals::audio_tracks(app, c, w, h),
            Modal::Cleanup(plan) => modals::cleanup(app, c, w, h, &plan),
            Modal::ExistingData(data) => modals::existing_data(app, c, w, h, &data),
        }
        c.pop_opacity();
    }
    if app.popup.is_some() != app.motion.popup_open {
        app.motion.popup_open = app.popup.is_some();
        app.motion.popup_at = std::time::Instant::now();
    }
    if let Some(popup) = app.popup.clone() {
        let r = popup.rect;
        app.hits.clear();
        let enter = app.motion.progress(app.motion.popup_at, 0.14);
        c.push_opacity(enter);
        c.round(
            Rect::new(r.x + 2., r.y + 8., r.w, r.h),
            10.,
            BG.alpha(0.7),
            None,
        );
        c.round(r, 10., RAISED, Some(BORDER));
        for (row, (i, v)) in popup
            .values
            .iter()
            .enumerate()
            .skip(popup.first_visible)
            .enumerate()
        {
            let item = Rect::new(r.x + 5., r.y + 6. + row as f32 * 35., r.w - 10., 34.);
            if item.bottom() > r.bottom() {
                break;
            }
            let disabled = v.ends_with(crate::i18n::tr("Indisponível"));
            let selected = popup.kind == ChoiceKind::Interface
                && InterfaceMode::ALL.get(i).copied() == Some(app.interface_mode());
            if !disabled {
                let amount = app
                    .motion
                    .hover_amount(item, app.hover == Some(app.hits.len()));
                if selected {
                    c.round(item, 6., GOLD_DARK, None);
                } else if amount > 0.01 {
                    c.round(item, 6., HOVER.alpha(amount), None);
                }
            }
            c.text(
                v,
                Rect::new(item.x + 10., item.y, item.w - 40., item.h),
                13.,
                if disabled {
                    DIM
                } else if selected {
                    GOLD
                } else {
                    TEXT
                },
                selected,
            );
            if selected {
                c.icon(
                    "check",
                    Rect::new(item.right() - 26., item.y + 10., 14., 14.),
                    GOLD,
                );
            }
            if !disabled {
                app.hit(item, Action::SelectChoice(popup.kind, i), v);
            }
        }
        c.pop_opacity();
    }
    if let Some(hit) = app.focus_index.and_then(|i| app.hits.get(i)) {
        c.outline(hit.rect.inset(-2.), BUTTON_RADIUS, GOLD, 1.5);
    }
    let waited = app.hover_since.elapsed().as_secs_f32();
    if waited > 0.7 && app.popup.is_none() && app.modal.is_none() {
        if let Some(hit) = app.hover.and_then(|i| app.hits.get(i)).filter(|hit| {
            !hit.label.is_empty()
                && !matches!(
                    hit.action,
                    Action::Seek(_) | Action::Slider(_, _) | Action::None | Action::Focus(_)
                )
        }) {
            let text = hit.label.clone();
            let r = hit.rect;
            let tw = (c.text_width(&text, 12., false) + 28.).clamp(80., 420.);
            let tr = Rect::new(
                r.x.min(w - tw - 12.).max(12.),
                if r.bottom() + 45. > h - 36. {
                    r.y - 37.
                } else {
                    r.bottom() + 7.
                },
                tw,
                30.,
            );
            let fade = if app.motion.enabled {
                ((waited - 0.7) / 0.12).clamp(0., 1.)
            } else {
                1.
            };
            c.push_opacity(fade);
            c.round(tr, 7., RAISED, Some(BORDER));
            c.center(&text, tr, 12., TEXT, false);
            c.pop_opacity();
        }
    }
    if let Some(toast) = app
        .toast
        .clone()
        .filter(|toast| app.modal.is_none() || toast.error || toast.warning)
    {
        let tone = if toast.warning {
            GOLD
        } else if toast.error {
            RED
        } else {
            GREEN
        };
        let enter = app.motion.progress(toast.since, 0.24);
        let width = 620f32.min(w - if mode.compact() { 40. } else { 280. });
        let r = Rect::new(
            w - width - 20.,
            h - if mode.compact() { 80. } else { 116. },
            width,
            65.,
        );
        c.push_opacity(enter);
        c.translate(0., (1. - enter) * 18.);
        c.round(
            Rect::new(r.x + 2., r.y + 6., r.w, r.h),
            12.,
            BG.alpha(0.6),
            None,
        );
        c.round(r, 12., RAISED, Some(tone.alpha(0.45)));
        c.round(Rect::new(r.x, r.y + 12., 3., r.h - 24.), 1.5, tone, None);
        c.circle(r.x + 30., r.y + 32., 12., tone.alpha(0.16));
        c.icon(
            if toast.warning {
                "warning"
            } else if toast.error {
                "close"
            } else {
                "check"
            },
            Rect::new(r.x + 22., r.y + 24., 16., 16.),
            tone,
        );
        c.text(
            &toast.text,
            Rect::new(
                r.x + 54.,
                r.y + 5.,
                r.w - if toast.action.is_some() { 230. } else { 107. },
                55.,
            ),
            13.5,
            TEXT,
            false,
        );
        c.reset_transform();
        if let Some(action) = toast.action {
            button(
                app,
                c,
                Rect::new(r.right() - 174., r.y + 15., 126., 35.),
                if matches!(action, Action::Page(Page::News)) {
                    crate::i18n::tr("Ver novidades")
                } else {
                    crate::i18n::tr("Ver detalhes")
                },
                "",
                action,
                Style::Secondary,
            );
        }
        button(
            app,
            c,
            Rect::new(r.right() - 40., r.y + 15., 28., 35.),
            "",
            "close",
            Action::DismissToast,
            Style::Ghost,
        );
        c.pop_opacity();
    }
    c.present().unwrap_or(false)
}
fn shell(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    c.fill(Rect::new(0., 0., w, 64.), SIDEBAR);
    c.fill(Rect::new(0., 64., 216., h - 100.), SIDEBAR);
    c.line(0., 64., w, 64., BORDER, 1.);
    c.line(216., 64., 216., h - 36., BORDER, 1.);
    c.logo(Rect::new(24., 15., 34., 34.));
    let brand = c.text_width("Wyzar", 22., true);
    label(c, "Wyzar", 70., 14., brand + 4., 22., GOLD, true);
    let rest = c.text_width("Capture", 22., true);
    label(c, "Capture", 71. + brand, 14., rest + 4., 22., TEXT, true);
    dropdown(
        app,
        c,
        Rect::new(w - 397., 15., 175., 35.),
        app.interface_mode().label(),
        ChoiceKind::Interface,
    );
    button(
        app,
        c,
        Rect::new(w - 204., 14., 39., 37.),
        "",
        "settings",
        Action::Page(Page::Settings),
        Style::Ghost,
    );
    for (offset, icon, name, action) in [
        (
            152.,
            "minimize",
            crate::i18n::tr("Minimizar"),
            Action::Minimize,
        ),
        (
            104.,
            if app.maximized { "restore" } else { "maximize" },
            if app.maximized {
                crate::i18n::tr("Restaurar")
            } else {
                crate::i18n::tr("Maximizar")
            },
            Action::Maximize,
        ),
        (56., "close", crate::i18n::tr("Fechar"), Action::Close),
    ] {
        let r = Rect::new(w - offset, 9., 48., 46.);
        let amount = app
            .motion
            .hover_amount(r, app.hover == Some(app.hits.len()));
        if amount > 0.01 {
            c.round(
                r,
                7.,
                if icon == "close" {
                    RED.alpha(amount)
                } else {
                    HOVER.alpha(amount)
                },
                None,
            );
        }
        c.icon(
            icon,
            Rect::new(r.x + 15., r.y + 14., 18., 18.),
            if amount > 0.5 && icon == "close" {
                Color::literal(0xffffff)
            } else {
                TEXT
            },
        );
        app.hit(r, action, name);
    }
    let nav = [
        (
            crate::i18n::tr("Início"),
            "home",
            Action::Page(Page::Home),
            app.page == Page::Home,
        ),
        (
            crate::i18n::tr("Gravação"),
            "record",
            Action::NewRecording,
            false,
        ),
        (
            crate::i18n::tr("Biblioteca"),
            "library",
            Action::Page(Page::Library),
            app.page == Page::Library,
        ),
        (
            crate::i18n::tr("Edição rápida"),
            "edit",
            Action::Page(Page::Editor),
            app.page == Page::Editor,
        ),
        (
            crate::i18n::tr("Configurações"),
            "settings",
            Action::Page(Page::Settings),
            app.page == Page::Settings,
        ),
    ];
    let secondary = [
        (crate::i18n::tr("Conta local"), "account", Page::About),
        (crate::i18n::tr("Ajuda e atalhos"), "help", Page::Help),
        (crate::i18n::tr("Componentes"), "timeline", Page::Components),
        (crate::i18n::tr("Novidades"), "news", Page::News),
    ];
    // One indicator slides between the active entries of both groups.
    let target = nav
        .iter()
        .position(|n| n.3)
        .map(|i| 91. + i as f32 * 59.)
        .or_else(|| {
            secondary
                .iter()
                .position(|s| s.2 == app.page)
                .filter(|_| !(app.settings.appearance.simple && app.page == Page::Home))
                .map(|i| 417. + i as f32 * 55.)
        });
    for (i, (title, icon, a, active)) in nav.into_iter().enumerate() {
        let r = Rect::new(20., 91. + i as f32 * 59., 180., 47.);
        button(
            app,
            c,
            r,
            title,
            icon,
            a,
            if active {
                Style::Selected
            } else {
                Style::Ghost
            },
        );
    }
    c.line(30., 399., 190., 399., BORDER, 1.);
    for (i, (title, icon, page)) in secondary.into_iter().enumerate() {
        if app.settings.appearance.simple && app.page == Page::Home && page != Page::Help {
            continue;
        }
        button(
            app,
            c,
            Rect::new(20., 417. + i as f32 * 55., 180., 45.),
            title,
            icon,
            Action::Page(page),
            if app.page == page {
                Style::Selected
            } else {
                Style::Ghost
            },
        );
    }
    if let Some(y) = target {
        let y = app.motion.nav_y(y);
        c.round(Rect::new(10., y + 12., 4., 23.), 2., GOLD, None);
    }
    // Real state of the recordings drive instead of a slogan.
    button(
        app,
        c,
        Rect::new(20., h - 128., 180., 42.),
        crate::i18n::tr("Suas pastas"),
        "folder",
        Action::OpenFolders,
        Style::Ghost,
    );
    if let Some((free, total)) = files::free_space(&app.settings.videos) {
        progress(
            c,
            Rect::new(34., h - 78., 152., 4.),
            1. - free as f32 / total.max(1) as f32,
            if free < app.settings.low_space_gb as u64 * 1_073_741_824 {
                RED
            } else {
                GOLD.alpha(0.8)
            },
        );
        label(
            c,
            &crate::i18n::format_text(
                crate::i18n::tr("{} livres de {}"),
                &[("0", size_label(free)), ("1", size_label(total))],
            ),
            34.,
            h - 70.,
            160.,
            11.,
            DIM,
            false,
        );
    }
    c.fill(Rect::new(0., h - 36., w, 36.), SIDEBAR);
    c.line(0., h - 36., w, h - 36., BORDER, 1.);
    let (state, color) = match app.state {
        RecordingState::Idle => (crate::i18n::tr("Pronto para gravar"), GREEN),
        RecordingState::Countdown => (crate::i18n::tr("Preparando gravação"), GOLD),
        RecordingState::Starting => (crate::i18n::tr("Iniciando captura…"), GOLD),
        RecordingState::Recording => (crate::i18n::tr("Gravando"), RED),
        RecordingState::Paused => (crate::i18n::tr("Gravação pausada"), GOLD),
        RecordingState::Finalizing => (crate::i18n::tr("Salvando gravação…"), GOLD),
    };
    if app.state == RecordingState::Recording && app.motion.enabled {
        let pulse = (app.animation_started.elapsed().as_secs_f32() * 2.).sin() * 0.5 + 0.5;
        c.circle(
            24.,
            h - 18.,
            6. + 4. * pulse,
            color.alpha(0.25 * (1. - pulse)),
        );
    }
    c.circle(24., h - 18., 5., color);
    let status = if matches!(
        app.state,
        RecordingState::Recording | RecordingState::Paused
    ) {
        {
            let __wyzar_i18n_0 = &(duration_label(app.seconds));
            crate::i18n::format_text(
                crate::i18n::tr("{state} · {}"),
                &[
                    ("state", format!("{}", state)),
                    ("0", format!("{}", __wyzar_i18n_0)),
                ],
            )
        }
    } else {
        state.into()
    };
    label(c, &status, 40., h - 29., 350., 12., TEXT, false);
    let text = if app.settings.recording_mode == 3 {
        {
            let __wyzar_i18n_0 = &(crate::updates::VERSION);
            crate::i18n::format_text(
                crate::i18n::tr("WAV · PCM 16 bits · 48 kHz · estéreo       |       v{}"),
                &[("0", format!("{}", __wyzar_i18n_0))],
            )
        }
    } else {
        {
            let __wyzar_i18n_0 = &(app.settings.width);
            let __wyzar_i18n_1 = &(app.settings.height);
            let __wyzar_i18n_2 = &(app.settings.fps);
            let __wyzar_i18n_3 = &(crate::updates::VERSION);
            crate::i18n::format_text(
                crate::i18n::tr(
                    "{} × {}   ·   {} FPS configurados   ·   MP4 / H.264       |       v{}",
                ),
                &[
                    ("0", format!("{}", __wyzar_i18n_0)),
                    ("1", format!("{}", __wyzar_i18n_1)),
                    ("2", format!("{}", __wyzar_i18n_2)),
                    ("3", format!("{}", __wyzar_i18n_3)),
                ],
            )
        }
    };
    c.text(
        &text,
        Rect::new(w - 620., h - 35., 595., 35.),
        12.,
        MUTED,
        false,
    );
}

fn title(c: &mut Canvas, title: &str, subtitle: &str, x: f32, y: f32, w: f32) {
    label(c, title, x, y + 2., w, PAGE_TITLE_FONT, TEXT, true);
    label(c, subtitle, x, y + 44., w, 14.5, MUTED, false);
}
fn source_card(app: &mut App, c: &mut Canvas, r: Rect, source: Source, compact: bool) {
    let selected = app.settings.source == source;
    panel(c, r);
    if app.hover == Some(app.hits.len()) {
        c.round(r.inset(1.), 10., HOVER.alpha(0.4), None);
    }
    if selected {
        c.outline(r, 10., GOLD, 2.);
        c.gradient(r.inset(2.), GOLD.alpha(0.07), GOLD.alpha(0.), false, 8.);
    }
    let name = match source {
        Source::Display => "monitor",
        Source::Window => "window",
        Source::Region => "region",
        Source::Game => "game",
    };
    let size = if compact { 30. } else { 34. };
    let short = r.h < 114.;
    c.icon(
        name,
        Rect::new(
            r.x + (r.w - size) / 2.,
            r.y + if short { 13. } else { 20. },
            size,
            size,
        ),
        if selected { GOLD } else { TEXT },
    );
    c.center(
        source.label(),
        Rect::new(r.x + 4., r.y + if short { 50. } else { 61. }, r.w - 8., 25.),
        14.,
        TEXT,
        true,
    );
    c.center(
        match source {
            Source::Display => crate::i18n::tr("Grave toda a tela"),
            Source::Window => crate::i18n::tr("Apenas uma janela"),
            Source::Region => crate::i18n::tr("Selecione uma área"),
            Source::Game => crate::i18n::tr("Janela de jogo"),
        },
        Rect::new(
            r.x + 4.,
            r.y + if short { 77. } else { 86. },
            r.w - 8.,
            if short { 18. } else { 23. },
        ),
        if short { 10. } else { 11. },
        MUTED,
        false,
    );
    if selected {
        c.circle(r.right() - 15., r.y + 15., 6., GOLD);
        c.icon("check", Rect::new(r.right() - 20., r.y + 10., 10., 10.), BG);
    }
    app.hit(r, Action::Source(source), source.label());
}
