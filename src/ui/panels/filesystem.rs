use std::path::{PathBuf, Path};
use std::fs;
use egui::{Pos2, Align2, FontId, Rect, Ui, ScrollArea, Vec2};
use crate::theme::Theme;
use super::draw_panel_frame;

#[derive(Clone)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub ext: String,
}

pub struct FileExplorerState {
    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
}

impl FileExplorerState {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
        let mut state = Self {
            current_path: PathBuf::from(home),
            entries: Vec::new(),
        };
        state.refresh();
        state
    }

    pub fn refresh(&mut self) {
        self.entries.clear();
        
        if let Ok(read_dir) = fs::read_dir(&self.current_path) {
            let mut dirs = Vec::new();
            let mut files = Vec::new();

            for entry in read_dir.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') { continue; } // Hide hidden files for a cleaner HUD

                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                let metadata = entry.metadata().ok();
                let size = metadata.map(|m| m.len()).unwrap_or(0);
                
                let ext = Path::new(&name)
                    .extension()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_lowercase();

                let fe = FileEntry { name, is_dir, size, ext };
                
                if is_dir {
                    dirs.push(fe);
                } else {
                    files.push(fe);
                }
            }

            // Sort alphabetically
            dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            self.entries.extend(dirs);
            self.entries.extend(files);
        }
    }
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} G", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} M", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.0} K", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn get_icon_for_file(ext: &str) -> &'static str {
    match ext {
        "rs" | "js" | "ts" | "py" | "html" | "css" | "json" | "toml" | "lock" => "", // Code/Config
        "md" | "txt" => "", // Document
        "png" | "jpg" | "jpeg" | "gif" | "svg" => "", // Image
        "mp3" | "wav" | "flac" => "", // Audio
        "mp4" | "mkv" | "avi" => "", // Video
        "zip" | "tar" | "gz" | "7z" => "", // Archive
        _ => "", // Generic file
    }
}

pub fn draw_filesystem(ui: &mut Ui, rect: Rect, explorer: &mut FileExplorerState, theme: &Theme) {
    let painter = ui.painter();
    draw_panel_frame(painter, rect, "FILESYSTEM", theme);

    let chamfer = 12.0;
    
    // Top row: Parent directory absolute path
    let header_y = rect.top() + 32.0;
    let header_x = rect.left() + chamfer + 4.0;
    
    let parent_path = explorer.current_path.parent().unwrap_or_else(|| Path::new("/"));
    let parent_str = parent_path.to_string_lossy();
    let current_name = explorer.current_path.file_name().unwrap_or_default().to_string_lossy();

    painter.text(
        Pos2::new(header_x, header_y),
        Align2::LEFT_TOP,
        format!("{}", parent_str),
        FontId::monospace(12.0),
        theme.dimmed(),
    );

    // Current folder name with icon
    painter.text(
        Pos2::new(header_x, header_y + 16.0),
        Align2::LEFT_TOP,
        format!("  {}", current_name),
        FontId::monospace(14.0),
        theme.high(),
    );

    // Separator line
    let sep_y = header_y + 36.0;
    painter.line_segment(
        [Pos2::new(header_x, sep_y), Pos2::new(rect.right() - chamfer - 4.0, sep_y)],
        egui::Stroke::new(1.0, theme.faint()),
    );

    // Scrollable area for entries (indented)
    let inner_rect = Rect::from_min_max(
        Pos2::new(header_x + 16.0, sep_y + 4.0), // Indent by 16px
        Pos2::new(rect.right() - 8.0, rect.bottom() - 12.0)
    );

    let mut navigate_to = None;
    let mut go_up = false;

    ui.allocate_ui_at_rect(inner_rect, |ui| {
        ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            let row_height = 18.0;
            let available_width = ui.available_width() - 8.0;

            // ".." row to go up
            let (rect_up, response_up) = ui.allocate_exact_size(Vec2::new(available_width, row_height), egui::Sense::click());
            if response_up.hovered() {
                ui.painter().rect_filled(rect_up, 0.0, theme.with_alpha(10));
            }
            if response_up.clicked() {
                go_up = true;
            }
            ui.painter().text(
                Pos2::new(rect_up.left(), rect_up.center().y),
                Align2::LEFT_CENTER,
                "  ..",
                FontId::monospace(13.0),
                theme.full(),
            );

            // Entries
            for entry in &explorer.entries {
                let (row_rect, response) = ui.allocate_exact_size(Vec2::new(available_width, row_height), egui::Sense::click());
                
                if response.hovered() {
                    ui.painter().rect_filled(row_rect, 0.0, theme.with_alpha(10));
                }

                if response.clicked() && entry.is_dir {
                    navigate_to = Some(entry.name.clone());
                }

                let icon = if entry.is_dir { "" } else { get_icon_for_file(&entry.ext) };
                let color = if entry.is_dir { theme.high() } else { theme.dimmed() };

                ui.painter().text(
                    Pos2::new(row_rect.left(), row_rect.center().y),
                    Align2::LEFT_CENTER,
                    format!("{}  {}", icon, entry.name),
                    FontId::monospace(13.0),
                    color,
                );

                if !entry.is_dir {
                    ui.painter().text(
                        Pos2::new(row_rect.right() - 8.0, row_rect.center().y),
                        Align2::RIGHT_CENTER,
                        format_size(entry.size),
                        FontId::monospace(11.0),
                        theme.low(),
                    );
                }
            }
        });
    });

    if let Some(target) = navigate_to {
        explorer.current_path.push(target);
        explorer.refresh();
    } else if go_up {
        if let Some(parent) = explorer.current_path.parent() {
            explorer.current_path = parent.to_path_buf();
            explorer.refresh();
        }
    }
}