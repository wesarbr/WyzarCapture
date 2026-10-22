use super::*;

pub(super) fn compact(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    let x = 240.;
    let width = w - x - 24.;
    title(
        c,
        crate::i18n::tr("Nova gravação"),
        crate::i18n::tr("Escolha a fonte, confira o áudio e comece."),
        x + 2.,
        84.,
        width,
    );
    button(
        app,
        c,
        Rect::new(w - 210., 92., 186., 40.),
        crate::i18n::tr("Ajustar gravação"),
        "settings",
        Action::NewRecording,
        Style::Secondary,
    );
    let rail = Rect::new(w - 306., 178., 282., h - 232.);
    panel(c, rail);
    label(
        c,
        crate::i18n::tr("Fonte de captura"),
        rail.x + 18.,
        rail.y + 16.,
        rail.w - 36.,
        17.,
        TEXT,
        true,
    );
    for (i, source) in Source::ALL.into_iter().enumerate() {
        source_card(
            app,
            c,
            Rect::new(
                rail.x + 14. + (i % 2) as f32 * 129.,
                rail.y + 61. + (i / 2) as f32 * 114.,
                124.,
                102.,
            ),
            source,
            true,
        );
    }
    let sy = rail.y + 301.;
    for (i, (name, t, on, icon)) in [
        (
            crate::i18n::tr("Microfone"),
            Toggle::Microphone,
            app.settings.microphone,
            "microphone",
        ),
        (
            crate::i18n::tr("Som do sistema"),
            Toggle::SystemAudio,
            app.settings.system_audio,
            "audio",
        ),
        (
            crate::i18n::tr("Webcam"),
            Toggle::Webcam,
            app.settings.webcam,
            "webcam",
        ),
        (
            crate::i18n::tr("Cursor"),
            Toggle::Cursor,
            app.settings.cursor,
            "cursor",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        if t == Toggle::Webcam && app.webcams.is_empty() {
            continue;
        }
        let yy = sy + i as f32 * 43.;
        c.icon(icon, Rect::new(rail.x + 18., yy + 4., 20., 20.), MUTED);
        label(
            c,
            name,
            rail.x + 51.,
            yy + 3.,
            rail.w - 114.,
            13.,
            TEXT,
            false,
        );
        toggle(
            app,
            c,
            Rect::new(rail.right() - 57., yy + 5., 37., 21.),
            on,
            t,
        );
    }
    button(
        app,
        c,
        Rect::new(rail.x + 14., rail.bottom() - 51., rail.w - 28., 36.),
        crate::i18n::tr("Pasta de destino"),
        "folder",
        Action::OpenFolders,
        Style::Secondary,
    );
    let preview = Rect::new(x, 178., rail.x - x - 16., h - 406.);
    panel(c, preview);
    dashboard::preview_content(app, c, preview.inset(11.), false);
    let status = if app.state == RecordingState::Recording {
        {
            let __wyzar_i18n_0 = &(duration_label(app.seconds));
            crate::i18n::format_text(
                crate::i18n::tr("Gravando · {}"),
                &[("0", format!("{}", __wyzar_i18n_0))],
            )
        }
    } else if app.state == RecordingState::Paused {
        {
            let __wyzar_i18n_0 = &(duration_label(app.seconds));
            crate::i18n::format_text(
                crate::i18n::tr("Pausado · {}"),
                &[("0", format!("{}", __wyzar_i18n_0))],
            )
        }
    } else if app.state == RecordingState::Countdown {
        crate::i18n::tr("Preparando · contagem no monitor").into()
    } else if app.state == RecordingState::Finalizing {
        crate::i18n::tr("Salvando sua gravação…").into()
    } else {
        crate::i18n::tr("Pronto para gravar").into()
    };
    label(
        c,
        &status,
        x + 3.,
        preview.bottom() + 14.,
        preview.w,
        16.,
        if app.state == RecordingState::Recording {
            RED
        } else {
            TEXT
        },
        true,
    );
    label(
        c,
        &{
            let __wyzar_i18n_0 = &(resolution(&app.settings));
            let __wyzar_i18n_1 = &(app.settings.fps);
            crate::i18n::format_text(
                crate::i18n::tr("{} · {} FPS · MP4 / H.264"),
                &[
                    ("0", format!("{}", __wyzar_i18n_0)),
                    ("1", format!("{}", __wyzar_i18n_1)),
                ],
            )
        },
        x + 3.,
        preview.bottom() + 45.,
        preview.w,
        12.,
        MUTED,
        false,
    );
    let controls = Rect::new(x, h - 160., preview.w, 98.);
    panel(c, controls);
    let active = matches!(
        app.state,
        RecordingState::Recording | RecordingState::Paused
    );
    button(
        app,
        c,
        Rect::new(x + 16., controls.y + 22., 225., 51.),
        if app.state == RecordingState::Idle {
            crate::i18n::tr("Nova gravação")
        } else if active {
            crate::i18n::tr("Parar e salvar")
        } else if app.state == RecordingState::Countdown {
            crate::i18n::tr("Cancelar contagem")
        } else {
            crate::i18n::tr("Processando…")
        },
        if active { "stop" } else { "record" },
        if app.state == RecordingState::Idle {
            Action::NewRecording
        } else if active || app.state == RecordingState::Countdown {
            Action::Stop
        } else {
            Action::None
        },
        if active {
            Style::Danger
        } else {
            Style::Primary
        },
    );
    if active {
        button(
            app,
            c,
            Rect::new(x + 253., controls.y + 22., 125., 51.),
            if app.state == RecordingState::Paused {
                crate::i18n::tr("Retomar")
            } else {
                crate::i18n::tr("Pausar")
            },
            "pause",
            Action::Pause,
            Style::Secondary,
        );
    }
    button(
        app,
        c,
        Rect::new(controls.right() - 169., controls.y + 22., 151., 51.),
        crate::i18n::tr("Capturar tela"),
        "resolution",
        Action::Screenshot,
        Style::Secondary,
    );
    label(
        c,
        &{
            let __wyzar_i18n_0 = &(app.settings.videos.display());
            crate::i18n::format_text(
                crate::i18n::tr("SALVAR EM  {}"),
                &[("0", format!("{}", __wyzar_i18n_0))],
            )
        },
        x + 3.,
        h - 58.,
        preview.w,
        10.,
        DIM,
        false,
    );
}
