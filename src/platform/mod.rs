use crate::error::PlatformError;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::LinuxKeyboardSimulator;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsKeyboardSimulator;

pub trait KeyboardSimulator {
    fn simulate_copy(&self) -> Result<(), PlatformError>;
    fn simulate_paste(&self) -> Result<(), PlatformError>;
}

#[cfg(target_os = "linux")]
pub fn create_simulator() -> impl KeyboardSimulator {
    LinuxKeyboardSimulator::new()
}

#[cfg(target_os = "windows")]
pub fn create_simulator() -> impl KeyboardSimulator {
    WindowsKeyboardSimulator::new()
}

pub fn simulate_copy() -> Result<(), PlatformError> {
    create_simulator().simulate_copy()
}

pub fn simulate_paste() -> Result<(), PlatformError> {
    create_simulator().simulate_paste()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "linux")]
    fn test_create_linux_simulator() {
        let simulator = create_simulator();
        assert!(std::any::type_name_of_val(&simulator).contains("LinuxKeyboardSimulator"));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_create_windows_simulator() {
        let simulator = create_simulator();
        assert!(std::any::type_name_of_val(&simulator).contains("WindowsKeyboardSimulator"));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_simulate_copy_returns_error_when_xdotool_missing() {
        let simulator = LinuxKeyboardSimulator::new();
        let result = simulator.simulate_copy();
        if !LinuxKeyboardSimulator::is_xdotool_available() {
            assert!(result.is_err());
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_simulate_paste_returns_error_when_xdotool_missing() {
        let simulator = LinuxKeyboardSimulator::new();
        let result = simulator.simulate_paste();
        if !LinuxKeyboardSimulator::is_xdotool_available() {
            assert!(result.is_err());
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_not_implemented() {
        let simulator = WindowsKeyboardSimulator::new();
        let result = simulator.simulate_copy();
        assert!(result.is_err());

        let result = simulator.simulate_paste();
        assert!(result.is_err());
    }
}
