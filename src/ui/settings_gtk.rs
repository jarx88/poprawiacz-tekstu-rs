use crate::config::Config;
use gtk4::glib;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use tracing::info;

pub struct SettingsDialog {
    dialog: adw::PreferencesWindow,
    openai_key: gtk4::Entry,
    openai_model: gtk4::Entry,
    anthropic_key: gtk4::Entry,
    anthropic_model: gtk4::Entry,
    gemini_key: gtk4::Entry,
    gemini_model: gtk4::Entry,
    deepseek_key: gtk4::Entry,
    deepseek_model: gtk4::Entry,
    highlight_diffs: gtk4::Switch,
}

fn create_entry_row(title: &str, value: &str, is_password: bool) -> (adw::ActionRow, gtk4::Entry) {
    let row = adw::ActionRow::builder().title(title).build();

    let entry = gtk4::Entry::builder()
        .text(value)
        .valign(gtk4::Align::Center)
        .hexpand(true)
        .visibility(!is_password)
        .build();

    if is_password {
        entry.add_css_class("monospace");
    }

    row.add_suffix(&entry);
    (row, entry)
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

        let (openai_key_row, openai_key) =
            create_entry_row("Klucz API", &config.api_keys.openai, true);
        openai_group.add(&openai_key_row);

        let (openai_model_row, openai_model) =
            create_entry_row("Model", &config.models.openai, false);
        openai_group.add(&openai_model_row);

        api_page.add(&openai_group);

        let anthropic_group = adw::PreferencesGroup::builder().title("Anthropic").build();

        let (anthropic_key_row, anthropic_key) =
            create_entry_row("Klucz API", &config.api_keys.anthropic, true);
        anthropic_group.add(&anthropic_key_row);

        let (anthropic_model_row, anthropic_model) =
            create_entry_row("Model", &config.models.anthropic, false);
        anthropic_group.add(&anthropic_model_row);

        api_page.add(&anthropic_group);

        let gemini_group = adw::PreferencesGroup::builder().title("Gemini").build();

        let (gemini_key_row, gemini_key) =
            create_entry_row("Klucz API", &config.api_keys.gemini, true);
        gemini_group.add(&gemini_key_row);

        let (gemini_model_row, gemini_model) =
            create_entry_row("Model", &config.models.gemini, false);
        gemini_group.add(&gemini_model_row);

        api_page.add(&gemini_group);

        let deepseek_group = adw::PreferencesGroup::builder().title("DeepSeek").build();

        let (deepseek_key_row, deepseek_key) =
            create_entry_row("Klucz API", &config.api_keys.deepseek, true);
        deepseek_group.add(&deepseek_key_row);

        let (deepseek_model_row, deepseek_model) =
            create_entry_row("Model", &config.models.deepseek, false);
        deepseek_group.add(&deepseek_model_row);

        api_page.add(&deepseek_group);

        dialog.add(&api_page);

        let settings_page = adw::PreferencesPage::builder()
            .title("Ustawienia")
            .icon_name("emblem-system-symbolic")
            .build();

        let display_group = adw::PreferencesGroup::builder()
            .title("Wyswietlanie")
            .build();

        let highlight_row = adw::ActionRow::builder()
            .title("Podswietlaj roznice")
            .subtitle("Zaznacz zmiany miedzy oryginalem a poprawionym tekstem")
            .build();

        let highlight_diffs = gtk4::Switch::builder()
            .valign(gtk4::Align::Center)
            .active(config.settings.highlight_diffs)
            .build();
        highlight_row.add_suffix(&highlight_diffs);
        highlight_row.set_activatable_widget(Some(&highlight_diffs));

        display_group.add(&highlight_row);
        settings_page.add(&display_group);

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
        }
    }

    pub fn present(&self) {
        self.dialog.present();
    }

    pub fn to_config(&self) -> Config {
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
                reasoning_effort: "high".to_string(),
                verbosity: "medium".to_string(),
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

        self.dialog.connect_close_request(move |_| {
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
                    reasoning_effort: "high".to_string(),
                    verbosity: "medium".to_string(),
                },
            };

            callback(config);
            info!("Settings saved");

            glib::Propagation::Proceed
        });
    }
}
