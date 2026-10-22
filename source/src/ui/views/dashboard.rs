use super::*;

pub(super) fn preview_content(app: &mut App, c: &mut Canvas, inner: Rect, details: bool) {
    if app.settings.recording_mode == 3 {
        c.round(inner, 8., PANEL, Some(BORDER));
        c.icon(
            "microphone",
            Rect::new(inner.x + inner.w / 2. - 26., inner.y + 38., 52., 60.),
            GOLD,
        );
        c.center(
            crate::i18n::tr("Gravador de áudio"),
            Rect::new(inner.x + 20., inner.y + 115., inner.w - 40., 34.),
            24.,
            TEXT,
            true,
        );
        c.center(
            &{
                let __wyzar_i18n_0 = &(duration_label(app.seconds));
                crate::i18n::format_text(
                    crate::i18n::tr("WAV · 48 kHz · {}"),
                    &[("0", format!("{}", __wyzar_i18n_0))],
                )
            },
            Rect::new(inner.x + 20., inner.y + 160., inner.w - 40., 28.),
            15.,
            MUTED,
            false,
        );
        for (i, (name, peak, on)) in [
            (
                crate::i18n::tr("Microfone"),
                app.mic_level,
                app.settings.microphone,
            ),
            (
                crate::i18n::tr("Sistema"),
                app.system_level,
                app.settings.system_audio,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let y = inner.y + 211. + i as f32 * 47.;
            label(
                c,
                name,
                inner.x + 24.,
                y,
                99.,
                13.,
                if on { TEXT } else { MUTED },
                false,
            );
            c.round(
                Rect::new(inner.x + 128., y + 8., inner.w - 155., 7.),
                3.,
                BORDER,
                None,
            );
            if peak > 0. {
                c.round(
                    Rect::new(
                        inner.x + 128.,
                        y + 8.,
                        (inner.w - 155.) * peak.clamp(0., 1.),
                        7.,
                    ),
                    3.,
                    GOLD,
                    None,
                );
            }
        }
        return;
    }
    c.clip(inner);
    let actual = app.preview.lock().ok().and_then(|p| p.clone());
    if let Some(f) = &actual {
        c.fitted_frame(f, inner);
    } else {
        c.wallpaper(inner, 0.65);
        c.icon(
            "monitor",
            Rect::new(
                inner.x + inner.w / 2. - 25.,
                inner.y + inner.h / 2. - 44.,
                50.,
                50.,
            ),
            GOLD,
        );
        c.center(
            crate::i18n::tr("Sua captura aparece aqui"),
            Rect::new(
                inner.x + 20.,
                inner.y + inner.h / 2. + 23.,
                inner.w - 40.,
                32.,
            ),
            18.,
            TEXT,
            true,
        );
    }
    c.gradient(
        Rect::new(inner.x, inner.y, inner.w, 68.),
        BG.alpha(0.8),
        BG.alpha(0.),
        false,
        0.,
    );
    c.unclip();
    if !app.webcams.is_empty() && app.state == RecordingState::Idle {
        let webcam = app.webcam_frame.lock().ok().and_then(|f| f.clone());
        if let Some(f) = webcam {
            c.fitted_frame(
                &f,
                Rect::new(inner.right() - 202., inner.bottom() - 155., 190., 107.),
            );
        }
        button(
            app,
            c,
            Rect::new(inner.right() - 202., inner.bottom() - 43., 190., 32.),
            crate::i18n::tr("Ampliar webcam"),
            "webcam",
            Action::CameraView,
            Style::Secondary,
        );
    }
    let active = matches!(
        app.state,
        RecordingState::Recording | RecordingState::Paused
    );
    let status = if active {
        format!(
            "{}  {}",
            if app.state == RecordingState::Paused {
                crate::i18n::tr("PAUSADO")
            } else {
                crate::i18n::tr("● REC")
            },
            duration_label(app.seconds)
        )
    } else if app.state == RecordingState::Countdown {
        crate::i18n::tr("PREPARANDO").into()
    } else if app.preview_enabled {
        crate::i18n::tr("● AO VIVO").into()
    } else if actual.is_some() {
        crate::i18n::tr("ÚLTIMO QUADRO").into()
    } else {
        crate::i18n::tr("PRÉVIA").into()
    };
    chip(
        c,
        Rect::new(
            inner.x + 12.,
            inner.y + 12.,
            if active { 180. } else { 151. },
            29.,
        ),
        &status,
        if active {
            RED
        } else if app.preview_enabled {
            GREEN
        } else {
            MUTED
        },
    );
    if details {
        for (i, (icon, text)) in [
            ("resolution", resolution(&app.settings)),
            ("fps", {
                let __wyzar_i18n_0 = &(app.settings.fps);
                crate::i18n::format_text(
                    crate::i18n::tr("{} FPS"),
                    &[("0", format!("{}", __wyzar_i18n_0))],
                )
            }),
            ("encoder", crate::i18n::tr("H.264 · MP4").into()),
        ]
        .into_iter()
        .enumerate()
        {
            let r = Rect::new(
                inner.right() - 149.,
                inner.y + 12. + i as f32 * 37.,
                137.,
                29.,
            );
            c.round(r, 6., BG.alpha(0.88), Some(BORDER));
            c.icon(icon, Rect::new(r.x + 9., r.y + 7., 15., 15.), MUTED);
            label(c, &text, r.x + 31., r.y + 3., 100., 11., TEXT, false);
        }
    }
    if app.state == RecordingState::Idle {
        button(
            app,
            c,
            Rect::new(inner.x + 12., inner.bottom() - 43., 209., 31.),
            if app.preview_enabled {
                crate::i18n::tr("Desativar prévia ao vivo")
            } else {
                crate::i18n::tr("Ativar prévia ao vivo")
            },
            "eye",
            Action::Preview,
            Style::Secondary,
        );
    } else if app.state == RecordingState::Countdown {
        c.round(
            Rect::new(inner.x + 12., inner.bottom() - 51., inner.w - 24., 39.),
            7.,
            BG.alpha(0.9),
            Some(BORDER),
        );
        c.center(
            crate::i18n::tr("A contagem regressiva aparece no monitor da captura"),
            Rect::new(inner.x + 20., inner.bottom() - 48., inner.w - 40., 32.),
            12.,
            GOLD,
            false,
        );
    }
}

pub(super) fn home(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    let x = 240.;
    let right = w - 344.;
    let main_w = right - x - 20.;
    // Functional header: what will be recorded, not a slogan.
    let hero = Rect::new(x - 24., 65., main_w + 44., 140.);
    c.clip(hero);
    c.wallpaper(
        Rect::new(hero.x + hero.w * 0.42, hero.y, hero.w * 0.58, hero.h),
        0.9,
    );
    c.gradient(
        Rect::new(hero.x + hero.w * 0.42, hero.y, hero.w * 0.3, hero.h),
        BG,
        BG.alpha(0.),
        true,
        0.,
    );
    c.unclip();
    label(
        c,
        if app.settings.recording_mode == 3 {
            crate::i18n::tr("Gravador de áudio")
        } else {
            crate::i18n::tr("Nova gravação")
        },
        x + 4.,
        92.,
        main_w,
        PAGE_TITLE_FONT,
        TEXT,
        true,
    );
    let audio_only = app.settings.recording_mode == 3;
    let mut specs: Vec<(&str, String, bool)> = vec![];
    if !audio_only {
        specs.push(("monitor", app.settings.source.label().into(), true));
        specs.push(("resolution", resolution(&app.settings), true));
        specs.push((
            "fps",
            crate::i18n::format_text(
                crate::i18n::tr("{} FPS"),
                &[("0", app.settings.fps.to_string())],
            ),
            true,
        ));
    }
    specs.push((
        "microphone",
        crate::i18n::tr("Microfone").into(),
        app.settings.microphone,
    ));
    specs.push((
        "audio",
        crate::i18n::tr("Áudio do sistema").into(),
        app.settings.system_audio,
    ));
    if !audio_only {
        specs.push((
            "webcam",
            crate::i18n::tr("Webcam").into(),
            app.settings.webcam,
        ));
    }
    let mut cx = x + 4.;
    for (icon, text, on) in specs {
        let tw = c.text_width(&text, 12.5, false);
        let r = Rect::new(cx, 148., tw + 44., 32.);
        if r.right() > x + main_w * 0.72 {
            break;
        }
        c.round(
            r,
            16.,
            if on { RAISED } else { PANEL.alpha(0.6) },
            Some(if on { BORDER } else { BORDER.alpha(0.5) }),
        );
        c.icon(
            icon,
            Rect::new(r.x + 12., r.y + 8., 16., 16.),
            if on { GOLD } else { DIM },
        );
        c.text(
            &text,
            Rect::new(r.x + 34., r.y, tw + 6., r.h),
            12.5,
            if on { TEXT } else { DIM },
            false,
        );
        cx = r.right() + 8.;
    }
    let py = 214.;
    let ph = (h - 478.).max(300.);
    let sources_w = 290.;
    let preview = Rect::new(x, py, main_w - sources_w - 14., ph);
    panel(c, preview);
    let inner = preview.inset(12.);
    preview_content(app, c, inner, true);
    let sr = Rect::new(preview.right() + 14., py, sources_w, ph);
    panel(c, sr);
    label(
        c,
        if app.settings.recording_mode == 3 {
            crate::i18n::tr("Modo de gravação")
        } else {
            crate::i18n::tr("Fonte de captura")
        },
        sr.x + 18.,
        sr.y + 14.,
        sr.w - 36.,
        18.,
        TEXT,
        true,
    );
    let cw = (sr.w - 42.) / 2.;
    let ch = (ph - 88.) / 2.;
    for (i, s) in Source::ALL.into_iter().enumerate() {
        if app.settings.recording_mode == 3 {
            let r = Rect::new(
                sr.x + 14. + (i % 2) as f32 * (cw + 14.),
                sr.y + 57. + (i / 2) as f32 * (ch + 12.),
                cw,
                ch,
            );
            panel(c, r);
            if i == 3 {
                c.outline(r, 9., GOLD, 2.);
            }
            c.icon(
                ["record", "monitor", "audio", "microphone"][i],
                Rect::new(r.x + r.w / 2. - 17., r.y + 27., 34., 34.),
                if i == 3 { GOLD } else { TEXT },
            );
            c.center(
                [
                    crate::i18n::tr("Vídeo + áudio"),
                    crate::i18n::tr("Vídeo silencioso"),
                    crate::i18n::tr("Tela + WAV"),
                    crate::i18n::tr("Somente áudio"),
                ][i],
                Rect::new(r.x + 5., r.y + 78., r.w - 10., 27.),
                12.,
                TEXT,
                true,
            );
            app.hit(
                r,
                Action::SelectChoice(ChoiceKind::RecordingMode, i),
                crate::i18n::tr(crate::audio::MODE_LABELS[i]),
            );
            continue;
        }
        source_card(
            app,
            c,
            Rect::new(
                sr.x + 14. + (i % 2) as f32 * (cw + 14.),
                sr.y + 57. + (i / 2) as f32 * (ch + 12.),
                cw,
                ch,
            ),
            s,
            true,
        );
    }
    let devices_y = preview.bottom() + 14.;
    let dw = (main_w - 36.) / 4.;
    for (i, (name, icon, t, on, sub)) in [
        (
            crate::i18n::tr("Microfone"),
            "microphone",
            Toggle::Microphone,
            app.settings.microphone,
            app.device_name(Toggle::Microphone),
        ),
        (
            crate::i18n::tr("Áudio do sistema"),
            "audio",
            Toggle::SystemAudio,
            app.settings.system_audio,
            app.device_name(Toggle::SystemAudio),
        ),
        (
            crate::i18n::tr("Webcam"),
            "webcam",
            Toggle::Webcam,
            app.settings.webcam,
            app.device_name(Toggle::Webcam),
        ),
        (
            crate::i18n::tr("Cursor"),
            "cursor",
            Toggle::Cursor,
            app.settings.cursor,
            if app.settings.click_effects {
                crate::i18n::tr("Destacar cliques").into()
            } else {
                crate::i18n::tr("Mostrar na gravação").into()
            },
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let r = Rect::new(x + i as f32 * (dw + 12.), devices_y, dw, 76.);
        if i == 2 && app.webcams.is_empty() && app.settings.recording_mode != 3 {
            button(
                app,
                c,
                r,
                crate::i18n::tr("Capturar tela"),
                "resolution",
                Action::Screenshot,
                Style::Secondary,
            );
            continue;
        }
        if app.settings.recording_mode == 3 && i >= 2 {
            panel(c, r);
            c.icon("sliders", Rect::new(r.x + 13., r.y + 24., 25., 25.), TEXT);
            label(
                c,
                if i == 2 {
                    crate::i18n::tr("Ganho do microfone")
                } else {
                    crate::i18n::tr("Ganho do sistema")
                },
                r.x + 49.,
                r.y + 9.,
                r.w - 57.,
                12.,
                TEXT,
                true,
            );
            dropdown(
                app,
                c,
                Rect::new(r.x + 49., r.y + 37., r.w - 62., 28.),
                &format!(
                    "{}%",
                    if i == 2 {
                        app.settings.mic_gain
                    } else {
                        app.settings.system_gain
                    }
                ),
                if i == 2 {
                    ChoiceKind::MicGain
                } else {
                    ChoiceKind::SystemGain
                },
            );
            continue;
        }
        panel(c, r);
        c.icon(icon, Rect::new(r.x + 14., r.y + 23., 28., 28.), TEXT);
        label(c, name, r.x + 54., r.y + 12., r.w - 104., 12., TEXT, true);
        label(c, &sub, r.x + 54., r.y + 39., r.w - 63., 11., MUTED, false);
        if i < 2 {
            let peak = if i == 0 {
                app.mic_level
            } else {
                app.system_level
            };
            for bar in 0..18 {
                let xx = r.x + 54. + bar as f32 * (r.w - 67.) / 18.;
                c.round(
                    Rect::new(xx, r.bottom() - 9., (r.w - 67.) / 18. - 2., 3.),
                    1.,
                    if on && (bar as f32 / 18.) < peak {
                        GOLD
                    } else {
                        BORDER
                    },
                    None,
                );
            }
        }
        toggle(
            app,
            c,
            Rect::new(r.right() - 48., r.y + 17., 34., 19.),
            on,
            t,
        );
    }
    let controls_y = devices_y + 104.;
    c.line(
        x,
        controls_y - 14.,
        x + main_w,
        controls_y - 14.,
        BORDER,
        1.,
    );
    let quality = Rect::new(x, controls_y + 7., 295., 78.);
    panel(c, quality);
    c.icon(
        "settings",
        Rect::new(x + 18., controls_y + 32., 25., 25.),
        TEXT,
    );
    label(
        c,
        crate::i18n::tr("Qualidade de gravação"),
        x + 57.,
        controls_y + 21.,
        225.,
        14.,
        TEXT,
        true,
    );
    label(
        c,
        &if app.settings.recording_mode == 3 {
            crate::i18n::tr("WAV · 48 kHz · PCM 16 bits").into()
        } else {
            {
                let __wyzar_i18n_0 = &(resolution(&app.settings));
                let __wyzar_i18n_1 = &(app.settings.fps);
                crate::i18n::format_text(
                    crate::i18n::tr("{} · {} FPS · H.264"),
                    &[
                        ("0", format!("{}", __wyzar_i18n_0)),
                        ("1", format!("{}", __wyzar_i18n_1)),
                    ],
                )
            }
        },
        x + 57.,
        controls_y + 47.,
        220.,
        11.,
        MUTED,
        false,
    );
    app.hit(
        quality,
        Action::OpenSettings(if app.settings.recording_mode == 3 {
            2
        } else {
            3
        }),
        crate::i18n::tr("Qualidade de gravação"),
    );
    let rcx = x + main_w * 0.57;
    let rcy = controls_y + 45.;
    let record_rect = Rect::new(rcx - 52., rcy - 52., 104., 104.);
    let hover = app
        .motion
        .hover_amount(record_rect, app.hover == Some(app.hits.len()));
    let live = app.state == RecordingState::Recording;
    // A slow pulse while recording; a soft halo on hover while idle.
    if live && app.motion.enabled {
        let t = (app.animation_started.elapsed().as_secs_f32() * 0.9).fract();
        c.ring(rcx, rcy, 48. + t * 14., RED.alpha(0.35 * (1. - t)), 2.);
    }
    c.glow(rcx, rcy, 64. + hover * 6., RED.alpha(0.10 + hover * 0.10));
    c.ring(rcx, rcy, 47., RED, 3.);
    c.circle(rcx, rcy, 32. + hover * 3., RED);
    if app.state == RecordingState::Recording || app.state == RecordingState::Paused {
        c.round(Rect::new(rcx - 12., rcy - 12., 24., 24.), 5., TEXT, None);
    }
    app.hit(
        Rect::new(rcx - 52., rcy - 52., 104., 104.),
        if app.state == RecordingState::Idle {
            Action::NewRecording
        } else {
            Action::Stop
        },
        crate::i18n::tr("Iniciar ou parar gravação"),
    );
    let pause = Rect::new(rcx - 142., controls_y + 9., 73., 77.);
    let stop = Rect::new(rcx + 69., controls_y + 9., 73., 77.);
    for (r, title, icon, key, a, enabled) in [
        (
            pause,
            if app.state == RecordingState::Paused {
                crate::i18n::tr("Retomar")
            } else {
                crate::i18n::tr("Pausar")
            },
            "pause",
            app.settings.pause_hotkey,
            Action::Pause,
            app.session.is_some(),
        ),
        (
            stop,
            crate::i18n::tr("Parar"),
            "stop",
            app.settings.stop_hotkey,
            Action::Stop,
            app.state != RecordingState::Idle,
        ),
    ] {
        panel(c, r);
        c.icon(
            icon,
            Rect::new(r.x + 25., r.y + 13., 23., 23.),
            if enabled { TEXT } else { DIM },
        );
        c.center(
            title,
            Rect::new(r.x, r.y + 40., r.w, 18.),
            11.,
            if enabled { TEXT } else { DIM },
            false,
        );
        c.center(
            &crate::settings::shortcuts::label(key),
            Rect::new(r.x, r.y + 58., r.w, 15.),
            10.,
            DIM,
            false,
        );
        if enabled {
            app.hit(r, a, title);
        }
    }
    let quick = Rect::new(x + main_w - 202., controls_y + 7., 202., 78.);
    panel(c, quick);
    label(
        c,
        crate::i18n::tr("Ações rápidas"),
        quick.x + 14.,
        quick.y + 8.,
        170.,
        12.,
        TEXT,
        true,
    );
    button(
        app,
        c,
        Rect::new(quick.x + 8., quick.y + 34., 88., 32.),
        crate::i18n::tr("Pasta"),
        "folder",
        Action::OpenVideos,
        Style::Ghost,
    );
    button(
        app,
        c,
        Rect::new(quick.x + 102., quick.y + 34., 92., 32.),
        crate::i18n::tr("Print"),
        "resolution",
        Action::Screenshot,
        Style::Ghost,
    );
    c.line(right - 1., 64., right - 1., h - 36., BORDER, 1.);
    label(
        c,
        crate::i18n::tr("Gravações recentes"),
        right + 20.,
        91.,
        245.,
        20.,
        TEXT,
        true,
    );
    button(
        app,
        c,
        Rect::new(w - 96., 90., 75., 30.),
        crate::i18n::tr("Ver tudo"),
        "",
        Action::Page(Page::Library),
        Style::Ghost,
    );
    if app.library.items.is_empty() {
        let r = Rect::new(right + 20., 147., 304., 330.);
        panel(c, r);
        c.icon("library", Rect::new(r.x + 128., r.y + 44., 48., 48.), GOLD);
        c.center(
            crate::i18n::tr("Nenhuma gravação ainda"),
            Rect::new(r.x + 15., r.y + 117., r.w - 30., 28.),
            15.,
            TEXT,
            true,
        );
        c.center(
            crate::i18n::tr("As gravações aparecerão na biblioteca"),
            Rect::new(r.x + 12., r.y + 158., r.w - 24., 24.),
            12.,
            MUTED,
            false,
        );
        c.center(
            crate::i18n::tr("assim que você salvar seu primeiro vídeo."),
            Rect::new(r.x + 12., r.y + 181., r.w - 24., 24.),
            12.,
            MUTED,
            false,
        );
        button(
            app,
            c,
            Rect::new(r.x + 31., r.y + 244., r.w - 62., 43.),
            crate::i18n::tr("Nova gravação"),
            "plus",
            Action::NewRecording,
            Style::Primary,
        );
    } else {
        let recent: Vec<_> = app.library.items.iter().take(4).cloned().collect();
        for (i, r) in recent.iter().enumerate() {
            let y = 148. + i as f32 * 109.;
            let thumb = Rect::new(right + 20., y, 112., 67.);
            c.round(thumb, 6., RAISED, Some(BORDER));
            c.clip(thumb.inset(2.));
            if let Some(f) = &r.thumbnail {
                c.frame(f, thumb.inset(2.));
            } else {
                c.wallpaper(thumb.inset(2.), 1.);
            }
            c.unclip();
            c.text(
                &r.title,
                Rect::new(right + 145., y + 1., 175., 28.),
                13.,
                TEXT,
                true,
            );
            label(
                c,
                &duration_label(r.duration),
                right + 145.,
                y + 32.,
                160.,
                11.,
                MUTED,
                false,
            );
            label(
                c,
                &size_label(r.bytes),
                right + 145.,
                y + 53.,
                160.,
                11.,
                MUTED,
                false,
            );
            app.hit(
                Rect::new(right + 20., y, 304., 85.),
                Action::Select(i),
                &r.title,
            );
            c.line(right + 20., y + 96., w - 20., y + 96., BORDER, 1.);
        }
    }
    let storage = Rect::new(right + 20., h - 284., 304., 122.);
    panel(c, storage);
    label(
        c,
        crate::i18n::tr("Armazenamento"),
        storage.x + 16.,
        storage.y + 13.,
        225.,
        16.,
        TEXT,
        true,
    );
    let destination = if app.settings.recording_mode == 3 {
        &app.settings.audios
    } else {
        &app.settings.videos
    };
    if let Some((free, total)) = files::free_space(destination) {
        let used = 1. - free as f32 / total.max(1) as f32;
        label(
            c,
            &{
                let __wyzar_i18n_0 = &(size_label(free));
                let __wyzar_i18n_1 = &(size_label(total));
                crate::i18n::format_text(
                    crate::i18n::tr("{} livres de {}"),
                    &[
                        ("0", format!("{}", __wyzar_i18n_0)),
                        ("1", format!("{}", __wyzar_i18n_1)),
                    ],
                )
            },
            storage.x + 16.,
            storage.y + 42.,
            275.,
            11.,
            MUTED,
            false,
        );
        progress(
            c,
            Rect::new(storage.x + 16., storage.y + 75., 272., 10.),
            used,
            GOLD,
        );
        label(
            c,
            &{
                let __wyzar_i18n_0 = &(used * 100.);
                crate::i18n::format_text(
                    crate::i18n::tr("{:.0}% utilizado"),
                    &[("0:.0", format!("{:.0}", __wyzar_i18n_0))],
                )
            },
            storage.x + 16.,
            storage.y + 92.,
            272.,
            11.,
            MUTED,
            false,
        );
    } else {
        label(
            c,
            crate::i18n::tr("Escolha sua pasta de vídeos"),
            storage.x + 16.,
            storage.y + 48.,
            272.,
            12.,
            MUTED,
            false,
        );
    }
    label(
        c,
        crate::i18n::tr("SALVAR EM"),
        right + 22.,
        h - 145.,
        280.,
        10.,
        DIM,
        true,
    );
    c.text(
        &destination.to_string_lossy(),
        Rect::new(right + 22., h - 118., 278., 27.),
        11.,
        MUTED,
        false,
    );
    button(
        app,
        c,
        Rect::new(right + 20., h - 82., 304., 32.),
        crate::i18n::tr("Alterar pasta de destino"),
        "folder",
        Action::ChooseFolder(if app.settings.recording_mode == 3 {
            5
        } else {
            0
        }),
        Style::Secondary,
    );
}
