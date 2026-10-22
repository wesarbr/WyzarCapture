use super::*;

pub(super) fn settings(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    let x = 240.;
    let width = w - x - 24.;
    c.clip(Rect::new(x, 65., width, 95.));
    c.wallpaper(Rect::new(w - 700., -180., 800., 420.), 0.55);
    c.gradient(Rect::new(x, 65., width, 95.), BG, BG.alpha(0.1), true, 0.);
    c.unclip();
    title(
        c,
        crate::i18n::tr("Configurações"),
        crate::i18n::tr("Captura, áudio, vídeo, atalhos, pastas e aparência."),
        x + 2.,
        84.,
        width,
    );
    let nav = Rect::new(x, 178., 190., h - 232.);
    panel(c, nav);
    let cats = [
        (crate::i18n::tr("Geral"), "settings"),
        (crate::i18n::tr("Captura"), "monitor"),
        (crate::i18n::tr("Áudio"), "audio"),
        (crate::i18n::tr("Vídeo"), "play"),
        (crate::i18n::tr("Atalhos"), "keyboard"),
        (crate::i18n::tr("Armazenamento"), "storage"),
        (crate::i18n::tr("Aparência"), "palette"),
        (crate::i18n::tr("Avançado"), "sliders"),
        (crate::i18n::tr("Conta local"), "account"),
        (crate::i18n::tr("Sobre"), "help"),
    ];
    for (i, (name, icon)) in cats.into_iter().enumerate() {
        button(
            app,
            c,
            Rect::new(x + 9., 190. + i as f32 * 51., 172., 43.),
            name,
            icon,
            Action::SettingsCategory(i),
            if app.category == i {
                Style::Selected
            } else {
                Style::Ghost
            },
        );
    }
    let content = Rect::new(x + 206., 178., width - 206., h - 252.);
    let gap = 14.;
    let half = (content.w - gap) / 2.;
    let s = app.settings.clone();
    match app.category {
        0 => {
            let left = Rect::new(content.x, 178., half, 260.);
            section(
                c,
                left,
                crate::i18n::tr("Geral"),
                crate::i18n::tr("Configurações principais do aplicativo."),
                "monitor",
            );
            label(
                c,
                crate::i18n::tr("Idioma"),
                left.x + 20.,
                left.y + 80.,
                200.,
                13.,
                TEXT,
                false,
            );
            dropdown(
                app,
                c,
                Rect::new(left.x + left.w * 0.46, left.y + 76., left.w * 0.5, 33.),
                s.language.name(),
                ChoiceKind::Language,
            );
            row_toggle(
                app,
                c,
                left.x + 20.,
                left.y + 126.,
                left.w - 40.,
                crate::i18n::tr("Iniciar com o Windows"),
                crate::i18n::tr("Abre o WyzarCapture automaticamente."),
                s.startup,
                Toggle::Startup,
            );
            row_toggle(
                app,
                c,
                left.x + 20.,
                left.y + 185.,
                left.w - 40.,
                crate::i18n::tr("Minimizar para a bandeja"),
                crate::i18n::tr("Ao fechar a janela do aplicativo."),
                s.tray,
                Toggle::Tray,
            );
            let pr = Rect::new(content.x + half + gap, 178., half, 260.);
            section(
                c,
                pr,
                crate::i18n::tr("Pré-visualização"),
                crate::i18n::tr("Resolução, FPS e formato atuais."),
                "monitor",
            );
            c.wallpaper(Rect::new(pr.x + 18., pr.y + 73., pr.w - 36., 137.), 1.);
            chip(
                c,
                Rect::new(pr.x + 30., pr.y + 85., 75., 26.),
                crate::i18n::tr("PRÉVIA"),
                MUTED,
            );
            label(
                c,
                &{
                    let __wyzar_i18n_0 = &(resolution(&s));
                    let __wyzar_i18n_1 = &(s.fps);
                    crate::i18n::format_text(
                        crate::i18n::tr("{}    ·    {} FPS    ·    MP4"),
                        &[
                            ("0", format!("{}", __wyzar_i18n_0)),
                            ("1", format!("{}", __wyzar_i18n_1)),
                        ],
                    )
                },
                pr.x + 20.,
                pr.bottom() - 37.,
                pr.w - 40.,
                13.,
                TEXT,
                false,
            );
            let cp = Rect::new(content.x, 452., half, 238.);
            capture_settings(app, c, cp);
            let vp = Rect::new(pr.x, 452., half, 238.);
            video_settings(app, c, vp);
            let dest = Rect::new(content.x, 704., content.w, (h - 759.).max(100.));
            section(
                c,
                dest,
                crate::i18n::tr("Nomeação e armazenamento"),
                crate::i18n::tr("Escolha onde suas gravações serão salvas."),
                "storage",
            );
            let folder = app.settings.videos.to_string_lossy().into_owned();
            c.text(
                &folder,
                Rect::new(dest.x + 22., dest.y + 75., dest.w - 211., 31.),
                13.,
                TEXT,
                false,
            );
            button(
                app,
                c,
                Rect::new(dest.right() - 179., dest.y + 73., 157., 35.),
                crate::i18n::tr("Escolher pasta"),
                "folder",
                Action::ChooseFolder(0),
                Style::Secondary,
            );
        }
        1 => {
            let r = Rect::new(content.x, 178., content.w, 360.);
            capture_settings(app, c, r);
            let r2 = Rect::new(content.x, 552., content.w, 190.);
            section(
                c,
                r2,
                crate::i18n::tr("Cursor e região"),
                crate::i18n::tr("Controles da fonte de captura."),
                "cursor",
            );
            row_toggle(
                app,
                c,
                r2.x + 24.,
                r2.y + 76.,
                r2.w / 2. - 40.,
                crate::i18n::tr("Mostrar cursor"),
                crate::i18n::tr("Incluir o ponteiro na gravação."),
                s.cursor,
                Toggle::Cursor,
            );
            row_toggle(
                app,
                c,
                r2.x + r2.w / 2. + 10.,
                r2.y + 76.,
                r2.w / 2. - 40.,
                crate::i18n::tr("Destacar cliques"),
                crate::i18n::tr("Realce dourado enquanto clica."),
                s.click_effects,
                Toggle::Clicks,
            );
        }
        2 => {
            let r = Rect::new(content.x, 178., content.w, 304.);
            section(
                c,
                r,
                crate::i18n::tr("Áudio"),
                crate::i18n::tr("Microfone e som do sistema podem ser gravados juntos."),
                "audio",
            );
            dropdown(
                app,
                c,
                Rect::new(r.right() - 301., r.y + 16., 277., 32.),
                crate::i18n::tr(crate::audio::MODE_LABELS[s.recording_mode.min(3)]),
                ChoiceKind::RecordingMode,
            );
            row_toggle(
                app,
                c,
                r.x + 24.,
                r.y + 76.,
                (r.w - 48.) / 2.,
                crate::i18n::tr("Microfone"),
                crate::i18n::tr("Fonte de entrada independente."),
                s.microphone,
                Toggle::Microphone,
            );
            let name = app.device_name(Toggle::Microphone);
            dropdown(
                app,
                c,
                Rect::new(r.x + 24., r.y + 137., r.w - 218., 36.),
                &name,
                ChoiceKind::Microphone,
            );
            dropdown(
                app,
                c,
                Rect::new(r.right() - 180., r.y + 137., 156., 36.),
                &{
                    let __wyzar_i18n_0 = &(s.mic_gain);
                    crate::i18n::format_text(
                        crate::i18n::tr("Ganho {}%"),
                        &[("0", format!("{}", __wyzar_i18n_0))],
                    )
                },
                ChoiceKind::MicGain,
            );
            row_toggle(
                app,
                c,
                r.x + 24.,
                r.y + 190.,
                (r.w - 48.) / 2.,
                crate::i18n::tr("Áudio do sistema"),
                crate::i18n::tr("Grava o som do dispositivo de saída escolhido."),
                s.system_audio,
                Toggle::SystemAudio,
            );
            let name = app.device_name(Toggle::SystemAudio);
            dropdown(
                app,
                c,
                Rect::new(r.x + 24., r.y + 249., r.w - 218., 36.),
                &name,
                ChoiceKind::Output,
            );
            dropdown(
                app,
                c,
                Rect::new(r.right() - 180., r.y + 249., 156., 36.),
                &{
                    let __wyzar_i18n_0 = &(s.system_gain);
                    crate::i18n::format_text(
                        crate::i18n::tr("Ganho {}%"),
                        &[("0", format!("{}", __wyzar_i18n_0))],
                    )
                },
                ChoiceKind::SystemGain,
            );
            // Processing switches sit beside their sources instead of over the
            // recording mode selector in the section header.
            for (title, y, on, which) in [
                (
                    crate::i18n::tr("Melhorar voz"),
                    76.,
                    s.voice_filter,
                    Toggle::VoiceFilter,
                ),
                (
                    crate::i18n::tr("Proteger picos"),
                    190.,
                    s.audio_limiter,
                    Toggle::AudioLimiter,
                ),
            ] {
                let x = r.x + (r.w - 48.) / 2. + 48.;
                label(
                    c,
                    title,
                    x,
                    r.y + y + 2.,
                    r.right() - x - 90.,
                    13.,
                    MUTED,
                    false,
                );
                toggle(
                    app,
                    c,
                    Rect::new(r.right() - 66., r.y + y + 5., 42., 23.),
                    on,
                    which,
                );
            }
            let wr = Rect::new(content.x, 496., content.w, 281.);
            section(
                c,
                wr,
                crate::i18n::tr("Webcam"),
                crate::i18n::tr("Câmera opcional sobreposta à captura."),
                "webcam",
            );
            toggle(
                app,
                c,
                Rect::new(wr.right() - 67., wr.y + 22., 42., 23.),
                s.webcam,
                Toggle::Webcam,
            );
            let cam = app.device_name(Toggle::Webcam);
            row_choice(
                app,
                c,
                wr.x + 24.,
                wr.y + 78.,
                wr.w - 48.,
                crate::i18n::tr("Dispositivo"),
                &cam,
                ChoiceKind::Webcam,
            );
            let half = (wr.w - 72.) / 2.;
            row_choice(
                app,
                c,
                wr.x + 24.,
                wr.y + 131.,
                half,
                crate::i18n::tr("Posição"),
                [
                    crate::i18n::tr("Superior esquerda"),
                    crate::i18n::tr("Superior direita"),
                    crate::i18n::tr("Inferior esquerda"),
                    crate::i18n::tr("Inferior direita"),
                ][s.webcam_position],
                ChoiceKind::WebcamPosition,
            );
            row_choice(
                app,
                c,
                wr.x + 48. + half,
                wr.y + 131.,
                half,
                crate::i18n::tr("Formato"),
                crate::ui::app::webcam_shapes()[s.webcam_shape.min(2)],
                ChoiceKind::WebcamShape,
            );
            row_toggle(
                app,
                c,
                wr.x + 48. + half,
                wr.y + 180.,
                half,
                crate::i18n::tr("Espelhar imagem"),
                "",
                s.webcam_mirror,
                Toggle::WebcamMirror,
            );
            row_choice(
                app,
                c,
                wr.x + 24.,
                wr.y + 184.,
                half,
                crate::i18n::tr("Tamanho"),
                &{
                    let __wyzar_i18n_0 = &(s.webcam_size);
                    crate::i18n::format_text(
                        crate::i18n::tr("{}% da largura"),
                        &[("0", format!("{}", __wyzar_i18n_0))],
                    )
                },
                ChoiceKind::WebcamSize,
            );
            button(
                app,
                c,
                Rect::new(wr.x + 24., wr.y + 235., 215., 32.),
                crate::i18n::tr("Atualizar dispositivos"),
                "webcam",
                Action::RefreshDevices,
                Style::Secondary,
            );
            button(
                app,
                c,
                Rect::new(wr.right() - 246., wr.y + 235., 222., 32.),
                if app.webcams.is_empty() {
                    crate::i18n::tr("Abrir gravador de áudio")
                } else {
                    crate::i18n::tr("Ampliar webcam")
                },
                if app.webcams.is_empty() {
                    "microphone"
                } else {
                    "webcam"
                },
                if app.webcams.is_empty() {
                    Action::NewAudioRecording
                } else {
                    Action::CameraView
                },
                Style::Secondary,
            );
        }
        3 => {
            let r = Rect::new(content.x, 178., content.w, 385.);
            video_settings(app, c, r);
            let r2 = Rect::new(content.x, 577., content.w, 161.);
            section(
                c,
                r2,
                crate::i18n::tr("Qualidade consistente"),
                crate::i18n::tr("A capacidade de captura depende da fonte e do hardware."),
                "encoder",
            );
            label(
                c,
                crate::i18n::tr("MP4 · H.264 para vídeo, AAC estéreo para áudio."),
                r2.x + 24.,
                r2.y + 79.,
                r2.w - 48.,
                15.,
                TEXT,
                false,
            );
            label(
                c,
                crate::i18n::tr(
                    "Compatibilidade automática: 720p/30 se o encoder recusar o perfil.",
                ),
                r2.x + 24.,
                r2.y + 112.,
                r2.w - 118.,
                13.,
                MUTED,
                false,
            );
            toggle(
                app,
                c,
                Rect::new(r2.right() - 70., r2.y + 111., 42., 23.),
                s.auto_fallback,
                Toggle::AutoFallback,
            );
        }
        4 => {
            let r = Rect::new(content.x, 178., content.w, 465.);
            section(
                c,
                r,
                crate::i18n::tr("Atalhos globais"),
                crate::i18n::tr("Disponíveis mesmo com outra janela em primeiro plano."),
                "timeline",
            );
            for (i, (name, key, k)) in [
                (
                    crate::i18n::tr("Iniciar gravação"),
                    s.record_hotkey,
                    ChoiceKind::RecordKey,
                ),
                (
                    crate::i18n::tr("Pausar / retomar"),
                    s.pause_hotkey,
                    ChoiceKind::PauseKey,
                ),
                (
                    crate::i18n::tr("Parar gravação"),
                    s.stop_hotkey,
                    ChoiceKind::StopKey,
                ),
                (
                    crate::i18n::tr("Capturar tela"),
                    s.screenshot_hotkey,
                    ChoiceKind::ScreenshotKey,
                ),
            ]
            .into_iter()
            .enumerate()
            {
                row_choice(
                    app,
                    c,
                    r.x + 24.,
                    r.y + 84. + i as f32 * 57.,
                    r.w - 48.,
                    name,
                    &crate::settings::shortcuts::label(key),
                    k,
                );
            }
            label(
                c,
                crate::i18n::tr(
                    "No editor: Espaço reproduz · S divide · Ctrl+Z desfaz · Ctrl+S salva",
                ),
                r.x + 24.,
                r.y + 330.,
                r.w - 48.,
                14.,
                MUTED,
                false,
            );
            button(
                app,
                c,
                Rect::new(r.x + 24., r.y + 382., 280., 38.),
                crate::i18n::tr("Restaurar atalhos padrão"),
                "settings",
                Action::ResetShortcuts,
                Style::Secondary,
            );
        }
        5 => {
            let r = Rect::new(content.x, content.y - app.scroll, content.w, 995.);
            c.clip(content);
            app.hit_clip = Some(content);
            app.scroll_max = (995. - content.h).max(0.);
            section(
                c,
                r,
                crate::i18n::tr("Armazenamento"),
                crate::i18n::tr("Pastas independentes, validadas e salvas no seu computador."),
                "storage",
            );
            let folders = [
                (crate::i18n::tr("Vídeos"), s.videos),
                (crate::i18n::tr("Capturas de tela"), s.screenshots),
                (crate::i18n::tr("Temporários e cache"), s.cache),
                (crate::i18n::tr("Exportações"), s.exports),
                (crate::i18n::tr("Projetos de edição"), s.projects),
                (crate::i18n::tr("Gravações de áudio"), s.audios),
            ];
            for (i, (name, path)) in folders.into_iter().enumerate() {
                let y = r.y + 70. + i as f32 * 66.;
                label(c, name, r.x + 23., y, r.w - 48., 13., TEXT, true);
                let fr = Rect::new(r.x + 23., y + 29., r.w - 163., 34.);
                c.round(fr, 6., BG, Some(BORDER));
                c.text(&path.to_string_lossy(), fr.inset(7.), 12., MUTED, false);
                button(
                    app,
                    c,
                    Rect::new(r.right() - 127., y + 29., 104., 34.),
                    crate::i18n::tr("Alterar"),
                    "folder",
                    Action::ChooseFolder(i),
                    Style::Secondary,
                );
            }
            row_choice(
                app,
                c,
                r.x + 23.,
                r.y + 483.,
                r.w - 46.,
                crate::i18n::tr("Organizar em subpastas"),
                match s.subfolders.as_str() {
                    "date" => crate::i18n::tr("Por data"),
                    "type" => crate::i18n::tr("Por tipo"),
                    "project" => crate::i18n::tr("Por projeto"),
                    _ => crate::i18n::tr("Sem subpastas"),
                },
                ChoiceKind::Subfolders,
            );
            row_choice(
                app,
                c,
                r.x + 23.,
                r.y + 531.,
                r.w - 46.,
                crate::i18n::tr("Alertar quando restar"),
                &{
                    let __wyzar_i18n_0 = &(s.low_space_gb);
                    crate::i18n::format_text(
                        crate::i18n::tr("{} GB"),
                        &[("0", format!("{}", __wyzar_i18n_0))],
                    )
                },
                ChoiceKind::SpaceAlert,
            );
            row_choice(
                app,
                c,
                r.x + 23.,
                r.y + 579.,
                r.w - 46.,
                crate::i18n::tr("Limite da biblioteca"),
                &if s.storage_limit_gb == 0 {
                    crate::i18n::tr("Sem limite").into()
                } else {
                    {
                        let __wyzar_i18n_0 = &(s.storage_limit_gb);
                        crate::i18n::format_text(
                            crate::i18n::tr("{} GB"),
                            &[("0", format!("{}", __wyzar_i18n_0))],
                        )
                    }
                },
                ChoiceKind::StorageLimit,
            );
            if s.subfolders == "project" {
                label(
                    c,
                    crate::i18n::tr("Nome do projeto"),
                    r.x + 23.,
                    r.y + 631.,
                    (r.w - 46.) * 0.45,
                    13.,
                    TEXT,
                    false,
                );
                field(
                    app,
                    c,
                    Rect::new(
                        r.x + 23. + (r.w - 46.) * 0.46,
                        r.y + 627.,
                        (r.w - 46.) * 0.54,
                        33.,
                    ),
                    &s.project_name,
                    Field::ProjectName,
                    crate::i18n::tr("Meu projeto"),
                );
            }
            label(
                c,
                crate::i18n::tr("Manutenção segura"),
                r.x + 23.,
                r.y + 698.,
                r.w - 46.,
                17.,
                TEXT,
                true,
            );
            for (i, name) in [
                crate::i18n::tr("Limpar cache"),
                crate::i18n::tr("Limpar miniaturas"),
                crate::i18n::tr("Limpar logs antigos"),
            ]
            .iter()
            .enumerate()
            {
                let bytes = crate::persistence::maintenance::for_kind(&app.settings, i)
                    .map(|p| p.bytes)
                    .unwrap_or(0);
                let rect = Rect::new(r.x + 23., r.y + 740. + i as f32 * 52., r.w - 46., 39.);
                button(
                    app,
                    c,
                    rect,
                    &{
                        let __wyzar_i18n_0 = &(crate::core::size_label(bytes));
                        crate::i18n::format_text(
                            crate::i18n::tr("{name} · {}"),
                            &[
                                ("name", format!("{}", name)),
                                ("0", format!("{}", __wyzar_i18n_0)),
                            ],
                        )
                    },
                    "trash",
                    Action::Cleanup(i),
                    Style::Secondary,
                );
            }
            button(
                app,
                c,
                Rect::new(r.x + 23., r.y + 911., r.w - 46., 39.),
                crate::i18n::tr("Importar dados existentes"),
                "folder",
                Action::ImportExistingData,
                Style::Secondary,
            );
            app.hit_clip = None;
            c.unclip();
        }
        6 => appearance_screen::appearance(app, c, content),
        7 => {
            let r = Rect::new(content.x, 178., content.w, 580.);
            section(
                c,
                r,
                crate::i18n::tr("Avançado"),
                crate::i18n::tr("Perfis, segurança dos arquivos e recuperação."),
                "filter",
            );
            row_toggle(
                app,
                c,
                r.x + 24.,
                r.y + 83.,
                r.w - 48.,
                crate::i18n::tr("Recuperação por segmentos"),
                crate::i18n::tr("Finaliza segmentos e preserva checkpoints locais."),
                s.recovery,
                Toggle::Recovery,
            );
            row_toggle(
                app,
                c,
                r.x + 24.,
                r.y + 219.,
                r.w - 48.,
                crate::i18n::tr("Notificações"),
                crate::i18n::tr("Mostrar avisos e mensagens do aplicativo."),
                s.notifications,
                Toggle::Notifications,
            );
            row_toggle(
                app,
                c,
                r.x + 24.,
                r.y + 151.,
                r.w - 48.,
                crate::i18n::tr("Redimensionar na GPU"),
                crate::i18n::tr(
                    "Automático, com fallback para CPU. A cópia de quadros ainda existe.",
                ),
                s.gpu_resize,
                Toggle::GpuResize,
            );
            row_choice(
                app,
                c,
                r.x + 24.,
                r.y + 292.,
                r.w - 48.,
                crate::i18n::tr("Perfil de gravação"),
                crate::i18n::tr("Carregar perfil salvo"),
                ChoiceKind::Profile,
            );
            button(
                app,
                c,
                Rect::new(r.x + 24., r.y + 347., 292., 38.),
                crate::i18n::tr("Salvar perfil atual"),
                "save",
                Action::SaveProfile,
                Style::Secondary,
            );
            label(
                c,
                &{
                    let __wyzar_i18n_0 = &(app.pending_recovery);
                    crate::i18n::format_text(
                        crate::i18n::tr("Sessões interrompidas encontradas: {}"),
                        &[("0", format!("{}", __wyzar_i18n_0))],
                    )
                },
                r.x + 24.,
                r.y + 410.,
                r.w - 48.,
                16.,
                TEXT,
                true,
            );
            label(
                c,
                crate::i18n::tr("O último segmento em aberto pode ser perdido após uma falha."),
                r.x + 24.,
                r.y + 449.,
                r.w - 48.,
                13.,
                MUTED,
                false,
            );
            label(
                c,
                crate::i18n::tr("Os segmentos ficam no disco escolhido para seus vídeos."),
                r.x + 24.,
                r.y + 476.,
                r.w - 48.,
                13.,
                MUTED,
                false,
            );
            button(
                app,
                c,
                Rect::new(r.x + 24., r.y + 523., 224., 35.),
                crate::i18n::tr("Abrir projeto salvo"),
                "folder",
                Action::OpenProject,
                Style::Secondary,
            );
            button(
                app,
                c,
                Rect::new(r.x + 265., r.y + 523., 240., 35.),
                crate::i18n::tr("Gravações interrompidas"),
                "record",
                Action::Page(Page::Recovery),
                Style::Secondary,
            );
            button(
                app,
                c,
                Rect::new(r.x + 522., r.y + 523., 190., 35.),
                crate::i18n::tr("Diagnóstico"),
                "filter",
                Action::Page(Page::Diagnostics),
                Style::Secondary,
            );
        }
        _ => {
            let r = Rect::new(content.x, 178., content.w, 350.);
            section(
                c,
                r,
                if app.category == 8 {
                    crate::i18n::tr("Conta local")
                } else {
                    "WyzarCapture"
                },
                crate::i18n::tr("Seu conteúdo continua no seu computador."),
                if app.category == 8 { "account" } else { "help" },
            );
            c.logo(Rect::new(r.x + 24., r.y + 92., 58., 58.));
            label(
                c,
                &format!("WyzarCapture {}", crate::updates::VERSION),
                r.x + 101.,
                r.y + 91.,
                r.w - 130.,
                26.,
                TEXT,
                true,
            );
            label(
                c,
                crate::i18n::tr("Capture. Edite. Crie."),
                r.x + 101.,
                r.y + 135.,
                r.w - 130.,
                16.,
                GOLD,
                false,
            );
            label(
                c,
                crate::i18n::tr("Preferências, perfis e biblioteca são armazenados localmente."),
                r.x + 24.,
                r.y + 206.,
                r.w - 48.,
                15.,
                MUTED,
                false,
            );
            label(
                c,
                crate::i18n::tr("Nenhuma conta ou assinatura é necessária."),
                r.x + 24.,
                r.y + 243.,
                r.w - 48.,
                15.,
                MUTED,
                false,
            );
            button(
                app,
                c,
                Rect::new(r.x + 24., r.y + 289., 200., 38.),
                crate::i18n::tr("Ajuda e atalhos"),
                "help",
                Action::Page(Page::Help),
                Style::Secondary,
            );
        }
    }
    c.circle(content.x + 5., h - 53., 3., GREEN);
    label(
        c,
        crate::i18n::tr("Preferências salvas automaticamente"),
        content.x + 17.,
        h - 65.,
        480.,
        11.,
        MUTED,
        false,
    );
}
pub(super) fn capture_settings(app: &mut App, c: &mut Canvas, r: Rect) {
    section(
        c,
        r,
        crate::i18n::tr("Captura"),
        crate::i18n::tr("Configure o que será capturado."),
        "monitor",
    );
    let s = app.settings.clone();
    let mon = app
        .monitors
        .get(s.monitor)
        .map(|m| m.name.clone())
        .unwrap_or(crate::i18n::tr("Nenhum monitor").into());
    row_choice(
        app,
        c,
        r.x + 20.,
        r.y + 76.,
        r.w - 40.,
        crate::i18n::tr("Monitor"),
        &mon,
        ChoiceKind::Monitor,
    );
    row_choice(
        app,
        c,
        r.x + 20.,
        r.y + 122.,
        r.w - 40.,
        crate::i18n::tr("Contagem regressiva"),
        &if s.countdown == 0 {
            crate::i18n::tr("Sem contagem").into()
        } else {
            {
                let __wyzar_i18n_0 = &(s.countdown);
                crate::i18n::format_text(
                    crate::i18n::tr("{} segundos"),
                    &[("0", format!("{}", __wyzar_i18n_0))],
                )
            }
        },
        ChoiceKind::Countdown,
    );
    let value = s.naming.clone();
    label(
        c,
        crate::i18n::tr("Regra de nomeação"),
        r.x + 20.,
        r.y + 174.,
        (r.w - 40.) * 0.45,
        13.,
        TEXT,
        false,
    );
    field(
        app,
        c,
        Rect::new(
            r.x + 20. + (r.w - 40.) * 0.46,
            r.y + 168.,
            (r.w - 40.) * 0.54,
            33.,
        ),
        &value,
        Field::Naming,
        "Wyzar_{data}_{hora}",
    );
    button(
        app,
        c,
        Rect::new(r.x + 20., r.y + 208., 185., 23.),
        crate::i18n::tr("Modelos de nome"),
        "",
        Action::OpenChoice(
            ChoiceKind::Naming,
            Rect::new(r.x + 20., r.y + 208., 220., 23.),
        ),
        Style::Ghost,
    );
    if r.h > 280. {
        label(
            c,
            crate::i18n::tr("Escolha a fonte no painel Início ou em Nova gravação."),
            r.x + 20.,
            r.y + 239.,
            r.w - 40.,
            14.,
            MUTED,
            false,
        );
        button(
            app,
            c,
            Rect::new(r.x + 20., r.y + 285., 220., 37.),
            crate::i18n::tr("Configurar gravação"),
            "record",
            Action::NewRecording,
            Style::Secondary,
        );
    }
}
pub(super) fn video_settings(app: &mut App, c: &mut Canvas, r: Rect) {
    section(
        c,
        r,
        crate::i18n::tr("Vídeo"),
        crate::i18n::tr("Qualidade e formato da gravação."),
        "record",
    );
    let s = app.settings.clone();
    let rw = r.w - 40.;
    row_choice(
        app,
        c,
        r.x + 20.,
        r.y + 76.,
        rw,
        crate::i18n::tr("Encoder"),
        if s.hardware {
            crate::i18n::tr("Automático · hardware")
        } else {
            crate::i18n::tr("Software")
        },
        ChoiceKind::Encoder,
    );
    row_choice(
        app,
        c,
        r.x + 20.,
        r.y + 120.,
        rw,
        crate::i18n::tr("Taxa de quadros"),
        &{
            let __wyzar_i18n_0 = &(s.fps);
            crate::i18n::format_text(
                crate::i18n::tr("{} FPS"),
                &[("0", format!("{}", __wyzar_i18n_0))],
            )
        },
        ChoiceKind::Fps,
    );
    row_choice(
        app,
        c,
        r.x + 20.,
        r.y + 164.,
        rw,
        crate::i18n::tr("Resolução de saída"),
        &resolution(&s),
        ChoiceKind::Resolution,
    );
    if r.h > 270. {
        row_choice(
            app,
            c,
            r.x + 20.,
            r.y + 214.,
            rw,
            crate::i18n::tr("Bitrate"),
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
            crate::i18n::tr("Formato de saída"),
            r.x + 20.,
            r.y + 276.,
            rw * 0.45,
            14.,
            TEXT,
            false,
        );
        chip(
            c,
            Rect::new(r.x + 20. + rw * 0.46, r.y + 273., rw * 0.54, 30.),
            crate::i18n::tr("MP4 · H.264 + AAC"),
            GOLD,
        );
    }
}
