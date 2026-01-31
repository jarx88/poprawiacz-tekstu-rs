use crate::error::{ApiError, DEFAULT_TIMEOUT, CONNECTION_TIMEOUT};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models";

#[derive(Debug, Serialize)]
struct GenerateContentRequest {
    contents: Vec<Content>,
    generation_config: GenerationConfig,
    safety_settings: Vec<SafetySetting>,
}

#[derive(Debug, Serialize, Clone)]
struct Content {
    role: String,
    parts: Vec<Part>,
}

#[derive(Debug, Serialize, Clone)]
struct Part {
    text: String,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    temperature: f32,
    max_output_tokens: u32,
}

#[derive(Debug, Serialize)]
struct SafetySetting {
    category: String,
    threshold: String,
}

#[derive(Debug, Deserialize)]
struct GenerateContentResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: CandidateContent,
}

#[derive(Debug, Deserialize)]
struct CandidateContent {
    parts: Vec<ContentPart>,
}

#[derive(Debug, Deserialize)]
struct ContentPart {
    text: String,
}

pub async fn correct_text_gemini(
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

    let url = format!(
        "{}/{}:generateContent?key={}",
        GEMINI_API_BASE, model, api_key
    );

    let contents = vec![
        Content {
            role: "user".to_string(),
            parts: vec![
                Part {
                    text: system_prompt.to_string(),
                },
                Part {
                    text: instruction_prompt.to_string(),
                },
                Part {
                    text: text_to_correct.to_string(),
                },
            ],
        },
    ];

    let request = GenerateContentRequest {
        contents,
        generation_config: GenerationConfig {
            temperature: 0.7,
            max_output_tokens: 3072,
        },
        safety_settings: vec![
            SafetySetting {
                category: "HARM_CATEGORY_HARASSMENT".to_string(),
                threshold: "BLOCK_NONE".to_string(),
            },
            SafetySetting {
                category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
                threshold: "BLOCK_NONE".to_string(),
            },
            SafetySetting {
                category: "HARM_CATEGORY_SEXUALLY_EXPLICIT".to_string(),
                threshold: "BLOCK_NONE".to_string(),
            },
            SafetySetting {
                category: "HARM_CATEGORY_DANGEROUS_CONTENT".to_string(),
                threshold: "BLOCK_NONE".to_string(),
            },
        ],
    };

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

    let completion: GenerateContentResponse = response.json().await.map_err(|e| {
        ApiError::Response(format!("Failed to parse response: {}", e))
    })?;

    completion
        .candidates
        .first()
        .and_then(|candidate| candidate.content.parts.first())
        .map(|part| part.text.trim().to_string())
        .ok_or_else(|| ApiError::Response("No text content in response".to_string()))
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
        match result.unwrap_err() {
            ApiError::Response(msg) => assert_eq!(msg, "API key is empty"),
            _ => panic!("Expected Response error"),
        }
    }

    #[tokio::test]
    async fn test_gemini_empty_model() {
        let result = correct_text_gemini(
            "test-key",
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
    async fn test_gemini_empty_text() {
        let result = correct_text_gemini(
            "test-key",
            "gemini-2.5-flash",
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
    async fn test_gemini_invalid_api_key() {
        let result = correct_text_gemini(
            "invalid-key",
            "gemini-2.5-flash",
            "test text",
            "Correct this",
            "You are a helpful assistant",
        )
        .await;

        assert!(result.is_err());
    }
}
