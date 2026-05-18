use egui::{Pos2, Align2, FontId, Rect, Painter};
use crate::app::AppState;
use crate::theme::Theme;
use super::draw_panel_frame;

pub fn draw_system_logs(painter: &Painter, rect: Rect, state: &AppState, theme: &Theme) {
    draw_panel_frame(painter, rect, "SYSTEM_LOGS", theme);

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