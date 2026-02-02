use crate::api::http_client::{get_client, get_streaming_client};
use crate::error::{ApiError, DEFAULT_TIMEOUT};
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models";

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<SystemInstruction>,
    #[serde(rename = "generationConfig")]
    generation_config: GenerationConfig,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    #[serde(rename = "thinkingConfig")]
    thinking_config: ThinkingConfig,
}

#[derive(Debug, Serialize)]
struct ThinkingConfig {
    #[serde(rename = "thinkingBudget")]
    thinking_budget: i32,
}

#[derive(Debug, Serialize)]
struct SystemInstruction {
    parts: Vec<TextPart>,
}

#[derive(Debug, Serialize)]
struct GeminiContent {
    role: String,
    parts: Vec<TextPart>,
}

#[derive(Debug, Serialize)]
struct TextPart {
    text: String,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: Option<CandidateContent>,
}

#[derive(Debug, Deserialize)]
struct CandidateContent {
    parts: Option<Vec<ContentPart>>,
}

#[derive(Debug, Deserialize)]
struct ContentPart {
    text: Option<String>,
}

pub async fn correct_text_gemini(
    api_key: &str,
    model: &str,
    text_to_correct: &str,
    instruction_prompt: &str,
    system_prompt: &str,
) -> Result<String, ApiError> {
    correct_text_gemini_with_callback::<fn(&str)>(
        api_key, model, text_to_correct, instruction_prompt, system_prompt, true, None
    ).await
}

pub async fn correct_text_gemini_with_callback<F>(
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

    let client = if streaming { get_streaming_client() } else { get_client() };

    let user_content = format!("{}\n\n---\n{}\n---", instruction_prompt, text_to_correct);

    let request = GeminiRequest {
        contents: vec![GeminiContent {
            role: "user".to_string(),
            parts: vec![TextPart { text: user_content }],
        }],
        system_instruction: Some(SystemInstruction {
            parts: vec![TextPart { text: system_prompt.to_string() }],
        }),
        generation_config: GenerationConfig {
            thinking_config: ThinkingConfig {
                thinking_budget: 0,
            },
        },
    };

    if streaming {
        stream_gemini_request_with_callback(client, api_key, model, request, on_chunk).await
    } else {
        batch_gemini_request(client, api_key, model, request).await
    }
}

async fn batch_gemini_request(
    client: &Client,
    api_key: &str,
    model: &str,
    request: GeminiRequest,
) -> Result<String, ApiError> {
    let url = format!("{}/{}:generateContent?key={}", GEMINI_API_BASE, model, api_key);

    let response = client
        .post(&url)
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

    let completion: GeminiResponse = response.json().await.map_err(|e| {
        ApiError::Response(format!("Failed to parse response: {}", e))
    })?;

    completion
        .candidates
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.content)
        .and_then(|c| c.parts)
        .and_then(|p| p.into_iter().next())
        .and_then(|p| p.text)
        .map(|t| t.trim().to_string())
        .ok_or_else(|| ApiError::Response("No text content in response".to_string()))
}

async fn stream_gemini_request_with_callback<F>(
    client: &Client,
    api_key: &str,
    model: &str,
    request: GeminiRequest,
    on_chunk: Option<F>,
) -> Result<String, ApiError>
where
    F: Fn(&str) + Send + 'static,
{
    let url = format!("{}/{}:streamGenerateContent?alt=sse&key={}", GEMINI_API_BASE, model, api_key);

    let response = client
        .post(&url)
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

                if let Ok(resp) = serde_json::from_str::<GeminiResponse>(data) {
                    if let Some(candidates) = resp.candidates {
                        if let Some(candidate) = candidates.first() {
                            if let Some(content) = &candidate.content {
                                if let Some(parts) = &content.parts {
                                    for part in parts {
                                        if let Some(text) = &part.text {
                                            if !text.is_empty() {
                                                collected_text.push_str(text);
                                                if let Some(ref callback) = on_chunk {
                                                    callback(text);
                                                }
                                            }
                                        }
                                    }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gemini_empty_api_key() {
        let result = correct_text_gemini(
            "",
            "gemini-2.5-flash",
            "test text",
            "Correct this",
            "You are a helpful assistant",
        )
        .await;

        assert!(result.is_err());
    }
}
