use crate::config::Config;
use gtk4::glib;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use tracing::info;

pub struct SettingsDialog {
    dialog: adw::PreferencesWindow,
    openai_key: adw::EntryRow,
    openai_model: adw::EntryRow,
    anthropic_key: adw::EntryRow,
    anthropic_model: adw::EntryRow,
    gemini_key: adw::EntryRow,
    gemini_model: adw::EntryRow,
    deepseek_key: adw::EntryRow,
    deepseek_model: adw::EntryRow,
    highlight_diffs: gtk4::Switch,
    reasoning_effort: adw::ComboRow,
    verbosity: adw::ComboRow,
}

impl SettingsDialog {
    pub fn new(parent: &adw::ApplicationWindow, config: &Config) -> Self {
        let dialog = adw::PreferencesWindow::builder()
            .title("Ustawienia")
            .transient_for(parent)
            .modal(true)
            .default_width(600)
            .default_height(700)
            .build();

        let api_page = adw::PreferencesPage::builder()
            .title("API")
            .icon_name("network-server-symbolic")
            .build();

        let openai_group = adw::PreferencesGroup::builder().title("OpenAI").build();

        let openai_key = adw::EntryRow::builder()
            .title("Klucz API")
            .text(&config.api_keys.openai)
            .build();
        openai_key.add_css_class("monospace");
        openai_group.add(&openai_key);

        let openai_model = adw::EntryRow::builder()
            .title("Model")
            .text(&config.models.openai)
            .build();
        openai_group.add(&openai_model);

        api_page.add(&openai_group);

        let anthropic_group = adw::PreferencesGroup::builder().title("Anthropic").build();

        let anthropic_key = adw::EntryRow::builder()
            .title("Klucz API")
            .text(&config.api_keys.anthropic)
            .build();
        anthropic_key.add_css_class("monospace");
        anthropic_group.add(&anthropic_key);

        let anthropic_model = adw::EntryRow::builder()
            .title("Model")
            .text(&config.models.anthropic)
            .build();
        anthropic_group.add(&anthropic_model);

        api_page.add(&anthropic_group);

        let gemini_group = adw::PreferencesGroup::builder().title("Gemini").build();

        let gemini_key = adw::EntryRow::builder()
            .title("Klucz API")
            .text(&config.api_keys.gemini)
            .build();
        gemini_key.add_css_class("monospace");
        gemini_group.add(&gemini_key);

        let gemini_model = adw::EntryRow::builder()
            .title("Model")
            .text(&config.models.gemini)
            .build();
        gemini_group.add(&gemini_model);

        api_page.add(&gemini_group);

        let deepseek_group = adw::PreferencesGroup::builder().title("DeepSeek").build();

        let deepseek_key = adw::EntryRow::builder()
            .title("Klucz API")
            .text(&config.api_keys.deepseek)
            .build();
        deepseek_key.add_css_class("monospace");
        deepseek_group.add(&deepseek_key);

        let deepseek_model = adw::EntryRow::builder()
            .title("Model")
            .text(&config.models.deepseek)
            .build();
        deepseek_group.add(&deepseek_model);

        api_page.add(&deepseek_group);

        dialog.add(&api_page);

        let settings_page = adw::PreferencesPage::builder()
            .title("Ustawienia")
            .icon_name("emblem-system-symbolic")
            .build();

        let display_group = adw::PreferencesGroup::builder()
            .title("Wyświetlanie")
            .build();

        let highlight_row = adw::ActionRow::builder()
            .title("Podświetlaj różnice")
            .subtitle("Zaznacz zmiany między oryginałem a poprawionym tekstem")
            .build();

        let highlight_diffs = gtk4::Switch::builder()
            .valign(gtk4::Align::Center)
            .active(config.settings.highlight_diffs)
            .build();
        highlight_row.add_suffix(&highlight_diffs);
        highlight_row.set_activatable_widget(Some(&highlight_diffs));

        display_group.add(&highlight_row);
        settings_page.add(&display_group);

        let ai_group = adw::PreferencesGroup::builder()
            .title("Ustawienia AI")
            .description("Parametry przetwarzania przez modele AI")
            .build();

        let effort_options = gtk4::StringList::new(&["low", "medium", "high"]);
        let reasoning_effort = adw::ComboRow::builder()
            .title("Reasoning Effort")
            .subtitle("Poziom dokładności rozumowania (dla modeli o1/o3)")
            .model(&effort_options)
            .build();
        let effort_idx = match config.ai_settings.reasoning_effort.as_str() {
            "low" => 0,
            "medium" => 1,
            "high" => 2,
            _ => 2,
        };
        reasoning_effort.set_selected(effort_idx);
        ai_group.add(&reasoning_effort);

        let verbosity_options = gtk4::StringList::new(&["low", "medium", "high"]);
        let verbosity = adw::ComboRow::builder()
            .title("Verbosity")
            .subtitle("Poziom szczegółowości odpowiedzi")
            .model(&verbosity_options)
            .build();
        let verb_idx = match config.ai_settings.verbosity.as_str() {
            "low" => 0,
            "medium" => 1,
            "high" => 2,
            _ => 1,
        };
        verbosity.set_selected(verb_idx);
        ai_group.add(&verbosity);

        settings_page.add(&ai_group);

        dialog.add(&settings_page);

        Self {
            dialog,
            openai_key,
            openai_model,
            anthropic_key,
            anthropic_model,
            gemini_key,
            gemini_model,
            deepseek_key,
            deepseek_model,
            highlight_diffs,
            reasoning_effort,
            verbosity,
        }
    }

    pub fn present(&self) {
        self.dialog.present();
    }

    pub fn to_config(&self) -> Config {
        let effort_options = ["low", "medium", "high"];
        let reasoning_effort = effort_options
            .get(self.reasoning_effort.selected() as usize)
            .unwrap_or(&"high")
            .to_string();

        let verbosity_options = ["low", "medium", "high"];
        let verbosity = verbosity_options
            .get(self.verbosity.selected() as usize)
            .unwrap_or(&"medium")
            .to_string();

        Config {
            api_keys: crate::config::ApiKeys {
                openai: self.openai_key.text().to_string(),
                anthropic: self.anthropic_key.text().to_string(),
                gemini: self.gemini_key.text().to_string(),
                deepseek: self.deepseek_key.text().to_string(),
            },
            models: crate::config::Models {
                openai: self.openai_model.text().to_string(),
                anthropic: self.anthropic_model.text().to_string(),
                gemini: self.gemini_model.text().to_string(),
                deepseek: self.deepseek_model.text().to_string(),
            },
            settings: crate::config::Settings {
                auto_startup: false,
                default_style: "normal".to_string(),
                highlight_diffs: self.highlight_diffs.is_active(),
            },
            ai_settings: crate::config::AiSettings {
                reasoning_effort,
                verbosity,
            },
        }
    }

    pub fn connect_save<F: Fn(Config) + 'static>(&self, callback: F) {
        let openai_key = self.openai_key.clone();
        let openai_model = self.openai_model.clone();
        let anthropic_key = self.anthropic_key.clone();
        let anthropic_model = self.anthropic_model.clone();
        let gemini_key = self.gemini_key.clone();
        let gemini_model = self.gemini_model.clone();
        let deepseek_key = self.deepseek_key.clone();
        let deepseek_model = self.deepseek_model.clone();
        let highlight_diffs = self.highlight_diffs.clone();
        let reasoning_effort_row = self.reasoning_effort.clone();
        let verbosity_row = self.verbosity.clone();
        let _dialog = self.dialog.clone();

        self.dialog.connect_close_request(move |_| {
            let effort_options = ["low", "medium", "high"];
            let reasoning_effort = effort_options
                .get(reasoning_effort_row.selected() as usize)
                .unwrap_or(&"high")
                .to_string();

            let verbosity_options = ["low", "medium", "high"];
            let verbosity = verbosity_options
                .get(verbosity_row.selected() as usize)
                .unwrap_or(&"medium")
                .to_string();

            let config = Config {
                api_keys: crate::config::ApiKeys {
                    openai: openai_key.text().to_string(),
                    anthropic: anthropic_key.text().to_string(),
                    gemini: gemini_key.text().to_string(),
                    deepseek: deepseek_key.text().to_string(),
                },
                models: crate::config::Models {
                    openai: openai_model.text().to_string(),
                    anthropic: anthropic_model.text().to_string(),
                    gemini: gemini_model.text().to_string(),
                    deepseek: deepseek_model.text().to_string(),
                },
                settings: crate::config::Settings {
                    auto_startup: false,
                    default_style: "normal".to_string(),
                    highlight_diffs: highlight_diffs.is_active(),
                },
                ai_settings: crate::config::AiSettings {
                    reasoning_effort,
                    verbosity,
                },
            };

            callback(config);
            info!("Settings saved");

            glib::Propagation::Proceed
        });
    }
}
