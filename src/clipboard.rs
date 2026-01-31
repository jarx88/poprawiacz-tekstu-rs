use arboard::Clipboard;
use std::fmt;

/// Clipboard-specific error type
#[derive(Debug, Clone)]
pub enum ClipboardError {
    /// Failed to access clipboard
    AccessFailed(String),
    /// Failed to read text from clipboard
    ReadFailed(String),
    /// Failed to write text to clipboard
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

/// Convert arboard errors to ClipboardError
impl From<arboard::Error> for ClipboardError {
    fn from(err: arboard::Error) -> Self {
        let msg = err.to_string();
        ClipboardError::AccessFailed(msg)
    }
}

/// Read text from system clipboard
///
/// # Returns
/// - `Ok(String)` - The text content from clipboard
/// - `Err(ClipboardError)` - If clipboard access fails or is unavailable
///
/// # Example
/// ```no_run
/// use poprawiacz_tekstu_rs::clipboard::read_text;
///
/// match read_text() {
///     Ok(text) => println!("Clipboard: {}", text),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn read_text() -> Result<String, ClipboardError> {
    let mut clipboard =
        Clipboard::new().map_err(|e| ClipboardError::AccessFailed(e.to_string()))?;

    clipboard
        .get_text()
        .map_err(|e| ClipboardError::ReadFailed(e.to_string()))
}

/// Write text to system clipboard
///
/// # Arguments
/// * `text` - The text to write to clipboard
///
/// # Returns
/// - `Ok(())` - If write succeeds
/// - `Err(ClipboardError)` - If clipboard access fails or write fails
///
/// # Example
/// ```no_run
/// use poprawiacz_tekstu_rs::clipboard::write_text;
///
/// match write_text("Hello, clipboard!") {
///     Ok(()) => println!("Text written to clipboard"),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn write_text(text: &str) -> Result<(), ClipboardError> {
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
    #[ignore] // Requires display/clipboard access - skip in headless CI
    fn test_read_text_success() {
        // First write known text
        let test_text = "test_read_clipboard_content";
        write_text(test_text).expect("Failed to write test text");

        // Then read it back
        let result = read_text();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_text);
    }

    #[test]
    #[ignore] // Requires display/clipboard access - skip in headless CI
    fn test_write_text_success() {
        let test_text = "test_write_clipboard_content";
        let result = write_text(test_text);
        assert!(result.is_ok());

        // Verify by reading back
        let read_result = read_text();
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), test_text);
    }

    #[test]
    #[ignore] // Requires display/clipboard access - skip in headless CI
    fn test_write_empty_string() {
        let result = write_text("");
        assert!(result.is_ok());

        let read_result = read_text();
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), "");
    }

    #[test]
    #[ignore] // Requires display/clipboard access - skip in headless CI
    fn test_write_multiline_text() {
        let test_text = "line1\nline2\nline3";
        let result = write_text(test_text);
        assert!(result.is_ok());

        let read_result = read_text();
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), test_text);
    }

    #[test]
    #[ignore] // Requires display/clipboard access - skip in headless CI
    fn test_write_unicode_text() {
        let test_text = "Hello 世界 🦀 Привет";
        let result = write_text(test_text);
        assert!(result.is_ok());

        let read_result = read_text();
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), test_text);
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

    #[test]
    fn test_clipboard_error_is_error_trait() {
        let err: Box<dyn std::error::Error> =
            Box::new(ClipboardError::AccessFailed("test".to_string()));
        assert!(!err.to_string().is_empty());
    }
}
