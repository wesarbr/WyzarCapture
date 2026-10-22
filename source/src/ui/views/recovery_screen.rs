use super::*;
pub(super) fn recovery(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    title(
        c,
        crate::i18n::tr("Gravações interrompidas encontradas"),
        crate::i18n::tr(
            "Recupere os segmentos finalizados. Seus arquivos originais são preservados.",
        ),
        242.,
        84.,
        w - 268.,
    );
    let area = Rect::new(240., 180., w - 266., h - 240.);
    if app.recovery_items.is_empty() {
        panel(c, Rect::new(area.x, area.y, area.w, 180.));
        label(
            c,
            crate::i18n::tr("Nenhuma sessão recuperável encontrada"),
            area.x + 26.,
            area.y + 28.,
            area.w - 52.,
            22.,
            TEXT,
            true,
        );
        label(
            c,
            crate::i18n::tr(
                "Diários antigos e sessões sem segmentos finalizados podem exigir análise manual.",
            ),
            area.x + 26.,
            area.y + 83.,
            area.w - 52.,
            14.,
            MUTED,
            false,
        );
        button(
            app,
            c,
            Rect::new(area.x + 26., area.y + 123., 200., 36.),
            crate::i18n::tr("Ir para biblioteca"),
            "library",
            Action::Page(Page::Library),
            Style::Secondary,
        );
        return;
    }
    c.clip(area);
    let items = app.recovery_items.clone();
    for (index, item) in items.iter().enumerate() {
        let card = Rect::new(
            area.x,
            area.y + index as f32 * 174. - app.scroll,
            area.w - 14.,
            158.,
        );
        panel(c, card);
        c.icon(
            "record",
            Rect::new(card.x + 24., card.y + 24., 26., 26.),
            GOLD,
        );
        label(
            c,
            &item
                .output
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            card.x + 66.,
            card.y + 20.,
            card.w - 90.,
            18.,
            TEXT,
            true,
        );
        label(
            c,
            &item.detail,
            card.x + 24.,
            card.y + 65.,
            card.w - 48.,
            13.,
            MUTED,
            false,
        );
        for (n, (text, icon, action, style)) in [
            (
                crate::i18n::tr("Recuperar"),
                "save",
                Action::Recover(index),
                Style::Primary,
            ),
            (
                crate::i18n::tr("Abrir pasta"),
                "folder",
                Action::RecoveryFolder(index),
                Style::Secondary,
            ),
            (
                crate::i18n::tr("Ver detalhes"),
                "help",
                Action::RecoveryDetails(index),
                Style::Secondary,
            ),
            (
                crate::i18n::tr("Descartar"),
                "trash",
                Action::DiscardRecovery(index),
                Style::Danger,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let r = Rect::new(card.x + 24. + n as f32 * 160., card.y + 105., 148., 34.);
            if r.y >= area.y && r.bottom() <= area.bottom() && !app.recovering {
                button(app, c, r, text, icon, action, style);
            }
        }
    }
    c.unclip();
    app.scroll_max = (items.len() as f32 * 174. - area.h).max(0.);
    label(
        c,
        if app.recovering {
            crate::i18n::tr("Montando e verificando o vídeo. Aguarde a conclusão antes de fechar.")
        } else {
            crate::i18n::tr(
                "O segmento em aberto no momento da falha pode estar ausente no resultado.",
            )
        },
        242.,
        h - 59.,
        w - 270.,
        12.,
        GOLD,
        false,
    );
}
pub(super) fn details(app: &mut App, c: &mut Canvas, w: f32, h: f32, index: usize, discard: bool) {
    let r = modals::modal_panel(c, w, h, 850., 330.);
    let Some(item) = app.recovery_items.get(index).cloned() else {
        return;
    };
    label(
        c,
        if discard {
            crate::i18n::tr("Descartar arquivos temporários?")
        } else {
            crate::i18n::tr("Detalhes da sessão interrompida")
        },
        r.x + 28.,
        r.y + 24.,
        r.w - 56.,
        24.,
        TEXT,
        true,
    );
    label(
        c,
        &item.detail,
        r.x + 28.,
        r.y + 87.,
        r.w - 56.,
        14.,
        MUTED,
        false,
    );
    c.paragraph(
        &{
            let __wyzar_i18n_0 = &(item.output.display());
            crate::i18n::format_text(
                crate::i18n::tr("Destino original: {}"),
                &[("0", format!("{}", __wyzar_i18n_0))],
            )
        },
        Rect::new(r.x + 28., r.y + 126., r.w - 56., 65.),
        13.,
        TEXT,
        false,
    );
    c.paragraph(if discard { crate::i18n::tr("O diário, os segmentos e o áudio auxiliar da sessão interrompida serão enviados à Lixeira. Um vídeo final já salvo e seus auxiliares serão preservados.") } else { crate::i18n::tr("A recuperação usa apenas segmentos finalizados e decodificáveis, sem prometer reconstrução do trecho em aberto. O resultado será salvo na sua pasta de vídeos atual.") }, Rect::new(r.x+28., r.y+199., r.w-56., 62.), 13., MUTED, false);
    button(
        app,
        c,
        Rect::new(r.right() - 320., r.bottom() - 57., 134., 36.),
        crate::i18n::tr("Fechar"),
        "",
        Action::CloseModal,
        Style::Secondary,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 171., r.bottom() - 57., 143., 36.),
        if discard {
            crate::i18n::tr("Mover à Lixeira")
        } else {
            crate::i18n::tr("Abrir pasta")
        },
        if discard { "trash" } else { "folder" },
        if discard {
            Action::ConfirmDiscardRecovery(index)
        } else {
            Action::RecoveryFolder(index)
        },
        if discard {
            Style::Danger
        } else {
            Style::Primary
        },
    );
}
