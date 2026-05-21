use egui::{Pos2, Rect, Painter, FontId, Align2, Color32};
use crate::theme::Theme;
use crate::ui::terminal::TerminalWidget;

pub const CHAMFER: f32 = 10.0;

fn panel_fill() -> Color32 {
    Color32::from_rgba_unmultiplied(2, 5, 4, 255)
}

pub fn draw_terminal(painter: &Painter, rect: Rect, term: &TerminalWidget, theme: &Theme) {
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
    painter.add(egui::Shape::convex_polygon(points.clone(), fill, egui::Stroke::NONE));
    painter.add(egui::Shape::closed_line(points, egui::Stroke::new(1.0, theme.mid())));

    painter.text(
        Pos2::new(rect.left() + clip + 4.0, rect.top() + 10.0),
        Align2::LEFT_TOP,
        "TERMINAL",
        FontId::monospace(15.0),
        theme.low(),
    );

    let buffer = term.get_buffer();
    let buf_guard = match buffer.lock() {
        Ok(g) => g,
        Err(_) => return,
    };

    let cell_w = 8.0;
    let cell_h = 16.0;
    let margin_x = rect.left() + clip + 4.0;
    let margin_y = rect.top() + 32.0;

    let cols = buf_guard.width();
    let rows = buf_guard.height();

    for row_idx in 0..rows {
        for col_idx in 0..cols {
            let cell = buf_guard.get_cell(col_idx, row_idx);

            let x = margin_x + col_idx as f32 * cell_w;
            let y = margin_y + row_idx as f32 * cell_h;

            if x + cell_w > rect.right() - clip - 4.0 || y + cell_h > rect.bottom() - clip - 4.0 {
                continue;
            }

            let bg = Color32::from_rgb(cell.bg[0], cell.bg[1], cell.bg[2]);
            if bg != Color32::TRANSPARENT && bg != Color32::from_rgb(0, 0, 0) {
                painter.rect_filled(Rect::from_min_size(Pos2::new(x, y), egui::vec2(cell_w, cell_h)), 0.0, bg);
            }

            if cell.c != ' ' {
                let fg = if cell.bold {
                    Color32::from_rgb(
                        (cell.fg[0] as u32 + 80).min(255) as u8,
                        (cell.fg[1] as u32 + 80).min(255) as u8,
                        (cell.fg[2] as u32 + 80).min(255) as u8,
                    )
                } else {
                    Color32::from_rgb(cell.fg[0], cell.fg[1], cell.fg[2])
                };

                painter.text(
                    Pos2::new(x, y),
                    Align2::LEFT_TOP,
                    cell.c.to_string(),
                    FontId::monospace(12.0),
                    fg,
                );
            }
        }
    }

    let (cursor_col, cursor_row) = buf_guard.cursor();
    let cursor_x = margin_x + cursor_col as f32 * cell_w;
    let cursor_y = margin_y + cursor_row as f32 * cell_h;
    painter.rect_filled(
        Rect::from_min_size(Pos2::new(cursor_x, cursor_y), egui::vec2(cell_w, cell_h)),
        0.0,
        theme.high(),
    );
}