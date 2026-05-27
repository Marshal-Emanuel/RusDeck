use egui::{Pos2, Rect, Stroke, Align2, FontId, Painter, Color32};
use crate::theme::Theme;

pub const CHAMFER: f32 = 12.0;

fn panel_fill() -> Color32 {
    Color32::from_rgba_unmultiplied(6, 10, 8, 255) // Matches deep terminal background [6,10,8]
}

pub fn draw_panel_frame(painter: &Painter, rect: Rect, label: &str, theme: &Theme) {
    let clip = CHAMFER;
    // Asymmetric chamfer matching the terminal (top-right and bottom-left)
    let points = vec![
        Pos2::new(rect.left(), rect.top()),
        Pos2::new(rect.right() - clip, rect.top()),
        Pos2::new(rect.right(), rect.top() + clip),
        Pos2::new(rect.right(), rect.bottom()),
        Pos2::new(rect.left() + clip, rect.bottom()),
        Pos2::new(rect.left(), rect.bottom() - clip),
    ];

    let fill = panel_fill();
    painter.add(egui::Shape::convex_polygon(points.clone(), fill, Stroke::NONE));
    painter.add(egui::Shape::closed_line(points, Stroke::new(1.0, theme.low())));

    // Neon Accent brackets matching the chamfers
    let thick_stroke = Stroke::new(2.5, theme.accent);
    // Top-Right Bracket
    painter.add(egui::Shape::line(vec![
        Pos2::new(rect.right() - clip - 12.0, rect.top()),
        Pos2::new(rect.right() - clip, rect.top()),
        Pos2::new(rect.right(), rect.top() + clip),
        Pos2::new(rect.right(), rect.top() + clip + 12.0),
    ], thick_stroke));
    // Bottom-Left Bracket
    painter.add(egui::Shape::line(vec![
        Pos2::new(rect.left() + clip + 12.0, rect.bottom()),
        Pos2::new(rect.left() + clip, rect.bottom()),
        Pos2::new(rect.left(), rect.bottom() - clip),
        Pos2::new(rect.left(), rect.bottom() - clip - 12.0),
    ], thick_stroke));

    // Futuristic dot indicator before the label
    let dot_x = rect.left() + clip + 4.0;
    let dot_y = rect.top() + 15.0;
    painter.rect_filled(
        Rect::from_min_size(Pos2::new(dot_x, dot_y), egui::vec2(4.0, 4.0)),
        0.0,
        theme.accent,
    );

    painter.text(
        Pos2::new(rect.left() + clip + 14.0, rect.top() + 10.0),
        Align2::LEFT_TOP,
        label,
        FontId::monospace(14.0),
        theme.high(), // Set high brightness for the panel title text
    );
}