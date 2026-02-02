use crate::config::{AiSettings, ApiKeys, Config, Models};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SettingsDialog {
    pub visible: bool,
    pub temp_openai_key: String,
    pub temp_anthropic_key: String,
    pub temp_gemini_key: String,
    pub temp_deepseek_key: String,
    pub temp_openai_model: String,
    pub temp_anthropic_model: String,
    pub temp_gemini_model: String,
    pub temp_deepseek_model: String,
    pub temp_reasoning_effort: String,
    pub temp_verbosity: String,
    pub validation_error: Option<String>,
}

impl Default for SettingsDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsDialog {
    pub fn new() -> Self {
        Self {
            visible: false,
            temp_openai_key: String::new(),
            temp_anthropic_key: String::new(),
            temp_gemini_key: String::new(),
            temp_deepseek_key: String::new(),
            temp_openai_model: String::new(),
            temp_anthropic_model: String::new(),
            temp_gemini_model: String::new(),
            temp_deepseek_model: String::new(),
            temp_reasoning_effort: "high".to_string(),
            temp_verbosity: "medium".to_string(),
            validation_error: None,
        }
    }

    pub fn load_from_config(&mut self, config: &Config) {
        self.temp_openai_key = config.api_keys.openai.clone();
        self.temp_anthropic_key = config.api_keys.anthropic.clone();
        self.temp_gemini_key = config.api_keys.gemini.clone();
        self.temp_deepseek_key = config.api_keys.deepseek.clone();
        self.temp_openai_model = config.models.openai.clone();
        self.temp_anthropic_model = config.models.anthropic.clone();
        self.temp_gemini_model = config.models.gemini.clone();
        self.temp_deepseek_model = config.models.deepseek.clone();
        self.temp_reasoning_effort = config.ai_settings.reasoning_effort.clone();
        self.temp_verbosity = config.ai_settings.verbosity.clone();
        self.validation_error = None;
    }

    /// Validate settings - ensure no empty API keys for enabled providers
    pub fn validate(&mut self) -> Result<(), String> {
        // Check if any API key is empty
        if self.temp_openai_key.trim().is_empty() {
            return Err("OpenAI API key cannot be empty".to_string());
        }
        if self.temp_anthropic_key.trim().is_empty() {
            return Err("Anthropic API key cannot be empty".to_string());
        }
        if self.temp_gemini_key.trim().is_empty() {
            return Err("Gemini API key cannot be empty".to_string());
        }
        if self.temp_deepseek_key.trim().is_empty() {
            return Err("DeepSeek API key cannot be empty".to_string());
        }

        // Check if any model is empty
        if self.temp_openai_model.trim().is_empty() {
            return Err("OpenAI model cannot be empty".to_string());
        }
        if self.temp_anthropic_model.trim().is_empty() {
            return Err("Anthropic model cannot be empty".to_string());
        }
        if self.temp_gemini_model.trim().is_empty() {
            return Err("Gemini model cannot be empty".to_string());
        }
        if self.temp_deepseek_model.trim().is_empty() {
            return Err("DeepSeek model cannot be empty".to_string());
        }

        self.validation_error = None;
        Ok(())
    }

    pub fn save_to_config(&mut self, config: &mut Config, path: &PathBuf) -> Result<(), String> {
        self.validate()?;

        config.api_keys = ApiKeys {
            openai: self.temp_openai_key.trim().to_string(),
            anthropic: self.temp_anthropic_key.trim().to_string(),
            gemini: self.temp_gemini_key.trim().to_string(),
            deepseek: self.temp_deepseek_key.trim().to_string(),
        };

        config.models = Models {
            openai: self.temp_openai_model.trim().to_string(),
            anthropic: self.temp_anthropic_model.trim().to_string(),
            gemini: self.temp_gemini_model.trim().to_string(),
            deepseek: self.temp_deepseek_model.trim().to_string(),
        };

        config.ai_settings = AiSettings {
            reasoning_effort: self.temp_reasoning_effort.clone(),
            verbosity: self.temp_verbosity.clone(),
        };

        config
            .save(path)
            .map_err(|e| format!("Failed to save config: {}", e))?;

        self.validation_error = None;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.temp_openai_key.clear();
        self.temp_anthropic_key.clear();
        self.temp_gemini_key.clear();
        self.temp_deepseek_key.clear();
        self.temp_openai_model.clear();
        self.temp_anthropic_model.clear();
        self.temp_gemini_model.clear();
        self.temp_deepseek_model.clear();
        self.temp_reasoning_effort = "high".to_string();
        self.temp_verbosity = "medium".to_string();
        self.validation_error = None;
    }

    /// Show the settings dialog
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the settings dialog
    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn show_window_content(&mut self, ctx: &egui::Context) -> Option<SettingsAction> {
        let mut action = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                action = self.render_content(ui);
            });
        });

        action
    }

    pub fn show_window(&mut self, ctx: &egui::Context) -> Option<SettingsAction> {
        let mut action = None;
        let mut visible = self.visible;

        egui::Window::new("⚙️ Ustawienia")
            .open(&mut visible)
            .resizable(true)
            .default_width(620.0)
            .default_height(850.0)
            .collapsible(false)
            .show(ctx, |ui| {
                action = self.render_content(ui);
            });

        self.visible = visible;
        action
    }

    fn render_content(&mut self, ui: &mut egui::Ui) -> Option<SettingsAction> {
        let mut action = None;

        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                ui.set_max_width(580.0);

                if let Some(error) = &self.validation_error {
                    egui::Frame::NONE
                        .fill(egui::Color32::from_rgb(80, 20, 20))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 60, 60)))
                        .corner_radius(egui::CornerRadius::same(8))
                        .inner_margin(egui::Margin::same(12))
                        .show(ui, |ui| {
                            ui.colored_label(
                                egui::Color32::from_rgb(248, 113, 113),
                                format!("❌ {}", error),
                            );
                        });
                    ui.add_space(12.0);
                }

                let api_section = |ui: &mut egui::Ui,
                                   icon: &str,
                                   name: &str,
                                   color: egui::Color32,
                                   key: &mut String,
                                   model: &mut String| {
                    egui::Frame::NONE
                        .fill(egui::Color32::from_rgb(38, 38, 46))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(55, 55, 65)))
                        .corner_radius(egui::CornerRadius::same(10))
                        .inner_margin(egui::Margin::same(16))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("{} {}", icon, name))
                                        .size(15.0)
                                        .strong()
                                        .color(color),
                                );
                            });

                            ui.add_space(8.0);

                            ui.label(
                                egui::RichText::new("API Key")
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(160, 160, 170)),
                            );
                            ui.add(
                                egui::TextEdit::singleline(key)
                                    .password(true)
                                    .desired_width(f32::INFINITY),
                            );

                            ui.add_space(8.0);

                            ui.label(
                                egui::RichText::new("Model")
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(160, 160, 170)),
                            );
                            ui.add(egui::TextEdit::singleline(model).desired_width(f32::INFINITY));
                        });
                };

                api_section(
                    ui,
                    "●",
                    "OpenAI",
                    egui::Color32::from_rgb(16, 163, 127),
                    &mut self.temp_openai_key,
                    &mut self.temp_openai_model,
                );
                ui.add_space(12.0);

                api_section(
                    ui,
                    "●",
                    "Anthropic",
                    egui::Color32::from_rgb(217, 119, 6),
                    &mut self.temp_anthropic_key,
                    &mut self.temp_anthropic_model,
                );
                ui.add_space(12.0);

                api_section(
                    ui,
                    "●",
                    "Gemini",
                    egui::Color32::from_rgb(66, 133, 244),
                    &mut self.temp_gemini_key,
                    &mut self.temp_gemini_model,
                );
                ui.add_space(12.0);

                api_section(
                    ui,
                    "●",
                    "DeepSeek",
                    egui::Color32::from_rgb(124, 58, 237),
                    &mut self.temp_deepseek_key,
                    &mut self.temp_deepseek_model,
                );
                ui.add_space(12.0);

                egui::Frame::NONE
                    .fill(egui::Color32::from_rgb(38, 38, 46))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(55, 55, 65)))
                    .corner_radius(egui::CornerRadius::same(10))
                    .inner_margin(egui::Margin::same(16))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("🤖 AI Settings")
                                .size(15.0)
                                .strong()
                                .color(egui::Color32::from_rgb(140, 140, 200)),
                        );

                        ui.add_space(12.0);

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Reasoning Effort:")
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(160, 160, 170)),
                            );
                            egui::ComboBox::from_id_salt("reasoning_effort")
                                .selected_text(&self.temp_reasoning_effort)
                                .width(150.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.temp_reasoning_effort,
                                        "minimal".to_string(),
                                        "Minimal",
                                    );
                                    ui.selectable_value(
                                        &mut self.temp_reasoning_effort,
                                        "low".to_string(),
                                        "Low",
                                    );
                                    ui.selectable_value(
                                        &mut self.temp_reasoning_effort,
                                        "medium".to_string(),
                                        "Medium",
                                    );
                                    ui.selectable_value(
                                        &mut self.temp_reasoning_effort,
                                        "high".to_string(),
                                        "High",
                                    );
                                });
                        });

                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Verbosity:")
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(160, 160, 170)),
                            );
                            ui.add_space(40.0);
                            egui::ComboBox::from_id_salt("verbosity")
                                .selected_text(&self.temp_verbosity)
                                .width(150.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.temp_verbosity,
                                        "low".to_string(),
                                        "Low",
                                    );
                                    ui.selectable_value(
                                        &mut self.temp_verbosity,
                                        "medium".to_string(),
                                        "Medium",
                                    );
                                    ui.selectable_value(
                                        &mut self.temp_verbosity,
                                        "high".to_string(),
                                        "High",
                                    );
                                });
                        });
                    });

                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;

                    let save_btn = egui::Button::new(
                        egui::RichText::new("💾 Zapisz")
                            .color(egui::Color32::WHITE)
                            .size(14.0),
                    )
                    .fill(egui::Color32::from_rgb(34, 197, 94))
                    .corner_radius(egui::CornerRadius::same(8))
                    .min_size(egui::vec2(120.0, 38.0));

                    if ui.add(save_btn).clicked() {
                        action = Some(SettingsAction::Save);
                    }

                    let cancel_btn = egui::Button::new(
                        egui::RichText::new("Anuluj")
                            .color(egui::Color32::WHITE)
                            .size(14.0),
                    )
                    .fill(egui::Color32::from_rgb(70, 70, 85))
                    .corner_radius(egui::CornerRadius::same(8))
                    .min_size(egui::vec2(120.0, 38.0));

                    if ui.add(cancel_btn).clicked() {
                        action = Some(SettingsAction::Cancel);
                    }
                });
            });

        action
    }
}

/// Actions that can be triggered from the settings dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsAction {
    Save,
    Cancel,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_settings_dialog_new() {
        let dialog = SettingsDialog::new();
        assert!(!dialog.visible);
        assert!(dialog.temp_openai_key.is_empty());
        assert!(dialog.temp_anthropic_key.is_empty());
        assert!(dialog.temp_gemini_key.is_empty());
        assert!(dialog.temp_deepseek_key.is_empty());
        assert!(dialog.validation_error.is_none());
    }

    #[test]
    fn test_settings_load_from_config() {
        let mut dialog = SettingsDialog::new();
        let config = Config::default();

        dialog.load_from_config(&config);

        assert_eq!(dialog.temp_openai_model, "gpt-5-mini");
        assert_eq!(dialog.temp_anthropic_model, "claude-3-7-sonnet-latest");
        assert_eq!(dialog.temp_gemini_model, "gemini-2.5-flash");
        assert_eq!(dialog.temp_deepseek_model, "deepseek-chat");
    }

    #[test]
    fn test_settings_validation_empty_openai_key() {
        let mut dialog = SettingsDialog::new();
        dialog.temp_openai_key = String::new();
        dialog.temp_anthropic_key = "test-key".to_string();
        dialog.temp_gemini_key = "test-key".to_string();
        dialog.temp_deepseek_key = "test-key".to_string();
        dialog.temp_openai_model = "gpt-4".to_string();
        dialog.temp_anthropic_model = "claude-3".to_string();
        dialog.temp_gemini_model = "gemini-1.5".to_string();
        dialog.temp_deepseek_model = "deepseek-chat".to_string();

        let result = dialog.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("OpenAI"));
    }

    #[test]
    fn test_settings_validation_empty_anthropic_key() {
        let mut dialog = SettingsDialog::new();
        dialog.temp_openai_key = "test-key".to_string();
        dialog.temp_anthropic_key = String::new();
        dialog.temp_gemini_key = "test-key".to_string();
        dialog.temp_deepseek_key = "test-key".to_string();
        dialog.temp_openai_model = "gpt-4".to_string();
        dialog.temp_anthropic_model = "claude-3".to_string();
        dialog.temp_gemini_model = "gemini-1.5".to_string();
        dialog.temp_deepseek_model = "deepseek-chat".to_string();

        let result = dialog.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Anthropic"));
    }

    #[test]
    fn test_settings_validation_empty_gemini_key() {
        let mut dialog = SettingsDialog::new();
        dialog.temp_openai_key = "test-key".to_string();
        dialog.temp_anthropic_key = "test-key".to_string();
        dialog.temp_gemini_key = String::new();
        dialog.temp_deepseek_key = "test-key".to_string();
        dialog.temp_openai_model = "gpt-4".to_string();
        dialog.temp_anthropic_model = "claude-3".to_string();
        dialog.temp_gemini_model = "gemini-1.5".to_string();
        dialog.temp_deepseek_model = "deepseek-chat".to_string();

        let result = dialog.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Gemini"));
    }

    #[test]
    fn test_settings_validation_empty_deepseek_key() {
        let mut dialog = SettingsDialog::new();
        dialog.temp_openai_key = "test-key".to_string();
        dialog.temp_anthropic_key = "test-key".to_string();
        dialog.temp_gemini_key = "test-key".to_string();
        dialog.temp_deepseek_key = String::new();
        dialog.temp_openai_model = "gpt-4".to_string();
        dialog.temp_anthropic_model = "claude-3".to_string();
        dialog.temp_gemini_model = "gemini-1.5".to_string();
        dialog.temp_deepseek_model = "deepseek-chat".to_string();

        let result = dialog.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("DeepSeek"));
    }

    #[test]
    fn test_settings_validation_empty_model() {
        let mut dialog = SettingsDialog::new();
        dialog.temp_openai_key = "test-key".to_string();
        dialog.temp_anthropic_key = "test-key".to_string();
        dialog.temp_gemini_key = "test-key".to_string();
        dialog.temp_deepseek_key = "test-key".to_string();
        dialog.temp_openai_model = String::new();
        dialog.temp_anthropic_model = "claude-3".to_string();
        dialog.temp_gemini_model = "gemini-1.5".to_string();
        dialog.temp_deepseek_model = "deepseek-chat".to_string();

        let result = dialog.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("OpenAI model"));
    }

    #[test]
    fn test_settings_validation_success() {
        let mut dialog = SettingsDialog::new();
        dialog.temp_openai_key = "sk-test".to_string();
        dialog.temp_anthropic_key = "sk-ant-test".to_string();
        dialog.temp_gemini_key = "AIza-test".to_string();
        dialog.temp_deepseek_key = "sk-test".to_string();
        dialog.temp_openai_model = "gpt-4".to_string();
        dialog.temp_anthropic_model = "claude-3".to_string();
        dialog.temp_gemini_model = "gemini-1.5".to_string();
        dialog.temp_deepseek_model = "deepseek-chat".to_string();

        let result = dialog.validate();
        assert!(result.is_ok());
        assert!(dialog.validation_error.is_none());
    }

    #[test]
    fn test_settings_validation_whitespace_only() {
        let mut dialog = SettingsDialog::new();
        dialog.temp_openai_key = "   ".to_string();
        dialog.temp_anthropic_key = "test-key".to_string();
        dialog.temp_gemini_key = "test-key".to_string();
        dialog.temp_deepseek_key = "test-key".to_string();
        dialog.temp_openai_model = "gpt-4".to_string();
        dialog.temp_anthropic_model = "claude-3".to_string();
        dialog.temp_gemini_model = "gemini-1.5".to_string();
        dialog.temp_deepseek_model = "deepseek-chat".to_string();

        let result = dialog.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_settings_save_to_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();

        let mut dialog = SettingsDialog::new();
        dialog.temp_openai_key = "sk-openai-test".to_string();
        dialog.temp_anthropic_key = "sk-ant-test".to_string();
        dialog.temp_gemini_key = "AIza-test".to_string();
        dialog.temp_deepseek_key = "sk-deepseek-test".to_string();
        dialog.temp_openai_model = "gpt-4-turbo".to_string();
        dialog.temp_anthropic_model = "claude-3-opus".to_string();
        dialog.temp_gemini_model = "gemini-1.5-pro".to_string();
        dialog.temp_deepseek_model = "deepseek-coder".to_string();

        let mut config = Config::default();
        let result = dialog.save_to_config(&mut config, &temp_path);

        assert!(result.is_ok());
        assert_eq!(config.api_keys.openai, "sk-openai-test");
        assert_eq!(config.api_keys.anthropic, "sk-ant-test");
        assert_eq!(config.api_keys.gemini, "AIza-test");
        assert_eq!(config.api_keys.deepseek, "sk-deepseek-test");
        assert_eq!(config.models.openai, "gpt-4-turbo");
        assert_eq!(config.models.anthropic, "claude-3-opus");
        assert_eq!(config.models.gemini, "gemini-1.5-pro");
        assert_eq!(config.models.deepseek, "deepseek-coder");
    }

    #[test]
    fn test_settings_save_to_config_persists_to_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();

        let mut dialog = SettingsDialog::new();
        dialog.temp_openai_key = "sk-openai-test".to_string();
        dialog.temp_anthropic_key = "sk-ant-test".to_string();
        dialog.temp_gemini_key = "AIza-test".to_string();
        dialog.temp_deepseek_key = "sk-deepseek-test".to_string();
        dialog.temp_openai_model = "gpt-4-turbo".to_string();
        dialog.temp_anthropic_model = "claude-3-opus".to_string();
        dialog.temp_gemini_model = "gemini-1.5-pro".to_string();
        dialog.temp_deepseek_model = "deepseek-coder".to_string();

        let mut config = Config::default();
        dialog.save_to_config(&mut config, &temp_path).unwrap();

        // Load the config from file and verify
        let loaded_config = Config::load(&temp_path).unwrap();
        assert_eq!(loaded_config.api_keys.openai, "sk-openai-test");
        assert_eq!(loaded_config.models.openai, "gpt-4-turbo");
    }

    #[test]
    fn test_settings_save_to_config_validation_fails() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();

        let mut dialog = SettingsDialog::new();
        dialog.temp_openai_key = String::new(); // Empty key
        dialog.temp_anthropic_key = "sk-ant-test".to_string();
        dialog.temp_gemini_key = "AIza-test".to_string();
        dialog.temp_deepseek_key = "sk-deepseek-test".to_string();
        dialog.temp_openai_model = "gpt-4".to_string();
        dialog.temp_anthropic_model = "claude-3".to_string();
        dialog.temp_gemini_model = "gemini-1.5".to_string();
        dialog.temp_deepseek_model = "deepseek-chat".to_string();

        let mut config = Config::default();
        let result = dialog.save_to_config(&mut config, &temp_path);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("OpenAI"));
    }

    #[test]
    fn test_settings_clear() {
        let mut dialog = SettingsDialog::new();
        dialog.temp_openai_key = "test".to_string();
        dialog.temp_anthropic_key = "test".to_string();
        dialog.temp_gemini_key = "test".to_string();
        dialog.temp_deepseek_key = "test".to_string();
        dialog.temp_openai_model = "gpt-4".to_string();
        dialog.validation_error = Some("error".to_string());

        dialog.clear();

        assert!(dialog.temp_openai_key.is_empty());
        assert!(dialog.temp_anthropic_key.is_empty());
        assert!(dialog.temp_gemini_key.is_empty());
        assert!(dialog.temp_deepseek_key.is_empty());
        assert!(dialog.temp_openai_model.is_empty());
        assert!(dialog.validation_error.is_none());
    }

    #[test]
    fn test_settings_show_hide() {
        let mut dialog = SettingsDialog::new();
        assert!(!dialog.visible);

        dialog.show();
        assert!(dialog.visible);

        dialog.hide();
        assert!(!dialog.visible);
    }

    #[test]
    fn test_settings_action_enum() {
        let save_action = SettingsAction::Save;
        let cancel_action = SettingsAction::Cancel;

        assert_eq!(save_action, SettingsAction::Save);
        assert_eq!(cancel_action, SettingsAction::Cancel);
        assert_ne!(save_action, cancel_action);
    }
}
