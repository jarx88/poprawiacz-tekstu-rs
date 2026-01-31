use super::KeyboardSimulator;
use crate::error::PlatformError;
use std::process::Command;
use tracing::{debug, warn};

pub struct LinuxKeyboardSimulator {
    xdotool_available: bool,
}

impl LinuxKeyboardSimulator {
    pub fn new() -> Self {
        let xdotool_available = Self::is_xdotool_available();
        if !xdotool_available {
            warn!("xdotool not found. Keyboard simulation will not be available. Install with: sudo apt install xdotool");
        } else {
            debug!("xdotool found, keyboard simulation enabled");
        }
        Self { xdotool_available }
    }

    pub fn is_xdotool_available() -> bool {
        Command::new("which")
            .arg("xdotool")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn execute_xdotool(&self, keys: &str) -> Result<(), PlatformError> {
        if !self.xdotool_available {
            return Err(PlatformError::ToolNotFound(
                "xdotool is not installed. Install with: sudo apt install xdotool".to_string(),
            ));
        }

        debug!("Simulating key press: {}", keys);

        let output = Command::new("xdotool")
            .args(["key", keys])
            .output()
            .map_err(|e| {
                PlatformError::CommandFailed(format!("Failed to execute xdotool: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PlatformError::CommandFailed(format!(
                "xdotool command failed: {}",
                stderr
            )));
        }

        debug!("Key press simulation completed successfully");
        Ok(())
    }
}

impl KeyboardSimulator for LinuxKeyboardSimulator {
    fn simulate_copy(&self) -> Result<(), PlatformError> {
        self.execute_xdotool("ctrl+c")
    }

    fn simulate_paste(&self) -> Result<(), PlatformError> {
        self.execute_xdotool("ctrl+v")
    }
}

impl Default for LinuxKeyboardSimulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_simulator() {
        let simulator = LinuxKeyboardSimulator::new();
        assert_eq!(
            simulator.xdotool_available,
            LinuxKeyboardSimulator::is_xdotool_available()
        );
    }

    #[test]
    fn test_default() {
        let simulator = LinuxKeyboardSimulator::default();
        assert_eq!(
            simulator.xdotool_available,
            LinuxKeyboardSimulator::is_xdotool_available()
        );
    }

    #[test]
    fn test_simulate_copy_without_xdotool() {
        let simulator = LinuxKeyboardSimulator {
            xdotool_available: false,
        };
        let result = simulator.simulate_copy();
        assert!(result.is_err());
        match result {
            Err(PlatformError::ToolNotFound(msg)) => {
                assert!(msg.contains("xdotool"));
            }
            _ => panic!("Expected ToolNotFound error"),
        }
    }

    #[test]
    fn test_simulate_paste_without_xdotool() {
        let simulator = LinuxKeyboardSimulator {
            xdotool_available: false,
        };
        let result = simulator.simulate_paste();
        assert!(result.is_err());
        match result {
            Err(PlatformError::ToolNotFound(msg)) => {
                assert!(msg.contains("xdotool"));
            }
            _ => panic!("Expected ToolNotFound error"),
        }
    }

    #[test]
    fn test_is_xdotool_available() {
        let available = LinuxKeyboardSimulator::is_xdotool_available();
        let which_result = Command::new("which")
            .arg("xdotool")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        assert_eq!(available, which_result);
    }

    #[test]
    #[ignore]
    fn test_simulate_copy_with_xdotool() {
        if !LinuxKeyboardSimulator::is_xdotool_available() {
            return;
        }
        let simulator = LinuxKeyboardSimulator::new();
        let result = simulator.simulate_copy();
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_simulate_paste_with_xdotool() {
        if !LinuxKeyboardSimulator::is_xdotool_available() {
            return;
        }
        let simulator = LinuxKeyboardSimulator::new();
        let result = simulator.simulate_paste();
        assert!(result.is_ok());
    }
}
