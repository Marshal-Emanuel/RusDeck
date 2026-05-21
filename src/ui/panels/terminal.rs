use egui::{Pos2, Rect, Painter, FontId, Align2, Color32};
use crate::theme::Theme;
use crate::ui::terminal::TerminalWidget;

pub const CHAMFER: f32 = 10.0;

fn panel_fill() -> Color32 {
    Color32::from_rgba_unmultiplied(8, 12, 10, 255)
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

    let text_color = theme.terminal_text;

    for row_idx in 0..rows {
        let y = margin_y + row_idx as f32 * cell_h;
        if y + cell_h > rect.bottom() - clip - 4.0 {
            break;
        }

        let line_start = 0;
        let mut line_end = 0;

        for col_idx in (0..cols).rev() {
            let cell = buf_guard.lines[row_idx][col_idx];
            if cell.c != ' ' {
                line_end = col_idx + 1;
                break;
            }
        }

        if line_end == 0 {
            continue;
        }

        for col_idx in line_start..line_end {
            let cell = buf_guard.lines[row_idx][col_idx];
            let x = margin_x + col_idx as f32 * cell_w;
            if x + cell_w > rect.right() - clip - 4.0 {
                break;
            }

            if cell.c != ' ' {
                painter.text(
                    Pos2::new(x, y),
                    Align2::LEFT_TOP,
                    cell.c.to_string(),
                    FontId::monospace(12.0),
                    text_color,
                );
            }
        }
    }

    let (cursor_col, cursor_row) = buf_guard.cursor();
    let cursor_x = margin_x + cursor_col as f32 * cell_w;
    let cursor_y = margin_y + cursor_row as f32 * cell_h;

    if cursor_x < rect.right() - clip - 4.0 && cursor_y < rect.bottom() - clip - 4.0 {
        painter.rect_filled(
            Rect::from_min_size(Pos2::new(cursor_x, cursor_y), egui::vec2(cell_w, cell_h)),
            0.0,
            Color32::from_rgba_unmultiplied(0, 255, 204, 60),
        );
    }
}
