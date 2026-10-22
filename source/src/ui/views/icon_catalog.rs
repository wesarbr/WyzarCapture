use super::*;
pub(super) fn catalog(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    let x = 240.;
    let width = w - x - 24.;
    let a = app.settings.appearance.clone();
    let search = app.icon_search.clone();
    field(
        app,
        c,
        Rect::new(x, 176., width - 321., 39.),
        &search,
        Field::IconSearch,
        crate::i18n::tr("Buscar ícone pelo nome…"),
    );
    button(
        app,
        c,
        Rect::new(w - 330., 176., 306., 39.),
        crate::i18n::tr("Temas, cor e estilo dos ícones"),
        "palette",
        Action::OpenSettings(6),
        Style::Secondary,
    );
    let preview = Rect::new(w - 307., 232., 283., h - 286.);
    panel(c, preview);
    label(
        c,
        crate::i18n::tr("Seu ícone"),
        preview.x + 18.,
        preview.y + 16.,
        preview.w - 36.,
        17.,
        TEXT,
        true,
    );
    c.icon(
        &app.selected_icon,
        Rect::new(preview.x + 102., preview.y + 79., 80., 80.),
        GOLD,
    );
    c.center(
        crate::ui::icons::label(&app.selected_icon),
        Rect::new(preview.x + 15., preview.y + 188., preview.w - 30., 29.),
        17.,
        TEXT,
        true,
    );
    c.center(
        if a.icon_overrides.contains_key(&app.selected_icon) {
            crate::i18n::tr("Vetor personalizado")
        } else {
            crate::i18n::tr("Vetor original")
        },
        Rect::new(preview.x + 15., preview.y + 226., preview.w - 30., 25.),
        12.,
        MUTED,
        false,
    );
    button(
        app,
        c,
        Rect::new(preview.x + 16., preview.y + 277., preview.w - 32., 41.),
        crate::i18n::tr("Importar SVG local"),
        "import",
        Action::ImportIcon,
        Style::Primary,
    );
    button(
        app,
        c,
        Rect::new(preview.x + 16., preview.y + 330., preview.w - 32., 38.),
        crate::i18n::tr("Restaurar este ícone"),
        "reset",
        Action::ResetIcon,
        Style::Secondary,
    );
    c.paragraph(crate::i18n::tr("SVG 24 × 24, até 64 KiB. Use um único path, sem imagens ou referências externas. A forma é salva localmente; a cor e a espessura acompanham suas preferências."),Rect::new(preview.x+18.,preview.y+391.,preview.w-36.,120.),13.,MUTED,false);
    let viewport = Rect::new(x, 232., preview.x - x - 15., h - 286.);
    let names: Vec<_> = super::super::icons::NAMES
        .iter()
        .filter(|name| {
            format!("{} {}", name, crate::ui::icons::label(name))
                .to_lowercase()
                .contains(&search.trim().to_lowercase())
        })
        .copied()
        .collect();
    let columns = 6;
    let cw = (viewport.w - 50.) / columns as f32;
    let total = (names.len().div_ceil(columns) as f32 * 100.).max(1.);
    app.scroll_max = (total - viewport.h).max(0.);
    app.scroll = app.scroll.clamp(0., app.scroll_max);
    c.clip(viewport);
    app.hit_clip = Some(viewport);
    for (i, name) in names.iter().enumerate() {
        let r = Rect::new(
            x + (i % columns) as f32 * (cw + 10.),
            viewport.y + (i / columns) as f32 * 100. - app.scroll,
            cw,
            89.,
        );
        let selected = app.selected_icon == *name;
        c.round(
            r,
            8.,
            if selected { GOLD_DARK } else { PANEL },
            Some(if selected { GOLD } else { BORDER }),
        );
        c.icon(
            name,
            Rect::new(r.x + (r.w - 29.) / 2., r.y + 15., 29., 29.),
            if selected { GOLD } else { TEXT },
        );
        c.center(
            crate::ui::icons::label(name),
            Rect::new(r.x + 4., r.y + 57., r.w - 8., 20.),
            10.,
            MUTED,
            false,
        );
        if a.icon_overrides.contains_key(*name) {
            c.circle(r.right() - 9., r.y + 9., 3., GOLD);
        }
        app.hit(
            r,
            Action::SelectIcon((*name).into()),
            crate::i18n::format_text(
                crate::i18n::tr("Personalizar ícone {name}"),
                &[("name", format!("{}", name))],
            ),
        );
    }
    if names.is_empty() {
        c.center(
            crate::i18n::tr("Nenhum ícone encontrado"),
            viewport,
            17.,
            MUTED,
            false,
        );
    }
    c.unclip();
    app.hit_clip = None;
}
