use crate::error::{ApiError, DEEPSEEK_TIMEOUT, CONNECTION_TIMEOUT};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEEPSEEK_API_URL: &str = "https://api.deepseek.com/chat/completions";

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Debug, Deserialize)]
struct MessageContent {
    content: String,
}

pub async fn correct_text_deepseek(
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
        .timeout(Duration::from_secs(DEEPSEEK_TIMEOUT))
        .connect_timeout(Duration::from_secs(CONNECTION_TIMEOUT))
        .build()
        .map_err(|e| ApiError::Connection(e.to_string()))?;

    let messages = vec![
        Message {
            role: "system".to_string(),
            content: system_prompt.to_string(),
        },
        Message {
            role: "user".to_string(),
            content: format!("{}\n\n---\n{}\n---", instruction_prompt, text_to_correct),
        },
    ];

    let request = ChatCompletionRequest {
        model: model.to_string(),
        messages,
        temperature: 0.7,
        max_tokens: 2000,
    };

    let response = client
        .post(DEEPSEEK_API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                ApiError::Timeout(format!("Request timed out after {}s", DEEPSEEK_TIMEOUT))
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

    let completion: ChatCompletionResponse = response.json().await.map_err(|e| {
        ApiError::Response(format!("Failed to parse response: {}", e))
    })?;

    completion
        .choices
        .first()
        .map(|choice| choice.message.content.trim().to_string())
        .ok_or_else(|| ApiError::Response("No choices in response".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deepseek_empty_api_key() {
        let result = correct_text_deepseek(
            "",
            "deepseek-chat",
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
    async fn test_deepseek_empty_model() {
        let result = correct_text_deepseek(
            "sk-test",
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
    async fn test_deepseek_empty_text() {
        let result = correct_text_deepseek(
            "sk-test",
            "deepseek-chat",
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
    async fn test_deepseek_invalid_api_key() {
        let result = correct_text_deepseek(
            "sk-invalid",
            "deepseek-chat",
            "test text",
            "Correct this",
            "You are a helpful assistant",
        )
        .await;

        assert!(result.is_err());
    }

    #[test]
    fn test_deepseek_uses_correct_timeout() {
        assert_eq!(DEEPSEEK_TIMEOUT, 35);
    }
}
