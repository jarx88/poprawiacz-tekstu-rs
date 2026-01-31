use poprawiacz_tekstu_rs::api::openai::correct_text_openai;
use poprawiacz_tekstu_rs::api::anthropic::correct_text_anthropic;
use poprawiacz_tekstu_rs::api::gemini::correct_text_gemini;
use poprawiacz_tekstu_rs::api::deepseek::correct_text_deepseek;
use poprawiacz_tekstu_rs::error::ApiError;

#[tokio::test]
async fn test_openai_empty_inputs_error_handling() {
    let result = correct_text_openai(
        "",
        "gpt-4",
        "test text",
        "Correct this",
        "You are helpful",
        false,
    )
    .await;

    assert!(matches!(result, Err(ApiError::Response(_))));
    if let Err(ApiError::Response(msg)) = result {
        assert_eq!(msg, "API key is empty");
    }
}

#[tokio::test]
async fn test_openai_empty_model_error() {
    let result = correct_text_openai(
        "sk-test",
        "",
        "test text",
        "Correct this",
        "You are helpful",
        false,
    )
    .await;

    assert!(matches!(result, Err(ApiError::Response(_))));
    if let Err(ApiError::Response(msg)) = result {
        assert_eq!(msg, "Model is empty");
    }
}

#[tokio::test]
async fn test_openai_empty_text_error() {
    let result = correct_text_openai(
        "sk-test",
        "gpt-4",
        "",
        "Correct this",
        "You are helpful",
        false,
    )
    .await;

    assert!(matches!(result, Err(ApiError::Response(_))));
    if let Err(ApiError::Response(msg)) = result {
        assert_eq!(msg, "Text to correct is empty");
    }
}

#[tokio::test]
async fn test_anthropic_empty_api_key_error() {
    let result = correct_text_anthropic(
        "",
        "claude-3-7-sonnet-latest",
        "test text",
        "Correct this",
        "You are helpful",
    )
    .await;

    assert!(matches!(result, Err(ApiError::Response(_))));
}

#[tokio::test]
async fn test_gemini_empty_api_key_error() {
    let result = correct_text_gemini(
        "",
        "gemini-2.5-flash",
        "test text",
        "Correct this",
        "You are helpful",
    )
    .await;

    assert!(matches!(result, Err(ApiError::Response(_))));
}

#[tokio::test]
async fn test_deepseek_empty_api_key_error() {
    let result = correct_text_deepseek(
        "",
        "deepseek-chat",
        "test text",
        "Correct this",
        "You are helpful",
    )
    .await;

    assert!(matches!(result, Err(ApiError::Response(_))));
}

#[tokio::test]
async fn test_openai_invalid_key_returns_error() {
    let result = correct_text_openai(
        "sk-invalid-key-12345",
        "gpt-4",
        "Fix this sentence",
        "Correct grammar",
        "You are a grammar assistant",
        false,
    )
    .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError::Connection(_) | ApiError::Response(_) | ApiError::Timeout(_) => {}
    }
}

#[tokio::test]
async fn test_anthropic_invalid_key_returns_error() {
    let result = correct_text_anthropic(
        "sk-ant-invalid",
        "claude-3-7-sonnet-latest",
        "Fix this sentence",
        "Correct grammar",
        "You are helpful",
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_gemini_invalid_key_returns_error() {
    let result = correct_text_gemini(
        "invalid-gemini-key",
        "gemini-2.5-flash",
        "Fix this sentence",
        "Correct grammar",
        "You are helpful",
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_deepseek_invalid_key_returns_error() {
    let result = correct_text_deepseek(
        "invalid-deepseek-key",
        "deepseek-chat",
        "Fix this sentence",
        "Correct grammar",
        "You are helpful",
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_api_error_types_are_distinguishable() {
    let connection_err = ApiError::Connection("Network unreachable".to_string());
    let response_err = ApiError::Response("400 Bad Request".to_string());
    let timeout_err = ApiError::Timeout("Request exceeded 25s".to_string());

    assert!(matches!(connection_err, ApiError::Connection(_)));
    assert!(matches!(response_err, ApiError::Response(_)));
    assert!(matches!(timeout_err, ApiError::Timeout(_)));

    assert!(connection_err.to_string().contains("Connection error"));
    assert!(response_err.to_string().contains("Response error"));
    assert!(timeout_err.to_string().contains("Timeout error"));
}

#[tokio::test]
async fn test_concurrent_api_calls_all_providers() {
    let text = "Test sentence";
    let instruction = "Correct this";
    let system = "You are helpful";

    let openai_future = correct_text_openai("sk-invalid", "gpt-4", text, instruction, system, false);
    let anthropic_future = correct_text_anthropic("sk-invalid", "claude-3-7-sonnet-latest", text, instruction, system);
    let gemini_future = correct_text_gemini("invalid", "gemini-2.5-flash", text, instruction, system);
    let deepseek_future = correct_text_deepseek("invalid", "deepseek-chat", text, instruction, system);

    let results = tokio::join!(
        openai_future,
        anthropic_future,
        gemini_future,
        deepseek_future
    );

    assert!(results.0.is_err());
    assert!(results.1.is_err());
    assert!(results.2.is_err());
    assert!(results.3.is_err());
}

#[tokio::test]
async fn test_streaming_vs_batch_mode_difference() {
    let api_key = "sk-invalid";
    let model = "gpt-4";
    let text = "Test";
    let instruction = "Fix";
    let system = "Assistant";

    let batch_result = correct_text_openai(api_key, model, text, instruction, system, false).await;
    let stream_result = correct_text_openai(api_key, model, text, instruction, system, true).await;

    assert!(batch_result.is_err());
    assert!(stream_result.is_err());
}

#[tokio::test]
async fn test_unicode_text_handling() {
    let text = "Hello 世界 🦀 Привет";
    let instruction = "Check grammar";
    let system = "You are helpful";

    let result = correct_text_openai("sk-test", "gpt-4", text, instruction, system, false).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_very_long_text_input() {
    let long_text = "word ".repeat(1000);
    let instruction = "Summarize";
    let system = "You are helpful";

    let result = correct_text_openai("sk-test", "gpt-4", &long_text, instruction, system, false).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_special_characters_in_prompts() {
    let text = "Test sentence";
    let instruction = r#"Fix "quotes" and 'apostrophes' and \backslashes\"#;
    let system = "You are helpful";

    let result = correct_text_openai("sk-test", "gpt-4", text, instruction, system, false).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_empty_system_prompt_allowed() {
    let result = correct_text_openai(
        "sk-test",
        "gpt-4",
        "Test text",
        "Fix this",
        "",
        false,
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_empty_instruction_allowed() {
    let result = correct_text_openai(
        "sk-test",
        "gpt-4",
        "Test text",
        "",
        "You are helpful",
        false,
    )
    .await;

    assert!(result.is_err());
}
