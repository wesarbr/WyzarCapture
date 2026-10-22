//! Floating recorder. Uses the same settings, actions and session as the full interface.
use super::*;

pub(super) fn render(app: &mut App, c: &mut Canvas, w: f32, h: f32, mode: InterfaceMode) {
    c.round(Rect::new(1., 1., w - 2., h - 2.), 14., BG, Some(BORDER));
    if app.settings.appearance.animated {
        c.clip(Rect::new(2., 57., w - 4., h - 59.));
        c.wallpaper(Rect::new(w * 0.48, 56., w * 0.52, h - 56.), 1.);
        c.fill(Rect::new(2., 57., w - 4., h - 59.), BG.alpha(0.88));
        c.unclip();
    }
    header(app, c, w, mode);
    if mode == InterfaceMode::CompactFocus {
        focused(app, c, w, h);
    } else {
        full(app, c, w, h);
    }
}
fn header(app: &mut App, c: &mut Canvas, w: f32, mode: InterfaceMode) {
    c.fill(Rect::new(2., 2., w - 4., 53.), SIDEBAR);
    c.line(1., 56., w - 1., 56., BORDER, 1.);
    c.logo(Rect::new(20., 13., 31., 31.));
    label(c, "Wyzar", 62., 13., 66., 21., GOLD, true);
    label(c, "Capture", 125., 13., 92., 21., TEXT, true);
    c.line(231., 17., 231., 40., BORDER, 1.);
    dropdown(
        app,
        c,
        Rect::new(246., 12., 214., 33.),
        mode.label(),
        ChoiceKind::Interface,
    );
    for (offset, icon, tip, action, selected) in [
        (
            236.,
            "expand",
            crate::i18n::tr("Interface padrão · Ctrl+Alt+1"),
            Action::SetInterface(InterfaceMode::Standard),
            false,
        ),
        (
            192.,
            "compact",
            if mode == InterfaceMode::CompactFocus {
                crate::i18n::tr("Expandir compacto · Ctrl+Alt+3")
            } else {
                crate::i18n::tr("Compacto com foco · Ctrl+Alt+4")
            },
            Action::SetInterface(if mode == InterfaceMode::CompactFocus {
                InterfaceMode::Compact
            } else {
                InterfaceMode::CompactFocus
            }),
            mode == InterfaceMode::CompactFocus,
        ),
        (
            148.,
            "pin",
            crate::i18n::tr("Manter sobre as outras janelas"),
            Action::Pin,
            app.settings.appearance.pinned,
        ),
        (
            104.,
            "minimize",
            crate::i18n::tr("Minimizar"),
            Action::Minimize,
            false,
        ),
        (
            60.,
            "close",
            crate::i18n::tr("Fechar"),
            Action::Close,
            false,
        ),
    ] {
        let r = Rect::new(w - offset, 9., 39., 39.);
        let hover = app.hover == Some(app.hits.len());
        if hover || selected {
            c.round(
                r,
                7.,
                if hover && icon == "close" {
                    RED
                } else if selected {
                    GOLD_DARK
                } else {
                    HOVER
                },
                None,
            );
        }
        c.icon(
            icon,
            Rect::new(r.x + 10., r.y + 10., 19., 19.),
            if hover && icon == "close" {
                Color::literal(0xffffff)
            } else if selected {
                GOLD
            } else {
                TEXT
            },
        );
        app.hit(r, action, tip);
    }
}
fn full(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    let card_y = 76.;
    let card_h = (h - 195.).max(213.);
    let col = (w - 256.) / 5.;
    for (i, title) in [
        crate::i18n::tr("Área"),
        crate::i18n::tr("Sons"),
        if app.webcams.is_empty() {
            "PrintScreen"
        } else {
            crate::i18n::tr("Dispositivo")
        },
        crate::i18n::tr("Qualidade"),
        crate::i18n::tr("FPS"),
    ]
    .into_iter()
    .enumerate()
    {
        let r = Rect::new(20. + i as f32 * (col + 12.), card_y, col, card_h);
        panel(c, r);
        c.center(title, Rect::new(r.x, r.y + 11., r.w, 29.), 17., TEXT, true);
        match i {
            0 => {
                let live = app.preview.lock().ok().and_then(|f| f.clone());
                if let Some(f) = live {
                    c.fitted_frame(&f, Rect::new(r.x + 10., r.y + 51., col - 20., 83.));
                } else {
                    c.icon(
                        app.settings
                            .source
                            .key()
                            .replace("display", "monitor")
                            .as_str(),
                        Rect::new(r.x + (col - 57.) / 2., r.y + 65., 57., 57.),
                        GOLD,
                    );
                }
                dropdown(
                    app,
                    c,
                    Rect::new(r.x + 12., r.bottom() - 51., col - 24., 36.),
                    app.settings.source.label(),
                    ChoiceKind::CaptureSource,
                );
            }
            1 => {
                c.icon(
                    "microphone",
                    Rect::new(r.x + 25., r.y + 55., 42., 55.),
                    if app.settings.microphone { GOLD } else { MUTED },
                );
                c.icon(
                    "audio",
                    Rect::new(r.right() - 70., r.y + 61., 43., 43.),
                    if app.settings.system_audio {
                        GOLD
                    } else {
                        MUTED
                    },
                );
                for (row, text, toggle_kind, on, peak) in [
                    (
                        0.,
                        crate::i18n::tr("Microfone"),
                        Toggle::Microphone,
                        app.settings.microphone,
                        app.mic_level,
                    ),
                    (
                        1.,
                        crate::i18n::tr("Sistema"),
                        Toggle::SystemAudio,
                        app.settings.system_audio,
                        app.system_level,
                    ),
                ] {
                    let yy = r.bottom() - 79. + row * 34.;
                    label(c, text, r.x + 15., yy + 1., col - 60., 13., TEXT, false);
                    toggle(
                        app,
                        c,
                        Rect::new(r.right() - 50., yy + 3., 34., 20.),
                        on,
                        toggle_kind,
                    );
                    if on && peak > 0. {
                        progress(c, Rect::new(r.x + 15., yy + 26., col - 30., 2.), peak, GOLD);
                    }
                }
            }
            2 => {
                if app.webcams.is_empty() {
                    c.icon(
                        "resolution",
                        Rect::new(r.x + (col - 48.) / 2., r.y + 54., 48., 48.),
                        GOLD,
                    );
                    button(
                        app,
                        c,
                        Rect::new(r.x + 10., r.y + 115., col - 20., 37.),
                        crate::i18n::tr("Capturar agora"),
                        "",
                        Action::TakeScreenshot,
                        Style::Primary,
                    );
                    button(
                        app,
                        c,
                        Rect::new(r.x + 10., r.bottom() - 49., col - 20., 34.),
                        crate::i18n::tr("Personalizar captura"),
                        "",
                        Action::Screenshot,
                        Style::Secondary,
                    );
                    continue;
                }
                let live = app.webcam_frame.lock().ok().and_then(|f| f.clone());
                if let Some(f) = live {
                    let rr = Rect::new(r.x + 12., r.y + 57., col - 24., 82.);
                    c.fitted_frame(&f, rr);
                    app.hit(rr, Action::CameraView, crate::i18n::tr("Ampliar webcam"));
                } else {
                    c.icon(
                        "webcam",
                        Rect::new(r.x + (col - 55.) / 2., r.y + 62., 55., 55.),
                        if app.settings.webcam { GOLD } else { MUTED },
                    );
                    app.hit(
                        Rect::new(r.x + 12., r.y + 59., col - 24., 78.),
                        Action::CameraView,
                        crate::i18n::tr("Ampliar webcam"),
                    );
                }
                toggle(
                    app,
                    c,
                    Rect::new(r.right() - 43., r.y + 42., 29., 17.),
                    app.settings.webcam,
                    Toggle::Webcam,
                );
                let name = app
                    .webcams
                    .iter()
                    .find(|d| d.id == app.settings.webcam_id)
                    .map(|d| d.name.clone())
                    .unwrap_or_else(|| {
                        if app.webcams.is_empty() {
                            crate::i18n::tr("Sem câmera").into()
                        } else {
                            crate::i18n::tr("Escolher câmera").into()
                        }
                    });
                dropdown(
                    app,
                    c,
                    Rect::new(r.x + 12., r.bottom() - 51., col - 24., 36.),
                    &name,
                    ChoiceKind::Webcam,
                );
            }
            3 => {
                let badge = Rect::new(r.x + (col - 70.) / 2., r.y + 69., 70., 48.);
                c.outline(badge, 8., GOLD, 3.);
                c.center(
                    if app.settings.height >= 2160 {
                        "4K"
                    } else if app.settings.height >= 1440 {
                        crate::i18n::tr("QHD")
                    } else {
                        crate::i18n::tr("HD")
                    },
                    badge,
                    26.,
                    GOLD,
                    true,
                );
                dropdown(
                    app,
                    c,
                    Rect::new(r.x + 12., r.bottom() - 51., col - 24., 36.),
                    &format!("{}p", app.settings.height),
                    ChoiceKind::Resolution,
                );
            }
            _ => {
                for (j, height) in [18., 34., 54., 34., 18.].into_iter().enumerate() {
                    c.round(
                        Rect::new(
                            r.x + (col - 58.) / 2. + j as f32 * 13.,
                            r.y + 93. - height / 2.,
                            5.,
                            height,
                        ),
                        2.5,
                        if j == 0 || j == 4 { DIM } else { GOLD },
                        None,
                    );
                }
                dropdown(
                    app,
                    c,
                    Rect::new(r.x + 12., r.bottom() - 51., col - 24., 36.),
                    &{
                        let __wyzar_i18n_0 = &(app.settings.fps);
                        crate::i18n::format_text(
                            crate::i18n::tr("{} FPS"),
                            &[("0", format!("{}", __wyzar_i18n_0))],
                        )
                    },
                    ChoiceKind::Fps,
                );
            }
        }
    }
    let divider = w - 187.;
    c.line(
        divider,
        card_y + 4.,
        divider,
        card_y + card_h - 4.,
        BORDER,
        1.,
    );
    recorder(
        app,
        c,
        Rect::new(divider + 16., card_y + 30., 151., 151.),
        false,
    );
    footer(app, c, Rect::new(20., h - 100., w - 40., 63.), false);
    status(app, c, w, h);
}
fn recorder(app: &mut App, c: &mut Canvas, r: Rect, small: bool) {
    let active = matches!(
        app.state,
        RecordingState::Recording | RecordingState::Paused
    );
    let ready = app.state == RecordingState::Idle;
    let countdown = app.state == RecordingState::Countdown;
    let tone = if ready || active || countdown {
        RED
    } else {
        DIM
    };
    let cx = r.x + r.w / 2.;
    let cy = r.y + r.h / 2.;
    c.circle(cx, cy, r.w / 2., tone.alpha(0.08));
    c.outline(r.inset(6.), r.w / 2., tone, 1.5);
    c.gradient(
        r.inset(13.),
        tone,
        if ready || active || countdown {
            Color::rgb(0xcd1126)
        } else {
            DIM
        },
        false,
        r.w / 2.,
    );
    let text = if ready {
        crate::i18n::tr("Gravar")
    } else if active {
        crate::i18n::tr("Parar")
    } else if countdown {
        crate::i18n::tr("Cancelar")
    } else if app.state == RecordingState::Starting {
        crate::i18n::tr("Preparando")
    } else {
        crate::i18n::tr("Salvando")
    };
    c.center(
        text,
        r,
        if small && text.len() > 8 {
            12.
        } else if small {
            16.
        } else {
            22.
        },
        Color::literal(0xffffff),
        true,
    );
    let action = if ready {
        Action::StartRecording
    } else if active || countdown {
        Action::Stop
    } else {
        Action::None
    };
    if !matches!(action, Action::None) {
        if app.hover == Some(app.hits.len()) {
            c.outline(r.inset(2.), r.w / 2., tone.alpha(0.8), 2.);
        }
        app.hit(
            r,
            action,
            if ready {
                crate::i18n::tr("Iniciar com as configurações atuais")
            } else if active {
                crate::i18n::tr("Parar e salvar")
            } else {
                crate::i18n::tr("Cancelar contagem")
            },
        );
    }
    if active {
        button(
            app,
            c,
            Rect::new(
                if small { r.x - 7. } else { r.x + 11. },
                r.bottom() + 6.,
                if small { 108. } else { r.w - 22. },
                30.,
            ),
            if app.state == RecordingState::Paused {
                crate::i18n::tr("Retomar")
            } else {
                crate::i18n::tr("Pausar")
            },
            if app.state == RecordingState::Paused {
                "play"
            } else {
                "pause"
            },
            Action::Pause,
            Style::Secondary,
        );
    } else if !small {
        let hint = if ready {
            {
                let __wyzar_i18n_0 = crate::settings::shortcuts::label(app.settings.record_hotkey);
                crate::i18n::format_text(
                    crate::i18n::tr("{} · iniciar"),
                    &[("0", format!("{}", __wyzar_i18n_0))],
                )
            }
        } else if countdown {
            crate::i18n::tr("Esc · cancelar").into()
        } else if app.state == RecordingState::Starting {
            crate::i18n::tr("Iniciando captura").into()
        } else {
            crate::i18n::tr("Aguarde a conclusão").into()
        };
        c.center(
            &hint,
            Rect::new(r.x, r.bottom() + 5., r.w, 29.),
            11.,
            MUTED,
            false,
        );
    }
}
fn target_control(app: &mut App, c: &mut Canvas, r: Rect) {
    let (text, kind, icon) = match app.settings.source {
        Source::Display => (
            app.monitors
                .get(app.settings.monitor)
                .map(|m| m.name.clone())
                .unwrap_or_else(|| crate::i18n::tr("Escolher monitor").into()),
            ChoiceKind::Monitor,
            "monitor",
        ),
        Source::Region => (
            app.region
                .map(|r| {
                    let __wyzar_i18n_0 = &(r.width);
                    let __wyzar_i18n_1 = &(r.height);
                    crate::i18n::format_text(
                        crate::i18n::tr("Região · {} × {}"),
                        &[
                            ("0", format!("{}", __wyzar_i18n_0)),
                            ("1", format!("{}", __wyzar_i18n_1)),
                        ],
                    )
                })
                .unwrap_or_else(|| crate::i18n::tr("Selecionar região").into()),
            ChoiceKind::Window,
            "region",
        ),
        _ => (
            if app.window.is_some() {
                app.window_name.clone()
            } else {
                crate::i18n::tr("Selecionar janela").into()
            },
            ChoiceKind::Window,
            "window",
        ),
    };
    c.icon(icon, Rect::new(r.x + 10., r.y + 10., 22., 22.), TEXT);
    dropdown(
        app,
        c,
        Rect::new(r.x + 42., r.y, r.w - 42., r.h),
        &text,
        kind,
    );
}
fn footer(app: &mut App, c: &mut Canvas, r: Rect, simple: bool) {
    panel(c, r);
    target_control(
        app,
        c,
        Rect::new(r.x + 9., r.y + 10., if simple { 243. } else { 272. }, 42.),
    );
    let start = r.x + if simple { 268. } else { 297. };
    for (i, (icon, action, on, tip)) in [
        (
            "microphone",
            Action::Toggle(Toggle::Microphone),
            app.settings.microphone,
            crate::i18n::tr("Ativar/desativar microfone"),
        ),
        (
            "audio",
            Action::Toggle(Toggle::SystemAudio),
            app.settings.system_audio,
            crate::i18n::tr("Ativar/desativar áudio do sistema"),
        ),
        (
            "webcam",
            Action::Toggle(Toggle::Webcam),
            app.settings.webcam,
            crate::i18n::tr("Ativar/desativar webcam"),
        ),
        (
            "cursor",
            Action::Toggle(Toggle::Cursor),
            app.settings.cursor,
            crate::i18n::tr("Mostrar/ocultar cursor"),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        if icon == "webcam" && app.webcams.is_empty() {
            continue;
        }
        let rr = Rect::new(
            start + i as f32 * if simple { 43. } else { 49. },
            r.y + 10.,
            if simple { 36. } else { 41. },
            42.,
        );
        button(
            app,
            c,
            rr,
            "",
            icon,
            action,
            if on {
                Style::Selected
            } else {
                Style::Secondary
            },
        );
        app.hits.last_mut().map(|hit| hit.label = tip.into());
    }
    let mut extras = vec![
        (
            "settings",
            Action::NewRecording,
            crate::i18n::tr("Ajustes de gravação"),
        ),
        (
            "resolution",
            Action::Screenshot,
            crate::i18n::tr("Capturar tela em PNG"),
        ),
    ];
    if simple {
        extras.push((
            "folder",
            Action::OpenFolders,
            crate::i18n::tr("Suas pastas"),
        ));
    }
    if !simple {
        extras.extend([
            (
                "folder",
                Action::OpenFolders,
                crate::i18n::tr("Suas pastas"),
            ),
            (
                "palette",
                Action::OpenChoice(ChoiceKind::Theme, Rect::new(r.right() - 96., r.y, 41., 42.)),
                crate::i18n::tr("Escolher tema"),
            ),
            (
                "help",
                Action::HelpTopic(12),
                crate::i18n::tr("Ajuda do modo compacto"),
            ),
        ]);
    }
    for (i, (icon, action, tip)) in extras.iter().enumerate() {
        let rr = Rect::new(
            r.right() - 10. - (extras.len() - i) as f32 * 49.,
            r.y + 10.,
            41.,
            42.,
        );
        button(app, c, rr, "", icon, action.clone(), Style::Ghost);
        app.hits.last_mut().map(|hit| hit.label = (*tip).into());
    }
    if !simple {
        let folder = app.settings.videos.display().to_string();
        label(
            c,
            &folder,
            start + 204.,
            r.y + 20.,
            (r.w - 770.).max(40.),
            10.,
            MUTED,
            false,
        );
    }
}
fn status(app: &App, c: &mut Canvas, w: f32, h: f32) {
    let (text, color) = match app.state {
        RecordingState::Recording => (
            {
                let __wyzar_i18n_0 = &(crate::core::timecode(app.seconds));
                crate::i18n::format_text(
                    crate::i18n::tr("Gravando · {}"),
                    &[("0", format!("{}", __wyzar_i18n_0))],
                )
            },
            RED,
        ),
        RecordingState::Paused => (
            {
                let __wyzar_i18n_0 = &(crate::core::timecode(app.seconds));
                crate::i18n::format_text(
                    crate::i18n::tr("Pausado · {}"),
                    &[("0", format!("{}", __wyzar_i18n_0))],
                )
            },
            GOLD,
        ),
        RecordingState::Countdown => (
            crate::i18n::tr("Contagem no monitor · Esc cancela").into(),
            GOLD,
        ),
        RecordingState::Starting => (crate::i18n::tr("Preparando a captura…").into(), GOLD),
        RecordingState::Finalizing => (crate::i18n::tr("Finalizando e salvando…").into(), GOLD),
        _ => (crate::i18n::tr("Pronto para gravar").into(), GREEN),
    };
    c.circle(28., h - 18., 3., color);
    label(c, &text, 39., h - 26., w * 0.6, 11., color, false);
    c.text(
        &if app.settings.recording_mode == 3 {
            crate::i18n::tr("WAV · 48 kHz · estéreo").into()
        } else {
            {
                let __wyzar_i18n_0 = &(app.settings.height);
                let __wyzar_i18n_1 = &(app.settings.fps);
                crate::i18n::format_text(
                    crate::i18n::tr("{}p · {} FPS · MP4"),
                    &[
                        ("0", format!("{}", __wyzar_i18n_0)),
                        ("1", format!("{}", __wyzar_i18n_1)),
                    ],
                )
            }
        },
        Rect::new(w - 217., h - 29., 196., 23.),
        11.,
        MUTED,
        false,
    );
}
fn focused(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    let y = 72.;
    let row = Rect::new(20., y, w - 166., 52.);
    dropdown(
        app,
        c,
        Rect::new(row.x, row.y + 8., 155., 36.),
        app.settings.source.label(),
        ChoiceKind::CaptureSource,
    );
    dropdown(
        app,
        c,
        Rect::new(row.x + 166., row.y + 8., 143., 36.),
        &format!("{}p", app.settings.height),
        ChoiceKind::Resolution,
    );
    dropdown(
        app,
        c,
        Rect::new(row.x + 320., row.y + 8., 132., 36.),
        &{
            let __wyzar_i18n_0 = &(app.settings.fps);
            crate::i18n::format_text(
                crate::i18n::tr("{} FPS"),
                &[("0", format!("{}", __wyzar_i18n_0))],
            )
        },
        ChoiceKind::Fps,
    );
    recorder(app, c, Rect::new(w - 118., y - 2., 94., 94.), true);
    footer(app, c, Rect::new(20., h - 100., w - 155., 63.), true);
    status(app, c, w, h);
}
