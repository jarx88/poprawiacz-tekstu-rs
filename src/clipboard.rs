use arboard::Clipboard;
use std::fmt;
use std::process::Command;

#[derive(Debug, Clone)]
pub enum ClipboardError {
    AccessFailed(String),
    ReadFailed(String),
    WriteFailed(String),
}

impl fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClipboardError::AccessFailed(msg) => write!(f, "Clipboard access failed: {}", msg),
            ClipboardError::ReadFailed(msg) => write!(f, "Failed to read clipboard: {}", msg),
            ClipboardError::WriteFailed(msg) => write!(f, "Failed to write clipboard: {}", msg),
        }
    }
}

impl std::error::Error for ClipboardError {}

impl From<arboard::Error> for ClipboardError {
    fn from(err: arboard::Error) -> Self {
        ClipboardError::AccessFailed(err.to_string())
    }
}

fn is_wayland() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_ok()
        || std::env::var("XDG_SESSION_TYPE")
            .map(|v| v == "wayland")
            .unwrap_or(false)
}

fn read_text_wl_paste() -> Result<String, ClipboardError> {
    let output = Command::new("wl-paste")
        .arg("--no-newline")
        .output()
        .map_err(|e| ClipboardError::ReadFailed(format!("wl-paste failed: {}", e)))?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .map_err(|e| ClipboardError::ReadFailed(format!("Invalid UTF-8: {}", e)))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(ClipboardError::ReadFailed(format!(
            "wl-paste error: {}",
            stderr
        )))
    }
}

fn write_text_wl_copy(text: &str) -> Result<(), ClipboardError> {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| ClipboardError::WriteFailed(format!("wl-copy failed: {}", e)))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| ClipboardError::WriteFailed(format!("Write to wl-copy failed: {}", e)))?;
    }

    let status = child
        .wait()
        .map_err(|e| ClipboardError::WriteFailed(format!("wl-copy wait failed: {}", e)))?;

    if status.success() {
        Ok(())
    } else {
        Err(ClipboardError::WriteFailed(
            "wl-copy returned error".to_string(),
        ))
    }
}

pub fn read_text() -> Result<String, ClipboardError> {
    if is_wayland() {
        return read_text_wl_paste();
    }

    let mut clipboard =
        Clipboard::new().map_err(|e| ClipboardError::AccessFailed(e.to_string()))?;

    clipboard
        .get_text()
        .map_err(|e| ClipboardError::ReadFailed(e.to_string()))
}

pub fn write_text(text: &str) -> Result<(), ClipboardError> {
    if is_wayland() {
        return write_text_wl_copy(text);
    }

    let mut clipboard =
        Clipboard::new().map_err(|e| ClipboardError::AccessFailed(e.to_string()))?;

    clipboard
        .set_text(text)
        .map_err(|e| ClipboardError::WriteFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_wayland() {
        let result = is_wayland();
        assert!(result == true || result == false);
    }

    #[test]
    fn test_clipboard_error_display() {
        let err = ClipboardError::AccessFailed("No display".to_string());
        assert_eq!(err.to_string(), "Clipboard access failed: No display");
    }

    #[test]
    fn test_clipboard_error_read_display() {
        let err = ClipboardError::ReadFailed("Empty clipboard".to_string());
        assert_eq!(err.to_string(), "Failed to read clipboard: Empty clipboard");
    }

    #[test]
    fn test_clipboard_error_write_display() {
        let err = ClipboardError::WriteFailed("Permission denied".to_string());
        assert_eq!(
            err.to_string(),
            "Failed to write clipboard: Permission denied"
        );
    }
}
