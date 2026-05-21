mod app;
mod monitor;
mod theme;
mod ui;

use std::sync::{Arc, RwLock};
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

    let terminal = TerminalWidget::new(80, 24)
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
}

impl eframe::App for RusDeckApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        let screen_rect = ctx.screen_rect();
        let cell_w = 8.0;
        let cell_h = 16.0;
        let margin_x = 20.0;
        let margin_y = 60.0;
        let clip = 10.0;
        let avail_w = screen_rect.width() - 2.0 * 8.0 - (clip + 4.0) * 2.0;
        let avail_h = screen_rect.height() - 48.0 - 8.0 - 200.0 - 8.0 - 2.0 * 8.0;

        let term_cols = ((avail_w / 2.0 - 20.0) / cell_w).floor() as usize;
        let term_rows = (avail_h / cell_h).floor() as usize;

        if term_cols > 10 && term_rows > 5 {
            self.terminal.resize(term_cols, term_rows);
        }

        {
            let state = self.state.read().unwrap();
            ui::draw(ctx, &state, &self.theme, &mut self.bg_cache, &mut self.terminal);
        }

        let _ = self.repaint_request_rx.try_recv();
        ctx.request_repaint();
    }
}