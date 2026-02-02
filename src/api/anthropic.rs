use crate::api::http_client::get_client;
use crate::error::{ApiError, DEFAULT_TIMEOUT};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

#[derive(Debug, Serialize)]
struct MessagesRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    system: String,
    temperature: f32,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    stream: bool,
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

#[derive(Debug, Deserialize)]
struct StreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    delta: Option<StreamDelta>,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    #[serde(rename = "type")]
    delta_type: Option<String>,
    text: Option<String>,
}

pub async fn correct_text_anthropic(
    api_key: &str,
    model: &str,
    text_to_correct: &str,
    instruction_prompt: &str,
    system_prompt: &str,
) -> Result<String, ApiError> {
    correct_text_anthropic_with_callback::<fn(&str)>(
        api_key, model, text_to_correct, instruction_prompt, system_prompt, true, None
    ).await
}

pub async fn correct_text_anthropic_with_callback<F>(
    api_key: &str,
    model: &str,
    text_to_correct: &str,
    instruction_prompt: &str,
    system_prompt: &str,
    streaming: bool,
    on_chunk: Option<F>,
) -> Result<String, ApiError>
where
    F: Fn(&str) + Send + 'static,
{
    if api_key.is_empty() {
        return Err(ApiError::Response("API key is empty".to_string()));
    }
    if model.is_empty() {
        return Err(ApiError::Response("Model is empty".to_string()));
    }
    if text_to_correct.is_empty() {
        return Err(ApiError::Response("Text to correct is empty".to_string()));
    }

    let client = get_client();

    let messages = vec![Message {
        role: "user".to_string(),
        content: format!("{}\n\n---\n{}\n---", instruction_prompt, text_to_correct),
    }];

    let request = MessagesRequest {
        model: model.to_string(),
        messages,
        max_tokens: 4096,
        system: system_prompt.to_string(),
        temperature: 0.7,
        stream: streaming,
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

    if streaming {
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
                    if let Ok(event) = serde_json::from_str::<StreamEvent>(data) {
                        if event.event_type == "content_block_delta" {
                            if let Some(delta) = event.delta {
                                if let Some(text) = delta.text {
                                    collected_text.push_str(&text);
                                    if let Some(ref callback) = on_chunk {
                                        callback(&text);
                                    }
                                }
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
    } else {
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
}
