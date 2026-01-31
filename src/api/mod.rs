pub mod openai;
pub mod anthropic;
pub mod gemini;
pub mod deepseek;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    OpenAI,
    Anthropic,
    Gemini,
    DeepSeek,
}

impl Provider {
    pub fn name(&self) -> &'static str {
        match self {
            Provider::OpenAI => "OpenAI",
            Provider::Anthropic => "Anthropic",
            Provider::Gemini => "Gemini",
            Provider::DeepSeek => "DeepSeek",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_names() {
        assert_eq!(Provider::OpenAI.name(), "OpenAI");
        assert_eq!(Provider::Anthropic.name(), "Anthropic");
        assert_eq!(Provider::Gemini.name(), "Gemini");
        assert_eq!(Provider::DeepSeek.name(), "DeepSeek");
    }
}
