use super::*;
pub(super) fn diagnostics(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    title(
        c,
        crate::i18n::tr("Diagnóstico de gravação"),
        crate::i18n::tr("Métricas locais da última sessão. Nenhum dado é enviado pela internet."),
        242.,
        84.,
        w - 268.,
    );
    let r = Rect::new(240., 180., w - 266., h - 280.);
    panel(c, r);
    if let Some(m) = app.metrics.clone() {
        let rows = [
            (
                crate::i18n::tr("FPS configurado / efetivo"),
                format!("{} / {:.2}", m.configured_fps, m.effective_fps()),
            ),
            (
                crate::i18n::tr("Quadros recebidos / submetidos"),
                format!("{} / {}", m.received, m.encoded),
            ),
            (
                crate::i18n::tr("Prazos de captura perdidos"),
                m.missed_deadlines.to_string(),
            ),
            (
                crate::i18n::tr("CPU do processo · média / pico"),
                format!("{:.1}% / {:.1}%", m.cpu_mean, m.cpu_peak),
            ),
            (
                crate::i18n::tr("Memória privada · atual / pico"),
                format!(
                    "{} / {}",
                    size_label(m.memory_final),
                    size_label(m.memory_peak)
                ),
            ),
            (crate::i18n::tr("Submissão ao encoder · média"), {
                let __wyzar_i18n_0 = &(m.encoder_mean_ms);
                crate::i18n::format_text(
                    crate::i18n::tr("{:.2} ms"),
                    &[("0:.2", format!("{:.2}", __wyzar_i18n_0))],
                )
            }),
            (crate::i18n::tr("Processamento por quadro · média"), {
                let __wyzar_i18n_0 = &(m.processing_mean_ms);
                crate::i18n::format_text(
                    crate::i18n::tr("{:.2} ms"),
                    &[("0:.2", format!("{:.2}", __wyzar_i18n_0))],
                )
            }),
            (crate::i18n::tr("Fila do encoder / atraso estimado"), {
                let __wyzar_i18n_0 = &(size_label(m.encoder_queue_bytes));
                let __wyzar_i18n_1 = &(m.encoder_delay_ms);
                crate::i18n::format_text(
                    crate::i18n::tr("{} / {:.1} ms"),
                    &[
                        ("0", format!("{}", __wyzar_i18n_0)),
                        ("1:.1", format!("{:.1}", __wyzar_i18n_1)),
                    ],
                )
            }),
            (crate::i18n::tr("Áudio preenchido / descartado"), {
                let __wyzar_i18n_0 = &(m.audio_padded_samples);
                let __wyzar_i18n_1 = &(m.audio_dropped_samples);
                crate::i18n::format_text(
                    crate::i18n::tr("{} / {} amostras"),
                    &[
                        ("0", format!("{}", __wyzar_i18n_0)),
                        ("1", format!("{}", __wyzar_i18n_1)),
                    ],
                )
            }),
            (crate::i18n::tr("Diferença dos clocks submetidos"), {
                let __wyzar_i18n_0 = &(m.submitted_av_difference_ms);
                crate::i18n::format_text(
                    crate::i18n::tr("{:.3} ms"),
                    &[("0:.3", format!("{:.3}", __wyzar_i18n_0))],
                )
            }),
            (
                crate::i18n::tr("Gravado / livre no destino"),
                format!("{} / {}", size_label(m.bytes), size_label(m.free_bytes)),
            ),
        ];
        for (i, (name, value)) in rows.into_iter().enumerate() {
            let y = r.y + 24. + i as f32 * 43.;
            label(c, name, r.x + 24., y, r.w * 0.53, 14., MUTED, false);
            label(
                c,
                &value,
                r.x + r.w * 0.58,
                y,
                r.w * 0.4 - 28.,
                14.,
                TEXT,
                true,
            );
            c.line(
                r.x + 24.,
                y + 31.,
                r.right() - 24.,
                y + 31.,
                BORDER.alpha(0.6),
                1.,
            );
        }
    } else {
        label(
            c,
            crate::i18n::tr("Inicie uma gravação para coletar métricas reais."),
            r.x + 26.,
            r.y + 29.,
            r.w - 52.,
            20.,
            TEXT,
            true,
        );
    }
    c.paragraph(crate::i18n::tr("Encoder: seleção automática solicitada ao Windows. Utilização de GPU e sincronismo físico A/V ainda não medidos. Prazos perdidos são uma estimativa de cadência, não uma contagem interna de descarte do WGC."), Rect::new(r.x+24.,r.bottom()-79.,r.w-48.,60.),13.,MUTED,false);
    button(
        app,
        c,
        Rect::new(r.x, h - 83., 197., 36.),
        crate::i18n::tr("Abrir pasta de logs"),
        "folder",
        Action::OpenLogs,
        Style::Secondary,
    );
    button(
        app,
        c,
        Rect::new(r.x + 210., h - 83., 202., 36.),
        crate::i18n::tr("Copiar diagnóstico"),
        "save",
        Action::CopyDiagnostics,
        Style::Primary,
    );
    button(
        app,
        c,
        Rect::new(r.x + 427., h - 83., 224., 36.),
        crate::i18n::tr("Formatos e dispositivos"),
        "encoder",
        Action::Page(Page::Capabilities),
        Style::Secondary,
    );
}
pub(super) fn error_details(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    let r = modals::modal_panel(c, w, h, 850., 410.);
    label(
        c,
        crate::i18n::tr("Detalhes da operação"),
        r.x + 28.,
        r.y + 24.,
        r.w - 56.,
        24.,
        TEXT,
        true,
    );
    c.paragraph(
        app.last_error
            .as_deref()
            .unwrap_or(crate::i18n::tr("Nenhum detalhe adicional disponível.")),
        Rect::new(r.x + 28., r.y + 90., r.w - 56., 205.),
        14.,
        MUTED,
        false,
    );
    label(
        c,
        crate::i18n::tr("Revise o dispositivo ou destino e tente novamente."),
        r.x + 28.,
        r.y + 306.,
        r.w - 56.,
        14.,
        TEXT,
        false,
    );
    button(
        app,
        c,
        Rect::new(r.x + 28., r.bottom() - 60., 210., 36.),
        crate::i18n::tr("Escolher destino"),
        "folder",
        Action::OpenFolders,
        Style::Secondary,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 368., r.bottom() - 60., 193., 36.),
        crate::i18n::tr("Copiar diagnóstico"),
        "save",
        Action::CopyDiagnostics,
        Style::Secondary,
    );
    button(
        app,
        c,
        Rect::new(r.right() - 161., r.bottom() - 60., 133., 36.),
        crate::i18n::tr("Fechar"),
        "",
        Action::CloseModal,
        Style::Primary,
    );
}
