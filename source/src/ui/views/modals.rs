use super::*;
pub(super) fn existing_data(
    app: &mut App,
    c: &mut Canvas,
    w: f32,
    h: f32,
    data: &crate::persistence::migration::Import,
) {
    let r = modal_panel(c, w, h, 800., 415.);
    label(
        c,
        crate::i18n::tr("Dados existentes encontrados"),
        r.x + 28.,
        r.y + 24.,
        r.w - 56.,
        25.,
        TEXT,
        true,
    );
    c.paragraph(&{ let __wyzar_i18n_0 = &(data.source.display()); let __wyzar_i18n_1 = &(data.files.len()); crate::i18n::format_text(crate::i18n::tr("{}\n\n{} arquivo(s) reconhecido(s): preferências, perfis e índice. Um backup dos arquivos atuais será criado antes da importação. Vídeos e projetos continuam nos locais originais."), &[("0", format!("{}", __wyzar_i18n_0)), ("1", format!("{}", __wyzar_i18n_1))]) },Rect::new(r.x+28.,r.y+84.,r.w-56.,218.),16.,MUTED,false);
    button(
        app,
        c,
        Rect::new(r.x + 28., r.bottom() - 66., 178., 40.),
        crate::i18n::tr("Ver pasta"),
        "folder",
        Action::ViewExistingData,
        Style::Secondary,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 328., r.bottom() - 66., 140., 40.),
        crate::i18n::tr("Ignorar"),
        "",
        Action::CloseModal,
        Style::Secondary,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 172., r.bottom() - 66., 144., 40.),
        crate::i18n::tr("Importar"),
        "import",
        Action::ConfirmImportData,
        Style::Primary,
    );
}
pub(super) fn cleanup(
    app: &mut App,
    c: &mut Canvas,
    w: f32,
    h: f32,
    plan: &crate::persistence::maintenance::Plan,
) {
    let r = modal_panel(c, w, h, 700., 360.);
    label(
        c,
        crate::i18n::tr("Confirmar limpeza"),
        r.x + 28.,
        r.y + 25.,
        r.w - 56.,
        25.,
        TEXT,
        true,
    );
    c.paragraph(&{ let __wyzar_i18n_0 = &(plan.label); let __wyzar_i18n_1 = &(crate::core::size_label(plan.bytes)); let __wyzar_i18n_2 = &(plan.files.len()); crate::i18n::format_text(crate::i18n::tr("{}\n{} em {} arquivo(s).\n\nSomente arquivos temporários reconhecidos são selecionados. Vídeos, áudio, capturas, biblioteca e projetos serão preservados."), &[("0", format!("{}", __wyzar_i18n_0)), ("1", format!("{}", __wyzar_i18n_1)), ("2", format!("{}", __wyzar_i18n_2))]) },Rect::new(r.x+28.,r.y+82.,r.w-56.,175.),16.,MUTED,false);
    button(
        app,
        c,
        Rect::new(r.right() - 328., r.bottom() - 66., 140., 40.),
        crate::i18n::tr("Cancelar"),
        "",
        Action::CloseModal,
        Style::Secondary,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 172., r.bottom() - 66., 144., 40.),
        crate::i18n::tr("Limpar"),
        "trash",
        Action::ConfirmCleanup,
        Style::Primary,
    );
}

pub(super) fn audio_tracks(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    let r = modal_panel(c, w, h, 1060., 690.);
    label(
        c,
        crate::i18n::tr("Trilhas de áudio"),
        r.x + 28.,
        r.y + 22.,
        600.,
        27.,
        TEXT,
        true,
    );
    label(
        c,
        crate::i18n::tr("Microfone e sistema independentes · edição não destrutiva"),
        r.x + 28.,
        r.y + 65.,
        850.,
        14.,
        MUTED,
        false,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 55., r.y + 19., 36., 36.),
        "",
        "close",
        Action::CloseModal,
        Style::Ghost,
    );
    button(
        app,
        c,
        Rect::new(r.x + 28., r.y + 105., 245., 35.),
        if app.project.tracks_active {
            crate::i18n::tr("Mix independente ativo")
        } else {
            crate::i18n::tr("Usar mix independente")
        },
        "audio",
        Action::ToggleTrackMix,
        if app.project.tracks_active {
            Style::Primary
        } else {
            Style::Secondary
        },
    );
    label(
        c,
        crate::i18n::tr("O mix ativo é usado na reprodução e na exportação."),
        r.x + 290.,
        r.y + 106.,
        700.,
        14.,
        MUTED,
        false,
    );
    let viewport = Rect::new(r.x + 20., r.y + 153., r.w - 40., 437.);
    c.clip(viewport);
    app.hit_clip = Some(viewport);
    let tracks = app.project.audio_tracks.clone();
    app.scroll_max = (tracks.len() as f32 * 137. - viewport.h).max(0.);
    if tracks.is_empty() {
        c.paragraph(crate::i18n::tr("Importe um WAV ou mantenha .mic.wav e .system.wav ao lado do vídeo antes de importá-lo. Projetos antigos continuam usando o áudio incorporado até ativar o mix independente."),viewport.inset(24.),17.,MUTED,false);
    }
    for (i, t) in tracks.iter().enumerate() {
        let y = viewport.y + i as f32 * 137. - app.scroll;
        let row = Rect::new(viewport.x + 7., y + 3., viewport.w - 14., 126.);
        panel(c, row);
        label(
            c,
            &format!(
                "{} · {}",
                i + 1,
                t.source.file_name().unwrap_or_default().to_string_lossy()
            ),
            row.x + 14.,
            row.y + 10.,
            row.w - 28.,
            15.,
            if t.mute { DIM } else { TEXT },
            true,
        );
        label(
            c,
            &{
                let __wyzar_i18n_0 = &(t.gain * 100.);
                let __wyzar_i18n_1 = &(t.offset);
                let __wyzar_i18n_2 = &(crate::core::timecode(t.start));
                let __wyzar_i18n_3 = &(crate::core::timecode(t.end));
                let __wyzar_i18n_4 = &(t.fade_in);
                crate::i18n::format_text(
                    crate::i18n::tr("Ganho {:.0}%  ·  Offset {:+.2}s  ·  {} → {}  ·  Fade {:.1}s"),
                    &[
                        ("0:.0", format!("{:.0}", __wyzar_i18n_0)),
                        ("1:+.2", format!("{:+.2}", __wyzar_i18n_1)),
                        ("2", format!("{}", __wyzar_i18n_2)),
                        ("3", format!("{}", __wyzar_i18n_3)),
                        ("4:.1", format!("{:.1}", __wyzar_i18n_4)),
                    ],
                )
            },
            row.x + 14.,
            row.y + 41.,
            row.w - 28.,
            12.,
            MUTED,
            false,
        );
        let names = [
            crate::i18n::tr("Mute"),
            crate::i18n::tr("Solo"),
            crate::i18n::tr("Ganho −"),
            crate::i18n::tr("Ganho +"),
            crate::i18n::tr("Offset −"),
            crate::i18n::tr("Offset +"),
            crate::i18n::tr("Fade"),
            crate::i18n::tr("Cortar"),
            crate::i18n::tr("Dividir"),
            crate::i18n::tr("Excluir"),
        ];
        let bw = (row.w - 28.) / 10.;
        for command in 0..10 {
            button(
                app,
                c,
                Rect::new(row.x + 14. + command as f32 * bw, row.y + 78., bw - 5., 33.),
                names[command],
                "",
                Action::TrackEdit(i, command),
                if (command == 0 && t.mute) || (command == 1 && t.solo) {
                    Style::Selected
                } else {
                    Style::Secondary
                },
            );
        }
    }
    app.hit_clip = None;
    c.unclip();
    label(
        c,
        crate::i18n::tr(
            "Cortar e dividir usam o cursor da timeline. Offset: passos de 50 ms. Fade: 0–2 s.",
        ),
        r.x + 28.,
        r.bottom() - 90.,
        r.w - 56.,
        12.,
        MUTED,
        false,
    );
    button(
        app,
        c,
        Rect::new(r.x + 28., r.bottom() - 57., 210., 35.),
        crate::i18n::tr("Importar WAV"),
        "plus",
        Action::Import,
        Style::Secondary,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 171., r.bottom() - 57., 143., 35.),
        crate::i18n::tr("Concluir"),
        "check",
        Action::CloseModal,
        Style::Primary,
    );
}

pub(super) fn modal_panel(c: &Canvas, w: f32, h: f32, width: f32, height: f32) -> Rect {
    let r = Rect::new((w - width) / 2., (h - height) / 2. + 13., width, height);
    c.round(
        Rect::new(r.x + 8., r.y + 12., r.w, r.h),
        17.,
        BG.alpha(0.65),
        None,
    );
    c.round(r, 15., Color::rgb(0x0f1317), Some(Color::rgb(0x3d454d)));
    r
}
pub(super) fn record_modal(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    let r = modal_panel(c, w, h, 1040., 785.);
    let x = r.x + 30.;
    let width = r.w - 60.;
    c.icon("record", Rect::new(x, r.y + 26., 33., 33.), GOLD);
    label(
        c,
        crate::i18n::tr("Nova gravação"),
        x + 56.,
        r.y + 18.,
        width - 110.,
        27.,
        TEXT,
        true,
    );
    label(
        c,
        crate::i18n::tr("Configure sua captura e comece a gravar com a melhor qualidade."),
        x + 56.,
        r.y + 58.,
        width - 110.,
        14.,
        MUTED,
        false,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 55., r.y + 16., 38., 38.),
        "",
        "close",
        Action::CloseModal,
        Style::Ghost,
    );
    dropdown(
        app,
        c,
        Rect::new(r.right() - 353., r.y + 24., 280., 34.),
        crate::i18n::tr(crate::audio::MODE_LABELS[app.settings.recording_mode.min(3)]),
        ChoiceKind::RecordingMode,
    );
    c.line(x, r.y + 102., x + width, r.y + 102., BORDER, 1.);
    if app.settings.recording_mode == 3 {
        audio_modal(app, c, r);
        return;
    }
    label(
        c,
        crate::i18n::tr("1. Selecione a fonte de captura"),
        x,
        r.y + 115.,
        width,
        15.,
        TEXT,
        true,
    );
    let s = app.settings.clone();
    if s.source != Source::Display {
        let text = if s.source == Source::Region {
            app.region
                .map(|r| {
                    let __wyzar_i18n_0 = &(r.width);
                    let __wyzar_i18n_1 = &(r.height);
                    crate::i18n::format_text(
                        crate::i18n::tr("Região: {} × {}"),
                        &[
                            ("0", format!("{}", __wyzar_i18n_0)),
                            ("1", format!("{}", __wyzar_i18n_1)),
                        ],
                    )
                })
                .unwrap_or(crate::i18n::tr("Selecionar área da tela").into())
        } else if app.window_name.is_empty() {
            crate::i18n::tr("Selecionar janela").into()
        } else {
            app.window_name.clone()
        };
        dropdown(
            app,
            c,
            Rect::new(x + width - 375., r.y + 112., 375., 31.),
            &text,
            ChoiceKind::Window,
        );
    }
    let cw = (width - 36.) / 4.;
    for (i, source) in Source::ALL.into_iter().enumerate() {
        source_card(
            app,
            c,
            Rect::new(x + i as f32 * (cw + 12.), r.y + 151., cw, 114.),
            source,
            false,
        );
    }
    label(
        c,
        crate::i18n::tr("2. Configure os dispositivos"),
        x,
        r.y + 281.,
        width,
        15.,
        TEXT,
        true,
    );
    let dw = (width - 24.) / 3.;
    for (i, (name, icon, t, on, kind)) in [
        (
            crate::i18n::tr("Microfone"),
            "microphone",
            Toggle::Microphone,
            s.microphone,
            ChoiceKind::Microphone,
        ),
        (
            crate::i18n::tr("Áudio do sistema"),
            "audio",
            Toggle::SystemAudio,
            s.system_audio,
            ChoiceKind::Output,
        ),
        (
            crate::i18n::tr("Webcam"),
            "webcam",
            Toggle::Webcam,
            s.webcam,
            ChoiceKind::Webcam,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let dr = Rect::new(x + i as f32 * (dw + 12.), r.y + 314., dw, 113.);
        if t == Toggle::Webcam && app.webcams.is_empty() {
            continue;
        }
        panel(c, dr);
        c.icon(icon, Rect::new(dr.x + 15., dr.y + 16., 24., 24.), TEXT);
        label(
            c,
            name,
            dr.x + 50.,
            dr.y + 14.,
            dr.w - 110.,
            14.,
            TEXT,
            true,
        );
        toggle(
            app,
            c,
            Rect::new(dr.right() - 51., dr.y + 17., 36., 20.),
            on,
            t,
        );
        let device = app.device_name(t);
        let cam = if t == Toggle::Webcam {
            app.webcam_frame.lock().ok().and_then(|f| f.clone())
        } else {
            None
        };
        dropdown(
            app,
            c,
            Rect::new(
                dr.x + 13.,
                dr.y + 60.,
                dr.w - if cam.is_some() { 110. } else { 26. },
                34.,
            ),
            &device,
            kind,
        );
        if let Some(frame) = cam {
            c.frame(&frame, Rect::new(dr.right() - 87., dr.y + 54., 74., 47.));
        }
    }
    label(
        c,
        crate::i18n::tr("3. Qualidade da gravação"),
        x,
        r.y + 444.,
        540.,
        15.,
        TEXT,
        true,
    );
    let qw = 176.;
    for (i, (name, res, bit, detail)) in [
        (
            crate::i18n::tr("Padrão"),
            (1920, 1080),
            20,
            crate::i18n::tr("1080p · 60 FPS"),
        ),
        (
            crate::i18n::tr("Alto"),
            (2560, 1440),
            40,
            crate::i18n::tr("1440p · 60 FPS"),
        ),
        (
            crate::i18n::tr("Ultra"),
            (3840, 2160),
            60,
            crate::i18n::tr("4K · 60 FPS"),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let qr = Rect::new(x + i as f32 * (qw + 11.), r.y + 478., qw, 88.);
        let selected = s.width == res.0 && s.bitrate == bit * 1_000_000 && s.fps == 60;
        panel(c, qr);
        if selected {
            c.outline(qr, 9., GOLD, 2.);
        }
        label(
            c,
            name,
            qr.x + 17.,
            qr.y + 13.,
            qr.w - 35.,
            16.,
            if selected { GOLD } else { TEXT },
            true,
        );
        label(
            c,
            detail,
            qr.x + 17.,
            qr.y + 47.,
            qr.w - 30.,
            11.,
            MUTED,
            false,
        );
        app.hit(
            qr,
            Action::Preset(i),
            crate::i18n::format_text(
                crate::i18n::tr("{name} · {bit} Mbps"),
                &[("name", format!("{}", name)), ("bit", format!("{}", bit))],
            ),
        );
    }
    let qx = x + 586.;
    label(
        c,
        crate::i18n::tr("4. Resolução e FPS"),
        qx,
        r.y + 444.,
        width - 586.,
        15.,
        TEXT,
        true,
    );
    dropdown(
        app,
        c,
        Rect::new(qx, r.y + 478., width - 586., 35.),
        &resolution(&s),
        ChoiceKind::Resolution,
    );
    dropdown(
        app,
        c,
        Rect::new(qx, r.y + 528., (width - 600.) / 2., 34.),
        &{
            let __wyzar_i18n_0 = &(s.fps);
            crate::i18n::format_text(
                crate::i18n::tr("{} FPS"),
                &[("0", format!("{}", __wyzar_i18n_0))],
            )
        },
        ChoiceKind::Fps,
    );
    dropdown(
        app,
        c,
        Rect::new(
            qx + (width - 600.) / 2. + 14.,
            r.y + 528.,
            (width - 600.) / 2.,
            34.,
        ),
        &{
            let __wyzar_i18n_0 = &(s.bitrate / 1_000_000);
            crate::i18n::format_text(
                crate::i18n::tr("{} Mbps"),
                &[("0", format!("{}", __wyzar_i18n_0))],
            )
        },
        ChoiceKind::Bitrate,
    );
    label(
        c,
        crate::i18n::tr("5. Opções adicionais"),
        x,
        r.y + 582.,
        width,
        15.,
        TEXT,
        true,
    );
    let opt_y = r.y + 619.;
    for (i, (title, on, t)) in [
        (crate::i18n::tr("Mostrar cursor"), s.cursor, Toggle::Cursor),
        (
            crate::i18n::tr("Destacar cliques"),
            s.click_effects,
            Toggle::Clicks,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let xx = x + i as f32 * 238.;
        toggle(app, c, Rect::new(xx, opt_y + 2., 35., 20.), on, t);
        label(c, title, xx + 48., opt_y - 2., 180., 13., TEXT, false);
    }
    dropdown(
        app,
        c,
        Rect::new(x + 522., opt_y - 3., 174., 32.),
        &{
            let __wyzar_i18n_0 = &(s.countdown);
            crate::i18n::format_text(
                crate::i18n::tr("Contagem: {} s"),
                &[("0", format!("{}", __wyzar_i18n_0))],
            )
        },
        ChoiceKind::Countdown,
    );
    button(
        app,
        c,
        Rect::new(x + 714., opt_y - 3., width - 714., 32.),
        crate::i18n::tr("Carregar perfil"),
        "save",
        Action::OpenChoice(
            ChoiceKind::Profile,
            Rect::new(x + 714., opt_y - 3., width - 714., 32.),
        ),
        Style::Secondary,
    );
    let folder = Rect::new(x, r.y + 662., width, 42.);
    c.round(folder, 7., BG, Some(BORDER));
    c.icon("folder", Rect::new(x + 12., folder.y + 11., 20., 20.), GOLD);
    label(
        c,
        crate::i18n::tr("Salvar em"),
        x + 44.,
        folder.y + 7.,
        75.,
        12.,
        MUTED,
        false,
    );
    c.text(
        &s.videos.to_string_lossy(),
        Rect::new(x + 128., folder.y + 2., width - 250., 38.),
        12.,
        TEXT,
        false,
    );
    button(
        app,
        c,
        Rect::new(folder.right() - 106., folder.y + 5., 98., 32.),
        crate::i18n::tr("Alterar"),
        "",
        Action::ChooseFolder(0),
        Style::Secondary,
    );
    c.line(x, r.y + 720., x + width, r.y + 720., BORDER, 1.);
    button(
        app,
        c,
        Rect::new(x, r.y + 736., 201., 36.),
        crate::i18n::tr("Salvar como perfil"),
        "save",
        Action::SaveProfile,
        Style::Secondary,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 359., r.y + 732., 115., 42.),
        crate::i18n::tr("Cancelar"),
        "",
        Action::CloseModal,
        Style::Secondary,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 229., r.y + 732., 199., 42.),
        crate::i18n::tr("Iniciar gravação"),
        "record",
        Action::StartRecording,
        Style::Primary,
    );
}
pub(super) fn audio_modal(app: &mut App, c: &mut Canvas, r: Rect) {
    let x = r.x + 30.;
    let width = r.w - 60.;
    let s = app.settings.clone();
    label(
        c,
        crate::i18n::tr("Gravação de áudio"),
        x,
        r.y + 124.,
        width,
        26.,
        TEXT,
        true,
    );
    label(
        c,
        crate::i18n::tr("Microfone, som do computador ou os dois. Sem abrir a captura de tela."),
        x,
        r.y + 168.,
        width,
        15.,
        MUTED,
        false,
    );
    for (i, (name, icon, t, on, kind, gain_kind, gain, level)) in [
        (
            crate::i18n::tr("Microfone"),
            "microphone",
            Toggle::Microphone,
            s.microphone,
            ChoiceKind::Microphone,
            ChoiceKind::MicGain,
            s.mic_gain,
            app.mic_level,
        ),
        (
            crate::i18n::tr("Áudio do sistema"),
            "audio",
            Toggle::SystemAudio,
            s.system_audio,
            ChoiceKind::Output,
            ChoiceKind::SystemGain,
            s.system_gain,
            app.system_level,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let cr = Rect::new(
            x + i as f32 * (width / 2. + 7.),
            r.y + 215.,
            (width - 14.) / 2.,
            210.,
        );
        panel(c, cr);
        c.icon(icon, Rect::new(cr.x + 20., cr.y + 21., 30., 30.), GOLD);
        label(
            c,
            name,
            cr.x + 67.,
            cr.y + 21.,
            cr.w - 142.,
            18.,
            TEXT,
            true,
        );
        toggle(
            app,
            c,
            Rect::new(cr.right() - 63., cr.y + 24., 40., 22.),
            on,
            t,
        );
        dropdown(
            app,
            c,
            Rect::new(cr.x + 20., cr.y + 80., cr.w - 40., 37.),
            &app.device_name(t),
            kind,
        );
        row_choice(
            app,
            c,
            cr.x + 20.,
            cr.y + 133.,
            cr.w - 40.,
            crate::i18n::tr("Ganho de gravação"),
            &format!("{gain}%"),
            gain_kind,
        );
        c.round(
            Rect::new(cr.x + 20., cr.bottom() - 21., cr.w - 40., 4.),
            2.,
            BORDER,
            None,
        );
        c.round(
            Rect::new(
                cr.x + 20.,
                cr.bottom() - 21.,
                (cr.w - 40.) * level.clamp(0., 1.),
                4.,
            ),
            2.,
            GOLD,
            None,
        );
    }
    row_toggle(
        app,
        c,
        x,
        r.y + 453.,
        width,
        crate::i18n::tr("Salvar também as fontes separadas"),
        crate::i18n::tr("Mantém .mic.wav e .system.wav sem ganho, além da mistura principal."),
        s.audio_sources,
        Toggle::AudioSources,
    );
    label(
        c,
        crate::i18n::tr("WAV · PCM 16 bits · 48 kHz · estéreo"),
        x,
        r.y + 539.,
        width,
        17.,
        TEXT,
        true,
    );
    label(
        c,
        crate::i18n::tr(
            "Sem compressão. Arquivos grandes usam RF64 automaticamente. Pausa e atalhos configuráveis.",
        ),
        x,
        r.y + 576.,
        width,
        13.,
        MUTED,
        false,
    );
    label(
        c,
        crate::i18n::tr("Salvar áudio em"),
        x,
        r.y + 630.,
        width,
        13.,
        TEXT,
        true,
    );
    c.round(
        Rect::new(x, r.y + 660., width - 120., 34.),
        6.,
        BG,
        Some(BORDER),
    );
    c.text(
        &s.audios.to_string_lossy(),
        Rect::new(x + 9., r.y + 665., width - 140., 24.),
        12.,
        MUTED,
        false,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 141., r.y + 660., 111., 34.),
        crate::i18n::tr("Alterar"),
        "folder",
        Action::ChooseFolder(5),
        Style::Secondary,
    );
    c.line(x, r.y + 720., x + width, r.y + 720., BORDER, 1.);
    button(
        app,
        c,
        Rect::new(x, r.y + 736., 201., 36.),
        crate::i18n::tr("Salvar como perfil"),
        "save",
        Action::SaveProfile,
        Style::Secondary,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 359., r.y + 732., 115., 42.),
        crate::i18n::tr("Cancelar"),
        "",
        Action::CloseModal,
        Style::Secondary,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 229., r.y + 732., 199., 42.),
        crate::i18n::tr("Gravar áudio"),
        "record",
        Action::StartRecording,
        Style::Primary,
    );
}
pub(super) fn export_modal(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    let r = modal_panel(c, w, h, 650., 573.);
    let x = r.x + 27.;
    c.icon("export", Rect::new(x, r.y + 25., 29., 29.), GOLD);
    label(
        c,
        crate::i18n::tr("Exportar vídeo"),
        x + 47.,
        r.y + 18.,
        500.,
        26.,
        TEXT,
        true,
    );
    label(
        c,
        crate::i18n::tr("Escolha a qualidade, o nome e a pasta do arquivo final."),
        x,
        r.y + 72.,
        r.w - 54.,
        14.,
        MUTED,
        false,
    );
    label(
        c,
        crate::i18n::tr("Nome do arquivo"),
        x,
        r.y + 118.,
        500.,
        13.,
        TEXT,
        true,
    );
    field(
        app,
        c,
        Rect::new(x, r.y + 148., r.w - 54., 39.),
        &app.export_name.clone(),
        Field::ExportName,
        crate::i18n::tr("Meu vídeo"),
    );
    let s = app.settings.clone();
    row_choice(
        app,
        c,
        x,
        r.y + 207.,
        r.w - 54.,
        crate::i18n::tr("Resolução"),
        &resolution(&s),
        ChoiceKind::Resolution,
    );
    row_choice(
        app,
        c,
        x,
        r.y + 256.,
        r.w - 54.,
        crate::i18n::tr("Qualidade"),
        &{
            let __wyzar_i18n_0 = &(s.bitrate / 1_000_000);
            crate::i18n::format_text(
                crate::i18n::tr("{} Mbps"),
                &[("0", format!("{}", __wyzar_i18n_0))],
            )
        },
        ChoiceKind::Bitrate,
    );
    row_choice(
        app,
        c,
        x,
        r.y + 305.,
        r.w - 54.,
        crate::i18n::tr("Quadros por segundo"),
        &{
            let __wyzar_i18n_0 = &(s.fps);
            crate::i18n::format_text(
                crate::i18n::tr("{} FPS"),
                &[("0", format!("{}", __wyzar_i18n_0))],
            )
        },
        ChoiceKind::Fps,
    );
    label(
        c,
        crate::i18n::tr("Pasta de exportação"),
        x,
        r.y + 362.,
        500.,
        13.,
        TEXT,
        true,
    );
    c.text(
        &s.exports.to_string_lossy(),
        Rect::new(x, r.y + 395., r.w - 182., 39.),
        13.,
        MUTED,
        false,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 135., r.y + 395., 108., 37.),
        crate::i18n::tr("Escolher"),
        "folder",
        Action::ChooseFolder(3),
        Style::Secondary,
    );
    chip(
        c,
        Rect::new(x, r.y + 456., 230., 29.),
        crate::i18n::tr(crate::container::VIDEO_DESCRIPTION),
        GOLD,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 267., r.y + 508., 106., 41.),
        crate::i18n::tr("Cancelar"),
        "",
        Action::CloseModal,
        Style::Secondary,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 148., r.y + 508., 121., 41.),
        crate::i18n::tr("Exportar"),
        "export",
        Action::BeginExport,
        Style::Primary,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 52., r.y + 17., 32., 32.),
        "",
        "close",
        Action::CloseModal,
        Style::Ghost,
    );
}
pub(super) fn text_modal(app: &mut App, c: &mut Canvas, w: f32, h: f32, f: Field, title: &str) {
    let r = modal_panel(c, w, h, 580., 252.);
    label(c, title, r.x + 27., r.y + 25., r.w - 54., 23., TEXT, true);
    field(
        app,
        c,
        Rect::new(r.x + 27., r.y + 92., r.w - 54., 42.),
        &app.text_buffer.clone(),
        f,
        crate::i18n::tr("Digite aqui…"),
    );
    if f == Field::Appearance(5) {
        label(
            c,
            crate::i18n::tr("Destaque, fundo, painéis e texto · separados por vírgula"),
            r.x + 27.,
            r.y + 143.,
            r.w - 54.,
            11.,
            MUTED,
            false,
        );
    }
    button(
        app,
        c,
        Rect::new(r.right() - 270., r.bottom() - 65., 108., 40.),
        crate::i18n::tr("Cancelar"),
        "",
        Action::CloseModal,
        Style::Secondary,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 148., r.bottom() - 65., 121., 40.),
        crate::i18n::tr("Salvar"),
        "check",
        Action::ConfirmText,
        Style::Primary,
    );
}
pub(super) fn delete_modal(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    let r = modal_panel(c, w, h, 660., 390.);
    label(
        c,
        crate::i18n::tr("Mover gravação para a Lixeira?"),
        r.x + 27.,
        r.y + 25.,
        r.w - 54.,
        23.,
        TEXT,
        true,
    );
    label(
        c,
        crate::i18n::tr("O vídeo e seus auxiliares serão tratados como uma gravação."),
        r.x + 27.,
        r.y + 88.,
        r.w - 54.,
        14.,
        MUTED,
        false,
    );
    let files = app
        .selected
        .and_then(|index| app.library.items.get(index))
        .map(|record| crate::library::bundle::files(&record.path))
        .unwrap_or_default();
    for (index, path) in files.iter().take(6).enumerate() {
        let y = r.y + 130. + index as f32 * 25.;
        c.icon("check", Rect::new(r.x + 29., y, 16., 16.), GOLD);
        label(
            c,
            &path.file_name().unwrap_or_default().to_string_lossy(),
            r.x + 57.,
            y - 4.,
            r.w - 84.,
            13.,
            MUTED,
            false,
        );
    }
    button(
        app,
        c,
        Rect::new(r.right() - 287., r.bottom() - 65., 108., 40.),
        crate::i18n::tr("Cancelar"),
        "",
        Action::CloseModal,
        Style::Secondary,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 165., r.bottom() - 65., 138., 40.),
        crate::i18n::tr("Mover à Lixeira"),
        "trash",
        Action::ConfirmDelete,
        Style::Danger,
    );
}
