use super::*;
pub(super) fn news(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    title(
        c,
        crate::i18n::tr("Novidades"),
        crate::i18n::tr("Mudanças de cada versão, disponíveis offline."),
        242.,
        84.,
        w - 268.,
    );
    chip(
        c,
        Rect::new(w - 260., 89., 234., 29.),
        &{
            let __wyzar_i18n_0 = &(crate::updates::VERSION);
            crate::i18n::format_text(
                crate::i18n::tr("Versão {}"),
                &[("0", format!("{}", __wyzar_i18n_0))],
            )
        },
        GOLD,
    );
    let area = Rect::new(240., 176., w - 266., h - 236.);
    c.clip(area);
    let mut y = area.y + 2. - app.scroll;
    for release in crate::updates::parse(crate::updates::CHANGELOG) {
        let lines = release
            .sections
            .iter()
            .map(|s| {
                44. + s
                    .items
                    .iter()
                    .map(|item| {
                        c.paragraph_height(crate::i18n::tr(item), area.w - 84., 13., false) + 14.
                    })
                    .sum::<f32>()
            })
            .sum::<f32>();
        let card = Rect::new(area.x, y, area.w - 16., lines + 76.);
        panel(c, card);
        label(
            c,
            &release.heading,
            card.x + 24.,
            y + 18.,
            card.w - 48.,
            22.,
            TEXT,
            true,
        );
        let mut row = y + 64.;
        for section in release.sections {
            label(
                c,
                crate::i18n::tr(&section.title),
                card.x + 24.,
                row,
                card.w - 48.,
                15.,
                GOLD,
                true,
            );
            row += 36.;
            for item in section.items {
                c.circle(card.x + 29., row + 9., 2.5, DIM);
                let text = crate::i18n::tr(&item);
                let height = c.paragraph_height(text, card.w - 68., 13., false);
                c.paragraph(
                    text,
                    Rect::new(card.x + 43., row - 3., card.w - 68., height + 3.),
                    13.,
                    MUTED,
                    false,
                );
                row += height + 14.;
            }
            row += 8.;
        }
        y += card.h + 16.;
    }
    c.unclip();
    app.scroll_max = (y + app.scroll - area.bottom()).max(0.);
    if app.scroll_max > 0. {
        let height = (area.h * area.h / (area.h + app.scroll_max)).max(36.);
        let track = Rect::new(area.right() - 5., area.y, 3., area.h);
        c.round(track, 2., BORDER, None);
        c.round(
            Rect::new(
                track.x,
                track.y + (track.h - height) * app.scroll / app.scroll_max,
                track.w,
                height,
            ),
            2.,
            GOLD,
            None,
        );
    }
}
