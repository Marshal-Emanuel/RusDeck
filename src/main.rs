mod app;
mod monitor;
mod theme;
mod ui;

use std::sync::{Arc, RwLock};
use std::time::Instant;
use app::AppState;
use theme::Theme;
use ui::background::BackgroundCache;
use ui::terminal::{TerminalWidget, TerminalTab};
use ui::panels::filesystem::FileExplorerState;

fn main() -> eframe::Result<()> {
    let state = Arc::new(RwLock::new(AppState::new()));

    eframe::run_native(
        "RusDeck",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_fullscreen(true)
                .with_decorations(false),
            ..Default::default()
        },
        Box::new(move |cc| {
            ui::setup_visuals(&cc.egui_ctx);

            let state_monitor = Arc::clone(&state);
            let ctx_monitor = cc.egui_ctx.clone();
            std::thread::spawn(move || {
                monitor::start_monitor_thread(state_monitor, ctx_monitor);
            });

            let terminal = TerminalWidget::new(100, 30, cc.egui_ctx.clone())
                .expect("Failed to create terminal");
            let terminals = vec![TerminalTab {
                title: "Term 1".to_string(),
                widget: terminal,
            }];

            Box::new(RusDeckApp {
                state,
                theme: Theme::default_white(),
                bg_cache: BackgroundCache::new(),
                terminals,
                active_terminal_idx: 0,
                file_explorer: FileExplorerState::new(),
                cursor_timer: Instant::now(),
            })
        }),
    )
}

struct RusDeckApp {
    state: Arc<RwLock<AppState>>,
    theme: Theme,
    bg_cache: BackgroundCache,
    terminals: Vec<TerminalTab>,
    active_terminal_idx: usize,
    file_explorer: FileExplorerState,
    cursor_timer: Instant,
}

impl eframe::App for RusDeckApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {


        let terminal_id = egui::Id::new("terminal_panel");
        let focused = ctx.memory(|m| m.has_focus(terminal_id));

        if !focused {
            ctx.memory_mut(|m| m.request_focus(terminal_id));
        }

        let now = Instant::now();
        if now.duration_since(self.cursor_timer) >= std::time::Duration::from_millis(500) {
            self.cursor_timer = now;
            if focused {
                ctx.request_repaint();
            }

            // 1. Process exit / cleanup dead terminals
            let mut i = 0;
            while i < self.terminals.len() {
                if !self.terminals[i].widget.is_alive() {
                    self.terminals.remove(i);
                    if self.active_terminal_idx >= self.terminals.len() && !self.terminals.is_empty() {
                        self.active_terminal_idx = self.terminals.len() - 1;
                    }
                } else {
                    i += 1;
                }
            }

            // If all terminals exited, spawn a new default one
            if self.terminals.is_empty() {
                if let Some(new_term) = TerminalWidget::new(100, 30, ctx.clone()) {
                    self.terminals.push(TerminalTab {
                        title: "Term 1".to_string(),
                        widget: new_term,
                    });
                }
                self.active_terminal_idx = 0;
            }

            // 2. Active CWD sync
            if let Some(active_tab) = self.terminals.get(self.active_terminal_idx) {
                if let Some(pid) = active_tab.widget.process_id() {
                    if let Ok(cwd) = std::fs::read_link(format!("/proc/{}/cwd", pid)) {
                        if self.file_explorer.current_path != cwd {
                            self.file_explorer.current_path = cwd;
                            self.file_explorer.refresh();
                        }
                    }
                }
            }
        }

        let dir_change = {
            let state = self.state.read().unwrap();
            ui::draw(
                ctx,
                &state,
                &self.theme,
                &mut self.bg_cache,
                &mut self.terminals,
                &mut self.active_terminal_idx,
                &mut self.file_explorer,
            )
        };

        if let Some(new_path) = dir_change {
            if let Some(active_tab) = self.terminals.get_mut(self.active_terminal_idx) {
                let cmd = format!("cd {}\n", new_path.to_string_lossy());
                active_tab.widget.write_input(cmd.as_bytes());
            }
        }

        ctx.request_repaint_after(std::time::Duration::from_millis(500));
    }
}
