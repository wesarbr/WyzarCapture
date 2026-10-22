use super::theme::*;
use crate::{i18n, platform::windows::graphics::Canvas};
pub struct Layout {
    pub previous: Rect,
    pub next: Rect,
    pub close: Rect,
    pub overflow: bool,
}
pub fn render(c: &mut Canvas, w: f32, h: f32, page: usize) -> Layout {
    let r = Rect::new((w - 970.).max(16.) / 2., 82., 970f32.min(w - 32.), h - 110.);
    c.fill(Rect::new(0., 0., w, h), BG.alpha(0.88));
    c.round(r, 12., PANEL, Some(BORDER));
    let text = crate::license::text(i18n::current());
    let mut paragraphs = text.split("\n\n");
    let title = paragraphs.next().unwrap_or("WyzarCapture");
    c.text(
        title,
        Rect::new(r.x + 25., r.y + 20., r.w - 50., 42.),
        24.,
        GOLD,
        true,
    );
    let page = page.min(crate::license::page_count().saturating_sub(1));
    let paragraph = paragraphs.nth(page).unwrap_or("").trim();
    let body = Rect::new(r.x + 27., r.y + 82., r.w - 54., r.h - 178.);
    let overflow = c.paragraph_height(paragraph, body.w, 17., false) > body.h;
    c.paragraph(paragraph, body, 17., TEXT, false);
    c.text(
        &format!("{}/{}", page + 1, crate::license::page_count()),
        Rect::new(r.x + 26., r.bottom() - 66., 80., 35.),
        14.,
        MUTED,
        false,
    );
    Layout {
        previous: Rect::new(r.right() - 460., r.bottom() - 68., 130., 40.),
        next: Rect::new(r.right() - 315., r.bottom() - 68., 130., 40.),
        close: Rect::new(r.right() - 170., r.bottom() - 68., 140., 40.),
        overflow,
    }
}
