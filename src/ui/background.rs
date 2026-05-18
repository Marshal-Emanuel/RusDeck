use egui::{Pos2, Rect, Painter, Vec2};
use crate::theme::Theme;

pub struct BackgroundCache {
    size: Vec2,
    dots: Vec<Pos2>,
}

impl BackgroundCache {
    pub fn new() -> Self {
        Self {
            size: Vec2::ZERO,
            dots: Vec::new(),
        }
    }

    fn rebuild(&mut self, rect: Rect) {
        self.size = rect.size();
        self.dots.clear();
        let mut x = rect.left();
        while x < rect.right() {
            let mut y = rect.top();
            while y < rect.bottom() {
                self.dots.push(Pos2::new(x, y));
                y += 14.0;
            }
            x += 14.0;
        }
    }
}

impl Default for BackgroundCache {
    fn default() -> Self {
        Self::new()
    }
}

pub fn draw_background(painter: &Painter, rect: Rect, theme: &Theme, cache: &mut BackgroundCache) {
    painter.rect_filled(rect, 0.0, theme.background);

    if cache.size != rect.size() {
        cache.rebuild(rect);
    }

    for &p in &cache.dots {
        painter.circle_filled(p, 0.5, theme.ghost());
    }
}