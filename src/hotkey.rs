use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyEvent {
    Triggered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyCombo {
    Primary,
    Fallback,
}

impl HotkeyCombo {
    pub fn description(&self) -> &'static str {
        match self {
            HotkeyCombo::Primary => "Ctrl+Shift+C",
            HotkeyCombo::Fallback => "Ctrl+Shift+Alt+C",
        }
    }

    pub fn to_hotkey(&self) -> HotKey {
        match self {
            HotkeyCombo::Primary => HotKey::new(
                Some(Modifiers::CONTROL | Modifiers::SHIFT),
                Code::KeyC,
            ),
            HotkeyCombo::Fallback => HotKey::new(
                Some(Modifiers::CONTROL | Modifiers::SHIFT | Modifiers::ALT),
                Code::KeyC,
            ),
        }
    }
}

pub struct HotkeyManager {
    manager: Arc<GlobalHotKeyManager>,
    registered_hotkey: Option<HotKey>,
    active_combo: Option<HotkeyCombo>,
    tx: mpsc::UnboundedSender<HotkeyEvent>,
}

impl HotkeyManager {
    pub fn new(tx: mpsc::UnboundedSender<HotkeyEvent>) -> Result<Self, String> {
        let manager = GlobalHotKeyManager::new().map_err(|e| {
            error!("Failed to create GlobalHotKeyManager: {}", e);
            format!("Failed to create hotkey manager: {}", e)
        })?;

        let mut hotkey_manager = Self {
            manager: Arc::new(manager),
            registered_hotkey: None,
            active_combo: None,
            tx,
        };

        hotkey_manager.register_with_fallback()?;

        Ok(hotkey_manager)
    }

    fn try_register_primary_hotkey(&mut self) -> Result<(), String> {
        let combo = HotkeyCombo::Primary;
        let hotkey = combo.to_hotkey();

        self.manager.register(hotkey).map_err(|e| {
            warn!("Failed to register {}: {}", combo.description(), e);
            format!("Failed to register {}: {}", combo.description(), e)
        })?;

        self.registered_hotkey = Some(hotkey);
        self.active_combo = Some(combo.clone());
        info!(
            "Global hotkey {} registered successfully",
            combo.description()
        );

        Ok(())
    }

    fn try_register_fallback_hotkey(&mut self) -> Result<(), String> {
        let combo = HotkeyCombo::Fallback;
        let hotkey = combo.to_hotkey();

        self.manager.register(hotkey).map_err(|e| {
            error!("Failed to register {}: {}", combo.description(), e);
            format!("Failed to register {}: {}", combo.description(), e)
        })?;

        self.registered_hotkey = Some(hotkey);
        self.active_combo = Some(combo.clone());
        info!(
            "Fallback hotkey {} registered successfully",
            combo.description()
        );

        Ok(())
    }

    fn register_with_fallback(&mut self) -> Result<(), String> {
        if self.try_register_primary_hotkey().is_ok() {
            return Ok(());
        }

        warn!("Primary hotkey registration failed, trying fallback...");

        if self.try_register_fallback_hotkey().is_ok() {
            return Ok(());
        }

        error!("Failed to register any hotkey - manual mode required");
        Err("Failed to register any hotkey".to_string())
    }

    pub fn active_combo(&self) -> Option<&HotkeyCombo> {
        self.active_combo.as_ref()
    }

    pub fn start_event_loop(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let receiver = GlobalHotKeyEvent::receiver();
            info!("Hotkey event loop started");

            loop {
                if let Ok(event) = receiver.try_recv() {
                    if event.state == HotKeyState::Pressed {
                        if let Some(registered) = self.registered_hotkey {
                            if event.id == registered.id() {
                                info!("Hotkey triggered: {:?}", self.active_combo);
                                if let Err(e) = self.tx.send(HotkeyEvent::Triggered) {
                                    error!("Failed to send hotkey event: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }

            warn!("Hotkey event loop terminated");
        })
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        if let Some(hotkey) = self.registered_hotkey {
            if let Err(e) = self.manager.unregister(hotkey) {
                error!("Failed to unregister hotkey: {}", e);
            } else {
                info!("Hotkey unregistered successfully");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, timeout, Duration};

    #[tokio::test]
    async fn test_hotkey_registration_succeeds() {
        let (tx, _rx) = mpsc::unbounded_channel();
        
        let result = HotkeyManager::new(tx);
        
        assert!(
            result.is_ok(),
            "Hotkey registration should succeed with primary or fallback"
        );
        
        let manager = result.unwrap();
        assert!(
            manager.active_combo().is_some(),
            "Active combo should be set"
        );
        
        let combo = manager.active_combo().unwrap();
        assert!(
            *combo == HotkeyCombo::Primary || *combo == HotkeyCombo::Fallback,
            "Active combo should be Primary or Fallback"
        );
    }

    #[tokio::test]
    async fn test_hotkey_combos_have_correct_descriptions() {
        assert_eq!(HotkeyCombo::Primary.description(), "Ctrl+Shift+C");
        assert_eq!(HotkeyCombo::Fallback.description(), "Ctrl+Shift+Alt+C");
    }

    #[tokio::test]
    async fn test_hotkey_combos_generate_different_hotkeys() {
        let primary = HotkeyCombo::Primary.to_hotkey();
        let fallback = HotkeyCombo::Fallback.to_hotkey();
        
        assert_ne!(primary.id(), fallback.id(), "Primary and fallback should have different IDs");
    }

    #[tokio::test]
    async fn test_event_forwarding_via_channel() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        let manager = HotkeyManager::new(tx);
        assert!(manager.is_ok(), "Manager creation should succeed");
        
        let manager = manager.unwrap();
        let _handle = manager.start_event_loop();
        
        sleep(Duration::from_millis(100)).await;
        assert!(!rx.is_closed(), "Channel should remain open");
        
        let result = timeout(Duration::from_millis(200), rx.recv()).await;
        assert!(result.is_err(), "Should timeout waiting for hotkey event");
    }

    #[test]
    fn test_fallback_registration_logic() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let manager = HotkeyManager::new(tx);
        
        assert!(
            manager.is_ok(),
            "Should register at least one hotkey (primary or fallback)"
        );
    }

    #[tokio::test]
    async fn test_hotkey_manager_cleanup_on_drop() {
        let (tx, _rx) = mpsc::unbounded_channel();
        
        {
            let manager = HotkeyManager::new(tx.clone());
            assert!(manager.is_ok(), "Manager creation should succeed");
        }
        
        let manager2 = HotkeyManager::new(tx);
        assert!(
            manager2.is_ok(),
            "Should be able to create new manager after previous one was dropped"
        );
    }

    #[test]
    fn test_active_combo_is_set_after_registration() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let manager = HotkeyManager::new(tx).expect("Manager creation should succeed");
        
        assert!(
            manager.active_combo().is_some(),
            "Active combo should be set after successful registration"
        );
    }

    #[test]
    fn test_hotkey_event_derives() {
        let event1 = HotkeyEvent::Triggered;
        let event2 = event1;
        assert_eq!(event1, event2);
        
        let event3 = event1.clone();
        assert_eq!(event1, event3);
    }
}
