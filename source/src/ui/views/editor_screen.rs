use super::*;

pub(super) fn editor(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    let x = 239.;
    let width = w - x - 23.;
    title(
        c,
        crate::i18n::tr("Editor de vídeo"),
        crate::i18n::tr("Corte, ajuste o áudio, aplique efeitos e exporte um novo MP4."),
        x + 2.,
        83.,
        width,
    );
    button(
        app,
        c,
        Rect::new(w - 477., 91., 43., 43.),
        "",
        "undo",
        Action::Undo,
        Style::Secondary,
    );
    if let Some(hit) = app
        .hits
        .last_mut()
        .filter(|h| matches!(h.action, Action::Undo))
    {
        hit.label = crate::i18n::tr("Desfazer · Ctrl+Z").into();
    }
    button(
        app,
        c,
        Rect::new(w - 425., 91., 43., 43.),
        "",
        "redo",
        Action::Redo,
        Style::Secondary,
    );
    if let Some(hit) = app
        .hits
        .last_mut()
        .filter(|h| matches!(h.action, Action::Redo))
    {
        hit.label = crate::i18n::tr("Refazer · Ctrl+Shift+Z").into();
    }
    button(
        app,
        c,
        Rect::new(w - 371., 91., 188., 43.),
        crate::i18n::tr("Salvar projeto"),
        "save",
        Action::SaveProject,
        Style::Secondary,
    );
    button(
        app,
        c,
        Rect::new(w - 170., 91., 147., 43.),
        crate::i18n::tr("Exportar"),
        "export",
        Action::Export,
        Style::Primary,
    );
    let media = Rect::new(x, 169., 254., 390.);
    panel(c, media);
    button(
        app,
        c,
        Rect::new(media.x + 128., media.y + 11., 112., 30.),
        crate::i18n::tr("Trilhas"),
        "audio",
        Action::AudioTracks,
        Style::Secondary,
    );
    label(
        c,
        crate::i18n::tr("Mídia"),
        media.x + 17.,
        media.y + 14.,
        100.,
        15.,
        GOLD,
        true,
    );
    c.line(
        media.x + 13.,
        media.y + 49.,
        media.x + 63.,
        media.y + 49.,
        GOLD,
        2.,
    );
    button(
        app,
        c,
        Rect::new(media.x + 13., media.y + 67., media.w - 26., 38.),
        crate::i18n::tr("Importar mídia"),
        "plus",
        Action::Import,
        Style::Secondary,
    );
    button(
        app,
        c,
        Rect::new(media.x + 13., media.y + 117., media.w - 26., 33.),
        crate::i18n::tr("Abrir projeto"),
        "folder",
        Action::OpenProject,
        Style::Ghost,
    );
    let clips = app.project.clips.clone();
    for clip in &clips {
        for ext in ["mic.wav", "system.wav"] {
            let path = clip.source.with_extension(ext);
            if !app.waveforms.contains_key(&path) {
                let data = crate::audio::waveform(&path, 350).unwrap_or_default();
                app.waveforms.insert(path, data);
            }
        }
    }
    if clips.is_empty() {
        c.icon(
            "library",
            Rect::new(media.x + 99., media.y + 193., 56., 56.),
            DIM,
        );
        c.center(
            crate::i18n::tr("Sua mídia começa aqui"),
            Rect::new(media.x + 10., media.y + 282., media.w - 20., 28.),
            14.,
            MUTED,
            false,
        );
    } else {
        for (i, clip) in clips.iter().take(4).enumerate() {
            let r = Rect::new(
                media.x + 13. + (i % 2) as f32 * 118.,
                media.y + 173. + (i / 2) as f32 * 103.,
                110.,
                90.,
            );
            let thumb = Rect::new(r.x, r.y, r.w, 64.);
            if let Some(f) = app
                .library
                .items
                .iter()
                .find(|r| r.path == clip.source)
                .and_then(|r| r.thumbnail.as_ref())
            {
                c.frame(f, thumb);
            } else {
                c.wallpaper(thumb, 1.);
            }
            if app.project.selected == i {
                c.outline(thumb, 4., GOLD, 2.);
            }
            label(
                c,
                &clip
                    .source
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
                r.x,
                r.y + 66.,
                r.w,
                11.,
                TEXT,
                false,
            );
            app.hit(
                r,
                Action::SelectClip(i),
                crate::i18n::tr("Selecionar clipe"),
            );
        }
    }
    let prop = Rect::new(w - 325., 169., 302., h - 221.);
    let preview = Rect::new(
        media.right() + 13.,
        169.,
        prop.x - media.right() - 26.,
        390.,
    );
    panel(c, preview);
    let screen = Rect::new(
        preview.x + 9.,
        preview.y + 10.,
        preview.w - 18.,
        preview.h - 83.,
    );
    c.clip(screen);
    if let Some(f) = &app.editor_frame {
        let ar = f.width as f32 / f.height as f32;
        let mut dest = screen;
        if screen.w / screen.h > ar {
            dest.w = screen.h * ar;
            dest.x += (screen.w - dest.w) / 2.;
        } else {
            dest.h = screen.w / ar;
            dest.y += (screen.h - dest.h) / 2.;
        }
        c.frame(f, dest);
    } else {
        c.wallpaper(screen, 0.65);
        c.center(
            if app.project.clips.is_empty() {
                crate::i18n::tr("Importe um vídeo para começar")
            } else {
                crate::i18n::tr("Carregando prévia…")
            },
            Rect::new(screen.x, screen.y + screen.h / 2. - 20., screen.w, 40.),
            18.,
            TEXT,
            true,
        );
    }
    c.unclip();
    chip(
        c,
        Rect::new(screen.right() - 128., screen.y + 10., 116., 28.),
        &{
            let __wyzar_i18n_0 = &(app.settings.height);
            let __wyzar_i18n_1 = &(app.settings.fps);
            crate::i18n::format_text(
                crate::i18n::tr("{}p · {} FPS"),
                &[
                    ("0", format!("{}", __wyzar_i18n_0)),
                    ("1", format!("{}", __wyzar_i18n_1)),
                ],
            )
        },
        TEXT,
    );
    let progress_y = screen.bottom() + 13.;
    progress(
        c,
        Rect::new(screen.x, progress_y, screen.w, 4.),
        (app.project.playhead / app.project.duration().max(0.01)) as f32,
        GOLD,
    );
    for n in 0..100 {
        app.hit(
            Rect::new(
                screen.x + screen.w * n as f32 / 100.,
                progress_y - 4.,
                screen.w / 100. + 1.,
                14.,
            ),
            Action::Seek(app.project.duration() * n as f64 / 100.),
            crate::i18n::tr("Navegar no vídeo"),
        );
    }
    label(
        c,
        &format!(
            "{}  /  {}",
            crate::core::timecode(app.project.playhead),
            crate::core::timecode(app.project.duration())
        ),
        preview.x + 16.,
        preview.bottom() - 45.,
        200.,
        12.,
        MUTED,
        false,
    );
    button(
        app,
        c,
        Rect::new(
            preview.x + preview.w / 2. - 23.,
            preview.bottom() - 50.,
            46.,
            40.,
        ),
        "",
        if app.playing { "pause" } else { "play" },
        Action::Play,
        Style::Ghost,
    );
    panel(c, prop);
    label(
        c,
        crate::i18n::tr("Propriedades"),
        prop.x + 17.,
        prop.y + 14.,
        prop.w - 34.,
        16.,
        GOLD,
        true,
    );
    c.line(
        prop.x + 13.,
        prop.y + 51.,
        prop.right() - 13.,
        prop.y + 51.,
        BORDER,
        1.,
    );
    label(
        c,
        crate::i18n::tr("VÍDEO SELECIONADO"),
        prop.x + 17.,
        prop.y + 67.,
        prop.w - 34.,
        11.,
        DIM,
        true,
    );
    let name = app
        .project
        .clips
        .get(app.project.selected)
        .map(|cl| {
            cl.source
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        })
        .unwrap_or(crate::i18n::tr("Nenhum clipe").into());
    label(
        c,
        &name,
        prop.x + 17.,
        prop.y + 98.,
        prop.w - 34.,
        14.,
        TEXT,
        true,
    );
    let selected = app.project.clips.get(app.project.selected).cloned();
    if let Some(clip) = &selected {
        label(
            c,
            &format!(
                "{} → {} · {}",
                crate::core::timecode(clip.start),
                crate::core::timecode(clip.end),
                crate::core::timecode(clip.duration())
            ),
            prop.x + 17.,
            prop.y + 125.,
            prop.w - 34.,
            11.,
            MUTED,
            false,
        );
    }
    for (i, (title, value, icon, action)) in [
        (
            crate::i18n::tr("Velocidade"),
            format!("{:.2}x", selected.as_ref().map(|c| c.speed).unwrap_or(1.)),
            "speed",
            Action::Speed,
        ),
        (
            crate::i18n::tr("Volume"),
            format!(
                "{:.0}%",
                selected.as_ref().map(|c| c.volume).unwrap_or(1.) * 100.
            ),
            "volume",
            Action::Volume,
        ),
        (
            crate::i18n::tr("Corte da tela"),
            if app.project.crop {
                crate::i18n::tr("Ampliado")
            } else {
                crate::i18n::tr("Original")
            }
            .into(),
            "crop",
            Action::Crop,
        ),
        (
            crate::i18n::tr("Fade"),
            if app.project.fade {
                crate::i18n::tr("Ativo")
            } else {
                crate::i18n::tr("Desativado")
            }
            .into(),
            "fade",
            Action::Fade,
        ),
        (
            crate::i18n::tr("Blur central"),
            if app.project.blur {
                crate::i18n::tr("Ativo")
            } else {
                crate::i18n::tr("Desativado")
            }
            .into(),
            "blur",
            Action::Blur,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let r = Rect::new(
            prop.x + 12.,
            prop.y + 175. + i as f32 * 63.,
            prop.w - 24.,
            59.,
        );
        panel(c, r);
        c.icon(icon, Rect::new(r.x + 12., r.y + 18., 21., 21.), TEXT);
        label(c, title, r.x + 45., r.y + 15., 140., 13., TEXT, true);
        c.text(
            &value,
            Rect::new(r.right() - 94., r.y, 82., r.h),
            12.,
            GOLD,
            true,
        );
        if !clips.is_empty() {
            app.hit(r, action, title);
        }
        if i < 2 && !clips.is_empty() {
            let sr = Rect::new(r.x + 45., r.y + 37., r.w - 151., 17.);
            let value = if i == 0 {
                ((selected.as_ref().map(|c| c.speed).unwrap_or(1.) - 0.25) / 1.75) as f32
            } else {
                selected.as_ref().map(|c| c.volume).unwrap_or(1.) / 1.5
            };
            progress(c, Rect::new(sr.x, sr.y + 7., sr.w, 4.), value, GOLD);
            c.circle(sr.x + sr.w * value.clamp(0., 1.), sr.y + 9., 5., TEXT);
            app.hit(
                sr,
                Action::Slider(
                    if i == 0 {
                        SliderKind::Speed
                    } else {
                        SliderKind::Volume
                    },
                    sr,
                ),
                title,
            );
        }
    }
    label(
        c,
        &{
            let __wyzar_i18n_0 = &(clips.len());
            let __wyzar_i18n_1 = &(app.project.text.len());
            crate::i18n::format_text(
                crate::i18n::tr("{} clipe(s) · {} legenda(s)"),
                &[
                    ("0", format!("{}", __wyzar_i18n_0)),
                    ("1", format!("{}", __wyzar_i18n_1)),
                ],
            )
        },
        prop.x + 17.,
        prop.y + 508.,
        prop.w - 34.,
        12.,
        MUTED,
        false,
    );
    c.paragraph(if app.project.tracks_active { crate::i18n::tr("Mix independente ativo. Abra Trilhas para mute, solo, ganho, offset, fade e cortes. Webcam integrada ao vídeo.") } else { crate::i18n::tr("Áudio incorporado ao vídeo. Abra Trilhas para ativar a mixagem dos WAVs independentes.") },Rect::new(prop.x+17.,prop.y+541.,prop.w-34.,79.),12.,DIM,false);
    let toolbar = Rect::new(x, 573., prop.x - x - 13., 55.);
    panel(c, toolbar);
    let tools = [
        (crate::i18n::tr("Cortar início"), "edit", Action::Trim),
        (crate::i18n::tr("Dividir"), "split", Action::Split),
        (crate::i18n::tr("Excluir"), "trash", Action::RemoveClip),
        (crate::i18n::tr("Velocidade"), "speed", Action::Speed),
        (crate::i18n::tr("Texto"), "text", Action::Text),
        (crate::i18n::tr("Seta"), "arrow", Action::Arrow),
        (crate::i18n::tr("Blur"), "blur", Action::Blur),
        (crate::i18n::tr("Crop"), "crop", Action::Crop),
        (crate::i18n::tr("Volume"), "volume", Action::Volume),
        (crate::i18n::tr("Fade"), "fade", Action::Fade),
    ];
    let tw = (toolbar.w - 16.) / 10.;
    for (i, (name, icon, a)) in tools.into_iter().enumerate() {
        let r = Rect::new(toolbar.x + 8. + i as f32 * tw, toolbar.y + 5., tw - 4., 45.);
        if app.hover == Some(app.hits.len()) {
            c.round(r, 5., HOVER, None);
        }
        let active = match &a {
            Action::Arrow => app.project.arrow,
            Action::Blur => app.project.blur,
            Action::Crop => app.project.crop,
            Action::Fade => app.project.fade,
            Action::Text => !app.project.text.is_empty(),
            _ => false,
        };
        if active {
            c.round(r, 5., GOLD_DARK, Some(GOLD.alpha(0.3)));
        }
        c.icon(
            icon,
            Rect::new(r.x + (r.w - 19.) / 2., r.y + 2., 19., 19.),
            if clips.is_empty() {
                DIM
            } else if active {
                GOLD
            } else {
                TEXT
            },
        );
        c.center(name, Rect::new(r.x, r.y + 25., r.w, 17.), 10., MUTED, false);
        if !clips.is_empty() {
            app.hit(r, a, name);
        }
    }
    let timeline = Rect::new(x, 641., prop.x - x - 13., h - 695.);
    panel(c, timeline);
    let label_w = 170.;
    let tx = timeline.x + label_w;
    let track_w = timeline.w - label_w - 10.;
    let duration = app.project.duration().max(1.);
    let ruler = Rect::new(tx, timeline.y + 6., track_w, 31.);
    for i in 0..7 {
        let xx = ruler.x + ruler.w * i as f32 / 6.;
        c.line(xx, ruler.y + 23., xx, ruler.y + 31., BORDER, 1.);
        label(
            c,
            &if duration < 10. {
                crate::core::timecode(duration * i as f64 / 6.)
            } else {
                duration_label(duration * i as f64 / 6.)
            },
            if i == 6 { xx - 68. } else { xx + 3. },
            ruler.y,
            78.,
            10.,
            DIM,
            false,
        );
    }
    for (i, (name, icon, color)) in [
        (crate::i18n::tr("Vídeo"), "play", GOLD),
        (crate::i18n::tr("Microfone"), "microphone", GREEN),
        (crate::i18n::tr("Áudio do sistema"), "audio", BLUE),
        (crate::i18n::tr("Webcam"), "webcam", PURPLE),
    ]
    .into_iter()
    .enumerate()
    {
        let y = timeline.y + 44. + i as f32 * 41.;
        c.line(
            timeline.x + 1.,
            y - 5.,
            timeline.right() - 1.,
            y - 5.,
            BORDER,
            1.,
        );
        c.icon(icon, Rect::new(timeline.x + 15., y + 6., 19., 19.), MUTED);
        label(c, name, timeline.x + 45., y + 3., 122., 12., TEXT, false);
        if i == 0 {
            let mut offset = 0.;
            for (index, clip) in clips.iter().enumerate() {
                let cr = Rect::new(
                    tx + track_w * (offset / duration) as f32,
                    y,
                    track_w * (clip.duration() / duration) as f32 - 3.,
                    30.,
                );
                c.round(
                    cr,
                    5.,
                    Color::rgb(0x242b31),
                    Some(if index == app.project.selected {
                        GOLD
                    } else {
                        BORDER
                    }),
                );
                if let Some(frame) = app
                    .library
                    .items
                    .iter()
                    .find(|r| r.path == clip.source)
                    .and_then(|r| r.thumbnail.as_ref())
                {
                    c.clip(cr.inset(2.));
                    for tile in 0..((cr.w / 80.).ceil() as usize).min(20) {
                        c.frame(
                            frame,
                            Rect::new(cr.x + tile as f32 * 80., cr.y + 2., 80., cr.h - 4.),
                        );
                    }
                    c.fill(cr.inset(2.), BG.alpha(0.35));
                    c.unclip();
                }
                c.text(
                    &clip
                        .source
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy(),
                    cr.inset(7.),
                    11.,
                    TEXT,
                    false,
                );
                app.hit(
                    cr,
                    Action::SelectClip(index),
                    crate::i18n::tr("Clipe na timeline"),
                );
                offset += clip.duration();
            }
        } else if i < 3 && app.project.tracks_active {
            for (index, track) in app
                .project
                .audio_tracks
                .clone()
                .iter()
                .enumerate()
                .filter(|(_, t)| if i == 1 { t.kind != 1 } else { t.kind == 1 })
            {
                let cr = Rect::new(
                    tx + track_w * (track.offset.max(0.) / duration) as f32,
                    y,
                    (track_w * (track.duration() / duration) as f32).min(track_w),
                    28.,
                );
                c.clip(Rect::new(tx, y, track_w, 30.));
                c.round(
                    cr,
                    5.,
                    color.alpha(if track.mute { 0.05 } else { 0.18 }),
                    Some(color.alpha(0.55)),
                );
                c.text(
                    &format!(
                        "{} · {:.0}%{}",
                        track
                            .source
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy(),
                        track.gain * 100.,
                        if track.mute {
                            crate::i18n::tr(" · mute")
                        } else if track.solo {
                            crate::i18n::tr(" · solo")
                        } else {
                            ""
                        }
                    ),
                    cr.inset(5.),
                    11.,
                    color,
                    false,
                );
                c.unclip();
                app.hit(
                    cr,
                    Action::SelectTrack(index),
                    crate::i18n::tr("Selecionar trilha · abra Trilhas para editar"),
                );
            }
        } else {
            let source = clips.first().map(|c| {
                c.source
                    .with_extension(if i == 1 { "mic.wav" } else { "system.wav" })
            });
            let exists = source.is_some_and(|p| p.exists());
            if i < 3 && exists {
                c.round(
                    Rect::new(tx, y, track_w, 28.),
                    5.,
                    color.alpha(0.17),
                    Some(color.alpha(0.5)),
                );
                label(
                    c,
                    if i == 1 {
                        crate::i18n::tr("Microfone · áudio preservado")
                    } else {
                        crate::i18n::tr("Sistema · áudio preservado")
                    },
                    tx + 12.,
                    y + 2.,
                    track_w - 24.,
                    11.,
                    color,
                    false,
                );
                if let Some(peaks) = clips.first().and_then(|clip| {
                    app.waveforms.get(&clip.source.with_extension(if i == 1 {
                        "mic.wav"
                    } else {
                        "system.wav"
                    }))
                }) {
                    for (n, peak) in peaks.iter().enumerate() {
                        let xx = tx + track_w * n as f32 / peaks.len().max(1) as f32;
                        let amp = (peak.sqrt() * 12.).max(0.5);
                        c.line(xx, y + 14. - amp, xx, y + 14. + amp, color.alpha(0.8), 1.);
                    }
                }
            } else {
                label(
                    c,
                    if i == 3 {
                        crate::i18n::tr("Webcam composta no vídeo, quando ativada")
                    } else {
                        crate::i18n::tr("Áudio incorporado ao vídeo")
                    },
                    tx + 10.,
                    y + 2.,
                    track_w - 20.,
                    10.,
                    DIM,
                    false,
                );
            }
        }
    }
    if !clips.is_empty() {
        let px = tx + track_w * (app.project.playhead / duration) as f32;
        c.line(px, timeline.y + 20., px, timeline.bottom() - 2., GOLD, 1.5);
        c.circle(px, timeline.y + 21., 5., GOLD);
        for i in 0..150 {
            app.hit(
                Rect::new(
                    tx + track_w * i as f32 / 150.,
                    timeline.y + 2.,
                    track_w / 150. + 1.,
                    34.,
                ),
                Action::Seek(duration * i as f64 / 150.),
                crate::i18n::tr("Posicionar cursor"),
            );
        }
    }
    if let Some(p) = app.export_progress {
        let r = Rect::new(prop.x + 15., prop.bottom() - 85., prop.w - 30., 64.);
        label(
            c,
            &{
                let __wyzar_i18n_0 = &(p * 100.);
                crate::i18n::format_text(
                    crate::i18n::tr("Exportando… {:.0}%"),
                    &[("0:.0", format!("{:.0}", __wyzar_i18n_0))],
                )
            },
            r.x,
            r.y,
            r.w,
            13.,
            TEXT,
            true,
        );
        progress(c, Rect::new(r.x, r.y + 35., r.w, 8.), p, GOLD);
    }
}
