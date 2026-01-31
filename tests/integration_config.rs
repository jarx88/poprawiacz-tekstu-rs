//! Integration tests for config persistence and validation
//! Tests full workflow: create config → save → load → verify equality

use poprawiacz_tekstu_rs::config::Config;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_config_roundtrip_persistence() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path();

    let mut config = Config::default();
    config.api_keys.openai = "sk-test-openai-key".to_string();
    config.api_keys.anthropic = "sk-test-anthropic-key".to_string();
    config.api_keys.gemini = "test-gemini-key".to_string();
    config.api_keys.deepseek = "test-deepseek-key".to_string();
    config.settings.auto_startup = true;
    config.settings.highlight_diffs = true;
    config.ai_settings.reasoning_effort = "low".to_string();

    config.save(temp_path).expect("Failed to save config");

    let loaded_config = Config::load(temp_path).expect("Failed to load config");

    assert_eq!(config, loaded_config);
    assert_eq!(loaded_config.api_keys.openai, "sk-test-openai-key");
    assert_eq!(loaded_config.settings.auto_startup, true);
    assert_eq!(loaded_config.ai_settings.reasoning_effort, "low");
}

#[test]
fn test_config_partial_update_persistence() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path();

    let config = Config::default();
    config.save(temp_path).unwrap();

    let mut loaded_config = Config::load(temp_path).unwrap();
    loaded_config.api_keys.openai = "new-key".to_string();
    loaded_config.settings.default_style = "concise".to_string();
    loaded_config.save(temp_path).unwrap();

    let final_config = Config::load(temp_path).unwrap();
    assert_eq!(final_config.api_keys.openai, "new-key");
    assert_eq!(final_config.settings.default_style, "concise");
    assert_eq!(final_config.models.anthropic, "claude-3-7-sonnet-latest");
}

#[test]
fn test_config_malformed_file_handling() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path();

    let malformed_toml = r#"
        [invalid_section]
        random_field = "value"
        [api_keys
        missing_bracket = true
    "#;

    fs::write(temp_path, malformed_toml).unwrap();

    let result = Config::load(temp_path);
    assert!(result.is_err());
}

#[test]
fn test_config_missing_required_fields() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path();

    let incomplete_toml = r#"
        [api_keys]
        OpenAI = "test-key"
    "#;

    fs::write(temp_path, incomplete_toml).unwrap();

    let result = Config::load(temp_path);
    assert!(result.is_err());
}

#[test]
fn test_config_case_sensitive_field_names() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path();

    let toml_with_correct_case = r#"
        [api_keys]
        OpenAI = "key1"
        Anthropic = "key2"
        Gemini = "key3"
        DeepSeek = "key4"

        [models]
        OpenAI = "gpt-5-mini"
        Anthropic = "claude-3-7-sonnet-latest"
        Gemini = "gemini-2.5-flash"
        DeepSeek = "deepseek-chat"

        [settings]
        AutoStartup = false
        DefaultStyle = "normal"
        HighlightDiffs = false

        [ai_settings]
        ReasoningEffort = "high"
        Verbosity = "medium"
    "#;

    fs::write(temp_path, toml_with_correct_case).unwrap();

    let config = Config::load(temp_path).expect("Should load with correct case");
    assert_eq!(config.api_keys.openai, "key1");
    assert_eq!(config.api_keys.anthropic, "key2");
}

#[test]
fn test_config_empty_api_keys_allowed() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path();

    let config = Config::default();
    config.save(temp_path).unwrap();

    let loaded = Config::load(temp_path).unwrap();
    assert_eq!(loaded.api_keys.openai, "");
    assert_eq!(loaded.api_keys.anthropic, "");
    assert_eq!(loaded.api_keys.gemini, "");
    assert_eq!(loaded.api_keys.deepseek, "");
}

#[test]
fn test_config_unicode_values() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path();

    let mut config = Config::default();
    config.api_keys.openai = "test-🔑-key".to_string();
    config.settings.default_style = "普通".to_string();
    config.save(temp_path).unwrap();

    let loaded = Config::load(temp_path).unwrap();
    assert_eq!(loaded.api_keys.openai, "test-🔑-key");
    assert_eq!(loaded.settings.default_style, "普通");
}

#[test]
fn test_config_very_long_api_keys() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path();

    let long_key = "sk-".to_string() + &"a".repeat(1000);
    let mut config = Config::default();
    config.api_keys.openai = long_key.clone();
    config.save(temp_path).unwrap();

    let loaded = Config::load(temp_path).unwrap();
    assert_eq!(loaded.api_keys.openai, long_key);
    assert_eq!(loaded.api_keys.openai.len(), 1003);
}

#[test]
fn test_config_special_characters_in_values() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path();

    let mut config = Config::default();
    config.api_keys.openai = r#"key"with"quotes"#.to_string();
    config.settings.default_style = "style\nwith\nnewlines".to_string();
    config.save(temp_path).unwrap();

    let loaded = Config::load(temp_path).unwrap();
    assert!(loaded.api_keys.openai.contains("quotes"));
}

#[test]
fn test_config_multiple_save_load_cycles() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path();

    let mut config = Config::default();

    for i in 0..10 {
        config.api_keys.openai = format!("key-{}", i);
        config.save(temp_path).unwrap();

        let loaded = Config::load(temp_path).unwrap();
        assert_eq!(loaded.api_keys.openai, format!("key-{}", i));

        config = loaded;
    }

    let final_config = Config::load(temp_path).unwrap();
    assert_eq!(final_config.api_keys.openai, "key-9");
}

#[test]
fn test_config_concurrent_save_operations() {
    use std::sync::Arc;
    use std::thread;

    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = Arc::new(temp_file.path().to_path_buf());

    let mut handles = vec![];

    for i in 0..5 {
        let path = Arc::clone(&temp_path);
        let handle = thread::spawn(move || {
            let mut config = Config::default();
            config.api_keys.openai = format!("thread-{}", i);
            config.save(path.as_path()).ok()
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let result = Config::load(temp_path.as_path());
    assert!(result.is_ok());
}

#[test]
fn test_config_file_permissions_preserved() {
    use std::os::unix::fs::PermissionsExt;

    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path();

    let config = Config::default();
    config.save(temp_path).unwrap();

    let metadata = fs::metadata(temp_path).unwrap();
    let permissions = metadata.permissions();

    assert!(permissions.mode() & 0o400 != 0);
}

#[test]
fn test_config_default_values_completeness() {
    let config = Config::default();

    assert!(!config.models.openai.is_empty());
    assert!(!config.models.anthropic.is_empty());
    assert!(!config.models.gemini.is_empty());
    assert!(!config.models.deepseek.is_empty());

    assert!(!config.settings.default_style.is_empty());
    assert!(!config.ai_settings.reasoning_effort.is_empty());
    assert!(!config.ai_settings.verbosity.is_empty());
}

#[test]
fn test_config_path_resolution() {
    let path = Config::get_config_path();
    assert!(path.to_str().unwrap().ends_with("config.toml"));
}

#[test]
fn test_config_toml_format_readability() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path();

    let config = Config::default();
    config.save(temp_path).unwrap();

    let content = fs::read_to_string(temp_path).unwrap();

    assert!(content.contains("[api_keys]"));
    assert!(content.contains("[models]"));
    assert!(content.contains("[settings]"));
    assert!(content.contains("[ai_settings]"));

    assert!(content.contains("OpenAI"));
    assert!(content.contains("Anthropic"));
    assert!(content.contains("Gemini"));
    assert!(content.contains("DeepSeek"));
}
