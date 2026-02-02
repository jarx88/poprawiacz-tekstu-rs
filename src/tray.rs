use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use tracing::{error, info};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    Show,
    Quit,
}

pub struct TrayManager {
    event_rx: Receiver<TrayEvent>,
    #[allow(dead_code)]
    event_tx: Sender<TrayEvent>,
}

impl TrayManager {
    pub fn new() -> Result<Self, String> {
        let (event_tx, event_rx) = mpsc::channel();

        let tx_clone = event_tx.clone();

        std::thread::spawn(move || {
            if let Err(e) = Self::run_tray_service(tx_clone) {
                error!("Tray service error: {}", e);
            }
        });

        info!("TrayManager initialized");

        Ok(Self { event_rx, event_tx })
    }

    pub fn poll_event(&mut self) -> Option<TrayEvent> {
        match self.event_rx.try_recv() {
            Ok(event) => Some(event),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => None,
        }
    }

    #[cfg(target_os = "linux")]
    fn run_tray_service(tx: Sender<TrayEvent>) -> Result<(), String> {
        use ksni::{Tray, TrayService};

        struct PoprawiaczTray {
            tx: Sender<TrayEvent>,
        }

        impl Tray for PoprawiaczTray {
            fn id(&self) -> String {
                "poprawiacz-tekstu-rs".into()
            }

            fn icon_name(&self) -> String {
                Self::get_icon_path()
            }

            fn title(&self) -> String {
                "PoprawiaczTekstuRs".into()
            }

            fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
                use ksni::menu::*;
                vec![
                    StandardItem {
                        label: "Pokaż".into(),
                        activate: Box::new(|tray: &mut Self| {
                            let _ = tray.tx.send(TrayEvent::Show);
                        }),
                        ..Default::default()
                    }
                    .into(),
                    MenuItem::Separator,
                    StandardItem {
                        label: "Zakończ".into(),
                        activate: Box::new(|tray: &mut Self| {
                            let _ = tray.tx.send(TrayEvent::Quit);
                        }),
                        ..Default::default()
                    }
                    .into(),
                ]
            }
        }

        impl PoprawiaczTray {
            fn get_icon_path() -> String {
                if let Ok(exe) = std::env::current_exe() {
                    if let Some(dir) = exe.parent() {
                        let icon = dir.join("assets").join("icon_24.png");
                        if icon.exists() {
                            return icon.to_string_lossy().to_string();
                        }
                        let icon = dir.join("icon_24.png");
                        if icon.exists() {
                            return icon.to_string_lossy().to_string();
                        }
                    }
                }
                "text-editor".into()
            }
        }

        let service = TrayService::new(PoprawiaczTray { tx });
        let _ = service.run();

        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    fn run_tray_service(_tx: Sender<TrayEvent>) -> Result<(), String> {
        Ok(())
    }
}
