use super::*;

pub(super) fn components(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    let x = 240.;
    title(
        c,
        crate::i18n::tr("Componentes de interface"),
        crate::i18n::tr("Botões, campos, seletores e ícones usados no aplicativo."),
        x + 2.,
        84.,
        w - x - 24.,
    );
    button(
        app,
        c,
        Rect::new(w - 303., 91., 128., 38.),
        crate::i18n::tr("Controles"),
        "sliders",
        Action::ComponentsTab(0),
        if app.components_tab == 0 {
            Style::Selected
        } else {
            Style::Secondary
        },
    );
    button(
        app,
        c,
        Rect::new(w - 164., 91., 140., 38.),
        crate::i18n::tr("Ícones"),
        "layers",
        Action::ComponentsTab(1),
        if app.components_tab == 1 {
            Style::Selected
        } else {
            Style::Secondary
        },
    );
    if app.components_tab == 1 {
        super::icon_catalog::catalog(app, c, w, h);
        return;
    }
    let width = w - x - 24.;
    let third = (width - 28.) / 3.;
    let y = 180.;
    for i in 0..3 {
        panel(c, Rect::new(x + i as f32 * (third + 14.), y, third, 165.));
    }
    label(
        c,
        crate::i18n::tr("01. Botões"),
        x + 18.,
        y + 13.,
        third - 36.,
        16.,
        TEXT,
        true,
    );
    button(
        app,
        c,
        Rect::new(x + 18., y + 59., third - 36., 37.),
        crate::i18n::tr("Nova gravação"),
        "record",
        Action::NewRecording,
        Style::Primary,
    );
    disabled(
        c,
        Rect::new(x + 18., y + 109., (third - 48.) / 2., 35.),
        crate::i18n::tr("Desabilitado"),
        "save",
    );
    button(
        app,
        c,
        Rect::new(
            x + 30. + (third - 48.) / 2.,
            y + 109.,
            (third - 48.) / 2.,
            35.,
        ),
        crate::i18n::tr("Perigo"),
        "trash",
        Action::None,
        Style::Danger,
    );
    let xx = x + third + 14.;
    label(
        c,
        crate::i18n::tr("02. Campos e seletores"),
        xx + 18.,
        y + 13.,
        third - 36.,
        16.,
        TEXT,
        true,
    );
    field(
        app,
        c,
        Rect::new(xx + 18., y + 59., third - 36., 36.),
        &app.search.clone(),
        Field::Search,
        crate::i18n::tr("Buscar gravações…"),
    );
    dropdown(
        app,
        c,
        Rect::new(xx + 18., y + 110., third - 36., 34.),
        crate::i18n::tr("60 FPS"),
        ChoiceKind::Fps,
    );
    let xx = x + 2. * (third + 14.);
    label(
        c,
        crate::i18n::tr("03. Estados e alternadores"),
        xx + 18.,
        y + 13.,
        third - 36.,
        16.,
        TEXT,
        true,
    );
    toggle(
        app,
        c,
        Rect::new(xx + 19., y + 65., 44., 24.),
        app.settings.cursor,
        Toggle::Cursor,
    );
    label(
        c,
        crate::i18n::tr("Ativado"),
        xx + 79.,
        y + 63.,
        110.,
        14.,
        TEXT,
        false,
    );
    toggle(
        app,
        c,
        Rect::new(xx + 19., y + 114., 44., 24.),
        app.settings.webcam,
        Toggle::Webcam,
    );
    label(
        c,
        crate::i18n::tr("Desativado"),
        xx + 79.,
        y + 112.,
        160.,
        14.,
        MUTED,
        false,
    );
    let row = Rect::new(x, 361., width, 184.);
    panel(c, row);
    label(
        c,
        crate::i18n::tr("04. Cards de captura"),
        x + 18.,
        row.y + 13.,
        width - 36.,
        16.,
        TEXT,
        true,
    );
    for (i, s) in Source::ALL.into_iter().enumerate() {
        source_card(
            app,
            c,
            Rect::new(x + 18. + i as f32 * 180., row.y + 53., 165., 116.),
            s,
            true,
        );
    }
    label(
        c,
        crate::i18n::tr("05. Status"),
        x + 780.,
        row.y + 15.,
        250.,
        16.,
        TEXT,
        true,
    );
    for (i, (name, color)) in [
        (crate::i18n::tr("● REC"), RED),
        (crate::i18n::tr("● PRONTO"), MUTED),
        (crate::i18n::tr("● PROCESSANDO"), GOLD),
        (crate::i18n::tr("✓ CONCLUÍDO"), GREEN),
    ]
    .into_iter()
    .enumerate()
    {
        chip(
            c,
            Rect::new(
                x + 780. + (i % 2) as f32 * 178.,
                row.y + 62. + (i / 2) as f32 * 48.,
                158.,
                30.,
            ),
            name,
            color,
        );
    }
    let r = Rect::new(x, 560., (width - 14.) / 2., h - 614.);
    panel(c, r);
    label(
        c,
        crate::i18n::tr("06. Ícones vetoriais"),
        r.x + 18.,
        r.y + 13.,
        r.w - 36.,
        16.,
        TEXT,
        true,
    );
    let names = [
        "home",
        "record",
        "library",
        "edit",
        "settings",
        "account",
        "help",
        "news",
        "monitor",
        "window",
        "region",
        "game",
        "microphone",
        "audio",
        "webcam",
        "cursor",
        "export",
        "trash",
        "folder",
        "save",
        "timeline",
        "search",
        "filter",
        "bell",
        "storage",
        "resolution",
        "fps",
        "encoder",
        "tag",
        "calendar",
        "clock",
        "volume",
    ];
    for (i, name) in names.into_iter().enumerate() {
        let xx = r.x + 22. + (i % 8) as f32 * 65.;
        let yy = r.y + 63. + (i / 8) as f32 * 53.;
        c.icon(
            name,
            Rect::new(xx, yy, 24., 24.),
            if i % 8 == 0 { GOLD } else { TEXT },
        );
    }
    let r2 = Rect::new(r.right() + 14., 560., (width - 14.) / 2., h - 614.);
    panel(c, r2);
    label(
        c,
        crate::i18n::tr("07. Progresso e notificações"),
        r2.x + 18.,
        r2.y + 13.,
        r2.w - 36.,
        16.,
        TEXT,
        true,
    );
    label(
        c,
        crate::i18n::tr("Exportação de vídeo"),
        r2.x + 18.,
        r2.y + 63.,
        r2.w - 36.,
        13.,
        TEXT,
        false,
    );
    progress(
        c,
        Rect::new(r2.x + 18., r2.y + 98., r2.w - 36., 9.),
        0.78,
        GOLD,
    );
    for (i, (name, color)) in [
        (crate::i18n::tr("Gravação salva com sucesso"), GREEN),
        (crate::i18n::tr("Espaço de armazenamento baixo"), GOLD),
        (crate::i18n::tr("Erro ao iniciar a gravação"), RED),
    ]
    .into_iter()
    .enumerate()
    {
        let rr = Rect::new(r2.x + 18., r2.y + 133. + i as f32 * 39., r2.w - 36., 32.);
        c.round(rr, 5., color.alpha(0.08), Some(color.alpha(0.3)));
        c.circle(rr.x + 17., rr.y + 16., 5., color);
        label(c, name, rr.x + 33., rr.y + 3., rr.w - 45., 12., TEXT, false);
    }
}
