use super::*;

pub(super) fn appearance(app: &mut App, c: &mut Canvas, viewport: Rect) {
    let a = app.settings.appearance.clone();
    let total = 1140.;
    app.scroll_max = (total - viewport.h).max(0.);
    app.scroll = app.scroll.clamp(0., app.scroll_max);
    app.hit_clip = Some(viewport);
    c.clip(viewport);
    let x = viewport.x;
    let y = viewport.y - app.scroll;
    let width = viewport.w;
    label(
        c,
        crate::i18n::tr("Temas e efeitos"),
        x + 2.,
        y,
        width,
        23.,
        TEXT,
        true,
    );
    label(
        c,
        crate::i18n::tr(
            "Cada tema tem cores e um efeito animado próprio. As escolhas são salvas automaticamente.",
        ),
        x + 2.,
        y + 36.,
        width,
        13.,
        MUTED,
        false,
    );
    let cw = (width - 28.) / 3.;
    for i in 0..9 {
        let (name, detail, accent, bg, surface) = if let Some(p) = PRESETS.get(i) {
            (
                crate::i18n::tr(p.name),
                crate::i18n::tr(p.detail),
                p.accent,
                p.background,
                p.surface,
            )
        } else {
            (
                crate::i18n::tr("Personalizado"),
                crate::i18n::tr("Suas cores · varredura de luz"),
                a.accent,
                a.background,
                a.surface,
            )
        };
        let r = Rect::new(
            x + (i % 3) as f32 * (cw + 14.),
            y + 79. + (i / 3) as f32 * 119.,
            cw,
            105.,
        );
        c.round(
            r,
            10.,
            Color::literal(surface),
            Some(if a.theme == i {
                Color::literal(accent)
            } else {
                BORDER
            }),
        );
        let top = Rect::new(r.x + 9., r.y + 9., r.w - 18., 35.);
        c.round(top, 5., Color::literal(bg), None);
        // Live sample of this theme's own effect, drawn with its palette.
        let saved = c.palette.clone();
        c.palette = Palette::new(&crate::settings::appearance::Appearance {
            theme: i,
            ..a.clone()
        });
        c.clip(top.inset(1.));
        c.wallpaper(top, 1.);
        c.unclip();
        c.palette = saved;
        for n in 0..3 {
            c.round(
                Rect::new(top.x + 9. + n as f32 * 18., top.y + 12., 12., 12.),
                3.,
                Color::literal(if n == 0 {
                    accent
                } else {
                    mix(surface, 0xf3f5f7, 0.12 + n as f32 * 0.06)
                }),
                None,
            );
        }
        if a.theme == i {
            c.icon(
                "check",
                Rect::new(top.right() - 25., top.y + 8., 18., 18.),
                Color::literal(accent),
            );
        }
        label(
            c,
            name,
            r.x + 12.,
            r.y + 49.,
            r.w - 24.,
            15.,
            Color::literal(if i == 8 { a.text } else { TEXT.0 }),
            true,
        );
        label(
            c,
            detail,
            r.x + 12.,
            r.y + 76.,
            r.w - 24.,
            11.,
            Color::literal(if i == 8 {
                mix(a.surface, a.text, 0.7)
            } else {
                0xa8b2bf
            }),
            false,
        );
        if app.hover == Some(app.hits.len()) {
            c.outline(r.inset(2.), 8., Color::literal(accent), 1.);
        }
        app.hit(
            r,
            Action::SetTheme(i),
            crate::i18n::format_text(
                crate::i18n::tr("Aplicar tema {name}"),
                &[("name", format!("{}", name))],
            ),
        );
    }
    let custom = Rect::new(x, y + 452., width, 142.);
    panel(c, custom);
    label(
        c,
        crate::i18n::tr("Sua paleta"),
        x + 18.,
        custom.y + 12.,
        width - 36.,
        16.,
        TEXT,
        true,
    );
    button(
        app,
        c,
        Rect::new(custom.right() - 208., custom.y + 8., 190., 31.),
        crate::i18n::tr("Paleta completa"),
        "palette",
        Action::EditAppearance(5),
        Style::Ghost,
    );
    label(
        c,
        crate::i18n::tr(
            "Editar uma cor ativa o tema personalizado; o contraste é verificado antes de salvar.",
        ),
        x + 18.,
        custom.y + 42.,
        width - 36.,
        12.,
        MUTED,
        false,
    );
    let bw = (width - 48.) / 4.;
    for (i, (title, value)) in [
        (crate::i18n::tr("Destaque"), a.accent),
        (crate::i18n::tr("Fundo"), a.background),
        (crate::i18n::tr("Painéis"), a.surface),
        (crate::i18n::tr("Texto"), a.text),
    ]
    .into_iter()
    .enumerate()
    {
        let r = Rect::new(x + 12. + i as f32 * (bw + 8.), custom.y + 83., bw, 42.);
        c.round(r, 7., RAISED, Some(BORDER));
        c.round(
            Rect::new(r.x + 11., r.y + 11., 20., 20.),
            5.,
            Color::literal(value),
            Some(DIM),
        );
        label(c, title, r.x + 42., r.y + 4., r.w - 48., 12., TEXT, true);
        label(
            c,
            &format!("#{value:06X}"),
            r.x + 42.,
            r.y + 22.,
            r.w - 48.,
            10.,
            MUTED,
            false,
        );
        app.hit(
            r,
            Action::EditAppearance(i),
            crate::i18n::format_text(
                crate::i18n::tr("Personalizar {title}"),
                &[("title", format!("{}", title))],
            ),
        );
    }
    let icons = Rect::new(x, y + 608., width, 178.);
    panel(c, icons);
    label(
        c,
        crate::i18n::tr("Ícones que acompanham você"),
        x + 18.,
        icons.y + 13.,
        width - 36.,
        16.,
        TEXT,
        true,
    );
    let col = (width - 48.) / 3.;
    label(
        c,
        crate::i18n::tr("Estilo"),
        x + 18.,
        icons.y + 49.,
        col,
        12.,
        MUTED,
        false,
    );
    dropdown(
        app,
        c,
        Rect::new(x + 18., icons.y + 73., col - 12., 35.),
        [
            crate::i18n::tr("Vinculado ao tema"),
            crate::i18n::tr("Contorno suave"),
            crate::i18n::tr("Duo com destaque"),
            crate::i18n::tr("Técnico"),
        ][a.icon_style],
        ChoiceKind::IconStyle,
    );
    label(
        c,
        crate::i18n::tr("Espessura"),
        x + col + 22.,
        icons.y + 49.,
        col,
        12.,
        MUTED,
        false,
    );
    dropdown(
        app,
        c,
        Rect::new(x + col + 22., icons.y + 73., col - 12., 35.),
        [
            crate::i18n::tr("Fino"),
            crate::i18n::tr("Equilibrado"),
            crate::i18n::tr("Forte"),
        ][a.icon_weight],
        ChoiceKind::IconWeight,
    );
    button(
        app,
        c,
        Rect::new(x + 2. * col + 26., icons.y + 73., col - 12., 35.),
        crate::i18n::tr("Cor dos ícones"),
        "palette",
        Action::EditAppearance(4),
        Style::Secondary,
    );
    button(
        app,
        c,
        Rect::new(x + 18., icons.y + 125., 260., 35.),
        crate::i18n::tr("Explorar e personalizar ícones"),
        "layers",
        Action::Page(Page::Components),
        Style::Ghost,
    );
    let options = Rect::new(x, y + 800., width, 261.);
    panel(c, options);
    row_toggle(
        app,
        c,
        x + 18.,
        options.y + 13.,
        width - 36.,
        crate::i18n::tr("Efeitos animados do tema"),
        crate::i18n::tr(
            "Movimento suave em cada tema. Pausa durante gravação e exportação e segue a opção de animações do Windows.",
        ),
        a.animated,
        Toggle::Animated,
    );
    label(
        c,
        crate::i18n::tr("Interface"),
        x + 18.,
        options.y + 79.,
        width - 350.,
        14.,
        TEXT,
        true,
    );
    label(
        c,
        crate::i18n::tr("Padrão, foco ou janela compacta."),
        x + 18.,
        options.y + 104.,
        width - 350.,
        12.,
        MUTED,
        false,
    );
    dropdown(
        app,
        c,
        Rect::new(x + width - 286., options.y + 84., 268., 36.),
        a.mode().label(),
        ChoiceKind::Interface,
    );
    row_toggle(
        app,
        c,
        x + 18.,
        options.y + 137.,
        width - 36.,
        crate::i18n::tr("Minimizar ao começar a gravar"),
        crate::i18n::tr("A contagem aparece no monitor; seus atalhos continuam ativos."),
        a.minimize_on_record,
        Toggle::MinimizeOnRecord,
    );
    row_toggle(
        app,
        c,
        x + 18.,
        options.y + 199.,
        width - 36.,
        crate::i18n::tr("Ocultar WyzarCapture da captura"),
        crate::i18n::tr(
            "Evita a repetição da prévia em tela e região. Desative para gravar o app.",
        ),
        a.exclude_self,
        Toggle::ExcludeSelf,
    );
    button(
        app,
        c,
        Rect::new(x, y + 1077., 215., 40.),
        crate::i18n::tr("Restaurar aparência"),
        "reset",
        Action::ResetAppearance,
        Style::Secondary,
    );
    label(
        c,
        crate::i18n::tr("Sem reiniciar o aplicativo."),
        x + 233.,
        y + 1087.,
        width - 240.,
        12.,
        MUTED,
        false,
    );
    c.unclip();
    app.hit_clip = None;
    if app.scroll_max > 0. {
        let ratio = viewport.h / total;
        c.round(
            Rect::new(
                viewport.right() - 5.,
                viewport.y + app.scroll / total * viewport.h,
                3.,
                viewport.h * ratio,
            ),
            1.5,
            DIM,
            None,
        );
    }
}
