//! System tray integration using tray-icon crate
//!
//! This module provides system tray functionality with platform-specific handling:
//! - Linux: GTK runtime in separate thread
//! - Windows/macOS: Standard tray icon integration
//!
//! # Example
//! ```no_run
//! use poprawiacz_tekstu_rs::tray::{TrayIcon, TrayEvent};
//!
//! let (tray, event_rx) = TrayIcon::new().expect("Failed to create tray icon");
//!
//! // Poll for events
//! while let Ok(event) = event_rx.try_recv() {
//!     match event {
//!         TrayEvent::Show => println!("Show window"),
//!         TrayEvent::Quit => println!("Quit app"),
//!     }
//! }
//! ```

use std::sync::mpsc::{self, Receiver, Sender};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuId, MenuItem},
    Icon, TrayIcon as TrayIconInner, TrayIconBuilder,
};

/// Events emitted by the system tray
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    /// User clicked "Show" menu item
    Show,
    /// User clicked "Quit" menu item
    Quit,
}

/// System tray icon with event forwarding
pub struct TrayIcon {
    #[allow(dead_code)]
    inner: Option<TrayIconInner>,
    show_id: MenuId,
    quit_id: MenuId,
    event_tx: Sender<TrayEvent>,
}

impl TrayIcon {
    /// Creates a new system tray icon with "Show" and "Quit" menu items
    ///
    /// Returns a tuple of (TrayIcon, Receiver<TrayEvent>) for event polling.
    ///
    /// # Platform-specific behavior
    /// - **Linux**: Spawns a separate thread for GTK runtime (required for tray to work)
    /// - **Windows/macOS**: Creates tray icon in current thread
    ///
    /// # Errors
    /// Returns error if:
    /// - GTK initialization fails (Linux only)
    /// - Tray icon creation fails
    /// - Icon loading fails
    pub fn new() -> anyhow::Result<(Self, Receiver<TrayEvent>)> {
        let (event_tx, event_rx) = mpsc::channel();

        let icon = Self::create_placeholder_icon()?;

        #[cfg(target_os = "linux")]
        {
            let (show_id, quit_id) = Self::init_gtk_thread(icon)?;
            Ok((
                Self {
                    inner: None,
                    show_id,
                    quit_id,
                    event_tx,
                },
                event_rx,
            ))
        }

        #[cfg(not(target_os = "linux"))]
        {
            let show_item = MenuItem::new("Show", true, None);
            let quit_item = MenuItem::new("Quit", true, None);
            let show_id = show_item.id().clone();
            let quit_id = quit_item.id().clone();

            let menu = Menu::new();
            menu.append(&show_item)?;
            menu.append(&quit_item)?;

            let tray = TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_icon(icon)
                .with_tooltip("PoprawiaczTekstuRs")
                .build()?;

            Ok((
                Self {
                    inner: Some(tray),
                    show_id,
                    quit_id,
                    event_tx,
                },
                event_rx,
            ))
        }
    }

    /// Polls for tray menu events and forwards them to the event channel
    ///
    /// Should be called periodically (e.g., in GUI event loop) to process menu clicks.
    pub fn poll_events(&self) {
        if let Ok(menu_event) = MenuEvent::receiver().try_recv() {
            let event = if menu_event.id == self.show_id {
                Some(TrayEvent::Show)
            } else if menu_event.id == self.quit_id {
                Some(TrayEvent::Quit)
            } else {
                None
            };

            if let Some(ev) = event {
                let _ = self.event_tx.send(ev);
            }
        }
    }

    /// Creates a placeholder icon (32x32 green square)
    fn create_placeholder_icon() -> anyhow::Result<Icon> {
        let size = 32;
        let mut rgba = Vec::with_capacity((size * size * 4) as usize);

        for _ in 0..(size * size) {
            rgba.push(16);
            rgba.push(163);
            rgba.push(127);
            rgba.push(255);
        }

        Ok(Icon::from_rgba(rgba, size, size)?)
    }

    /// Initializes GTK in a separate thread (Linux only)
    ///
    /// This is required because egui uses winit which doesn't use GTK on Linux,
    /// but tray-icon requires GTK for system tray integration.
    #[cfg(target_os = "linux")]
    fn init_gtk_thread(icon: Icon) -> anyhow::Result<(MenuId, MenuId)> {
        use std::sync::{Arc, Mutex};

        let ids = Arc::new(Mutex::new(None));
        let ids_clone = ids.clone();

        std::thread::spawn(move || {
            if let Err(e) = gtk::init() {
                eprintln!("Failed to initialize GTK: {}", e);
                return;
            }

            let show_item = MenuItem::new("Show", true, None);
            let quit_item = MenuItem::new("Quit", true, None);
            let show_id = show_item.id().clone();
            let quit_id = quit_item.id().clone();

            *ids_clone.lock().unwrap() = Some((show_id, quit_id));

            let menu = Menu::new();
            if menu.append(&show_item).is_err() {
                eprintln!("Failed to append Show item");
                return;
            }
            if menu.append(&quit_item).is_err() {
                eprintln!("Failed to append Quit item");
                return;
            }

            let _tray = match TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_icon(icon)
                .with_tooltip("PoprawiaczTekstuRs")
                .build()
            {
                Ok(tray) => tray,
                Err(e) => {
                    eprintln!("Failed to build tray icon: {}", e);
                    return;
                }
            };

            gtk::main();
        });

        std::thread::sleep(std::time::Duration::from_millis(100));

        let result = ids
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Failed to get menu IDs"));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_event_enum() {
        let show = TrayEvent::Show;
        let quit = TrayEvent::Quit;

        assert_eq!(show, TrayEvent::Show);
        assert_eq!(quit, TrayEvent::Quit);
        assert_ne!(show, quit);
    }

    #[test]
    fn test_placeholder_icon_creation() {
        let icon = TrayIcon::create_placeholder_icon();
        assert!(icon.is_ok(), "Placeholder icon creation should succeed");
    }

    #[test]
    fn test_tray_icon_creation() {
        #[cfg(not(target_os = "linux"))]
        {
            let result = TrayIcon::new();
            match result {
                Ok((tray, rx)) => {
                    tray.poll_events();
                    assert!(rx.try_recv().is_err());
                }
                Err(e) => {
                    eprintln!("Tray creation failed (expected in headless): {}", e);
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let result = TrayIcon::new();
            match result {
                Ok((tray, rx)) => {
                    tray.poll_events();
                    assert!(rx.try_recv().is_err());
                }
                Err(e) => {
                    eprintln!("Tray creation failed on Linux: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_event_channel_communication() {
        let (tx, rx) = mpsc::channel();

        tx.send(TrayEvent::Show).unwrap();
        tx.send(TrayEvent::Quit).unwrap();

        assert_eq!(rx.recv().unwrap(), TrayEvent::Show);
        assert_eq!(rx.recv().unwrap(), TrayEvent::Quit);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn test_menu_items_creation() {
        let show_item = MenuItem::new("Show", true, None);
        let quit_item = MenuItem::new("Quit", true, None);

        assert_ne!(show_item.id(), quit_item.id());
    }
}
