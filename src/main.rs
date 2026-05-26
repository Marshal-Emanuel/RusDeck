mod app;
mod monitor;
mod theme;
mod ui;

use std::sync::{Arc, RwLock};
use std::time::Instant;
use app::AppState;
use theme::Theme;
use ui::background::BackgroundCache;
use ui::terminal::TerminalWidget;

fn main() -> eframe::Result<()> {
    let state = Arc::new(RwLock::new(AppState::new()));
    let (repaint_request_tx, repaint_request_rx) = std::sync::mpsc::channel::<()>();

    let state_monitor = Arc::clone(&state);
    let tx = repaint_request_tx;
    std::thread::spawn(move || {
        monitor::start_monitor_thread(state_monitor, tx);
    });

    let terminal = TerminalWidget::new(100, 30)
        .expect("Failed to create terminal");

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
            Box::new(RusDeckApp {
                state,
                theme: Theme::default_white(),
                repaint_request_rx,
                bg_cache: BackgroundCache::new(),
                terminal,
                cursor_timer: Instant::now(),
            })
        }),
    )
}

struct RusDeckApp {
    state: Arc<RwLock<AppState>>,
    theme: Theme,
    repaint_request_rx: std::sync::mpsc::Receiver<()>,
    bg_cache: BackgroundCache,
    terminal: TerminalWidget,
    cursor_timer: Instant,
}

impl eframe::App for RusDeckApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

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
        }

        {
            let state = self.state.read().unwrap();
            ui::draw(ctx, &state, &self.theme, &mut self.bg_cache, &mut self.terminal);
        }

        if self.repaint_request_rx.try_recv().is_ok() {
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(std::time::Duration::from_secs(1));
        }
    }
}
