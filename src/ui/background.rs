use egui::{Pos2, Rect, Painter};
use crate::theme::Theme;

pub struct BackgroundCache;

impl BackgroundCache {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BackgroundCache {
    fn default() -> Self {
        Self::new()
    }
}

pub fn draw_background(painter: &Painter, rect: Rect, theme: &Theme) {
    painter.rect_filled(rect, 0.0, theme.background);

    let mut x = rect.left();
    while x < rect.right() {
        let mut y = rect.top();
        while y < rect.bottom() {
            painter.circle_filled(Pos2::new(x, y), 0.5, theme.ghost());
            y += 14.0;
        }
        x += 14.0;
    }
}