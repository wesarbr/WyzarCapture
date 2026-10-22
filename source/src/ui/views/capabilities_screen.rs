use super::*;
pub(super) fn capabilities(app: &mut App, c: &mut Canvas, w: f32, h: f32) {
    title(
        c,
        crate::i18n::tr("Formatos e dispositivos"),
        crate::i18n::tr(
            "Verificação local do encoder. Aceitar um formato não certifica a taxa sustentada.",
        ),
        242.,
        84.,
        w - 268.,
    );
    let r = Rect::new(240., 180., w - 266., 440.);
    panel(c, r);
    let column = (r.w - 220.) / 4.;
    for (index, fps) in [24, 30, 60, 120].into_iter().enumerate() {
        label(
            c,
            &crate::i18n::format_text(crate::i18n::tr("{fps} FPS"), &[("fps", format!("{}", fps))]),
            r.x + 210. + index as f32 * column,
            r.y + 20.,
            column - 12.,
            17.,
            TEXT,
            true,
        );
    }
    for (row, (width, height)) in [(1280, 720), (1920, 1080), (2560, 1440), (3840, 2160)]
        .into_iter()
        .enumerate()
    {
        let y = r.y + 83. + row as f32 * 79.;
        label(
            c,
            &crate::i18n::format_text(
                crate::i18n::tr("{width} × {height}"),
                &[
                    ("width", format!("{}", width)),
                    ("height", format!("{}", height)),
                ],
            ),
            r.x + 24.,
            y,
            170.,
            16.,
            TEXT,
            true,
        );
        for (index, fps) in [24, 30, 60, 120].into_iter().enumerate() {
            let label = crate::encoder::capabilities::label(&app.capabilities, width, height, fps);
            chip(
                c,
                Rect::new(
                    r.x + 202. + index as f32 * column,
                    y - 3.,
                    column - 15.,
                    30.,
                ),
                label,
                match label {
                    v if v == crate::i18n::tr("Formato verificado") => GREEN,
                    v if v == crate::i18n::tr("Indisponível") => DIM,
                    _ => GOLD,
                },
            );
        }
        c.line(r.x + 24., y + 48., r.right() - 24., y + 48., BORDER, 1.);
    }
    label(
        c,
        &{
            let __wyzar_i18n_0 = &(app.monitors.len());
            let __wyzar_i18n_1 = &(app.microphones.len());
            let __wyzar_i18n_2 = &(app.outputs.len());
            let __wyzar_i18n_3 = &(app.webcams.len());
            crate::i18n::format_text(
                crate::i18n::tr(
                    "Monitores: {}   ·   Entradas de áudio: {}   ·   Saídas de áudio: {}   ·   Câmeras enumeradas: {}",
                ),
                &[
                    ("0", format!("{}", __wyzar_i18n_0)),
                    ("1", format!("{}", __wyzar_i18n_1)),
                    ("2", format!("{}", __wyzar_i18n_2)),
                    ("3", format!("{}", __wyzar_i18n_3)),
                ],
            )
        },
        r.x + 24.,
        r.bottom() - 33.,
        r.w - 48.,
        13.,
        MUTED,
        false,
    );
    let notes = Rect::new(r.x, r.bottom() + 16., r.w, 110.);
    panel(c, notes);
    c.paragraph(&{ let __wyzar_i18n_0 = &(app.inventory.adapters.first().map(String::as_str).unwrap_or(crate::i18n::tr("Adapter não identificado"))); let __wyzar_i18n_1 = &(app.inventory.recommendation()); let __wyzar_i18n_2 = &(app.inventory.encoders.iter().filter(|e|e.hardware).count()); let __wyzar_i18n_3 = &(app.inventory.encoders.iter().filter(|e|!e.hardware).count()); crate::i18n::format_text(crate::i18n::tr("{}\n{}\nMFTs H.264: {} hardware, {} software. Enumeração não certifica velocidade sustentada."), &[("0", format!("{}", __wyzar_i18n_0)), ("1", format!("{}", __wyzar_i18n_1)), ("2", format!("{}", __wyzar_i18n_2)), ("3", format!("{}", __wyzar_i18n_3))]) },notes.inset(18.),13.,MUTED,false);
    if app.probing_capabilities {
        disabled(
            c,
            Rect::new(r.x, h - 83., 235., 36.),
            crate::i18n::tr("Verificando formatos…"),
            "encoder",
        );
    } else {
        button(
            app,
            c,
            Rect::new(r.x, h - 83., 235., 36.),
            crate::i18n::tr("Verificar formatos"),
            "encoder",
            Action::ProbeCapabilities,
            Style::Primary,
        );
    }
    button(
        app,
        c,
        Rect::new(r.x + 250., h - 83., 210., 36.),
        crate::i18n::tr("Voltar ao diagnóstico"),
        "filter",
        Action::Page(Page::Diagnostics),
        Style::Secondary,
    );
    if app.performance_running {
        disabled(
            c,
            Rect::new(r.x + 477., h - 83., 230., 36.),
            crate::i18n::tr("Medindo por 5 segundos…"),
            "encoder",
        );
    } else {
        button(
            app,
            c,
            Rect::new(r.x + 477., h - 83., 230., 36.),
            crate::i18n::tr("Testar desempenho"),
            "encoder",
            Action::PerformanceTest,
            Style::Secondary,
        );
    }
}
