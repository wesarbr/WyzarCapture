use super::*;

pub(super) fn library(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    let x = 240.;
    let rail = w - 334.;
    let width = rail - x - 20.;
    title(
        c,
        crate::i18n::tr("Biblioteca"),
        crate::i18n::tr("Vídeos, áudios e capturas das suas pastas."),
        x + 2.,
        85.,
        width,
    );
    let total_bytes: u64 = app.library.items.iter().map(|r| r.bytes).sum();
    for (i, (value, sub, icon)) in [
        (
            app.library.items.len().to_string(),
            crate::i18n::tr("Gravações"),
            "library",
        ),
        (
            size_label(total_bytes),
            crate::i18n::tr("Na biblioteca"),
            "storage",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let r = Rect::new(rail - 380. + i as f32 * 181., 89., 167., 74.);
        panel(c, r);
        c.icon(icon, Rect::new(r.x + 15., r.y + 22., 26., 26.), GOLD);
        label(c, &value, r.x + 54., r.y + 8., 105., 22., TEXT, true);
        label(c, sub, r.x + 54., r.y + 41., 105., 11., MUTED, false);
    }
    let search = Rect::new(x, 190., width - 396., 42.);
    field(
        app,
        c,
        search,
        &app.search.clone(),
        Field::Search,
        crate::i18n::tr("Buscar por nome ou tag…"),
    );
    button(
        app,
        c,
        Rect::new(search.right() + 12., 190., 123., 42.),
        if app.oldest {
            crate::i18n::tr("Mais antigos")
        } else {
            crate::i18n::tr("Mais recentes")
        },
        "",
        Action::Sort,
        Style::Secondary,
    );
    button(
        app,
        c,
        Rect::new(search.right() + 147., 190., 110., 42.),
        crate::i18n::tr("Reindexar"),
        "",
        Action::Reindex,
        Style::Secondary,
    );
    button(
        app,
        c,
        Rect::new(search.right() + 269., 190., 127., 42.),
        crate::i18n::tr("Importar"),
        "plus",
        Action::Import,
        Style::Primary,
    );
    let fav = app.library.items.iter().filter(|r| r.favorite).count();
    for (i, title) in [
        {
            let __wyzar_i18n_0 = &(app.library.items.len());
            crate::i18n::format_text(
                crate::i18n::tr("Todos ({})"),
                &[("0", format!("{}", __wyzar_i18n_0))],
            )
        },
        crate::i18n::tr("Recentes").into(),
        crate::i18n::format_text(
            crate::i18n::tr("Favoritos ({fav})"),
            &[("fav", format!("{}", fav))],
        ),
    ]
    .iter()
    .enumerate()
    {
        button(
            app,
            c,
            Rect::new(x + i as f32 * 155., 249., 143., 40.),
            title,
            "",
            Action::LibraryTab(i),
            if app.library_tab == i {
                Style::Primary
            } else {
                Style::Secondary
            },
        );
    }
    button(
        app,
        c,
        Rect::new(rail - 67., 249., 47., 40.),
        "",
        "timeline",
        Action::ListView,
        Style::Secondary,
    );
    let mut visible = app
        .library
        .visible(&app.search, app.library_tab == 2, app.oldest);
    if app.library_tab == 1 {
        visible.truncate(12);
    }
    let grid = Rect::new(x, 308., width, h - 361.);
    let cols: usize = if app.list_view { 1 } else { 3 };
    let row_height = if app.list_view { 110. } else { 259. };
    app.scroll_max = (visible.len().div_ceil(cols) as f32 * row_height - grid.h).max(0.);
    app.scroll = app.scroll.min(app.scroll_max);
    app.hit_clip = Some(grid);
    c.clip(grid);
    if visible.is_empty() {
        panel(c, Rect::new(grid.x, grid.y, grid.w, grid.h - 8.));
        c.icon(
            "library",
            Rect::new(grid.x + grid.w / 2. - 30., grid.y + 60., 60., 60.),
            GOLD,
        );
        c.center(
            if app.search.is_empty() {
                crate::i18n::tr("Nenhuma gravação ainda")
            } else {
                crate::i18n::tr("Nenhuma gravação encontrada")
            },
            Rect::new(grid.x + 20., grid.y + 158., grid.w - 40., 40.),
            24.,
            TEXT,
            true,
        );
        c.center(
            crate::i18n::tr("Grave sua tela ou importe um vídeo para começar."),
            Rect::new(grid.x + 20., grid.y + 212., grid.w - 40., 30.),
            15.,
            MUTED,
            false,
        );
        button(
            app,
            c,
            Rect::new(grid.x + grid.w / 2. - 105., grid.y + 278., 210., 45.),
            crate::i18n::tr("Nova gravação"),
            "record",
            Action::NewRecording,
            Style::Primary,
        );
    } else {
        let cols = if app.list_view { 1 } else { 3 };
        let cw = (grid.w - (cols as f32 - 1.) * 12.) / cols as f32;
        let ch = if app.list_view { 98. } else { 247. };
        for (n, &index) in visible.iter().enumerate() {
            let r = Rect::new(
                x + (n % cols) as f32 * (cw + 12.),
                grid.y + (n / cols) as f32 * (ch + 12.) - app.scroll,
                cw,
                ch,
            );
            if r.bottom() < grid.y || r.y > grid.bottom() {
                continue;
            }
            let rec = app.library.items[index].clone();
            panel(c, r);
            if app.selected == Some(index) {
                c.outline(r, 10., GOLD, 2.);
            }
            let thumb = if app.list_view {
                Rect::new(r.x + 9., r.y + 9., 142., 80.)
            } else {
                Rect::new(r.x + 9., r.y + 9., r.w - 18., 128.)
            };
            c.clip(thumb);
            if let Some(f) = &rec.thumbnail {
                c.frame(f, thumb);
            } else {
                c.wallpaper(thumb, 0.7);
            }
            c.unclip();
            chip(
                c,
                Rect::new(thumb.right() - 61., thumb.bottom() - 28., 55., 23.),
                &duration_label(rec.duration),
                TEXT,
            );
            let (tx, ty, tw) = if app.list_view {
                (r.x + 168., r.y + 15., r.w - 190.)
            } else {
                (r.x + 12., r.y + 146., r.w - 24.)
            };
            label(c, &rec.title, tx, ty, tw, 15., TEXT, true);
            let tags = if rec.tags.is_empty() {
                crate::i18n::tr("Vídeo").into()
            } else {
                rec.tags.join(" · ")
            };
            label(c, &tags, tx, ty + 30., tw, 11., MUTED, false);
            label(
                c,
                &if rec
                    .path
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("wav"))
                {
                    {
                        let __wyzar_i18n_0 = &(size_label(rec.bytes));
                        crate::i18n::format_text(
                            crate::i18n::tr("WAV · 48 kHz · {}"),
                            &[("0", format!("{}", __wyzar_i18n_0))],
                        )
                    }
                } else {
                    format!("{} × {} · {}", rec.width, rec.height, size_label(rec.bytes))
                },
                tx,
                ty + 56.,
                tw,
                11.,
                MUTED,
                false,
            );
            if !app.list_view {
                label(
                    c,
                    &files::timestamp_label(rec.modified),
                    r.x + 12.,
                    r.bottom() - 29.,
                    r.w - 128.,
                    10.,
                    MUTED,
                    false,
                );
                chip(
                    c,
                    Rect::new(r.right() - 108., r.bottom() - 30., 96., 21.),
                    if rec.missing {
                        crate::i18n::tr("Ausente")
                    } else if rec.problem.is_some() {
                        crate::i18n::tr("Erro")
                    } else if rec.duration <= 0. {
                        crate::i18n::tr("Indexando")
                    } else {
                        crate::i18n::tr("Concluída")
                    },
                    if rec.missing || rec.problem.is_some() {
                        RED
                    } else if rec.duration <= 0. {
                        GOLD
                    } else {
                        GREEN
                    },
                );
            }
            app.hit(
                Rect::new(
                    r.x,
                    r.y.max(grid.y),
                    r.w,
                    r.bottom().min(grid.bottom()) - r.y.max(grid.y),
                ),
                Action::Select(index),
                &rec.title,
            );
            let star = Rect::new(thumb.right() - 33., thumb.y + 5., 27., 27.);
            c.round(star, 5., BG.alpha(0.8), Some(BORDER));
            c.icon(
                "star",
                star.inset(5.),
                if rec.favorite { GOLD } else { MUTED },
            );
            app.hit(
                star,
                Action::Favorite(index),
                crate::i18n::tr("Marcar ou desmarcar favorito"),
            );
        }
    }
    c.unclip();
    app.hit_clip = None;
    if app.scroll_max > 0. {
        let thumb = (grid.h * grid.h / (grid.h + app.scroll_max)).max(32.);
        c.round(
            Rect::new(grid.right() - 4., grid.y, 3., grid.h),
            2.,
            BORDER,
            None,
        );
        c.round(
            Rect::new(
                grid.right() - 4.,
                grid.y + (grid.h - thumb) * app.scroll / app.scroll_max,
                3.,
                thumb,
            ),
            2.,
            GOLD,
            None,
        );
    }
    c.line(rail, 64., rail, h - 36., BORDER, 1.);
    let rx = rail + 19.;
    let rw = w - rx - 19.;
    let selected = app.selected.and_then(|i| app.library.items.get(i)).cloned();
    label(
        c,
        selected
            .as_ref()
            .map(|r| r.title.as_str())
            .unwrap_or(crate::i18n::tr("Detalhes da gravação")),
        rx,
        91.,
        rw,
        21.,
        TEXT,
        true,
    );
    let preview = Rect::new(rx, 144., rw, 177.);
    panel(c, preview);
    c.clip(preview.inset(2.));
    if let Some(f) = selected.as_ref().and_then(|r| r.thumbnail.as_ref()) {
        c.frame(f, preview.inset(2.));
    } else {
        c.wallpaper(preview.inset(2.), 0.8);
    }
    c.unclip();
    c.circle(
        preview.x + preview.w / 2.,
        preview.y + preview.h / 2.,
        26.,
        BG.alpha(0.85),
    );
    c.icon(
        "play",
        Rect::new(
            preview.x + preview.w / 2. - 11.,
            preview.y + preview.h / 2. - 12.,
            24.,
            24.,
        ),
        TEXT,
    );
    app.hit(
        preview,
        Action::PlaySelected,
        crate::i18n::tr("Reproduzir gravação"),
    );
    label(
        c,
        crate::i18n::tr("Informações"),
        rx,
        344.,
        rw,
        18.,
        TEXT,
        true,
    );
    if let Some(rec) = selected {
        for (i, (name, value, icon)) in [
            (
                crate::i18n::tr("Arquivo"),
                rec.path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into(),
                "library",
            ),
            (
                if rec.width == 0 {
                    crate::i18n::tr("Formato")
                } else {
                    crate::i18n::tr("Resolução")
                },
                if rec.width == 0 {
                    crate::i18n::tr("WAV · PCM 16 bits").into()
                } else {
                    format!("{} × {}", rec.width, rec.height)
                },
                "monitor",
            ),
            (
                if rec.width == 0 {
                    crate::i18n::tr("Amostragem")
                } else {
                    crate::i18n::tr("Taxa de quadros")
                },
                if rec.width == 0 {
                    crate::i18n::tr("48 kHz · estéreo").into()
                } else {
                    {
                        let __wyzar_i18n_0 = &(rec.fps);
                        crate::i18n::format_text(
                            crate::i18n::tr("{:.0} FPS"),
                            &[("0:.0", format!("{:.0}", __wyzar_i18n_0))],
                        )
                    }
                },
                "fps",
            ),
            (crate::i18n::tr("Tamanho"), size_label(rec.bytes), "storage"),
            (
                crate::i18n::tr("Duração"),
                duration_label(rec.duration),
                "clock",
            ),
            (
                crate::i18n::tr("Data do arquivo"),
                files::timestamp_label(rec.modified),
                "calendar",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let y = 390. + i as f32 * 37.;
            c.icon(icon, Rect::new(rx + 2., y + 2., 17., 17.), MUTED);
            label(c, name, rx + 31., y - 2., 118., 12., MUTED, false);
            label(c, &value, rx + 151., y - 2., rw - 151., 12., TEXT, false);
        }
        button(
            app,
            c,
            Rect::new(rx, 625., rw, 34.),
            &if rec.tags.is_empty() {
                crate::i18n::tr("Adicionar tags").into()
            } else {
                rec.tags.join(", ")
            },
            "tag",
            Action::Tags,
            Style::Secondary,
        );
        let actions = [
            (
                crate::i18n::tr("Reproduzir"),
                "play",
                Action::PlaySelected,
                Style::Primary,
            ),
            (
                if rec.missing {
                    crate::i18n::tr("Reimportar")
                } else {
                    crate::i18n::tr("Renomear")
                },
                "edit",
                if rec.missing {
                    Action::Relink
                } else {
                    Action::Rename
                },
                Style::Secondary,
            ),
            (
                crate::i18n::tr("Abrir pasta"),
                "folder",
                Action::OpenSelectedFolder,
                Style::Secondary,
            ),
            (
                crate::i18n::tr("Editar"),
                "edit",
                Action::EditSelected,
                Style::Secondary,
            ),
            (
                crate::i18n::tr("Exportar"),
                "export",
                Action::Export,
                Style::Secondary,
            ),
            (
                crate::i18n::tr("Excluir"),
                "trash",
                Action::Delete,
                Style::Danger,
            ),
        ];
        label(c, crate::i18n::tr("Ações"), rx, 683., rw, 18., TEXT, true);
        for (i, (name, icon, a, style)) in actions.into_iter().enumerate() {
            let r = Rect::new(
                rx + (i % 3) as f32 * (rw + 12.) / 3.,
                722. + (i / 3) as f32 * 59.,
                (rw - 24.) / 3.,
                51.,
            );
            button(app, c, r, "", "", a.clone(), style);
            let col = if matches!(style, Style::Primary) {
                BG
            } else if matches!(style, Style::Danger) {
                RED
            } else {
                TEXT
            };
            c.icon(
                icon,
                Rect::new(r.x + (r.w - 20.) / 2., r.y + 6., 20., 20.),
                col,
            );
            c.center(name, Rect::new(r.x, r.y + 29., r.w, 17.), 11., col, false);
        }
    } else {
        label(
            c,
            crate::i18n::tr("Selecione uma gravação"),
            rx + 8.,
            399.,
            rw - 16.,
            15.,
            TEXT,
            true,
        );
        label(
            c,
            crate::i18n::tr("para ver informações e ações."),
            rx + 8.,
            431.,
            rw - 16.,
            13.,
            MUTED,
            false,
        );
    }
}
