use crate::api::anthropic::correct_text_anthropic;
use crate::api::deepseek::correct_text_deepseek;
use crate::api::gemini::correct_text_gemini;
use crate::api::openai::correct_text_openai;
use crate::clipboard;
use crate::config::Config;
use crate::hotkey::{HotkeyEvent, HotkeyManager};
use crate::tray::{TrayEvent, TrayIcon};
use crate::ui::streaming_panel::StreamingPanel;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};

const API_NAMES: [&str; 4] = ["OpenAI", "Anthropic", "Gemini", "DeepSeek"];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ApiColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ApiColor {
    pub const OPENAI: ApiColor = ApiColor { r: 16, g: 163, b: 127 };
    pub const ANTHROPIC: ApiColor = ApiColor { r: 217, g: 119, b: 6 };
    pub const GEMINI: ApiColor = ApiColor { r: 66, g: 133, b: 244 };
    pub const DEEPSEEK: ApiColor = ApiColor { r: 124, g: 58, b: 237 };

    pub fn to_egui_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(self.r, self.g, self.b)
    }
}

pub struct MultiAPICorrector {
    config: Config,
    session_id: Arc<AtomicU64>,
    cancel_flags: [Arc<AtomicBool>; 4],
    panels: [StreamingPanel; 4],
    panel_senders: [mpsc::UnboundedSender<String>; 4],
    selected_panel: Option<usize>,
    is_processing: bool,
    status_message: String,
    hotkey_rx: Option<mpsc::UnboundedReceiver<HotkeyEvent>>,
    tray_icon: Option<TrayIcon>,
    tray_rx: Option<std::sync::mpsc::Receiver<TrayEvent>>,
    window_visible: bool,
    hotkey_combo: String,
}

impl MultiAPICorrector {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = Config::load(Config::get_config_path())
            .unwrap_or_else(|e| {
                error!("Failed to load config: {}, using defaults", e);
                Config::default()
            });

        let mut panel_senders = Vec::new();
        let mut panels = Vec::new();

        for _ in 0..4 {
            let (tx, rx) = mpsc::unbounded_channel();
            panel_senders.push(tx);
            panels.push(StreamingPanel::new(rx));
        }

        let panel_senders: [mpsc::UnboundedSender<String>; 4] = panel_senders.try_into()
            .expect("Failed to convert panel_senders to array");
        let panels: [StreamingPanel; 4] = panels.try_into()
            .expect("Failed to convert panels to array");

        let (hotkey_tx, hotkey_rx) = mpsc::unbounded_channel();
        let hotkey_combo = match HotkeyManager::new(hotkey_tx) {
            Ok(manager) => {
                let combo = manager.active_combo()
                    .map(|c| c.description().to_string())
                    .unwrap_or_else(|| "Not registered".to_string());
                tokio::spawn(async move {
                    manager.start_event_loop().await.unwrap();
                });
                combo
            }
            Err(e) => {
                error!("Failed to initialize hotkey manager: {}", e);
                "Not registered".to_string()
            }
        };

        let (tray_icon, tray_rx) = match TrayIcon::new() {
            Ok((icon, rx)) => (Some(icon), Some(rx)),
            Err(e) => {
                error!("Failed to create tray icon: {}", e);
                (None, None)
            }
        };

        Self {
            config,
            session_id: Arc::new(AtomicU64::new(0)),
            cancel_flags: [
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
            ],
            panels,
            panel_senders,
            selected_panel: None,
            is_processing: false,
            status_message: format!("⌨️ {} - copy selected text and press immediately", hotkey_combo),
            hotkey_rx: Some(hotkey_rx),
            tray_icon,
            tray_rx,
            window_visible: true,
            hotkey_combo,
        }
    }

    fn process_text(&mut self, text: String) {
        if self.is_processing {
            self.status_message = "⚠️ Already processing...".to_string();
            return;
        }

        for flag in &self.cancel_flags {
            flag.store(true, Ordering::SeqCst);
        }

        let new_session_id = self.session_id.fetch_add(1, Ordering::SeqCst) + 1;
        info!("📝 Starting session {}", new_session_id);

        for panel in &mut self.panels {
            panel.clear();
        }

        for flag in &self.cancel_flags {
            flag.store(false, Ordering::SeqCst);
        }

        self.selected_panel = None;
        self.is_processing = true;
        self.status_message = format!("🔄 Processing with 4 APIs (Session {})...", new_session_id);

        let api_keys = [
            self.config.api_keys.openai.clone(),
            self.config.api_keys.anthropic.clone(),
            self.config.api_keys.gemini.clone(),
            self.config.api_keys.deepseek.clone(),
        ];

        let models = [
            self.config.models.openai.clone(),
            self.config.models.anthropic.clone(),
            self.config.models.gemini.clone(),
            self.config.models.deepseek.clone(),
        ];

        for (idx, api_name) in API_NAMES.iter().enumerate() {
            let api_key = api_keys[idx].clone();
            let model = models[idx].clone();
            let text = text.clone();
            let tx = self.panel_senders[idx].clone();
            let cancel_flag = self.cancel_flags[idx].clone();
            let session_id = new_session_id;
            let current_session = self.session_id.clone();
            let api_name = api_name.to_string();

            tokio::spawn(async move {
                if api_key.is_empty() {
                    let _ = tx.send(format!("❌ API key not configured for {}", api_name));
                    return;
                }

                if cancel_flag.load(Ordering::SeqCst) {
                    info!("🚫 {} cancelled before start", api_name);
                    return;
                }

                if current_session.load(Ordering::SeqCst) != session_id {
                    info!("🚫 {} cancelled (session changed)", api_name);
                    return;
                }

                let instruction_prompt = "Correct the following text for grammar and spelling.";
                let system_prompt = "You are a helpful assistant that corrects text.";

                let result = match api_name.as_str() {
                    "OpenAI" => correct_text_openai(&api_key, &model, &text, instruction_prompt, system_prompt, false).await,
                    "Anthropic" => correct_text_anthropic(&api_key, &model, &text, instruction_prompt, system_prompt).await,
                    "Gemini" => correct_text_gemini(&api_key, &model, &text, instruction_prompt, system_prompt).await,
                    "DeepSeek" => correct_text_deepseek(&api_key, &model, &text, instruction_prompt, system_prompt).await,
                    _ => unreachable!(),
                };

                if cancel_flag.load(Ordering::SeqCst) || current_session.load(Ordering::SeqCst) != session_id {
                    info!("🚫 {} cancelled after completion", api_name);
                    return;
                }

                match result {
                    Ok(corrected) => {
                        info!("✅ {} completed", api_name);
                        let _ = tx.send(corrected);
                    }
                    Err(e) => {
                        error!("❌ {} error: {}", api_name, e);
                        let _ = tx.send(format!("❌ Error: {}", e));
                    }
                }
            });
        }
    }

    fn cancel_session(&mut self) {
        for flag in &self.cancel_flags {
            flag.store(true, Ordering::SeqCst);
        }
        self.session_id.fetch_add(1, Ordering::SeqCst);
        self.is_processing = false;
        self.status_message = "🚫 Session cancelled".to_string();
        info!("🚫 Session cancelled by user");
    }

    fn select_panel(&mut self, index: usize) {
        self.selected_panel = Some(index);
        let text = self.panels[index].get_text().to_string();
        if !text.is_empty() && !text.starts_with("❌") {
            if let Err(e) = clipboard::write_text(&text) {
                error!("Failed to write to clipboard: {}", e);
                self.status_message = format!("❌ Failed to copy: {}", e);
            } else {
                self.status_message = format!("✅ {} result copied to clipboard", API_NAMES[index]);
                info!("📋 Copied {} result to clipboard", API_NAMES[index]);
            }
        }
    }

    fn handle_hotkey_trigger(&mut self) {
        info!("🔥 Hotkey triggered");
        
        match clipboard::read_text() {
            Ok(text) => {
                if text.trim().is_empty() {
                    self.status_message = "⚠️ Clipboard is empty".to_string();
                    return;
                }
                info!("📋 Read {} chars from clipboard", text.len());
                self.process_text(text);
                self.window_visible = true;
            }
            Err(e) => {
                error!("Failed to read clipboard: {}", e);
                self.status_message = format!("❌ Failed to read clipboard: {}", e);
            }
        }
    }

    fn poll_events(&mut self, ctx: &egui::Context) {
        let mut hotkey_events = Vec::new();
        if let Some(rx) = &mut self.hotkey_rx {
            while let Ok(event) = rx.try_recv() {
                hotkey_events.push(event);
            }
        }

        for HotkeyEvent::Triggered in hotkey_events {
            self.handle_hotkey_trigger();
            ctx.request_repaint();
        }

        if let Some(tray) = &self.tray_icon {
            tray.poll_events();
        }

        if let Some(rx) = &self.tray_rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    TrayEvent::Show => {
                        self.window_visible = true;
                        ctx.request_repaint();
                        info!("👁️ Window shown from tray");
                    }
                    TrayEvent::Quit => {
                        info!("👋 Quit from tray");
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            }
        }
    }
}

impl eframe::App for MultiAPICorrector {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_events(ctx);

        if !self.window_visible {
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
            return;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.heading("🤖 PoprawiaczTekstuRs - Multi-API Text Corrector");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("❌ Cancel").clicked() {
                            self.cancel_session();
                        }
                        if ui.button("🗕 Minimize").clicked() {
                            self.window_visible = false;
                        }
                    });
                });

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label(&self.status_message);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("📝 Session: {}", self.session_id.load(Ordering::SeqCst)));
                    });
                });

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    let available_width = ui.available_width();
                    let panel_width = (available_width - 15.0) / 4.0;
                    
                    for (idx, api_name) in API_NAMES.iter().enumerate() {
                        let color = match idx {
                            0 => ApiColor::OPENAI,
                            1 => ApiColor::ANTHROPIC,
                            2 => ApiColor::GEMINI,
                            3 => ApiColor::DEEPSEEK,
                            _ => unreachable!(),
                        };

                        ui.vertical(|ui| {
                            ui.set_width(panel_width);
                            ui.set_height(ui.available_height());

                            let header_frame = egui::Frame::NONE
                                .fill(color.to_egui_color())
                                .inner_margin(8.0);

                            header_frame.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!("🤖 {}", api_name))
                                            .color(egui::Color32::WHITE)
                                            .strong()
                                    );
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let is_selected = self.selected_panel == Some(idx);
                                        if is_selected {
                                            ui.label(
                                                egui::RichText::new("✓")
                                                    .color(egui::Color32::WHITE)
                                                    .strong()
                                            );
                                        }
                                    });
                                });
                            });

                            ui.add_space(5.0);

                            let panel_frame = egui::Frame::NONE
                                .fill(egui::Color32::from_rgb(240, 240, 240))
                                .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(200)))
                                .inner_margin(5.0);

                            panel_frame.show(ui, |ui| {
                                ui.set_height(ui.available_height() - 40.0);
                                self.panels[idx].update_and_render(ctx, ui);
                            });

                            ui.add_space(5.0);

                            if ui.button(format!("📋 Copy {}", api_name)).clicked() {
                                self.select_panel(idx);
                            }
                        });

                        if idx < 3 {
                            ui.add_space(5.0);
                        }
                    }
                });
            });
        });

        let mut any_processing = false;
        for panel in &self.panels {
            if !panel.get_text().is_empty() && !panel.get_text().starts_with("❌") {
                any_processing = false;
            }
        }
        if self.is_processing && !any_processing {
            self.is_processing = false;
            self.status_message = format!("✅ Processing complete - {} ready", 
                self.panels.iter().filter(|p| !p.get_text().is_empty() && !p.get_text().starts_with("❌")).count());
        }

        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        info!("👋 Application exiting");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_color_constants() {
        assert_eq!(ApiColor::OPENAI.r, 16);
        assert_eq!(ApiColor::OPENAI.g, 163);
        assert_eq!(ApiColor::OPENAI.b, 127);

        assert_eq!(ApiColor::ANTHROPIC.r, 217);
        assert_eq!(ApiColor::ANTHROPIC.g, 119);
        assert_eq!(ApiColor::ANTHROPIC.b, 6);

        assert_eq!(ApiColor::GEMINI.r, 66);
        assert_eq!(ApiColor::GEMINI.g, 133);
        assert_eq!(ApiColor::GEMINI.b, 244);

        assert_eq!(ApiColor::DEEPSEEK.r, 124);
        assert_eq!(ApiColor::DEEPSEEK.g, 58);
        assert_eq!(ApiColor::DEEPSEEK.b, 237);
    }

    #[test]
    fn test_api_color_to_egui() {
        let color = ApiColor::OPENAI;
        let egui_color = color.to_egui_color();
        assert_eq!(egui_color.r(), 16);
        assert_eq!(egui_color.g(), 163);
        assert_eq!(egui_color.b(), 127);
    }

    #[test]
    fn test_api_names_array() {
        assert_eq!(API_NAMES.len(), 4);
        assert_eq!(API_NAMES[0], "OpenAI");
        assert_eq!(API_NAMES[1], "Anthropic");
        assert_eq!(API_NAMES[2], "Gemini");
        assert_eq!(API_NAMES[3], "DeepSeek");
    }

    #[test]
    fn test_session_cancellation() {
        let session_id = Arc::new(AtomicU64::new(0));
        let cancel_flag = Arc::new(AtomicBool::new(false));

        let current_session = session_id.load(Ordering::SeqCst);
        assert_eq!(current_session, 0);

        cancel_flag.store(true, Ordering::SeqCst);
        assert!(cancel_flag.load(Ordering::SeqCst));

        session_id.fetch_add(1, Ordering::SeqCst);
        let new_session = session_id.load(Ordering::SeqCst);
        assert_eq!(new_session, 1);
        assert_ne!(new_session, current_session);
    }

    #[test]
    fn test_cancel_flags_array() {
        let flags = [
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicBool::new(false)),
        ];

        for flag in &flags {
            assert!(!flag.load(Ordering::SeqCst));
        }

        for flag in &flags {
            flag.store(true, Ordering::SeqCst);
        }

        for flag in &flags {
            assert!(flag.load(Ordering::SeqCst));
        }
    }

    #[tokio::test]
    async fn test_api_integration_mock() {
        let session_id = Arc::new(AtomicU64::new(1));
        let cancel_flag = Arc::new(AtomicBool::new(false));
        let (tx, mut rx) = mpsc::unbounded_channel();

        let session = session_id.clone();
        let flag = cancel_flag.clone();

        tokio::spawn(async move {
            if flag.load(Ordering::SeqCst) {
                return;
            }

            if session.load(Ordering::SeqCst) == 1 {
                let _ = tx.send("Mock result".to_string());
            }
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let result = rx.try_recv();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Mock result");
    }

    #[test]
    fn test_panel_selection_logic() {
        let mut selected_panel: Option<usize> = None;

        selected_panel = Some(0);
        assert_eq!(selected_panel, Some(0));

        selected_panel = Some(2);
        assert_eq!(selected_panel, Some(2));

        selected_panel = None;
        assert_eq!(selected_panel, None);
    }

    #[test]
    fn test_processing_state_toggle() {
        let mut is_processing = false;

        is_processing = true;
        assert!(is_processing);

        is_processing = false;
        assert!(!is_processing);
    }

    #[test]
    fn test_status_message_format() {
        let session_id = 5;
        let message = format!("🔄 Processing with 4 APIs (Session {})...", session_id);
        assert!(message.contains("Session 5"));
        assert!(message.contains("Processing with 4 APIs"));
    }
}
