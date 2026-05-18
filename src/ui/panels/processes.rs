use egui::{Pos2, Align2, FontId, Rect, Painter};
use crate::app::AppState;
use crate::theme::Theme;
use super::draw_panel_frame;

pub fn draw_processes(painter: &Painter, rect: Rect, state: &AppState, theme: &Theme) {
    draw_panel_frame(painter, rect, "PROCESSES", theme);

    let chamfer = 10.0;
    let col_pid = rect.left() + chamfer + 4.0;
    let col_name = col_pid + 48.0;
    let col_cpu = rect.right() - chamfer - 74.0;
    let col_mem = rect.right() - chamfer - 12.0;

    let bar_max_width = col_mem - col_cpu - 24.0;

    let mut y = rect.top() + 38.0;
    let row_h = 28.0;

    painter.text(Pos2::new(col_pid, y), Align2::LEFT_TOP, "PID",         FontId::monospace(13.0), theme.dimmed());
    painter.text(Pos2::new(col_name, y), Align2::LEFT_TOP, "NAME",         FontId::monospace(13.0), theme.dimmed());
    painter.text(Pos2::new(col_cpu, y), Align2::LEFT_TOP, "CPU%",         FontId::monospace(13.0), theme.dimmed());
    painter.text(Pos2::new(col_mem, y), Align2::RIGHT_TOP, "MEM%",         FontId::monospace(13.0), theme.dimmed());
    y += 20.0;

    for proc in state.processes.iter().take(8) {
        let name = if proc.name.len() > 10 {
            format!("{}...", &proc.name[..8])
        } else {
            proc.name.clone()
        };

        painter.text(
            Pos2::new(col_pid, y),
            Align2::LEFT_TOP,
            format!("{}", proc.pid),
            FontId::monospace(14.0),
            theme.dimmed(),
        );
        painter.text(
            Pos2::new(col_name, y),
            Align2::LEFT_TOP,
            name,
            FontId::monospace(14.0),
            theme.low(),
        );

        let cpu_color = if proc.cpu_pct > 50.0 {
            egui::Color32::from_rgba_unmultiplied(255, 80, 80, 255)
        } else if proc.cpu_pct > 20.0 {
            egui::Color32::from_rgba_unmultiplied(255, 180, 60, 255)
        } else {
            theme.full()
        };

        let cpu_bar_w = (proc.cpu_pct / 100.0).min(1.0) * bar_max_width;
        let mem_bar_w = (proc.mem_pct / 100.0).min(1.0) * bar_max_width;

        draw_bar(painter, Pos2::new(col_cpu, y + row_h - 5.0), cpu_bar_w, cpu_color);
        draw_bar(painter, Pos2::new(col_mem - bar_max_width, y + row_h - 5.0), mem_bar_w, theme.full());

        painter.text(
            Pos2::new(col_cpu, y),
            Align2::LEFT_TOP,
            format!("{:>4.1}", proc.cpu_pct),
            FontId::monospace(14.0),
            theme.full(),
        );
        painter.text(
            Pos2::new(col_mem, y),
            Align2::RIGHT_TOP,
            format!("{:>4.1}", proc.mem_pct),
            FontId::monospace(14.0),
            theme.full(),
        );

        y += row_h;
    }
}

fn draw_bar(painter: &Painter, pos: Pos2, width: f32, color: egui::Color32) {
    if width < 1.0 { return; }
    painter.rect_filled(Rect::from_min_size(pos, egui::vec2(width, 2.0)), 0.0, color);
}