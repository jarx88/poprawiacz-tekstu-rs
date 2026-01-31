use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub api_keys: ApiKeys,
    pub models: Models,
    pub settings: Settings,
    pub ai_settings: AiSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiKeys {
    #[serde(rename = "OpenAI")]
    pub openai: String,
    #[serde(rename = "Anthropic")]
    pub anthropic: String,
    #[serde(rename = "Gemini")]
    pub gemini: String,
    #[serde(rename = "DeepSeek")]
    pub deepseek: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Models {
    #[serde(rename = "OpenAI")]
    pub openai: String,
    #[serde(rename = "Anthropic")]
    pub anthropic: String,
    #[serde(rename = "Gemini")]
    pub gemini: String,
    #[serde(rename = "DeepSeek")]
    pub deepseek: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    #[serde(rename = "AutoStartup")]
    pub auto_startup: bool,
    #[serde(rename = "DefaultStyle")]
    pub default_style: String,
    #[serde(rename = "HighlightDiffs")]
    pub highlight_diffs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AiSettings {
    #[serde(rename = "ReasoningEffort")]
    pub reasoning_effort: String,
    #[serde(rename = "Verbosity")]
    pub verbosity: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            api_keys: ApiKeys {
                openai: String::new(),
                anthropic: String::new(),
                gemini: String::new(),
                deepseek: String::new(),
            },
            models: Models {
                openai: "gpt-5-mini".to_string(),
                anthropic: "claude-3-7-sonnet-latest".to_string(),
                gemini: "gemini-2.5-flash".to_string(),
                deepseek: "deepseek-chat".to_string(),
            },
            settings: Settings {
                auto_startup: false,
                default_style: "normal".to_string(),
                highlight_diffs: false,
            },
            ai_settings: AiSettings {
                reasoning_effort: "high".to_string(),
                verbosity: "medium".to_string(),
            },
        }
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let toml_string = toml::to_string_pretty(self)?;
        fs::write(path, toml_string)?;
        Ok(())
    }

    pub fn get_config_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join("config.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.models.openai, "gpt-5-mini");
        assert_eq!(config.models.anthropic, "claude-3-7-sonnet-latest");
        assert_eq!(config.models.gemini, "gemini-2.5-flash");
        assert_eq!(config.models.deepseek, "deepseek-chat");
        assert_eq!(config.settings.auto_startup, false);
        assert_eq!(config.ai_settings.reasoning_effort, "high");
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();

        let original_config = Config::default();
        original_config.save(temp_path).unwrap();

        let loaded_config = Config::load(temp_path).unwrap();
        assert_eq!(original_config, loaded_config);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();

        assert!(toml_str.contains("OpenAI"));
        assert!(toml_str.contains("Anthropic"));
        assert!(toml_str.contains("Gemini"));
        assert!(toml_str.contains("DeepSeek"));
    }

    #[test]
    fn test_config_fields_exist() {
        let config = Config::default();
        let _api_keys = &config.api_keys;
        let _models = &config.models;
        let _settings = &config.settings;
        let _ai_settings = &config.ai_settings;
    }
}
