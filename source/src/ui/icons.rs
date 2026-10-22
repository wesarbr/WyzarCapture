pub const NAMES: &[&str] = &[
    "pin",
    "home",
    "record",
    "library",
    "edit",
    "settings",
    "monitor",
    "window",
    "region",
    "game",
    "microphone",
    "audio",
    "webcam",
    "cursor",
    "export",
    "trash",
    "play",
    "pause",
    "stop",
    "folder",
    "save",
    "search",
    "filter",
    "bell",
    "storage",
    "resolution",
    "fps",
    "encoder",
    "tag",
    "calendar",
    "clock",
    "clip",
    "volume",
    "speed",
    "text",
    "blur",
    "crop",
    "split",
    "help",
    "account",
    "news",
    "check",
    "close",
    "arrow",
    "timeline",
    "fade",
    "chevron",
    "plus",
    "star",
    "minimize",
    "maximize",
    "restore",
    "palette",
    "keyboard",
    "sliders",
    "sparkles",
    "compact",
    "expand",
    "undo",
    "redo",
    "book",
    "shield",
    "reset",
    "download",
    "import",
    "eye",
    "moon",
    "sun",
    "layers",
    "pointer",
    "lock",
    "warning",
    "info",
];
pub fn svg(name: &str) -> &'static str {
    match name {
        "pin" => include_str!("../../assets/icons/pin.svg"),
        "minimize" => include_str!("../../assets/icons/minimize.svg"),
        "maximize" => include_str!("../../assets/icons/maximize.svg"),
        "restore" => include_str!("../../assets/icons/restore.svg"),
        "palette" => include_str!("../../assets/icons/palette.svg"),
        "keyboard" => include_str!("../../assets/icons/keyboard.svg"),
        "sliders" => include_str!("../../assets/icons/sliders.svg"),
        "sparkles" => include_str!("../../assets/icons/sparkles.svg"),
        "compact" => include_str!("../../assets/icons/compact.svg"),
        "expand" => include_str!("../../assets/icons/expand.svg"),
        "undo" => include_str!("../../assets/icons/undo.svg"),
        "redo" => include_str!("../../assets/icons/redo.svg"),
        "book" => include_str!("../../assets/icons/book.svg"),
        "shield" => include_str!("../../assets/icons/shield.svg"),
        "reset" => include_str!("../../assets/icons/reset.svg"),
        "download" => include_str!("../../assets/icons/download.svg"),
        "import" => include_str!("../../assets/icons/import.svg"),
        "eye" => include_str!("../../assets/icons/eye.svg"),
        "moon" => include_str!("../../assets/icons/moon.svg"),
        "sun" => include_str!("../../assets/icons/sun.svg"),
        "layers" => include_str!("../../assets/icons/layers.svg"),
        "pointer" => include_str!("../../assets/icons/pointer.svg"),
        "lock" => include_str!("../../assets/icons/lock.svg"),
        "warning" => include_str!("../../assets/icons/warning.svg"),
        "info" => include_str!("../../assets/icons/info.svg"),
        "star" => include_str!("../../assets/icons/star.svg"),
        "home" => include_str!("../../assets/icons/home.svg"),
        "record" => include_str!("../../assets/icons/record.svg"),
        "library" => include_str!("../../assets/icons/library.svg"),
        "edit" => include_str!("../../assets/icons/edit.svg"),
        "settings" => include_str!("../../assets/icons/settings.svg"),
        "monitor" => include_str!("../../assets/icons/monitor.svg"),
        "window" => include_str!("../../assets/icons/window.svg"),
        "region" => include_str!("../../assets/icons/region.svg"),
        "game" => include_str!("../../assets/icons/game.svg"),
        "microphone" => include_str!("../../assets/icons/microphone.svg"),
        "audio" => include_str!("../../assets/icons/audio.svg"),
        "webcam" => include_str!("../../assets/icons/webcam.svg"),
        "cursor" => include_str!("../../assets/icons/cursor.svg"),
        "export" => include_str!("../../assets/icons/export.svg"),
        "trash" => include_str!("../../assets/icons/trash.svg"),
        "play" => include_str!("../../assets/icons/play.svg"),
        "pause" => include_str!("../../assets/icons/pause.svg"),
        "stop" => include_str!("../../assets/icons/stop.svg"),
        "folder" => include_str!("../../assets/icons/folder.svg"),
        "save" => include_str!("../../assets/icons/save.svg"),
        "search" => include_str!("../../assets/icons/search.svg"),
        "filter" => include_str!("../../assets/icons/filter.svg"),
        "bell" => include_str!("../../assets/icons/bell.svg"),
        "storage" => include_str!("../../assets/icons/storage.svg"),
        "resolution" => include_str!("../../assets/icons/resolution.svg"),
        "fps" => include_str!("../../assets/icons/fps.svg"),
        "encoder" => include_str!("../../assets/icons/encoder.svg"),
        "tag" => include_str!("../../assets/icons/tag.svg"),
        "calendar" => include_str!("../../assets/icons/calendar.svg"),
        "clock" => include_str!("../../assets/icons/clock.svg"),
        "volume" => include_str!("../../assets/icons/volume.svg"),
        "speed" => include_str!("../../assets/icons/speed.svg"),
        "text" => include_str!("../../assets/icons/text.svg"),
        "blur" => include_str!("../../assets/icons/blur.svg"),
        "crop" => include_str!("../../assets/icons/crop.svg"),
        "split" => include_str!("../../assets/icons/split.svg"),
        "help" => include_str!("../../assets/icons/help.svg"),
        "account" => include_str!("../../assets/icons/account.svg"),
        "news" => include_str!("../../assets/icons/news.svg"),
        "check" => include_str!("../../assets/icons/check.svg"),
        "close" => include_str!("../../assets/icons/close.svg"),
        "arrow" => include_str!("../../assets/icons/arrow.svg"),
        "timeline" => include_str!("../../assets/icons/timeline.svg"),
        "fade" => include_str!("../../assets/icons/fade.svg"),
        "chevron" => include_str!("../../assets/icons/chevron.svg"),
        "plus" => include_str!("../../assets/icons/plus.svg"),
        _ => include_str!("../../assets/icons/clip.svg"),
    }
}
pub fn path(name: &str) -> &'static str {
    svg(name)
        .split("<path d=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .unwrap_or("")
}
pub fn label(name: &str) -> &str {
    match name {
        "pin" => crate::i18n::tr("Fixar janela"),
        "home" => crate::i18n::tr("Início"),
        "record" => crate::i18n::tr("Gravação"),
        "library" => crate::i18n::tr("Biblioteca"),
        "edit" => crate::i18n::tr("Editar"),
        "settings" => crate::i18n::tr("Configurações"),
        "monitor" => crate::i18n::tr("Tela inteira"),
        "window" => crate::i18n::tr("Janela"),
        "region" => crate::i18n::tr("Região"),
        "game" => crate::i18n::tr("Jogo"),
        "microphone" => crate::i18n::tr("Microfone"),
        "audio" => crate::i18n::tr("Som do sistema"),
        "webcam" => crate::i18n::tr("Webcam"),
        "cursor" => crate::i18n::tr("Cursor"),
        "export" => crate::i18n::tr("Exportar"),
        "trash" => crate::i18n::tr("Excluir"),
        "play" => crate::i18n::tr("Reproduzir"),
        "pause" => crate::i18n::tr("Pausar"),
        "stop" => crate::i18n::tr("Parar"),
        "folder" => crate::i18n::tr("Pasta"),
        "save" => crate::i18n::tr("Salvar"),
        "search" => crate::i18n::tr("Buscar"),
        "filter" => crate::i18n::tr("Filtros"),
        "bell" => crate::i18n::tr("Notificações"),
        "storage" => crate::i18n::tr("Armazenamento"),
        "resolution" => crate::i18n::tr("Resolução"),
        "fps" => crate::i18n::tr("Quadros por segundo"),
        "encoder" => crate::i18n::tr("Encoder"),
        "tag" => crate::i18n::tr("Tags"),
        "calendar" => crate::i18n::tr("Calendário"),
        "clock" => crate::i18n::tr("Duração"),
        "clip" => crate::i18n::tr("Clipe"),
        "volume" => crate::i18n::tr("Volume"),
        "speed" => crate::i18n::tr("Velocidade"),
        "text" => crate::i18n::tr("Texto"),
        "blur" => crate::i18n::tr("Blur"),
        "crop" => crate::i18n::tr("Cortar tela"),
        "split" => crate::i18n::tr("Dividir"),
        "help" => crate::i18n::tr("Ajuda"),
        "account" => crate::i18n::tr("Conta local"),
        "news" => crate::i18n::tr("Novidades"),
        "check" => crate::i18n::tr("Confirmar"),
        "close" => crate::i18n::tr("Fechar"),
        "arrow" => crate::i18n::tr("Seta"),
        "timeline" => crate::i18n::tr("Timeline"),
        "fade" => crate::i18n::tr("Fade"),
        "chevron" => crate::i18n::tr("Abrir opções"),
        "plus" => crate::i18n::tr("Adicionar"),
        "star" => crate::i18n::tr("Favorito"),
        "minimize" => crate::i18n::tr("Minimizar"),
        "maximize" => crate::i18n::tr("Maximizar"),
        "restore" => crate::i18n::tr("Restaurar janela"),
        "palette" => crate::i18n::tr("Paleta"),
        "keyboard" => crate::i18n::tr("Teclado"),
        "sliders" => crate::i18n::tr("Ajustes"),
        "sparkles" => crate::i18n::tr("Efeitos"),
        "compact" => crate::i18n::tr("Modo foco"),
        "expand" => crate::i18n::tr("Expandir"),
        "undo" => crate::i18n::tr("Desfazer"),
        "redo" => crate::i18n::tr("Refazer"),
        "book" => crate::i18n::tr("Tutorial"),
        "shield" => crate::i18n::tr("Privacidade"),
        "reset" => crate::i18n::tr("Restaurar"),
        "download" => crate::i18n::tr("Baixar"),
        "import" => crate::i18n::tr("Importar"),
        "eye" => crate::i18n::tr("Prévia"),
        "moon" => crate::i18n::tr("Noite"),
        "sun" => crate::i18n::tr("Luz"),
        "layers" => crate::i18n::tr("Camadas"),
        "pointer" => crate::i18n::tr("Ponteiro"),
        "lock" => crate::i18n::tr("Bloquear"),
        "warning" => crate::i18n::tr("Atenção"),
        "info" => crate::i18n::tr("Informações"),
        _ => name,
    }
}
#[derive(Clone, Debug)]
pub enum Command {
    Move(f32, f32),
    Line(f32, f32),
    Cubic(f32, f32, f32, f32, f32, f32),
    Quad(f32, f32, f32, f32),
    Arc(f32, f32, f32, bool, bool, f32, f32),
    Close,
}
// Small SVG path reader shared with the original vector icon set; no SVG engine dependency.
pub fn parse(path: &str) -> Vec<Command> {
    let mut tokens = vec![];
    let mut n = String::new();
    for c in path.chars() {
        if c.is_alphabetic() {
            if !n.is_empty() {
                tokens.push(n.clone());
                n.clear();
            }
            tokens.push(c.to_string());
        } else if c == '-' || c == '+' {
            if !n.is_empty() {
                tokens.push(n.clone());
                n.clear();
            }
            n.push(c);
        } else if c.is_ascii_digit() || c == '.' {
            n.push(c);
        } else if !n.is_empty() {
            tokens.push(n.clone());
            n.clear();
        }
    }
    if !n.is_empty() {
        tokens.push(n);
    }
    let (mut i, mut op, mut x, mut y, mut sx, mut sy) = (0, 'M', 0., 0., 0., 0.);
    let mut out = vec![];
    let mut control: Option<(char, f32, f32)> = None;
    while i < tokens.len() {
        if let Some(c) = tokens[i].chars().next().filter(|c| c.is_alphabetic()) {
            op = c;
            i += 1;
            if c == 'z' || c == 'Z' {
                out.push(Command::Close);
                x = sx;
                y = sy;
                control = None;
                continue;
            }
        }
        let count = match op.to_ascii_uppercase() {
            'M' | 'L' | 'T' => 2,
            'H' | 'V' => 1,
            'Q' | 'S' => 4,
            'C' => 6,
            'A' => 7,
            _ => break,
        };
        if i + count > tokens.len() {
            break;
        }
        let v: Vec<f32> = tokens[i..i + count]
            .iter()
            .map(|s| s.parse().unwrap_or(0.))
            .collect();
        i += count;
        let relative = op.is_lowercase();
        let (ox, oy) = if relative { (x, y) } else { (0., 0.) };
        match op.to_ascii_uppercase() {
            'M' => {
                x = v[0] + ox;
                y = v[1] + oy;
                sx = x;
                sy = y;
                out.push(Command::Move(x, y));
                op = if relative { 'l' } else { 'L' }
            }
            'L' => {
                x = v[0] + ox;
                y = v[1] + oy;
                out.push(Command::Line(x, y));
            }
            'T' => {
                let (a, b) = control
                    .filter(|(kind, _, _)| *kind == 'Q')
                    .map(|(_, a, b)| (2. * x - a, 2. * y - b))
                    .unwrap_or((x, y));
                x = v[0] + ox;
                y = v[1] + oy;
                out.push(Command::Quad(a, b, x, y));
            }
            'S' => {
                let (a, b) = control
                    .filter(|(kind, _, _)| *kind == 'C')
                    .map(|(_, a, b)| (2. * x - a, 2. * y - b))
                    .unwrap_or((x, y));
                x = v[2] + ox;
                y = v[3] + oy;
                out.push(Command::Cubic(a, b, v[0] + ox, v[1] + oy, x, y));
            }
            'H' => {
                x = v[0] + ox;
                out.push(Command::Line(x, y));
            }
            'V' => {
                y = v[0] + oy;
                out.push(Command::Line(x, y));
            }
            'C' => {
                x = v[4] + ox;
                y = v[5] + oy;
                out.push(Command::Cubic(
                    v[0] + ox,
                    v[1] + oy,
                    v[2] + ox,
                    v[3] + oy,
                    x,
                    y,
                ));
            }
            'Q' => {
                x = v[2] + ox;
                y = v[3] + oy;
                out.push(Command::Quad(v[0] + ox, v[1] + oy, x, y));
            }
            'A' => {
                x = v[5] + ox;
                y = v[6] + oy;
                out.push(Command::Arc(v[0], v[1], v[2], v[3] != 0., v[4] != 0., x, y));
            }
            _ => {
                x = v[count - 2] + ox;
                y = v[count - 1] + oy;
                out.push(Command::Line(x, y));
            }
        }
        control = match out.last() {
            Some(Command::Quad(a, b, _, _)) => Some(('Q', *a, *b)),
            Some(Command::Cubic(_, _, a, b, _, _)) => Some(('C', *a, *b)),
            _ => None,
        };
    }
    out
}

pub fn validate_path(path: &str) -> Result<(), String> {
    if path.len() > 8192 || !path.trim_start().starts_with(['M', 'm']) {
        return Err(crate::i18n::tr("O vetor deve começar com M e ter até 8 KiB.").into());
    }
    let mut tokens = Vec::new();
    let mut num = String::new();
    for ch in path.chars().chain(Some(' ')) {
        if ch.is_ascii_alphabetic()
            || ch == '-'
            || ch == '+'
            || ch == ','
            || ch.is_ascii_whitespace()
        {
            if !num.is_empty() {
                tokens.push(num.clone());
                num.clear();
            }
            if ch.is_ascii_alphabetic() {
                if !"MmLlHhVvCcSsQqTtAaZz".contains(ch) {
                    return Err(crate::i18n::tr("Comando de vetor não suportado.").into());
                }
                tokens.push(ch.to_string());
            } else if ch == '-' || ch == '+' {
                num.push(ch);
            }
        } else if ch.is_ascii_digit() || ch == '.' {
            num.push(ch);
        } else {
            return Err(crate::i18n::tr("Conteúdo inválido no vetor.").into());
        }
    }
    let (mut i, mut arity) = (0, 0);
    while i < tokens.len() {
        if let Some(op) = tokens[i].chars().next().filter(|c| c.is_ascii_alphabetic()) {
            arity = match op.to_ascii_uppercase() {
                'M' | 'L' | 'T' => 2,
                'H' | 'V' => 1,
                'Q' | 'S' => 4,
                'C' => 6,
                'A' => 7,
                'Z' => 0,
                _ => unreachable!(),
            };
            i += 1;
            if arity == 0 {
                continue;
            }
        }
        if arity == 0 || i + arity > tokens.len() {
            return Err(crate::i18n::tr("Parâmetros incompletos no vetor.").into());
        }
        for v in &tokens[i..i + arity] {
            let n = v
                .parse::<f32>()
                .map_err(|_| crate::i18n::tr("Número inválido no vetor"))?;
            if !n.is_finite() || n.abs() > 96. {
                return Err(crate::i18n::tr("Coordenadas fora do limite do ícone 24 × 24.").into());
            }
        }
        i += arity;
    }
    if parse(path).len() < 2 {
        return Err(crate::i18n::tr("O vetor precisa de ao menos dois comandos.").into());
    }
    Ok(())
}
pub fn import_svg(path: &std::path::Path) -> Result<String, String> {
    use std::io::Read;
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    file.take(65537)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 65536 {
        return Err(crate::i18n::tr("O SVG deve ter até 64 KiB.").into());
    }
    let svg = String::from_utf8(bytes).map_err(|_| crate::i18n::tr("O SVG precisa usar UTF-8"))?;
    let lower = svg.to_ascii_lowercase();
    if [
        "<!",
        "<script",
        "<image",
        "<use",
        "<foreignobject",
        "href=",
        "url(",
    ]
    .iter()
    .any(|s| lower.contains(s))
        || svg.matches("<path").count() != 1
        || !svg.contains("viewBox=\"0 0 24 24\"")
    {
        return Err(crate::i18n::tr("Use um SVG local 24 × 24 com um único path; combine subformas no mesmo path, sem imagens ou referências externas.").into());
    }
    let start = svg.find("<path").ok_or(crate::i18n::tr("Path ausente"))?;
    let path = svg[start..]
        .split("d=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .ok_or(crate::i18n::tr("Use o atributo d entre aspas duplas"))?;
    validate_path(path)?;
    Ok(path.into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_bundled_vector_is_valid() {
        for name in NAMES {
            assert!(validate_path(path(name)).is_ok(), "{name}");
        }
    }
    #[test]
    fn rejects_invalid_and_unbounded_paths() {
        for p in [
            "M0 0L1..2 3",
            "M0 0L1",
            "M0 0L1e309 2",
            "M0 0X1 2",
            "M0 0L999999 0",
        ] {
            assert!(validate_path(p).is_err(), "{p}");
        }
    }
    #[test]
    fn smooth_curves_reflect_the_previous_control() {
        let p = parse("M1 2q2 3 4 5t6 7");
        assert!(matches!(p[2], Command::Quad(7., 9., 11., 14.)));
    }
}
