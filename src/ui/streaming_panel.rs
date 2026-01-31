use tokio::sync::mpsc;

pub struct StreamingPanel {
    text: String,
    rx: mpsc::UnboundedReceiver<String>,
    auto_scroll: bool,
}

impl StreamingPanel {
    pub fn new(rx: mpsc::UnboundedReceiver<String>) -> Self {
        Self {
            text: String::new(),
            rx,
            auto_scroll: true,
        }
    }

    pub fn clear(&mut self) {
        self.text.clear();
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    pub fn set_auto_scroll(&mut self, enabled: bool) {
        self.auto_scroll = enabled;
    }

    pub fn update_and_render(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let mut received_new_text = false;

        while let Ok(chunk) = self.rx.try_recv() {
            self.text.push_str(&chunk);
            received_new_text = true;
        }

        if received_new_text {
            ctx.request_repaint();
        }

        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .stick_to_bottom(self.auto_scroll)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.text.as_str())
                        .desired_width(f32::INFINITY)
                        .interactive(false),
                );
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_creation() {
        let (_tx, rx) = mpsc::unbounded_channel();
        let panel = StreamingPanel::new(rx);
        assert_eq!(panel.get_text(), "");
        assert!(panel.auto_scroll);
    }

    #[test]
    fn test_clear_text() {
        let (_tx, rx) = mpsc::unbounded_channel();
        let mut panel = StreamingPanel::new(rx);
        panel.text = "test content".to_string();
        panel.clear();
        assert_eq!(panel.get_text(), "");
    }

    #[test]
    fn test_auto_scroll_toggle() {
        let (_tx, rx) = mpsc::unbounded_channel();
        let mut panel = StreamingPanel::new(rx);
        assert!(panel.auto_scroll);
        panel.set_auto_scroll(false);
        assert!(!panel.auto_scroll);
        panel.set_auto_scroll(true);
        assert!(panel.auto_scroll);
    }

    #[tokio::test]
    async fn test_channel_message_delivery() {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut panel = StreamingPanel::new(rx);

        tx.send("Hello, ".to_string()).unwrap();
        tx.send("world!".to_string()).unwrap();

        while let Ok(chunk) = panel.rx.try_recv() {
            panel.text.push_str(&chunk);
        }

        assert_eq!(panel.get_text(), "Hello, world!");
    }

    #[tokio::test]
    async fn test_text_accumulation() {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut panel = StreamingPanel::new(rx);

        for i in 1..=5 {
            tx.send(format!("Line {}\n", i)).unwrap();
        }

        while let Ok(chunk) = panel.rx.try_recv() {
            panel.text.push_str(&chunk);
        }

        assert_eq!(panel.get_text(), "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\n");
    }

    #[test]
    fn test_panel_state_management() {
        let (_tx, rx) = mpsc::unbounded_channel();
        let mut panel = StreamingPanel::new(rx);

        panel.text = "Initial text".to_string();
        assert_eq!(panel.get_text(), "Initial text");

        panel.text.push_str(" + more");
        assert_eq!(panel.get_text(), "Initial text + more");

        panel.clear();
        assert_eq!(panel.get_text(), "");
    }

    #[tokio::test]
    async fn test_empty_channel() {
        let (_tx, rx) = mpsc::unbounded_channel();
        let mut panel = StreamingPanel::new(rx);

        assert!(panel.rx.try_recv().is_err());
        assert_eq!(panel.get_text(), "");
    }

    #[tokio::test]
    async fn test_channel_closed() {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut panel = StreamingPanel::new(rx);

        tx.send("Message".to_string()).unwrap();
        drop(tx);

        while let Ok(chunk) = panel.rx.try_recv() {
            panel.text.push_str(&chunk);
        }

        assert_eq!(panel.get_text(), "Message");
        assert!(panel.rx.try_recv().is_err());
    }
}
