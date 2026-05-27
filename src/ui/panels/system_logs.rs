use egui::{Pos2, Align2, FontId, Rect, Painter, Color32};
use crate::app::AppState;
use crate::theme::Theme;

pub fn draw_system_logs(painter: &Painter, rect: Rect, state: &AppState, theme: &Theme) {
    let clip = 12.0; // Chamfer matching the normal panels
    let step_h = 24.0;
    let step_x = rect.right() - 250.0; // Adjust step width for logs panel

    let points = vec![
        Pos2::new(rect.left(), rect.top()),
        Pos2::new(rect.right() - clip, rect.top()),
        Pos2::new(rect.right(), rect.top() + clip),
        Pos2::new(rect.right(), rect.bottom() - step_h),
        Pos2::new(step_x, rect.bottom() - step_h),
        Pos2::new(step_x - step_h, rect.bottom()),
        Pos2::new(rect.left() + clip, rect.bottom()),
        Pos2::new(rect.left(), rect.bottom() - clip),
    ];

    let fill = Color32::from_rgba_unmultiplied(6, 10, 8, 255);
    painter.add(egui::Shape::convex_polygon(points.clone(), fill, egui::Stroke::NONE));
    painter.add(egui::Shape::closed_line(points, egui::Stroke::new(1.0, theme.low())));

    // Nested segments in the notch
    let h_height = 12.0; 
    let chunk_widths = vec![80.0, 10.0, 25.0];
    let mut current_x = step_x + 15.0;
    
    for &w in &chunk_widths {
        if current_x + w + h_height > rect.right() - 10.0 { break; }
        let chunk_poly = vec![
            Pos2::new(current_x, rect.bottom()),
            Pos2::new(current_x + w, rect.bottom()),
            Pos2::new(current_x + w + h_height, rect.bottom() - h_height),
            Pos2::new(current_x + h_height, rect.bottom() - h_height),
        ];
        painter.add(egui::Shape::convex_polygon(chunk_poly, theme.accent, egui::Stroke::NONE));
        current_x += w + 6.0;
    }

    let thick_stroke = egui::Stroke::new(2.5, theme.accent);
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
        "SYSTEM_LOGS",
        FontId::monospace(14.0),
        theme.high(),
    );

    let chamfer = 10.0;
    let ts_width = 200.0;
    let text_x = rect.left() + chamfer + 4.0;
    let msg_x = text_x + ts_width;
    let msg_max_w = rect.right() - chamfer - 4.0 - msg_x;

    let mut y = rect.top() + 36.0;
    let row_h = 22.0;
    let max_rows = ((rect.height() - 44.0) / row_h) as usize;

    for (i, log) in state.logs.iter().take(max_rows).enumerate() {
        let base_alpha = (max_rows - i) as f32 / max_rows as f32 * 0.8 + 0.2;

        painter.text(
            Pos2::new(text_x, y),
            Align2::LEFT_TOP,
            &log.timestamp,
            FontId::monospace(13.0),
            theme.with_alpha((base_alpha * 120.0) as u8),
        );
        let msg = if msg_max_w > 30.0 {
            truncate_text(&log.message, msg_max_w, 13.0)
        } else {
            &log.message
        };
        painter.text(
            Pos2::new(msg_x, y),
            Align2::LEFT_TOP,
            msg,
            FontId::monospace(13.0),
            theme.with_alpha((base_alpha * 200.0) as u8),
        );
        y += row_h;
    }
}

fn truncate_text<'a>(text: &'a str, max_px: f32, font_size: f32) -> &'a str {
    let char_w = font_size * 0.6;
    let max_chars = (max_px / char_w) as usize;
    if text.len() > max_chars && max_chars > 5 {
        let truncate_at = max_chars - 3;
        if let Some((idx, _)) = text.char_indices().nth(truncate_at) {
            &text[..idx]
        } else {
            text
        }
    } else {
        text
    }
}