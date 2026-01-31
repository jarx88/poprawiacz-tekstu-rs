pub mod error;
pub mod config;
pub mod api;
pub mod ui;
pub mod platform;
pub mod hotkey;
pub mod clipboard;
pub mod diff;
pub mod app;
pub mod tray;

#[cfg(test)]
mod tests {
    #[test]
    fn lib_compiles() {
        assert!(true);
    }
}
