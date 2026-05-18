use egui::{Pos2, Rect, Painter, Vec2, epaint::Mesh};
use crate::theme::Theme;

const DOT_SPACING: f32 = 14.0;
const DOT_RADIUS: f32 = 1.0;

pub struct BackgroundCache {
    size: Vec2,
    mesh: Option<Mesh>,
}

impl BackgroundCache {
    pub fn new() -> Self {
        Self {
            size: Vec2::ZERO,
            mesh: None,
        }
    }

    fn rebuild(&mut self, rect: Rect, theme: &Theme) {
        self.size = rect.size();
        let color = theme.ghost();
        let mut mesh = Mesh::default();

        let mut x = rect.left() + DOT_SPACING;
        while x < rect.right() {
            let mut y = rect.top() + DOT_SPACING;
            while y < rect.bottom() {
                let cx = x;
                let cy = y;
                let r = DOT_RADIUS;
                let idx = mesh.vertices.len() as u32;
                mesh.colored_vertex(Pos2::new(cx, cy - r), color);
                mesh.colored_vertex(Pos2::new(cx + r, cy), color);
                mesh.colored_vertex(Pos2::new(cx, cy + r), color);
                mesh.colored_vertex(Pos2::new(cx - r, cy), color);
                mesh.add_triangle(idx, idx + 1, idx + 2);
                mesh.add_triangle(idx, idx + 2, idx + 3);
                y += DOT_SPACING;
            }
            x += DOT_SPACING;
        }

        self.mesh = Some(mesh);
    }
}

impl Default for BackgroundCache {
    fn default() -> Self {
        Self::new()
    }
}

pub fn draw_background(painter: &Painter, rect: Rect, theme: &Theme, cache: &mut BackgroundCache) {
    painter.rect_filled(rect, 0.0, theme.background);

    if cache.mesh.is_none() || cache.size != rect.size() {
        cache.rebuild(rect, theme);
    }

    if let Some(mesh) = &cache.mesh {
        painter.add(mesh.clone());
    }
}