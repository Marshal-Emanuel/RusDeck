use egui::{Pos2, Rect, Stroke, Align2, FontId, Painter, Color32};
use crate::theme::Theme;

pub const CHAMFER: f32 = 10.0;

fn panel_fill() -> Color32 {
    Color32::from_rgba_unmultiplied(8, 12, 10, 255)
}

pub fn draw_panel_frame(painter: &Painter, rect: Rect, label: &str, theme: &Theme) {
    let clip = CHAMFER;
    let points = vec![
        Pos2::new(rect.left() + clip, rect.top()),
        Pos2::new(rect.right() - clip, rect.top()),
        Pos2::new(rect.right(), rect.top() + clip),
        Pos2::new(rect.right(), rect.bottom() - clip),
        Pos2::new(rect.right() - clip, rect.bottom()),
        Pos2::new(rect.left() + clip, rect.bottom()),
        Pos2::new(rect.left(), rect.bottom() - clip),
        Pos2::new(rect.left(), rect.top() + clip),
    ];

    let fill = panel_fill();
    painter.add(egui::Shape::convex_polygon(points.clone(), fill, Stroke::NONE));
    painter.add(egui::Shape::closed_line(points, Stroke::new(1.0, theme.mid())));

    painter.text(
        Pos2::new(rect.left() + clip + 4.0, rect.top() + 6.0),
        Align2::LEFT_TOP,
        label,
        FontId::monospace(9.0),
        theme.low(),
    );
}