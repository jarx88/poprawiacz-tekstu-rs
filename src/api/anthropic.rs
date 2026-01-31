use crate::error::{ApiError, DEFAULT_TIMEOUT, CONNECTION_TIMEOUT};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

#[derive(Debug, Serialize)]
struct MessagesRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    system: String,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
}

pub async fn correct_text_anthropic(
    api_key: &str,
    model: &str,
    text_to_correct: &str,
    instruction_prompt: &str,
    system_prompt: &str,
) -> Result<String, ApiError> {
    if api_key.is_empty() {
        return Err(ApiError::Response("API key is empty".to_string()));
    }
    if model.is_empty() {
        return Err(ApiError::Response("Model is empty".to_string()));
    }
    if text_to_correct.is_empty() {
        return Err(ApiError::Response("Text to correct is empty".to_string()));
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT))
        .connect_timeout(Duration::from_secs(CONNECTION_TIMEOUT))
        .build()
        .map_err(|e| ApiError::Connection(e.to_string()))?;

    let messages = vec![Message {
        role: "user".to_string(),
        content: format!("{}\n\n---\n{}\n---", instruction_prompt, text_to_correct),
    }];

    let request = MessagesRequest {
        model: model.to_string(),
        messages,
        max_tokens: 2048,
        system: system_prompt.to_string(),
        temperature: 0.7,
    };

    let response = client
        .post(ANTHROPIC_API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                ApiError::Timeout(format!("Request timed out after {}s", DEFAULT_TIMEOUT))
            } else if e.is_connect() {
                ApiError::Connection(e.to_string())
            } else {
                ApiError::Response(e.to_string())
            }
        })?;

    if !response.status().is_success() {
        return Err(ApiError::Response(format!(
            "HTTP {}: {}",
            response.status(),
            response.text().await.unwrap_or_default()
        )));
    }

    let completion: MessagesResponse = response.json().await.map_err(|e| {
        ApiError::Response(format!("Failed to parse response: {}", e))
    })?;

    completion
        .content
        .into_iter()
        .find_map(|block| match block {
            ContentBlock::Text { text } => Some(text),
        })
        .ok_or_else(|| ApiError::Response("No text content in response".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_anthropic_empty_api_key() {
        let result = correct_text_anthropic(
            "",
            "claude-3-sonnet",
            "test text",
            "Correct this",
            "You are a helpful assistant",
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::Response(msg) => assert_eq!(msg, "API key is empty"),
            _ => panic!("Expected Response error"),
        }
    }

    #[tokio::test]
    async fn test_anthropic_empty_model() {
        let result = correct_text_anthropic(
            "sk-ant-test",
            "",
            "test text",
            "Correct this",
            "You are a helpful assistant",
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::Response(msg) => assert_eq!(msg, "Model is empty"),
            _ => panic!("Expected Response error"),
        }
    }

    #[tokio::test]
    async fn test_anthropic_empty_text() {
        let result = correct_text_anthropic(
            "sk-ant-test",
            "claude-3-sonnet",
            "",
            "Correct this",
            "You are a helpful assistant",
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::Response(msg) => assert_eq!(msg, "Text to correct is empty"),
            _ => panic!("Expected Response error"),
        }
    }

    #[tokio::test]
    async fn test_anthropic_invalid_api_key() {
        let result = correct_text_anthropic(
            "sk-ant-invalid",
            "claude-3-sonnet",
            "test text",
            "Correct this",
            "You are a helpful assistant",
        )
        .await;

        assert!(result.is_err());
    }
}
