use super::*;
pub(super) fn help(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    use crate::ui::help::TOPICS;
    let x = 240.;
    let width = w - x - 24.;
    title(
        c,
        crate::i18n::tr("Central de ajuda"),
        crate::i18n::tr("Guias, atalhos e solução de problemas, disponíveis offline."),
        x + 2.,
        84.,
        width,
    );
    let nav = Rect::new(x, 178., 300., h - 232.);
    panel(c, nav);
    let search = app.help_search.clone();
    field(
        app,
        c,
        Rect::new(x + 13., 193., 274., 37.),
        &search,
        Field::HelpSearch,
        crate::i18n::tr("Buscar por áudio, corte, pasta…"),
    );
    let query = search.trim().to_lowercase();
    let filtered: Vec<_> = TOPICS
        .iter()
        .enumerate()
        .filter(|(_, t)| {
            format!(
                "{} {} {} {}",
                crate::i18n::tr(t.title),
                crate::i18n::tr(t.keywords),
                crate::i18n::tr(t.intro),
                t.steps
                    .iter()
                    .map(|s| crate::i18n::tr(s))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
            .to_lowercase()
            .contains(&query)
        })
        .map(|(i, _)| i)
        .collect();
    if !filtered.contains(&app.help_topic) {
        if let Some(i) = filtered.first() {
            app.help_topic = *i;
        }
    }
    let row_height = ((nav.bottom() - 256.) / filtered.len().max(1) as f32).min(43.);
    for (row, i) in filtered.iter().enumerate() {
        let t = &TOPICS[*i];
        button(
            app,
            c,
            Rect::new(
                x + 10.,
                244. + row as f32 * row_height,
                280.,
                (row_height - 4.).max(18.),
            ),
            crate::i18n::tr(t.title),
            t.icon,
            Action::HelpTopic(*i),
            if app.help_topic == *i {
                Style::Selected
            } else {
                Style::Ghost
            },
        );
    }
    if filtered.is_empty() {
        c.paragraph(
            crate::i18n::tr(
                "Nenhum guia encontrado. Tente palavras como áudio, pasta, corte ou atalhos.",
            ),
            Rect::new(x + 23., 270., 254., 110.),
            15.,
            MUTED,
            false,
        );
    }
    let right = x + 316.;
    let rw = w - right - 24.;
    let health = Rect::new(right, 178., rw, 97.);
    panel(c, health);
    c.icon("sparkles", Rect::new(right + 17., 195., 23., 23.), GOLD);
    label(
        c,
        crate::i18n::tr("Seu próximo passo"),
        right + 52.,
        192.,
        rw - 70.,
        15.,
        TEXT,
        true,
    );
    let (message, target) = if app.state == RecordingState::Recording {
        (
            crate::i18n::tr(
                "Gravação em andamento. Use o atalho de parar para finalizar e salvar.",
            ),
            Action::OpenSettings(4),
        )
    } else if app.state == RecordingState::Countdown {
        (
            crate::i18n::tr(
                "A contagem está sobre o monitor. O atalho de parar cancela antes de gravar.",
            ),
            Action::HelpTopic(0),
        )
    } else if app.last_error.is_some() {
        (
            crate::i18n::tr(
                "Há um erro recente. Abra os detalhes para entender a causa e o próximo passo.",
            ),
            Action::ShowErrorDetails,
        )
    } else if app.settings.microphone && app.microphones.is_empty() {
        (
            crate::i18n::tr(
                "Microfone ativo, mas sem dispositivo detectado. Atualize os dispositivos em Áudio.",
            ),
            Action::OpenSettings(2),
        )
    } else if files::free_space(&app.settings.videos)
        .is_some_and(|(free, _)| free < (app.settings.low_space_gb as u64) * 1_073_741_824)
    {
        (
            crate::i18n::tr(
                "Pouco espaço na pasta de vídeos. Escolha outro destino antes de gravar.",
            ),
            Action::OpenSettings(5),
        )
    } else if !app.project.clips.is_empty() {
        (
            crate::i18n::tr(
                "Seu projeto está disponível. Edite os clipes e salve antes de exportar.",
            ),
            Action::Page(Page::Editor),
        )
    } else {
        (
            crate::i18n::tr(
                "Comece com uma amostra curta para conferir enquadramento, áudio e pasta.",
            ),
            Action::NewRecording,
        )
    };
    c.paragraph(
        message,
        Rect::new(right + 18., 228., rw - 167., 38.),
        13.,
        MUTED,
        false,
    );
    button(
        app,
        c,
        Rect::new(health.right() - 137., 229., 120., 32.),
        crate::i18n::tr("Abrir"),
        "arrow",
        target,
        Style::Secondary,
    );
    let viewport = Rect::new(right, 290., rw, h - 344.);
    panel(c, viewport);
    if filtered.is_empty() {
        app.scroll = 0.;
        app.scroll_max = 0.;
        c.icon(
            "search",
            Rect::new(viewport.x + 35., viewport.y + 44., 37., 37.),
            GOLD,
        );
        label(
            c,
            crate::i18n::tr("Vamos encontrar sua resposta"),
            viewport.x + 35.,
            viewport.y + 109.,
            viewport.w - 70.,
            23.,
            TEXT,
            true,
        );
        c.paragraph(crate::i18n::tr("Nenhum guia corresponde a essa busca. Tente uma palavra como áudio, corte, pasta ou atalhos, ou limpe a busca para ver todos os tutoriais."),Rect::new(viewport.x+35.,viewport.y+169.,viewport.w-70.,94.),16.,MUTED,false);
        button(
            app,
            c,
            Rect::new(viewport.x + 35., viewport.y + 288., 191., 43.),
            crate::i18n::tr("Limpar busca"),
            "reset",
            Action::ResetHelpSearch,
            Style::Primary,
        );
        return;
    }
    let t = &TOPICS[app.help_topic.min(TOPICS.len() - 1)];
    let intro_height = c
        .paragraph_height(crate::i18n::tr(t.intro), rw - 50., 15., false)
        .max(36.);
    let step_heights: Vec<f32> = t
        .steps
        .iter()
        .map(|s| {
            c.paragraph_height(crate::i18n::tr(s), rw - 96., 15., false)
                .max(50.)
                + 22.
        })
        .collect();
    let tip_height = c
        .paragraph_height(crate::i18n::tr(t.tip), rw - 110., 13., false)
        .max(54.)
        + 24.;
    let steps_start = 100. + intro_height;
    let tip_start = steps_start + step_heights.iter().sum::<f32>() + 4.;
    let actions_start = tip_start + tip_height + 20.;
    let total = actions_start + 80. + if app.help_topic == 9 { 179. } else { 0. };
    app.scroll_max = (total - viewport.h).max(0.);
    app.scroll = app.scroll.clamp(0., app.scroll_max);
    c.clip(viewport.inset(1.));
    app.hit_clip = Some(viewport);
    let y = viewport.y - app.scroll;
    c.icon(t.icon, Rect::new(right + 24., y + 25., 28., 28.), GOLD);
    label(
        c,
        crate::i18n::tr(t.title),
        right + 67.,
        y + 23.,
        rw - 93.,
        25.,
        TEXT,
        true,
    );
    c.paragraph(
        crate::i18n::tr(t.intro),
        Rect::new(right + 25., y + 74., rw - 50., intro_height),
        15.,
        MUTED,
        false,
    );
    let mut sy = y + steps_start;
    for (i, step) in t.steps.iter().enumerate() {
        c.circle(right + 38., sy + 13., 13., GOLD_DARK);
        c.center(
            &(i + 1).to_string(),
            Rect::new(right + 24., sy - 1., 28., 28.),
            12.,
            GOLD,
            true,
        );
        c.paragraph(
            crate::i18n::tr(step),
            Rect::new(right + 68., sy - 2., rw - 96., step_heights[i] - 14.),
            15.,
            TEXT,
            false,
        );
        sy += step_heights[i];
    }
    let tip = Rect::new(right + 24., y + tip_start, rw - 48., tip_height);
    c.round(tip, 8., GOLD_DARK, Some(GOLD.alpha(0.23)));
    c.icon("info", Rect::new(tip.x + 14., tip.y + 16., 19., 19.), GOLD);
    c.paragraph(
        crate::i18n::tr(t.tip),
        Rect::new(tip.x + 47., tip.y + 12., tip.w - 62., tip.h - 24.),
        13.,
        TEXT,
        false,
    );
    let mut by = y + actions_start;
    if app.help_topic == 9 {
        for (i, (label, key)) in [
            (
                crate::i18n::tr("Iniciar gravação"),
                app.settings.record_hotkey,
            ),
            (
                crate::i18n::tr("Pausar / retomar"),
                app.settings.pause_hotkey,
            ),
            (
                crate::i18n::tr("Parar / cancelar contagem"),
                app.settings.stop_hotkey,
            ),
            (
                crate::i18n::tr("Capturar tela"),
                app.settings.screenshot_hotkey,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let ry = by + i as f32 * 40.;
            chip(
                c,
                Rect::new(right + 24., ry, 190., 28.),
                &crate::settings::shortcuts::label(key),
                GOLD,
            );
            super::label(c, label, right + 228., ry + 2., rw - 260., 14., TEXT, false);
        }
        by += 179.;
    }
    let (name, action) = if let Some(i) = t.category {
        (
            crate::i18n::tr("Abrir configurações"),
            Action::OpenSettings(i),
        )
    } else if app.help_topic == 5 || app.help_topic == 6 {
        (crate::i18n::tr("Abrir editor"), Action::Page(Page::Editor))
    } else {
        (crate::i18n::tr("Nova gravação"), Action::NewRecording)
    };
    button(
        app,
        c,
        Rect::new(right + 24., by, 234., 41.),
        name,
        "arrow",
        action,
        Style::Primary,
    );
    button(
        app,
        c,
        Rect::new(right + 272., by, 217., 41.),
        crate::i18n::tr("Diagnóstico local"),
        "sliders",
        Action::Page(Page::Diagnostics),
        Style::Secondary,
    );
    c.unclip();
    app.hit_clip = None;
    if app.scroll_max > 0. {
        c.round(
            Rect::new(
                viewport.right() - 6.,
                viewport.y + app.scroll / total * viewport.h,
                3.,
                viewport.h * viewport.h / total,
            ),
            1.5,
            DIM,
            None,
        );
    }
}
