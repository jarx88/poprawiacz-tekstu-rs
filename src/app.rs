use crate::api::anthropic::correct_text_anthropic_with_callback;
use crate::api::deepseek::correct_text_deepseek_with_callback;
use crate::api::gemini::correct_text_gemini_with_callback;
use crate::api::openai::correct_text_openai_with_callback;
use crate::clipboard;
use crate::config::Config;
use crate::prompts::{get_instruction_prompt, get_system_prompt, CorrectionStyle};
use crate::ui::settings_gtk::SettingsDialog;

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::pango;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tracing::error;

const API_NAMES: [&str; 4] = ["OpenAI", "Anthropic", "Gemini", "DeepSeek"];
const API_COLORS: [(u8, u8, u8); 4] = [
    (16, 163, 127),
    (217, 119, 6),
    (66, 133, 244),
    (124, 58, 237),
];

enum ApiEvent {
    Chunk(String),
    Complete(Result<String, String>, f64),
}

#[derive(Clone)]
struct ApiPanel {
    container: gtk4::Box,
    header_label: gtk4::Label,
    time_label: gtk4::Label,
    spinner: gtk4::Spinner,
    text_view: gtk4::TextView,
    select_button: gtk4::Button,
}

struct AppState {
    config: Config,
    original_text: String,
    results: [Option<String>; 4],
    selected_panel: Option<usize>,
    is_processing: bool,
    cancel_flags: [Arc<AtomicBool>; 4],
}

pub struct MainWindow {
    window: adw::ApplicationWindow,
    panels: [ApiPanel; 4],
    status_label: gtk4::Label,
    paste_button: gtk4::Button,
    cancel_button: gtk4::Button,
    settings_button: gtk4::Button,
    state: Rc<RefCell<AppState>>,
}

impl MainWindow {
    pub fn new(app: &adw::Application) -> Self {
        let config = Config::load(Config::get_config_path()).unwrap_or_default();

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Poprawiacz Tekstu")
            .default_width(1200)
            .default_height(800)
            .build();

        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

        let header = adw::HeaderBar::new();
        let title = adw::WindowTitle::new("Poprawiacz Tekstu", "Korekta tekstu przez AI");
        header.set_title_widget(Some(&title));
        main_box.append(&header);

        let content_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        content_box.set_margin_start(12);
        content_box.set_margin_end(12);
        content_box.set_margin_top(12);
        content_box.set_margin_bottom(12);

        let panels_grid = gtk4::Grid::new();
        panels_grid.set_row_spacing(12);
        panels_grid.set_column_spacing(12);
        panels_grid.set_row_homogeneous(true);
        panels_grid.set_column_homogeneous(true);
        panels_grid.set_vexpand(true);

        let panels: [ApiPanel; 4] = std::array::from_fn(|i| {
            let panel = Self::create_api_panel(i);
            let row = (i / 2) as i32;
            let col = (i % 2) as i32;
            panels_grid.attach(&panel.container, col, row, 1, 1);
            panel
        });

        content_box.append(&panels_grid);

        let bottom_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        bottom_box.set_halign(gtk4::Align::Center);
        bottom_box.set_margin_top(12);

        let paste_button = gtk4::Button::with_label("📋 Wklej i przetwórz");
        paste_button.add_css_class("suggested-action");
        paste_button.add_css_class("pill");
        bottom_box.append(&paste_button);

        let cancel_button = gtk4::Button::with_label("❌ Anuluj");
        cancel_button.add_css_class("destructive-action");
        cancel_button.add_css_class("pill");
        cancel_button.set_sensitive(false);
        bottom_box.append(&cancel_button);

        let settings_button = gtk4::Button::with_label("⚙️ Ustawienia");
        settings_button.add_css_class("pill");
        bottom_box.append(&settings_button);

        content_box.append(&bottom_box);

        let status_label = gtk4::Label::new(Some("Gotowy. Naciśnij Ctrl+Shift+C lub wklej tekst."));
        status_label.add_css_class("dim-label");
        status_label.set_margin_top(8);
        content_box.append(&status_label);

        main_box.append(&content_box);
        window.set_content(Some(&main_box));

        let state = Rc::new(RefCell::new(AppState {
            config,
            original_text: String::new(),
            results: [None, None, None, None],
            selected_panel: None,
            is_processing: false,
            cancel_flags: std::array::from_fn(|_| Arc::new(AtomicBool::new(false))),
        }));

        let main_window = Self {
            window,
            panels,
            status_label,
            paste_button,
            cancel_button,
            settings_button,
            state,
        };

        main_window.connect_signals();
        main_window
    }

    fn create_api_panel(index: usize) -> ApiPanel {
        let (r, g, b) = API_COLORS[index];
        let color_css = format!("rgba({}, {}, {}, 0.15)", r, g, b);
        let border_css = format!("rgb({}, {}, {})", r, g, b);

        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        container.add_css_class("card");
        container.set_vexpand(true);

        let css = format!(
            "box {{ background: {}; border: 2px solid {}; border-radius: 12px; }}",
            color_css, border_css
        );
        let provider = gtk4::CssProvider::new();
        provider.load_from_string(&css);
        gtk4::style_context_add_provider_for_display(
            &container.display(),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        header_box.set_margin_start(12);
        header_box.set_margin_end(12);
        header_box.set_margin_top(8);
        header_box.set_margin_bottom(4);

        let header_label = gtk4::Label::new(Some(API_NAMES[index]));
        header_label.add_css_class("heading");
        header_label.set_hexpand(true);
        header_label.set_halign(gtk4::Align::Start);
        header_box.append(&header_label);

        let time_label = gtk4::Label::new(Some(""));
        time_label.add_css_class("dim-label");
        header_box.append(&time_label);

        let spinner = gtk4::Spinner::new();
        spinner.set_visible(false);
        header_box.append(&spinner);

        container.append(&header_box);

        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_margin_start(8);
        scrolled.set_margin_end(8);
        scrolled.set_margin_bottom(8);

        let text_view = gtk4::TextView::new();
        text_view.set_editable(false);
        text_view.set_wrap_mode(gtk4::WrapMode::Word);
        text_view.set_left_margin(8);
        text_view.set_right_margin(8);
        text_view.set_top_margin(8);
        text_view.set_bottom_margin(8);
        text_view.buffer().set_text("Oczekiwanie na tekst...");

        let font_desc = pango::FontDescription::from_string("Monospace 11");
        let attr_list = pango::AttrList::new();
        let attr = pango::AttrFontDesc::new(&font_desc);
        attr_list.insert(attr);

        scrolled.set_child(Some(&text_view));
        container.append(&scrolled);

        let select_button = gtk4::Button::with_label("✅ Wybierz");
        select_button.set_margin_start(8);
        select_button.set_margin_end(8);
        select_button.set_margin_bottom(8);
        select_button.set_sensitive(false);
        select_button.add_css_class("pill");
        container.append(&select_button);

        ApiPanel {
            container,
            header_label,
            time_label,
            spinner,
            text_view,
            select_button,
        }
    }

    fn connect_signals(&self) {
        let state = self.state.clone();
        let panels = self.panels.clone();
        let status_label = self.status_label.clone();
        let cancel_button = self.cancel_button.clone();

        self.paste_button.connect_clicked({
            let state = state.clone();
            let panels = panels.clone();
            let status_label = status_label.clone();
            let cancel_button = cancel_button.clone();
            move |_| {
                if let Ok(text) = clipboard::read_text() {
                    if !text.is_empty() {
                        Self::process_text(
                            state.clone(),
                            panels.clone(),
                            status_label.clone(),
                            cancel_button.clone(),
                            text,
                        );
                    }
                }
            }
        });

        self.cancel_button.connect_clicked({
            let state = state.clone();
            let status_label = status_label.clone();
            let cancel_button = cancel_button.clone();
            move |_| {
                let state_ref = state.borrow();
                for flag in &state_ref.cancel_flags {
                    flag.store(true, Ordering::SeqCst);
                }
                drop(state_ref);
                status_label.set_text("Anulowano.");
                cancel_button.set_sensitive(false);
            }
        });

        let window = self.window.clone();
        self.settings_button.connect_clicked({
            let state = state.clone();
            move |_| {
                let config = state.borrow().config.clone();
                let dialog = SettingsDialog::new(
                    window.downcast_ref::<adw::ApplicationWindow>().unwrap(),
                    &config,
                );
                let state_clone = state.clone();
                dialog.connect_save(move |new_config| {
                    if let Err(e) = new_config.save(Config::get_config_path()) {
                        error!("Failed to save config: {}", e);
                    }
                    state_clone.borrow_mut().config = new_config;
                });
                dialog.present();
            }
        });

        for (i, panel) in self.panels.iter().enumerate() {
            let state = state.clone();
            let status_label = status_label.clone();
            panel.select_button.connect_clicked(move |_| {
                let state_ref = state.borrow();
                if let Some(ref result) = state_ref.results[i] {
                    if let Err(e) = clipboard::write_text(result) {
                        error!("Failed to copy to clipboard: {}", e);
                    } else {
                        drop(state_ref);
                        state.borrow_mut().selected_panel = Some(i);
                        status_label.set_text(&format!(
                            "Skopiowano wynik z {} do schowka!",
                            API_NAMES[i]
                        ));
                    }
                }
            });
        }
    }

    fn process_text(
        state: Rc<RefCell<AppState>>,
        panels: [ApiPanel; 4],
        status_label: gtk4::Label,
        cancel_button: gtk4::Button,
        text: String,
    ) {
        {
            let mut state_mut = state.borrow_mut();
            state_mut.original_text = text.clone();
            state_mut.is_processing = true;
            state_mut.results = [None, None, None, None];
            state_mut.selected_panel = None;
            for flag in &state_mut.cancel_flags {
                flag.store(false, Ordering::SeqCst);
            }
        }

        status_label.set_text("Przetwarzanie...");
        cancel_button.set_sensitive(true);

        for panel in &panels {
            panel.spinner.set_visible(true);
            panel.spinner.start();
            panel.time_label.set_text("");
            panel.text_view.buffer().set_text("🔄 Przetwarzanie...");
            panel.select_button.set_sensitive(false);
        }

        let config = state.borrow().config.clone();
        let cancel_flags = state.borrow().cancel_flags.clone();

        let system_prompt = get_system_prompt(CorrectionStyle::Normal);
        let instruction = get_instruction_prompt(CorrectionStyle::Normal);

        for i in 0..4 {
            let text = text.clone();
            let config = config.clone();
            let system = system_prompt.to_string();
            let instr = instruction.to_string();
            let cancel = cancel_flags[i].clone();
            let state = state.clone();
            let panel = panels[i].clone();
            let status_label = status_label.clone();
            let cancel_button = cancel_button.clone();

            let (tx, rx) = async_channel::unbounded::<ApiEvent>();

            glib::spawn_future_local({
                let panel = panel.clone();
                let state = state.clone();
                let status_label = status_label.clone();
                let cancel_button = cancel_button.clone();
                async move {
                    while let Ok(event) = rx.recv().await {
                        match event {
                            ApiEvent::Chunk(chunk) => {
                                let buffer = panel.text_view.buffer();
                                let mut end = buffer.end_iter();
                                buffer.insert(&mut end, &chunk);
                            }
                            ApiEvent::Complete(result, elapsed) => {
                                panel.spinner.stop();
                                panel.spinner.set_visible(false);
                                panel.time_label.set_text(&format!("({:.1}s)", elapsed));

                                match result {
                                    Ok(ref text) => {
                                        panel.text_view.buffer().set_text(text);
                                        panel.select_button.set_sensitive(true);
                                        state.borrow_mut().results[i] = Some(text.to_string());
                                    }
                                    Err(ref e) => {
                                        panel.text_view.buffer().set_text(&format!("❌ Błąd: {}", e));
                                    }
                                }

                                let completed = state
                                    .borrow()
                                    .results
                                    .iter()
                                    .filter(|r| r.is_some())
                                    .count();
                                if completed == 4 {
                                    state.borrow_mut().is_processing = false;
                                    cancel_button.set_sensitive(false);
                                    status_label.set_text("Gotowe! Wybierz najlepszy wynik.");
                                }
                                break;
                            }
                        }
                    }
                }
            });

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let start = Instant::now();

                    let tx_chunk = tx.clone();
                    let cancel_check = cancel.clone();

                    let chunk_callback = move |chunk: &str| {
                        if !cancel_check.load(Ordering::SeqCst) {
                            let _ = tx_chunk.send_blocking(ApiEvent::Chunk(chunk.to_string()));
                        }
                    };

                    let result = match i {
                        0 => {
                            correct_text_openai_with_callback(
                                &config.api_keys.openai,
                                &config.models.openai,
                                &text,
                                &instr,
                                &system,
                                true,
                                Some(chunk_callback),
                            )
                            .await
                        }
                        1 => {
                            correct_text_anthropic_with_callback(
                                &config.api_keys.anthropic,
                                &config.models.anthropic,
                                &text,
                                &instr,
                                &system,
                                true,
                                Some(chunk_callback),
                            )
                            .await
                        }
                        2 => {
                            correct_text_gemini_with_callback(
                                &config.api_keys.gemini,
                                &config.models.gemini,
                                &text,
                                &instr,
                                &system,
                                true,
                                Some(chunk_callback),
                            )
                            .await
                        }
                        3 => {
                            correct_text_deepseek_with_callback(
                                &config.api_keys.deepseek,
                                &config.models.deepseek,
                                &text,
                                &instr,
                                &system,
                                true,
                                Some(chunk_callback),
                            )
                            .await
                        }
                        _ => unreachable!(),
                    };

                    let elapsed = start.elapsed().as_secs_f64();
                    let _ = tx.send_blocking(ApiEvent::Complete(
                        result.map_err(|e| e.to_string()),
                        elapsed,
                    ));
                });
            });
        }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
