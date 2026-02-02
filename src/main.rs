use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use libadwaita as adw;
use tracing_subscriber::{self, EnvFilter};

use poprawiacz_tekstu_rs::app::MainWindow;

const APP_ID: &str = "io.github.jarx88.poprawiacz-tekstu-rs";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("poprawiacz_tekstu_rs=info")),
        )
        .init();

    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    app.connect_activate(|app| {
        if let Some(window) = app.active_window() {
            window.set_visible(true);
            window.present();
        }
    });

    app.connect_startup(|app| {
        let window = MainWindow::new(app);
        window.present();
    });

    app.connect_command_line(|app, cmd| {
        let args: Vec<String> = cmd.arguments().iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect();

        if args.contains(&"--paste".to_string()) || args.contains(&"-p".to_string()) {
            if let Some(window) = app.active_window() {
                window.set_visible(true);
                window.present();
                
                if let Some(main_window) = window.downcast_ref::<adw::ApplicationWindow>() {
                    for widget in main_window.observe_children().into_iter() {
                        if let Ok(child) = widget {
                            if let Some(btn) = find_paste_button(&child) {
                                btn.emit_clicked();
                                break;
                            }
                        }
                    }
                }
            }
        } else {
            app.activate();
        }
        0
    });

    app.run()
}

fn find_paste_button(widget: &glib::Object) -> Option<gtk4::Button> {
    if let Some(btn) = widget.downcast_ref::<gtk4::Button>() {
        if let Some(label) = btn.label() {
            if label.contains("Wklej") {
                return Some(btn.clone());
            }
        }
    }
    
    if let Some(container) = widget.downcast_ref::<gtk4::Widget>() {
        let mut child = container.first_child();
        while let Some(c) = child {
            if let Some(btn) = find_paste_button(c.upcast_ref()) {
                return Some(btn);
            }
            child = c.next_sibling();
        }
    }
    
    None
}
