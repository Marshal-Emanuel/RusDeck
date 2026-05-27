use egui::{Pos2, Align2, FontId, Rect, Painter};
use crate::app::AppState;
use crate::theme::Theme;
use super::draw_panel_frame;

pub fn draw_processes(painter: &Painter, rect: Rect, state: &AppState, theme: &Theme) {
    draw_panel_frame(painter, rect, "PROCESSES", theme);

    let col_pid = rect.left() + 14.0;
    let col_name = col_pid + 54.0;
    let col_mem = rect.right() - 14.0;
    let col_cpu = col_mem - 58.0;

    let mut y = rect.top() + 38.0;
    let row_h = 22.0; // More compact row height to fit more processes
    let font_size = 12.0;

    // Table Headers
    painter.text(Pos2::new(col_pid, y), Align2::LEFT_TOP, "PID", FontId::monospace(font_size), theme.dimmed());
    painter.text(Pos2::new(col_name, y), Align2::LEFT_TOP, "NAME", FontId::monospace(font_size), theme.dimmed());
    painter.text(Pos2::new(col_cpu, y), Align2::RIGHT_TOP, "CPU%", FontId::monospace(font_size), theme.dimmed());
    painter.text(Pos2::new(col_mem, y), Align2::RIGHT_TOP, "MEM%", FontId::monospace(font_size), theme.dimmed());
    y += 20.0;

    // Calculate how many processes can fit in the remaining height
    let available_h = (rect.bottom() - 10.0) - y;
    let count = ((available_h / row_h).floor() as usize).max(8);

    // Dynamically calculate the maximum characters for process name to avoid overlap
    let name_max_w = col_cpu - col_name - 16.0;
    let char_w = font_size * 0.6; // Approximate width of a monospace character
    let max_chars = ((name_max_w / char_w).floor() as usize).max(5);

    for proc in state.processes.iter().take(count) {
        let name = if proc.name.len() > max_chars {
            format!("{}...", &proc.name[..max_chars.saturating_sub(3)])
        } else {
            proc.name.clone()
        };

        // Render PID
        painter.text(
            Pos2::new(col_pid, y),
            Align2::LEFT_TOP,
            format!("{}", proc.pid),
            FontId::monospace(font_size),
            theme.dimmed(),
        );

        // Render Name
        painter.text(
            Pos2::new(col_name, y),
            Align2::LEFT_TOP,
            name,
            FontId::monospace(font_size),
            theme.low(),
        );

        // Render CPU%
        let cpu_color = if proc.cpu_pct > 50.0 {
            egui::Color32::from_rgba_unmultiplied(255, 80, 80, 255)
        } else if proc.cpu_pct > 20.0 {
            egui::Color32::from_rgba_unmultiplied(255, 180, 60, 255)
        } else {
            theme.full()
        };

        painter.text(
            Pos2::new(col_cpu, y),
            Align2::RIGHT_TOP,
            format!("{:.1}%", proc.cpu_pct),
            FontId::monospace(font_size),
            cpu_color,
        );

        // Render MEM%
        painter.text(
            Pos2::new(col_mem, y),
            Align2::RIGHT_TOP,
            format!("{:.1}%", proc.mem_pct),
            FontId::monospace(font_size),
            theme.full(),
        );

        y += row_h;
    }
}