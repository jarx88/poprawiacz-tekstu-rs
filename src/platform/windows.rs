use super::KeyboardSimulator;
use crate::error::PlatformError;
use tracing::warn;

pub struct WindowsKeyboardSimulator;

impl WindowsKeyboardSimulator {
    pub fn new() -> Self {
        warn!("Windows keyboard simulation not yet implemented");
        Self
    }
}

impl KeyboardSimulator for WindowsKeyboardSimulator {
    fn simulate_copy(&self) -> Result<(), PlatformError> {
        Err(PlatformError::NotSupported(
            "Windows keyboard simulation not yet implemented. TODO: Implement with Win32 SendInput API".to_string(),
        ))
    }

    fn simulate_paste(&self) -> Result<(), PlatformError> {
        Err(PlatformError::NotSupported(
            "Windows keyboard simulation not yet implemented. TODO: Implement with Win32 SendInput API".to_string(),
        ))
    }
}

impl Default for WindowsKeyboardSimulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_simulator() {
        let _simulator = WindowsKeyboardSimulator::new();
    }

    #[test]
    fn test_default() {
        let _simulator = WindowsKeyboardSimulator::default();
    }

    #[test]
    fn test_simulate_copy_not_implemented() {
        let simulator = WindowsKeyboardSimulator::new();
        let result = simulator.simulate_copy();
        assert!(result.is_err());
        match result {
            Err(PlatformError::NotSupported(msg)) => {
                assert!(msg.contains("Win32 SendInput API"));
            }
            _ => panic!("Expected NotSupported error"),
        }
    }

    #[test]
    fn test_simulate_paste_not_implemented() {
        let simulator = WindowsKeyboardSimulator::new();
        let result = simulator.simulate_paste();
        assert!(result.is_err());
        match result {
            Err(PlatformError::NotSupported(msg)) => {
                assert!(msg.contains("Win32 SendInput API"));
            }
            _ => panic!("Expected NotSupported error"),
        }
    }
}
