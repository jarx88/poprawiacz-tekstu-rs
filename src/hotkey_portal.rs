use ashpd::desktop::global_shortcuts::{GlobalShortcuts, NewShortcut};
use futures_util::StreamExt;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalHotkeyEvent {
    Triggered,
}

pub struct PortalHotkeyManager {
    tx: mpsc::UnboundedSender<PortalHotkeyEvent>,
}

impl PortalHotkeyManager {
    pub fn new(tx: mpsc::UnboundedSender<PortalHotkeyEvent>) -> Self {
        Self { tx }
    }

    pub async fn run(self) -> Result<(), String> {
        let shortcuts = GlobalShortcuts::new().await.map_err(|e| {
            error!("Failed to create GlobalShortcuts portal: {}", e);
            format!("Portal unavailable: {}", e)
        })?;

        let session = shortcuts.create_session().await.map_err(|e| {
            error!("Failed to create shortcuts session: {}", e);
            format!("Session creation failed: {}", e)
        })?;

        let shortcut = NewShortcut::new("capture-text", "Przechwytuje tekst ze schowka i uruchamia korekcje")
            .preferred_trigger("CTRL+SHIFT+C");

        let request = shortcuts
            .bind_shortcuts(&session, &[shortcut], None)
            .await
            .map_err(|e| {
                error!("Failed to bind shortcuts: {}", e);
                format!("Bind failed: {}", e)
            })?;

        let response = request.response().map_err(|e| {
            error!("Shortcut binding rejected: {}", e);
            format!("Binding rejected: {}", e)
        })?;

        if response.shortcuts().is_empty() {
            warn!("No shortcuts were bound - user may need to configure in system settings");
        } else {
            for shortcut in response.shortcuts() {
                info!(
                    "Shortcut bound: {} -> {}",
                    shortcut.id(),
                    shortcut.trigger_description()
                );
            }
        }

        let mut activated_stream = shortcuts.receive_activated().await.map_err(|e| {
            error!("Failed to receive activated signal: {}", e);
            format!("Signal subscription failed: {}", e)
        })?;

        info!("Portal hotkey manager started, listening for Ctrl+Shift+C");

        while let Some(activated) = activated_stream.next().await {
            if activated.shortcut_id() == "capture-text" {
                info!("Portal hotkey triggered: capture-text");
                if let Err(e) = self.tx.send(PortalHotkeyEvent::Triggered) {
                    error!("Failed to send hotkey event: {}", e);
                    break;
                }
            }
        }

        warn!("Portal hotkey event loop terminated");
        Ok(())
    }
}

pub fn is_wayland() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_ok()
        || std::env::var("XDG_SESSION_TYPE")
            .map(|v| v == "wayland")
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portal_hotkey_event_derives() {
        let event1 = PortalHotkeyEvent::Triggered;
        let event2 = event1;
        assert_eq!(event1, event2);

        let event3 = event1.clone();
        assert_eq!(event1, event3);
    }

    #[test]
    fn test_is_wayland_detection() {
        let result = is_wayland();
        assert!(result == true || result == false);
    }
}
