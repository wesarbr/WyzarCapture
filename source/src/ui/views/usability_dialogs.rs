use super::*;

pub(super) fn shortcut(app: &mut App, c: &mut Canvas, w: f32, h: f32, index: usize) {
    let r = Rect::new((w - 760.) / 2., (h - 330.) / 2., 760., 330.);
    panel(c, r);
    label(
        c,
        crate::i18n::tr("Pressione seu atalho"),
        r.x + 28.,
        r.y + 24.,
        r.w - 56.,
        25.,
        TEXT,
        true,
    );
    let names = [
        "Iniciar gravação",
        "Pausar / retomar",
        "Parar gravação",
        "Capturar tela",
    ];
    c.center(
        crate::i18n::tr(names[index.min(3)]),
        Rect::new(r.x + 24., r.y + 82., r.w - 48., 40.),
        21.,
        GOLD,
        true,
    );
    c.paragraph(crate::i18n::tr("Pressione uma tecla ou combinação com Ctrl, Alt, Shift ou Win. Esc cancela. Atalhos reservados pelo Windows ou usados por outro programa podem ficar indisponíveis."), Rect::new(r.x + 28., r.y + 142., r.w - 56., 100.), 16., MUTED, false);
    button(
        app,
        c,
        Rect::new(r.right() - 172., r.bottom() - 62., 144., 38.),
        crate::i18n::tr("Cancelar"),
        "",
        Action::CloseModal,
        Style::Secondary,
    );
}
pub(super) fn folders(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    let r = Rect::new((w - 950.) / 2., (h - 600.) / 2. + 16., 950., 600.);
    panel(c, r);
    label(
        c,
        crate::i18n::tr("Suas pastas"),
        r.x + 28.,
        r.y + 22.,
        760.,
        26.,
        TEXT,
        true,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 60., r.y + 18., 36., 36.),
        "",
        "close",
        Action::CloseModal,
        Style::Ghost,
    );
    let paths = [
        app.settings.videos.clone(),
        app.settings.screenshots.clone(),
        app.settings.cache.clone(),
        app.settings.exports.clone(),
        app.settings.projects.clone(),
        app.settings.audios.clone(),
    ];
    for (i, name) in [
        "Vídeos",
        "Capturas de tela",
        "Temporários e cache",
        "Exportações",
        "Projetos de edição",
        "Gravações de áudio",
    ]
    .into_iter()
    .enumerate()
    {
        let y = r.y + 78. + i as f32 * 66.;
        label(
            c,
            crate::i18n::tr(name),
            r.x + 28.,
            y,
            540.,
            14.,
            TEXT,
            true,
        );
        label(
            c,
            &paths[i].display().to_string(),
            r.x + 28.,
            y + 24.,
            r.w - 310.,
            11.,
            MUTED,
            false,
        );
        button(
            app,
            c,
            Rect::new(r.right() - 258., y + 10., 106., 36.),
            crate::i18n::tr("Abrir"),
            "folder",
            Action::OpenFolder(i),
            Style::Secondary,
        );
        button(
            app,
            c,
            Rect::new(r.right() - 140., y + 10., 112., 36.),
            crate::i18n::tr("Alterar"),
            "settings",
            Action::ChooseFolder(i),
            Style::Secondary,
        );
    }
    button(
        app,
        c,
        Rect::new(r.x + 28., r.bottom() - 105., 420., 40.),
        crate::i18n::tr("Escolher raiz para todas as mídias"),
        "folder",
        Action::ChooseMediaRoot,
        Style::Primary,
    );
    button(
        app,
        c,
        Rect::new(r.x + 464., r.bottom() - 105., r.w - 492., 40.),
        crate::i18n::tr("Restaurar pastas do Windows"),
        "storage",
        Action::RestoreMediaDefaults,
        Style::Secondary,
    );
    label(
        c,
        crate::i18n::tr(
            "Novos arquivos usarão os destinos escolhidos. Arquivos existentes permanecem nas pastas originais.",
        ),
        r.x + 28.,
        r.bottom() - 50.,
        r.w - 56.,
        12.,
        MUTED,
        false,
    );
}
pub(super) fn screenshot(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    let r = Rect::new((w - 830.) / 2., (h - 570.) / 2. + 15., 830., 570.);
    panel(c, r);
    label(
        c,
        crate::i18n::tr("Captura de tela · PrintScreen"),
        r.x + 28.,
        r.y + 22.,
        r.w - 95.,
        25.,
        TEXT,
        true,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 60., r.y + 18., 36., 36.),
        "",
        "close",
        Action::CloseModal,
        Style::Ghost,
    );
    let s = app.settings.clone();
    let sources = ["Tela inteira", "Janela", "Região", "Usar fonte da gravação"];
    row_choice(
        app,
        c,
        r.x + 28.,
        r.y + 83.,
        r.w - 56.,
        crate::i18n::tr("Fonte de captura"),
        crate::i18n::tr(sources[s.screenshot_mode]),
        ChoiceKind::ScreenshotSource,
    );
    let effective_source = match s.screenshot_mode {
        0 => Source::Display,
        1 => Source::Window,
        2 => Source::Region,
        _ => s.source,
    };
    let target = if matches!(effective_source, Source::Window | Source::Game) {
        app.window_name.clone()
    } else if effective_source == Source::Region {
        app.region
            .map(|r| format!("{} × {}", r.width, r.height))
            .unwrap_or_default()
    } else {
        app.monitors
            .get(s.monitor)
            .map(|m| m.name.clone())
            .unwrap_or_default()
    };
    row_choice(
        app,
        c,
        r.x + 28.,
        r.y + 143.,
        r.w - 56.,
        crate::i18n::tr("Selecionar destino da captura"),
        if target.is_empty() {
            crate::i18n::tr("Selecionar")
        } else {
            &target
        },
        if effective_source == Source::Display {
            ChoiceKind::Monitor
        } else {
            ChoiceKind::Window
        },
    );
    row_choice(
        app,
        c,
        r.x + 28.,
        r.y + 203.,
        r.w - 56.,
        crate::i18n::tr("Atraso antes da captura"),
        &format!("{} s", s.screenshot_delay),
        ChoiceKind::ScreenshotDelay,
    );
    for (i, name, on, toggle_kind) in [
        (
            0,
            "Incluir cursor",
            s.screenshot_cursor,
            Toggle::ScreenshotCursor,
        ),
        (
            1,
            "Copiar imagem para colar",
            s.screenshot_clipboard,
            Toggle::ScreenshotClipboard,
        ),
    ] {
        let y = r.y + 275. + i as f32 * 44.;
        label(
            c,
            crate::i18n::tr(name),
            r.x + 28.,
            y,
            r.w - 120.,
            15.,
            TEXT,
            false,
        );
        toggle(
            app,
            c,
            Rect::new(r.right() - 76., y, 42., 23.),
            on,
            toggle_kind,
        );
    }
    label(
        c,
        &s.screenshots.display().to_string(),
        r.x + 28.,
        r.y + 378.,
        r.w - 56.,
        12.,
        MUTED,
        false,
    );
    label(
        c,
        &crate::i18n::format_text(
            crate::i18n::tr("Atalho: {} · PNG na resolução original da fonte"),
            &[("0", crate::settings::shortcuts::label(s.screenshot_hotkey))],
        ),
        r.x + 28.,
        r.y + 415.,
        r.w - 56.,
        13.,
        GOLD,
        false,
    );
    button(
        app,
        c,
        Rect::new(r.x + 28., r.bottom() - 70., 165., 42.),
        crate::i18n::tr("Abrir pasta"),
        "folder",
        Action::OpenFolder(1),
        Style::Secondary,
    );
    button(
        app,
        c,
        Rect::new(r.x + 205., r.bottom() - 70., 155., 42.),
        crate::i18n::tr("Alterar"),
        "settings",
        Action::ChooseFolder(1),
        Style::Secondary,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 258., r.bottom() - 70., 230., 42.),
        crate::i18n::tr("Capturar agora"),
        "resolution",
        Action::TakeScreenshot,
        Style::Primary,
    );
}
pub(super) fn camera(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    let r = Rect::new(22., 78., w - 44., h - 100.);
    panel(c, r);
    label(
        c,
        crate::i18n::tr("Webcam · visualização ampliada"),
        r.x + 20.,
        r.y + 16.,
        r.w - 410.,
        23.,
        TEXT,
        true,
    );
    let name = app.device_name(Toggle::Webcam);
    dropdown(
        app,
        c,
        Rect::new(r.right() - 360., r.y + 16., 286., 36.),
        &name,
        ChoiceKind::Webcam,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 58., r.y + 16., 36., 36.),
        "",
        "close",
        Action::CloseModal,
        Style::Ghost,
    );
    let frame_rect = Rect::new(r.x + 18., r.y + 70., r.w - 36., r.h - 125.);
    c.fill(frame_rect, Color::literal(0x000000));
    let frame = app.webcam_frame.lock().ok().and_then(|f| f.clone());
    if let Some(frame) = frame {
        c.fitted_frame(&frame, frame_rect);
    } else {
        c.center(
            crate::i18n::tr("Aguardando imagem da câmera…"),
            frame_rect,
            20.,
            MUTED,
            false,
        );
    }
    label(
        c,
        crate::i18n::tr(
            "Prévia com proporção preservada. Esc fecha. Ative Webcam na gravação para incluir a câmera no vídeo.",
        ),
        r.x + 20.,
        r.bottom() - 41.,
        r.w - 40.,
        13.,
        MUTED,
        false,
    );
}
