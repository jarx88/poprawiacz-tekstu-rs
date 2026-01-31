use poprawiacz_tekstu_rs::config::Config;
use poprawiacz_tekstu_rs::api::openai::correct_text_openai;
use poprawiacz_tekstu_rs::api::Provider;
use poprawiacz_tekstu_rs::error::{ApiError, DEFAULT_TIMEOUT, CONNECTION_TIMEOUT, DEEPSEEK_TIMEOUT};
use tempfile::NamedTempFile;

#[test]
fn test_full_config_to_api_workflow_structure() {
    let config = Config::default();

    assert!(!config.models.openai.is_empty());
    assert!(!config.models.anthropic.is_empty());
    assert!(!config.models.gemini.is_empty());
    assert!(!config.models.deepseek.is_empty());

    let api_key = &config.api_keys.openai;
    let model = &config.models.openai;

    assert_eq!(api_key, "");
    assert_eq!(model, "gpt-5-mini");
}

#[tokio::test]
async fn test_workflow_config_load_and_api_call() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path();

    let mut config = Config::default();
    config.api_keys.openai = "sk-test-key".to_string();
    config.models.openai = "gpt-4".to_string();
    config.save(temp_path).unwrap();

    let loaded_config = Config::load(temp_path).unwrap();

    let result = correct_text_openai(
        &loaded_config.api_keys.openai,
        &loaded_config.models.openai,
        "Test sentence",
        "Correct this",
        "You are helpful",
        false,
    )
    .await;

    assert!(result.is_err());
}

#[test]
fn test_provider_enum_matches_config_structure() {
    assert_eq!(Provider::OpenAI.name(), "OpenAI");
    assert_eq!(Provider::Anthropic.name(), "Anthropic");
    assert_eq!(Provider::Gemini.name(), "Gemini");
    assert_eq!(Provider::DeepSeek.name(), "DeepSeek");
}

#[tokio::test]
async fn test_all_providers_with_config_models() {
    let config = Config::default();

    let providers = [
        (Provider::OpenAI, &config.models.openai, &config.api_keys.openai),
        (Provider::Anthropic, &config.models.anthropic, &config.api_keys.anthropic),
        (Provider::Gemini, &config.models.gemini, &config.api_keys.gemini),
        (Provider::DeepSeek, &config.models.deepseek, &config.api_keys.deepseek),
    ];

    for (provider, model, key) in providers {
        assert!(!model.is_empty(), "Model for {} should not be empty", provider.name());
        assert_eq!(key, "", "Default API key for {} should be empty", provider.name());
    }
}

#[test]
fn test_timeout_constants_consistency() {
    assert!(DEFAULT_TIMEOUT > CONNECTION_TIMEOUT);
    assert!(DEEPSEEK_TIMEOUT > DEFAULT_TIMEOUT);
    
    assert_eq!(DEFAULT_TIMEOUT, 25);
    assert_eq!(CONNECTION_TIMEOUT, 8);
    assert_eq!(DEEPSEEK_TIMEOUT, 35);
}

#[test]
fn test_config_settings_affect_workflow() {
    let mut config1 = Config::default();
    config1.settings.highlight_diffs = false;
    config1.ai_settings.reasoning_effort = "low".to_string();

    let mut config2 = Config::default();
    config2.settings.highlight_diffs = true;
    config2.ai_settings.reasoning_effort = "high".to_string();

    assert_ne!(config1.settings.highlight_diffs, config2.settings.highlight_diffs);
    assert_ne!(config1.ai_settings.reasoning_effort, config2.ai_settings.reasoning_effort);
}

#[tokio::test]
async fn test_workflow_invalid_config_handling() {
    let config = Config::default();

    let result = correct_text_openai(
        &config.api_keys.openai,
        &config.models.openai,
        "",
        "Instruction",
        "System prompt",
        false,
    )
    .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ApiError::Response(_)));
}

#[test]
fn test_config_modification_workflow() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path();

    let config = Config::default();
    config.save(temp_path).unwrap();

    let mut loaded = Config::load(temp_path).unwrap();
    loaded.api_keys.openai = "new-key".to_string();
    loaded.settings.auto_startup = true;
    loaded.save(temp_path).unwrap();

    let final_config = Config::load(temp_path).unwrap();
    assert_eq!(final_config.api_keys.openai, "new-key");
    assert_eq!(final_config.settings.auto_startup, true);
}

#[test]
fn test_multiple_providers_from_single_config() {
    let mut config = Config::default();
    config.api_keys.openai = "openai-key".to_string();
    config.api_keys.anthropic = "anthropic-key".to_string();
    config.api_keys.gemini = "gemini-key".to_string();
    config.api_keys.deepseek = "deepseek-key".to_string();

    let keys = vec![
        &config.api_keys.openai,
        &config.api_keys.anthropic,
        &config.api_keys.gemini,
        &config.api_keys.deepseek,
    ];

    for (i, key) in keys.iter().enumerate() {
        assert!(!key.is_empty(), "Key {} should not be empty", i);
    }
}

#[tokio::test]
async fn test_workflow_with_streaming_enabled() {
    let config = Config::default();

    let result = correct_text_openai(
        "sk-test",
        &config.models.openai,
        "Test text",
        "Fix this",
        "You are helpful",
        true,
    )
    .await;

    assert!(result.is_err());
}

#[test]
fn test_ai_settings_from_config() {
    let config = Config::default();

    assert_eq!(config.ai_settings.reasoning_effort, "high");
    assert_eq!(config.ai_settings.verbosity, "medium");

    let valid_reasoning_efforts = ["low", "medium", "high"];
    let valid_verbosities = ["low", "medium", "high"];

    assert!(
        valid_reasoning_efforts.contains(&config.ai_settings.reasoning_effort.as_str()),
        "Default reasoning effort should be valid"
    );

    assert!(
        valid_verbosities.contains(&config.ai_settings.verbosity.as_str()),
        "Default verbosity should be valid"
    );
}

#[test]
fn test_default_style_from_config() {
    let config = Config::default();
    assert_eq!(config.settings.default_style, "normal");

    let valid_styles = ["normal", "concise", "detailed"];
    assert!(
        valid_styles.contains(&config.settings.default_style.as_str()),
        "Default style should be valid"
    );
}

#[tokio::test]
async fn test_error_propagation_through_workflow() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path();

    let mut config = Config::default();
    config.api_keys.openai = "".to_string();
    config.save(temp_path).unwrap();

    let loaded = Config::load(temp_path).unwrap();

    let result = correct_text_openai(
        &loaded.api_keys.openai,
        &loaded.models.openai,
        "Test",
        "Fix",
        "System",
        false,
    )
    .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError::Response(msg) => assert_eq!(msg, "API key is empty"),
        _ => panic!("Expected Response error for empty API key"),
    }
}

#[test]
fn test_config_clone_and_modify() {
    let original = Config::default();
    let mut cloned = original.clone();

    cloned.api_keys.openai = "modified".to_string();

    assert_eq!(original.api_keys.openai, "");
    assert_eq!(cloned.api_keys.openai, "modified");
}

#[test]
fn test_config_equality_check() {
    let config1 = Config::default();
    let config2 = Config::default();

    assert_eq!(config1, config2);

    let mut config3 = Config::default();
    config3.api_keys.openai = "different".to_string();

    assert_ne!(config1, config3);
}
