use super::*;

pub(super) fn about(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    let r = Rect::new(240., 91., w - 264., h - 147.);
    panel(c, r);
    c.wallpaper(Rect::new(r.x + 2., r.y + 2., r.w - 4., 250.), 0.45);
    c.logo(Rect::new(r.x + 42., r.y + 48., 72., 72.));
    label(
        c,
        "WyzarCapture",
        r.x + 139.,
        r.y + 48.,
        600.,
        39.,
        TEXT,
        true,
    );
    label(
        c,
        crate::i18n::tr("Grave. Crie. Compartilhe."),
        r.x + 141.,
        r.y + 113.,
        600.,
        20.,
        GOLD,
        false,
    );
    label(
        c,
        crate::i18n::tr("Gravação, edição e biblioteca locais"),
        r.x + 44.,
        r.y + 304.,
        r.w - 88.,
        30.,
        TEXT,
        true,
    );
    label(
        c,
        crate::i18n::tr("Suas gravações, preferências e projetos ficam no seu computador."),
        r.x + 44.,
        r.y + 370.,
        r.w - 88.,
        17.,
        MUTED,
        false,
    );
    label(
        c,
        &{
            let __wyzar_i18n_0 = &(crate::updates::VERSION);
            crate::i18n::format_text(
                crate::i18n::tr("Versão {} · Em desenvolvimento"),
                &[("0", format!("{}", __wyzar_i18n_0))],
            )
        },
        r.x + 44.,
        r.y + 417.,
        r.w - 88.,
        15.,
        MUTED,
        false,
    );
    button(
        app,
        c,
        Rect::new(r.x + 44., r.y + 493., 233., 46.),
        crate::i18n::tr("Configurar pastas"),
        "folder",
        Action::OpenSettings(5),
        Style::Primary,
    );
    button(
        app,
        c,
        Rect::new(r.x + 300., r.y + 493., 233., 46.),
        crate::i18n::tr("Licença"),
        "shield",
        Action::ShowLicense,
        Style::Secondary,
    );
}
