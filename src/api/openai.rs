use crate::error::{ApiError, DEFAULT_TIMEOUT, CONNECTION_TIMEOUT};
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
    stream: bool,
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

#[derive(Debug, Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: Delta,
}

#[derive(Debug, Deserialize)]
struct Delta {
    #[serde(default)]
    content: Option<String>,
}

pub async fn correct_text_openai(
    api_key: &str,
    model: &str,
    text_to_correct: &str,
    instruction_prompt: &str,
    system_prompt: &str,
    streaming: bool,
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
        max_tokens: 2048,
        stream: streaming,
    };

    if streaming {
        stream_openai_request(&client, api_key, request).await
    } else {
        batch_openai_request(&client, api_key, request).await
    }
}

async fn batch_openai_request(
    client: &Client,
    api_key: &str,
    request: ChatCompletionRequest,
) -> Result<String, ApiError> {
    let response = client
        .post(OPENAI_API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
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

    let completion: ChatCompletionResponse = response.json().await.map_err(|e| {
        ApiError::Response(format!("Failed to parse response: {}", e))
    })?;

    completion
        .choices
        .first()
        .map(|choice| choice.message.content.clone())
        .ok_or_else(|| ApiError::Response("No choices in response".to_string()))
}

async fn stream_openai_request(
    client: &Client,
    api_key: &str,
    request: ChatCompletionRequest,
) -> Result<String, ApiError> {
    let response = client
        .post(OPENAI_API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
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

    let mut stream = response.bytes_stream();
    let mut collected_text = String::new();
    let mut buffer = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| ApiError::Response(e.to_string()))?;
        let chunk_str = String::from_utf8_lossy(&chunk);
        buffer.push_str(&chunk_str);

        for line in buffer.lines() {
            if line.starts_with("data: ") {
                let data = &line[6..];
                if data.trim() == "[DONE]" {
                    break;
                }

                if let Ok(chunk_data) = serde_json::from_str::<StreamChunk>(data) {
                    if let Some(choice) = chunk_data.choices.first() {
                        if let Some(content) = &choice.delta.content {
                            collected_text.push_str(content);
                        }
                    }
                }
            }
        }

        buffer.clear();
    }

    if collected_text.is_empty() {
        Err(ApiError::Response("No content in streaming response".to_string()))
    } else {
        Ok(collected_text.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_openai_empty_api_key() {
        let result = correct_text_openai(
            "",
            "gpt-4",
            "test text",
            "Correct this",
            "You are a helpful assistant",
            false,
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::Response(msg) => assert_eq!(msg, "API key is empty"),
            _ => panic!("Expected Response error"),
        }
    }

    #[tokio::test]
    async fn test_openai_empty_model() {
        let result = correct_text_openai(
            "sk-test",
            "",
            "test text",
            "Correct this",
            "You are a helpful assistant",
            false,
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::Response(msg) => assert_eq!(msg, "Model is empty"),
            _ => panic!("Expected Response error"),
        }
    }

    #[tokio::test]
    async fn test_openai_empty_text() {
        let result = correct_text_openai(
            "sk-test",
            "gpt-4",
            "",
            "Correct this",
            "You are a helpful assistant",
            false,
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::Response(msg) => assert_eq!(msg, "Text to correct is empty"),
            _ => panic!("Expected Response error"),
        }
    }

    #[tokio::test]
    async fn test_openai_invalid_api_key() {
        let result = correct_text_openai(
            "sk-invalid",
            "gpt-4",
            "test text",
            "Correct this",
            "You are a helpful assistant",
            false,
        )
        .await;

        assert!(result.is_err());
    }
}
