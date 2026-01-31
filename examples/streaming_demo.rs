use eframe::egui;
use poprawiacz_tekstu_rs::ui::StreamingPanel;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

struct StreamingDemoApp {
    panel: StreamingPanel,
    tx: mpsc::UnboundedSender<String>,
    is_streaming: Arc<AtomicBool>,
    stream_count: usize,
}

impl StreamingDemoApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let panel = StreamingPanel::new(rx);

        Self {
            panel,
            tx,
            is_streaming: Arc::new(AtomicBool::new(false)),
            stream_count: 0,
        }
    }

    fn start_streaming(&mut self) {
        if self.is_streaming.load(Ordering::Relaxed) {
            return;
        }

        self.panel.clear();
        self.is_streaming.store(true, Ordering::Relaxed);
        self.stream_count += 1;

        let tx = self.tx.clone();
        let is_streaming = self.is_streaming.clone();
        let stream_id = self.stream_count;

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let _ = tx.send(format!("=== Stream {} Started ===\n\n", stream_id));

                let sentences = [
                    "This is a demonstration of streaming text updates.",
                    "Each sentence appears progressively with a delay.",
                    "The panel automatically scrolls to show new content.",
                    "You can stop the stream at any time using the button below.",
                    "The channel pattern allows async code to communicate with the GUI.",
                    "This architecture prevents blocking the UI thread.",
                    "Text accumulates in the panel as it arrives.",
                    "Perfect for displaying API responses in real-time!",
                ];

                for (i, sentence) in sentences.iter().enumerate() {
                    if !is_streaming.load(Ordering::Relaxed) {
                        let _ = tx.send("\n[Stream cancelled by user]\n".to_string());
                        break;
                    }

                    let _ = tx.send(format!("{}. {}\n\n", i + 1, sentence));
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }

                if is_streaming.load(Ordering::Relaxed) {
                    let _ = tx.send(format!("\n=== Stream {} Completed ===\n", stream_id));
                    is_streaming.store(false, Ordering::Relaxed);
                }
            });
        });
    }

    fn stop_streaming(&mut self) {
        if self.is_streaming.load(Ordering::Relaxed) {
            self.is_streaming.store(false, Ordering::Relaxed);
        }
    }
}

impl eframe::App for StreamingDemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Streaming Panel Demo");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        !self.is_streaming.load(Ordering::Relaxed),
                        egui::Button::new("Start Streaming"),
                    )
                    .clicked()
                {
                    self.start_streaming();
                }

                if ui
                    .add_enabled(
                        self.is_streaming.load(Ordering::Relaxed),
                        egui::Button::new("Stop Streaming"),
                    )
                    .clicked()
                {
                    self.stop_streaming();
                }

                if ui.button("Clear Panel").clicked() {
                    self.panel.clear();
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.label(format!(
                "Status: {}",
                if self.is_streaming.load(Ordering::Relaxed) {
                    "Streaming..."
                } else {
                    "Idle"
                }
            ));

            ui.add_space(10.0);

            ui.group(|ui| {
                ui.set_min_height(400.0);
                self.panel.update_and_render(ctx, ui);
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Streaming Panel Demo"),
        ..Default::default()
    };

    eframe::run_native(
        "Streaming Demo",
        options,
        Box::new(|cc| Ok(Box::new(StreamingDemoApp::new(cc)))),
    )
}
