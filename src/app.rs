use crate::api::anthropic::correct_text_anthropic;
use crate::api::deepseek::correct_text_deepseek;
use crate::api::gemini::correct_text_gemini;
use crate::api::openai::correct_text_openai_with_callback;
use crate::clipboard;
use crate::config::Config;
use crate::diff_gtk::set_text_with_diff;
use crate::hotkey::{HotkeyEvent, HotkeyManager};
use crate::prompts::{get_instruction_prompt, get_system_prompt, CorrectionStyle};
use crate::tray::TrayManager;
use crate::ui::SettingsDialog;

use gtk4::prelude::*;
use gtk4::{gdk, glib};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tracing::{error, info};

const API_NAMES: [&str; 4] = ["OpenAI", "Anthropic", "Gemini", "DeepSeek"];

#[derive(Clone, Copy)]
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

    pub fn for_index(index: usize) -> ApiColor {
        match index {
            0 => Self::OPENAI,
            1 => Self::ANTHROPIC,
            2 => Self::GEMINI,
            3 => Self::DEEPSEEK,
            _ => Self::OPENAI,
        }
    }

    pub fn to_css(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
    
    pub fn to_rgba(&self, alpha: f32) -> String {
        format!("rgba({}, {}, {}, {})", self.r, self.g, self.b, alpha)
    }
}

#[derive(Clone)]
struct PanelState {
    text_view: gtk4::TextView,
    spinner: gtk4::Spinner,
    progress_bar: gtk4::ProgressBar,
    time_label: gtk4::Label,
    status_icon: gtk4::Label,
    name_label: gtk4::Label,
    header_box: gtk4::Box,
    use_button: gtk4::Button,
    cancel_button: gtk4::Button,
    result_text: Rc<RefCell<String>>,
    start_time: Rc<RefCell<Option<Instant>>>,
    is_processing: Rc<RefCell<bool>>,
    is_completed: Rc<RefCell<bool>>,
    has_error: Rc<RefCell<bool>>,
}

struct AppState {
    config: Rc<RefCell<Config>>,
    session_id: Arc<AtomicU64>,
    cancel_flags: [Arc<AtomicBool>; 4],
    original_text: Rc<RefCell<String>>,
    panels: [PanelState; 4],
    status_label: gtk4::Label,
    session_label: gtk4::Label,
    api_counter_label: gtk4::Label,
    hint_label: gtk4::Label,
    completed_count: Rc<RefCell<u32>>,
    window: adw::ApplicationWindow,
}

pub struct MainWindow;

impl MainWindow {
    pub fn new(app: &adw::Application) -> adw::ApplicationWindow {
        let config_path = Config::get_config_path();
        let config = Config::load(&config_path).unwrap_or_default();
        
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("PoprawiaczTekstuRs - Multi-API")
            .default_width(1200)
            .default_height(800)
            .build();

        Self::setup_layer_shell(&window);
        Self::apply_css();

        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        main_box.add_css_class("main-container");

        let (header, settings_btn, paste_btn) = Self::build_header();
        main_box.append(&header);

        let (info_bar, status_label, session_label, api_counter_label, hint_label) = Self::build_info_bar();
        main_box.append(&info_bar);

        let panels_grid = gtk4::Grid::builder()
            .row_spacing(12)
            .column_spacing(12)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .hexpand(true)
            .vexpand(true)
            .build();

        let panels = Self::create_panels();
        
        for (i, panel) in panels.iter().enumerate() {
            let row = (i / 2) as i32;
            let col = (i % 2) as i32;
            
            let panel_frame = Self::build_panel_frame(i, panel);
            panels_grid.attach(&panel_frame, col, row, 1, 1);
        }

        main_box.append(&panels_grid);

        let (toolbar, cancel_btn, original_btn, hide_btn) = Self::build_toolbar();
        main_box.append(&toolbar);

        window.set_content(Some(&main_box));

        let state = Rc::new(RefCell::new(AppState {
            config: Rc::new(RefCell::new(config)),
            session_id: Arc::new(AtomicU64::new(0)),
            cancel_flags: std::array::from_fn(|_| Arc::new(AtomicBool::new(false))),
            original_text: Rc::new(RefCell::new(String::new())),
            panels: panels.clone(),
            status_label,
            session_label,
            api_counter_label,
            hint_label,
            completed_count: Rc::new(RefCell::new(0)),
            window: window.clone(),
        }));

        Self::connect_panel_buttons(state.clone());
        
        Self::connect_buttons(
            state.clone(),
            settings_btn,
            paste_btn,
            cancel_btn,
            original_btn,
            hide_btn,
            window.clone(),
        );

        Self::setup_hotkey(state.clone());
        Self::setup_tray(window.clone());
        Self::setup_close_handler(window.clone());

        window
    }

    fn setup_layer_shell(_window: &adw::ApplicationWindow) {
        #[cfg(feature = "wayland")]
        {
            if gtk4_layer_shell::is_supported() {
                gtk4_layer_shell::init_for_window(_window);
                info!("Layer shell initialized - window will hide from dock");
            }
        }
    }

    fn apply_css() {
        let css = r#"
            .main-container {
                background-color: #1e1e23;
            }
            .info-bar {
                background-color: #252530;
                padding: 8px 16px;
                border-bottom: 1px solid #3a3a45;
            }
            .status-label {
                font-size: 15px;
                font-weight: bold;
                color: #ffffff;
            }
            .info-label {
                font-size: 13px;
                color: #a0a0a0;
                margin-left: 16px;
            }
            .hint-label {
                font-size: 13px;
                color: #808080;
                margin-left: 16px;
            }
            .panel-frame {
                border-radius: 8px;
                background-color: #2a2a32;
                border: 1px solid #3a3a45;
            }
            .panel-title {
                font-weight: bold;
                font-size: 14px;
                color: white;
                padding: 8px 12px;
            }
            .time-label {
                font-size: 12px;
                color: rgba(255,255,255,0.7);
                padding-right: 8px;
            }
            .status-icon {
                font-size: 16px;
                padding-left: 8px;
            }
            .cancel-btn {
                padding: 2px 6px;
                min-width: 24px;
                min-height: 24px;
                background: rgba(255,255,255,0.1);
                border-radius: 4px;
            }
            .cancel-btn:hover {
                background: rgba(255,0,0,0.3);
            }
            .toolbar {
                background-color: #252530;
                padding: 12px;
                border-top: 1px solid #3a3a45;
            }
            .use-button {
                font-weight: bold;
                padding: 8px 16px;
                border-radius: 6px;
                color: white;
            }
            .use-button:disabled {
                opacity: 0.5;
            }
            .use-button-0 { background-color: #10a37f; }
            .use-button-0:hover { background-color: #0d8a6a; }
            .use-button-1 { background-color: #d97706; }
            .use-button-1:hover { background-color: #b86305; }
            .use-button-2 { background-color: #4285f4; }
            .use-button-2:hover { background-color: #3367d6; }
            .use-button-3 { background-color: #7c3aed; }
            .use-button-3:hover { background-color: #6429c9; }
            textview {
                background-color: #2a2a32;
                color: #e0e0e0;
                font-family: system-ui, -apple-system, sans-serif;
                font-size: 13px;
            }
            textview text {
                background-color: #2a2a32;
                color: #e0e0e0;
            }
            .panel-header-0 { background-color: #10a37f; border-radius: 8px 8px 0 0; }
            .panel-header-1 { background-color: #d97706; border-radius: 8px 8px 0 0; }
            .panel-header-2 { background-color: #4285f4; border-radius: 8px 8px 0 0; }
            .panel-header-3 { background-color: #7c3aed; border-radius: 8px 8px 0 0; }
            progressbar trough {
                min-height: 3px;
                background-color: rgba(255,255,255,0.1);
            }
            progressbar progress {
                min-height: 3px;
                background-color: rgba(255,255,255,0.8);
            }
        "#;

        let provider = gtk4::CssProvider::new();
        provider.load_from_data(css);

        gtk4::style_context_add_provider_for_display(
            &gdk::Display::default().expect("Could not get display"),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    fn build_header() -> (adw::HeaderBar, gtk4::Button, gtk4::Button) {
        let header = adw::HeaderBar::new();
        header.set_title_widget(Some(&gtk4::Label::new(Some("PoprawiaczTekstuRs - Multi-API"))));

        let settings_btn = gtk4::Button::from_icon_name("emblem-system-symbolic");
        settings_btn.set_tooltip_text(Some("Ustawienia"));
        header.pack_end(&settings_btn);

        let paste_btn = gtk4::Button::with_label("📋 Wklej tekst");
        paste_btn.add_css_class("suggested-action");
        header.pack_start(&paste_btn);

        (header, settings_btn, paste_btn)
    }

    fn build_info_bar() -> (gtk4::Box, gtk4::Label, gtk4::Label, gtk4::Label, gtk4::Label) {
        let info_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        info_bar.add_css_class("info-bar");

        let status_label = gtk4::Label::new(Some("⌨️ Ctrl+Shift+C - zaznacz tekst i naciśnij"));
        status_label.add_css_class("status-label");
        status_label.set_halign(gtk4::Align::Start);
        info_bar.append(&status_label);

        let session_label = gtk4::Label::new(Some("📝 Sesja: 0"));
        session_label.add_css_class("info-label");
        info_bar.append(&session_label);

        let api_counter_label = gtk4::Label::new(Some("🤖 API: 0/4"));
        api_counter_label.add_css_class("info-label");
        info_bar.append(&api_counter_label);

        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        info_bar.append(&spacer);

        let hint_label = gtk4::Label::new(Some(""));
        hint_label.add_css_class("hint-label");
        info_bar.append(&hint_label);

        (info_bar, status_label, session_label, api_counter_label, hint_label)
    }

    fn create_panels() -> [PanelState; 4] {
        std::array::from_fn(|i| {
            let text_view = gtk4::TextView::builder()
                .editable(false)
                .wrap_mode(gtk4::WrapMode::Word)
                .cursor_visible(false)
                .left_margin(12)
                .right_margin(12)
                .top_margin(12)
                .bottom_margin(12)
                .build();
            text_view.buffer().set_text("Oczekiwanie na tekst...");

            let spinner = gtk4::Spinner::new();
            spinner.set_visible(false);

            let progress_bar = gtk4::ProgressBar::new();
            progress_bar.set_visible(false);
            progress_bar.set_fraction(0.0);

            let status_icon = gtk4::Label::new(Some(""));
            status_icon.add_css_class("status-icon");

            let time_label = gtk4::Label::new(None);
            time_label.add_css_class("time-label");

            let name_label = gtk4::Label::new(Some(API_NAMES[i]));
            name_label.add_css_class("panel-title");

            let header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
            header_box.add_css_class(&format!("panel-header-{}", i));

            let cancel_button = gtk4::Button::with_label("✕");
            cancel_button.add_css_class("cancel-btn");
            cancel_button.add_css_class("flat");
            cancel_button.set_sensitive(false);
            cancel_button.set_tooltip_text(Some("Anuluj to API"));

            header_box.append(&status_icon);
            header_box.append(&name_label);
            header_box.append(&spinner);
            header_box.append(&time_label);
            
            let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
            spacer.set_hexpand(true);
            header_box.append(&spacer);
            
            header_box.append(&cancel_button);

            let use_button = gtk4::Button::with_label(&format!("📋 Użyj {}", API_NAMES[i]));
            use_button.add_css_class("use-button");
            use_button.add_css_class(&format!("use-button-{}", i));
            use_button.set_sensitive(false);

            PanelState {
                text_view,
                spinner,
                progress_bar,
                time_label,
                status_icon,
                name_label,
                header_box,
                use_button,
                cancel_button,
                result_text: Rc::new(RefCell::new(String::new())),
                start_time: Rc::new(RefCell::new(None)),
                is_processing: Rc::new(RefCell::new(false)),
                is_completed: Rc::new(RefCell::new(false)),
                has_error: Rc::new(RefCell::new(false)),
            }
        })
    }

    fn build_panel_frame(index: usize, panel: &PanelState) -> gtk4::Frame {
        let frame = gtk4::Frame::new(None);
        frame.add_css_class("panel-frame");
        frame.set_hexpand(true);
        frame.set_vexpand(true);

        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        
        vbox.append(&panel.header_box);
        vbox.append(&panel.progress_bar);

        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .hexpand(true)
            .vexpand(true)
            .child(&panel.text_view)
            .build();

        vbox.append(&scrolled);

        let button_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        button_box.set_margin_start(8);
        button_box.set_margin_end(8);
        button_box.set_margin_bottom(8);
        button_box.set_margin_top(4);
        
        panel.use_button.set_hexpand(true);
        button_box.append(&panel.use_button);
        
        vbox.append(&button_box);
        frame.set_child(Some(&vbox));

        frame
    }

    fn build_toolbar() -> (gtk4::Box, gtk4::Button, gtk4::Button, gtk4::Button) {
        let toolbar = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        toolbar.set_margin_start(12);
        toolbar.set_margin_end(12);
        toolbar.set_margin_bottom(12);
        toolbar.add_css_class("toolbar");

        let cancel_btn = gtk4::Button::with_label("❌ Anuluj wszystko");
        cancel_btn.add_css_class("destructive-action");
        toolbar.append(&cancel_btn);

        let original_btn = gtk4::Button::with_label("⚙️ Ustawienia");
        toolbar.append(&original_btn);

        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        toolbar.append(&spacer);

        let hide_btn = gtk4::Button::with_label("🔽 Minimalizuj");
        toolbar.append(&hide_btn);

        (toolbar, cancel_btn, original_btn, hide_btn)
    }

    fn connect_panel_buttons(state: Rc<RefCell<AppState>>) {
        let state_ref = state.borrow();
        
        for (i, panel) in state_ref.panels.iter().enumerate() {
            let state_clone = state.clone();
            let panel_clone = panel.clone();
            let index = i;
            
            panel.use_button.connect_clicked(move |_| {
                Self::use_api_result(&state_clone, index, &panel_clone);
            });

            let state_clone = state.clone();
            let index = i;
            
            panel.cancel_button.connect_clicked(move |_| {
                Self::cancel_single_api(&state_clone, index);
            });
        }
    }

    fn use_api_result(state: &Rc<RefCell<AppState>>, index: usize, panel: &PanelState) {
        let text = panel.result_text.borrow().clone();
        if text.is_empty() {
            return;
        }

        if let Err(e) = clipboard::write_text(&text) {
            error!("Failed to copy text: {}", e);
            return;
        }

        info!("Copied result from {} to clipboard", API_NAMES[index]);

        let state_ref = state.borrow();
        state_ref.window.set_visible(false);
        drop(state_ref);

        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(300));
            
            #[cfg(target_os = "linux")]
            {
                let _ = std::process::Command::new("xdotool")
                    .args(["key", "ctrl+v"])
                    .spawn();
            }
            
            #[cfg(target_os = "windows")]
            {
                use std::process::Command;
                let _ = Command::new("powershell")
                    .args(["-Command", "[System.Windows.Forms.SendKeys]::SendWait('^v')"])
                    .spawn();
            }
        });

        info!("Used result from {} and simulated Ctrl+V", API_NAMES[index]);
    }

    fn cancel_single_api(state: &Rc<RefCell<AppState>>, index: usize) {
        let state_ref = state.borrow();
        
        state_ref.cancel_flags[index].store(true, Ordering::SeqCst);
        
        let panel = &state_ref.panels[index];
        panel.spinner.stop();
        panel.spinner.set_visible(false);
        panel.progress_bar.set_visible(false);
        panel.cancel_button.set_sensitive(false);
        panel.status_icon.set_text("❌");
        panel.name_label.set_text(&format!("{} (anulowano)", API_NAMES[index]));
        panel.text_view.buffer().set_text("❌ Anulowano");
        *panel.is_processing.borrow_mut() = false;
        *panel.has_error.borrow_mut() = true;

        info!("Cancelled API {}", API_NAMES[index]);
    }

    fn connect_buttons(
        state: Rc<RefCell<AppState>>,
        settings_btn: gtk4::Button,
        paste_btn: gtk4::Button,
        cancel_btn: gtk4::Button,
        original_btn: gtk4::Button,
        hide_btn: gtk4::Button,
        window: adw::ApplicationWindow,
    ) {
        let state_clone = state.clone();
        paste_btn.connect_clicked(move |_| {
            glib::spawn_future_local({
                let state = state_clone.clone();
                async move {
                    Self::handle_hotkey_triggered(&state).await;
                }
            });
        });

        let state_clone = state.clone();
        cancel_btn.connect_clicked(move |_| {
            Self::cancel_all_processing(&state_clone);
        });

        let state_clone = state.clone();
        let window_clone = window.clone();
        original_btn.connect_clicked(move |_| {
            let state_ref = state_clone.borrow();
            let config = state_ref.config.borrow().clone();
            drop(state_ref);
            
            let dialog = SettingsDialog::new(&window_clone, &config);
            
            let state_for_save = state_clone.clone();
            dialog.connect_save(move |new_config| {
                let config_path = Config::get_config_path();
                if let Err(e) = new_config.save(&config_path) {
                    error!("Failed to save config: {}", e);
                } else {
                    let state_ref = state_for_save.borrow();
                    *state_ref.config.borrow_mut() = new_config;
                    info!("Settings saved successfully");
                }
            });
            
            dialog.present();
        });

        let window_weak = window.downgrade();
        hide_btn.connect_clicked(move |_| {
            if let Some(win) = window_weak.upgrade() {
                win.set_visible(false);
                info!("Window hidden to tray");
            }
        });

        let state_clone = state.clone();
        let window_clone = window.clone();
        settings_btn.connect_clicked(move |_| {
            let state_ref = state_clone.borrow();
            let config = state_ref.config.borrow().clone();
            drop(state_ref);
            
            let dialog = SettingsDialog::new(&window_clone, &config);
            
            let state_for_save = state_clone.clone();
            dialog.connect_save(move |new_config| {
                let config_path = Config::get_config_path();
                if let Err(e) = new_config.save(&config_path) {
                    error!("Failed to save config: {}", e);
                } else {
                    let state_ref = state_for_save.borrow();
                    *state_ref.config.borrow_mut() = new_config;
                    info!("Settings saved successfully");
                }
            });
            
            dialog.present();
        });
    }

    fn cancel_all_processing(state: &Rc<RefCell<AppState>>) {
        let state_ref = state.borrow();
        
        for flag in &state_ref.cancel_flags {
            flag.store(true, Ordering::SeqCst);
        }
        
        for (i, panel) in state_ref.panels.iter().enumerate() {
            panel.spinner.stop();
            panel.spinner.set_visible(false);
            panel.progress_bar.set_visible(false);
            panel.progress_bar.set_fraction(0.0);
            panel.cancel_button.set_sensitive(false);
            
            if *panel.is_processing.borrow() {
                panel.status_icon.set_text("❌");
                panel.name_label.set_text(&format!("{} (anulowano)", API_NAMES[i]));
                panel.text_view.buffer().set_text("❌ Anulowano");
                *panel.is_processing.borrow_mut() = false;
            }
        }
        
        state_ref.status_label.set_text("❌ Anulowano przetwarzanie");
        state_ref.hint_label.set_text("");
        
        info!("Cancelled all processing");
    }

    fn show_original_text_dialog(parent: &adw::ApplicationWindow, text: &str) {
        let dialog = gtk4::Window::builder()
            .title("Oryginalny tekst")
            .transient_for(parent)
            .modal(true)
            .default_width(500)
            .default_height(400)
            .build();

        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);

        let text_view = gtk4::TextView::builder()
            .editable(false)
            .wrap_mode(gtk4::WrapMode::Word)
            .build();
        text_view.buffer().set_text(text);

        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .hexpand(true)
            .vexpand(true)
            .child(&text_view)
            .build();

        vbox.append(&scrolled);

        let button_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        button_box.set_halign(gtk4::Align::End);

        let text_clone = text.to_string();
        let copy_btn = gtk4::Button::with_label("📋 Kopiuj");
        copy_btn.connect_clicked(move |_| {
            let _ = clipboard::write_text(&text_clone);
        });
        button_box.append(&copy_btn);

        let dialog_weak = dialog.downgrade();
        let close_btn = gtk4::Button::with_label("Zamknij");
        close_btn.connect_clicked(move |_| {
            if let Some(d) = dialog_weak.upgrade() {
                d.close();
            }
        });
        button_box.append(&close_btn);

        vbox.append(&button_box);
        dialog.set_child(Some(&vbox));
        dialog.present();
    }

    fn setup_close_handler(window: adw::ApplicationWindow) {
        window.connect_close_request(move |win| {
            win.set_visible(false);
            info!("Window hidden (close intercepted)");
            glib::Propagation::Stop
        });
    }

    fn setup_hotkey(state: Rc<RefCell<AppState>>) {
        let (async_tx, async_rx) = async_channel::unbounded::<HotkeyEvent>();
        
        std::thread::spawn(move || {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            
            if let Ok(_manager) = HotkeyManager::new(tx) {
                info!("Hotkey manager created");
                
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    while let Some(event) = rx.recv().await {
                        let _ = async_tx.send(event).await;
                    }
                });
            }
        });

        glib::spawn_future_local(async move {
            while let Ok(event) = async_rx.recv().await {
                match event {
                    HotkeyEvent::Triggered => {
                        info!("Hotkey triggered");
                        let state_ref = state.borrow();
                        state_ref.window.set_visible(true);
                        state_ref.window.present();
                        drop(state_ref);
                        Self::handle_hotkey_triggered(&state).await;
                    }
                }
            }
        });
    }

    async fn handle_hotkey_triggered(state: &Rc<RefCell<AppState>>) {
        if let Ok(text) = clipboard::read_text() {
            if !text.is_empty() {
                Self::prepare_processing_session(state, &text);
                
                let state_ref = state.borrow();
                let config = state_ref.config.borrow().clone();
                let cancel_flags = state_ref.cancel_flags.clone();
                let session = state_ref.session_id.load(Ordering::SeqCst);
                drop(state_ref);

                Self::process_with_apis(state.clone(), text, config, cancel_flags, session).await;
            } else {
                let state_ref = state.borrow();
                state_ref.status_label.set_text("⚠️ Brak tekstu w schowku");
            }
        }
    }

    fn prepare_processing_session(state: &Rc<RefCell<AppState>>, text: &str) {
        let state_ref = state.borrow();
        
        *state_ref.original_text.borrow_mut() = text.to_string();
        
        let session = state_ref.session_id.fetch_add(1, Ordering::SeqCst) + 1;
        state_ref.session_label.set_text(&format!("📝 Sesja: {}", session));
        
        *state_ref.completed_count.borrow_mut() = 0;
        state_ref.api_counter_label.set_text("🤖 API: 0/4");
        
        state_ref.status_label.set_text("🔄 Wysyłanie do 4 API równocześnie...");
        state_ref.hint_label.set_text(&format!("({} znaków)", text.len()));
        
        for flag in &state_ref.cancel_flags {
            flag.store(false, Ordering::SeqCst);
        }
        
        for (i, panel) in state_ref.panels.iter().enumerate() {
            *panel.is_processing.borrow_mut() = true;
            *panel.is_completed.borrow_mut() = false;
            *panel.has_error.borrow_mut() = false;
            *panel.start_time.borrow_mut() = Some(Instant::now());
            *panel.result_text.borrow_mut() = String::new();
            
            panel.spinner.set_visible(true);
            panel.spinner.start();
            panel.progress_bar.set_visible(true);
            panel.progress_bar.set_fraction(0.0);
            panel.cancel_button.set_sensitive(true);
            panel.use_button.set_sensitive(false);
            panel.status_icon.set_text("🤖");
            panel.name_label.set_text(API_NAMES[i]);
            panel.time_label.set_text("");
            panel.text_view.buffer().set_text("🔄 Przygotowanie...");
        }
    }

    async fn process_with_apis(
        state: Rc<RefCell<AppState>>,
        text: String,
        config: Config,
        cancel_flags: [Arc<AtomicBool>; 4],
        session: u64,
    ) {
        let system_prompt = get_system_prompt(CorrectionStyle::Normal);
        let instruction = get_instruction_prompt(CorrectionStyle::Normal);

        let (tx, rx) = async_channel::unbounded::<(usize, Result<String, String>)>();

        for i in 0..4 {
            let text = text.clone();
            let config = config.clone();
            let system = system_prompt.to_string();
            let instr = instruction.to_string();
            let cancel = cancel_flags[i].clone();
            let tx = tx.clone();

            tokio::spawn(async move {
                let result = match i {
                    0 => correct_text_openai_with_callback::<fn(&str)>(
                        &config.api_keys.openai,
                        &config.models.openai,
                        &text,
                        &instr,
                        &system,
                        true,
                        None,
                    ).await,
                    1 => correct_text_anthropic(
                        &config.api_keys.anthropic,
                        &config.models.anthropic,
                        &text,
                        &instr,
                        &system,
                    ).await,
                    2 => correct_text_gemini(
                        &config.api_keys.gemini,
                        &config.models.gemini,
                        &text,
                        &instr,
                        &system,
                    ).await,
                    3 => correct_text_deepseek(
                        &config.api_keys.deepseek,
                        &config.models.deepseek,
                        &text,
                        &instr,
                        &system,
                    ).await,
                    _ => Err(crate::error::ApiError::Response("Unknown API".to_string())),
                };

                if !cancel.load(Ordering::SeqCst) {
                    let _ = tx.send((i, result.map_err(|e| e.to_string()))).await;
                }
            });
        }

        drop(tx);

        while let Ok((index, result)) = rx.recv().await {
            Self::update_panel_result(&state, index, result, session);
        }

        Self::finalize_processing(&state);
    }

    fn update_panel_result(
        state: &Rc<RefCell<AppState>>,
        index: usize,
        result: Result<String, String>,
        _session: u64,
    ) {
        let state_ref = state.borrow();
        let panel = &state_ref.panels[index];
        
        panel.spinner.stop();
        panel.spinner.set_visible(false);
        panel.progress_bar.set_visible(false);
        panel.cancel_button.set_sensitive(false);
        *panel.is_processing.borrow_mut() = false;

        let elapsed = panel.start_time.borrow()
            .map(|t| t.elapsed().as_secs_f64())
            .unwrap_or(0.0);

        match result {
            Ok(corrected) => {
                *panel.result_text.borrow_mut() = corrected.clone();
                *panel.is_completed.borrow_mut() = true;
                
                panel.status_icon.set_text("✅");
                panel.name_label.set_text(&format!("{} ({:.1}s)", API_NAMES[index], elapsed));
                panel.use_button.set_sensitive(true);
                
                let original = state_ref.original_text.borrow().clone();
                let highlight = state_ref.config.borrow().settings.highlight_diffs;
                set_text_with_diff(&panel.text_view.buffer(), &original, &corrected, highlight);
                
                let mut count = state_ref.completed_count.borrow_mut();
                *count += 1;
                state_ref.api_counter_label.set_text(&format!("🤖 API: {}/4", *count));
            }
            Err(e) => {
                *panel.has_error.borrow_mut() = true;
                
                panel.status_icon.set_text("❌");
                panel.name_label.set_text(&format!("{} (błąd)", API_NAMES[index]));
                panel.text_view.buffer().set_text(&format!("❌ Błąd: {}", e));
                panel.use_button.set_sensitive(false);
            }
        }
    }

    fn finalize_processing(state: &Rc<RefCell<AppState>>) {
        let state_ref = state.borrow();
        let completed = *state_ref.completed_count.borrow();
        
        if completed > 0 {
            state_ref.status_label.set_text(&format!("✅ Gotowe! Otrzymano {} wyników", completed));
            state_ref.hint_label.set_text("Wybierz najlepszy wynik i kliknij 'Użyj'");
        } else {
            state_ref.status_label.set_text("❌ Wszystkie API zwróciły błędy");
            state_ref.hint_label.set_text("Sprawdź klucze API w ustawieniach");
        }
    }

    fn setup_tray(window: adw::ApplicationWindow) {
        let window_weak = window.downgrade();
        
        if let Ok(tray) = TrayManager::new() {
            let tray = Rc::new(RefCell::new(tray));
            let tray_clone = tray.clone();
            
            glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                if let Some(event) = tray_clone.borrow_mut().poll_event() {
                    match event {
                        crate::tray::TrayEvent::Show => {
                            if let Some(win) = window_weak.upgrade() {
                                win.set_visible(true);
                                win.present();
                                info!("Window shown from tray");
                            }
                        }
                        crate::tray::TrayEvent::Quit => {
                            if let Some(win) = window_weak.upgrade() {
                                win.application().map(|app| app.quit());
                            }
                        }
                    }
                }
                glib::ControlFlow::Continue
            });
        }
    }
}
