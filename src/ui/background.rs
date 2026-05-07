use egui::{Color32, ColorImage, TextureHandle};
use crate::theme::Theme;

pub struct BackgroundCache {
    texture: Option<TextureHandle>,
    width: f32,
    height: f32,
}

impl BackgroundCache {
    pub fn new() -> Self {
        Self {
            texture: None,
            width: 0.0,
            height: 0.0,
        }
    }

    pub fn get(&mut self, ctx: &egui::Context, w: f32, h: f32, theme: &Theme) -> &TextureHandle {
        if self.width != w || self.height != h {
            let color_image = generate_dot_grid(w, h, theme);
            self.texture = Some(ctx.load_texture("rusdeck_bg", color_image, egui::TextureOptions::LINEAR));
            self.width = w;
            self.height = h;
        }
        self.texture.as_ref().unwrap()
    }
}

impl Default for BackgroundCache {
    fn default() -> Self {
        Self::new()
    }
}

fn generate_dot_grid(w: f32, h: f32, theme: &Theme) -> ColorImage {
    let spacing = 14.0;
    let cols = ((w / spacing).ceil() as usize).max(1);
    let rows = ((h / spacing).ceil() as usize).max(1);

    let mut pixels = Vec::with_capacity(cols * rows);
    let [ar, ag, ab, _] = theme.accent.to_array();

    for row in 0..rows {
        for col in 0..cols {
            let x = col as f32 * spacing;
            let y = row as f32 * spacing;

            let dx = (x - w / 2.0) / (w / 2.0).max(1.0);
            let dy = (y - h / 2.0) / (h / 2.0).max(1.0);
            let dist = (dx * dx + dy * dy).sqrt();

            let brightness = if dist < 0.3 {
                0.06
            } else if dist < 0.7 {
                0.04
            } else {
                0.02
            };

            let alpha = (brightness * 255.0) as u8;
            pixels.push(Color32::from_rgba_unmultiplied(ar, ag, ab, alpha));
        }
    }

    ColorImage { size: [cols, rows], pixels }
}

pub fn draw_background(
    cache: &mut BackgroundCache,
    ctx: &egui::Context,
    painter: &egui::Painter,
    rect: egui::Rect,
    theme: &Theme,
) {
    let w = rect.width();
    let h = rect.height();

    painter.rect_filled(rect, 0.0, theme.background);

    if w > 0.0 && h > 0.0 {
        let tex = cache.get(ctx, w, h, theme);
        let tex_size = tex.size();
        let uvs = egui::Rect::from_min_max(
            egui::Pos2::ZERO,
            egui::Pos2::new(
                (w / (tex_size[0] as f32 * 14.0 / w)).min(1.0),
                (h / (tex_size[1] as f32 * 14.0 / h)).min(1.0),
            ),
        );
        painter.image(tex.id(), rect, uvs, Color32::WHITE);
    }
}