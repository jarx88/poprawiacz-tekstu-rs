pub mod error;
pub mod config;
pub mod api;
pub mod ui;
pub mod platform;
pub mod hotkey;
pub mod tray;
pub mod clipboard;
pub mod diff;

#[cfg(test)]
mod tests {
    #[test]
    fn lib_compiles() {
        assert!(true);
    }
}
