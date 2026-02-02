//! System promptów dla różnych stylów korekty tekstu
//!
//! Port z Python: utils/prompts.py
//! Obsługuje 7 różnych stylów: normal, professional, translate_en, translate_pl,
//! change_meaning, summary, prompt

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Style korekty tekstu
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CorrectionStyle {
    /// Standardowa korekta gramatyczna i ortograficzna
    Normal,
    /// Profesjonalny, formalny ton
    Professional,
    /// Tłumaczenie na angielski
    TranslateEn,
    /// Tłumaczenie na polski
    TranslatePl,
    /// Zmiana znaczenia tekstu
    ChangeMeaning,
    /// Podsumowanie tekstu
    Summary,
    /// Przekształcenie w prompt/instrukcję
    Prompt,
}

impl CorrectionStyle {
    /// Parsuje string do CorrectionStyle
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "professional" => Self::Professional,
            "translate_en" => Self::TranslateEn,
            "translate_pl" => Self::TranslatePl,
            "change_meaning" => Self::ChangeMeaning,
            "summary" => Self::Summary,
            "prompt" => Self::Prompt,
            _ => Self::Normal,
        }
    }

    /// Zwraca wszystkie dostępne style
    pub fn all() -> &'static [CorrectionStyle] {
        &[
            Self::Normal,
            Self::Professional,
            Self::TranslateEn,
            Self::TranslatePl,
            Self::ChangeMeaning,
            Self::Summary,
            Self::Prompt,
        ]
    }

    /// Zwraca opis stylu po polsku (dla UI)
    pub fn display_name_pl(&self) -> &'static str {
        match self {
            Self::Normal => "Standardowa korekta",
            Self::Professional => "Zmień na profesjonalny ton",
            Self::TranslateEn => "Przetłumacz na angielski",
            Self::TranslatePl => "Przetłumacz na polski",
            Self::ChangeMeaning => "Zmień znaczenie",
            Self::Summary => "Podsumowanie",
            Self::Prompt => "Przekształć w instrukcję",
        }
    }

    /// Zwraca emoji dla stylu (dla UI)
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Normal => "✏️",
            Self::Professional => "✨",
            Self::TranslateEn => "🇺🇸",
            Self::TranslatePl => "🇵🇱",
            Self::ChangeMeaning => "🔄",
            Self::Summary => "📝",
            Self::Prompt => "💡",
        }
    }
}

/// Instrukcje dla różnych stylów korekty
static INSTRUCTIONS: Lazy<HashMap<CorrectionStyle, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();

    m.insert(CorrectionStyle::Normal, 
        "Correct the following text, preserving its formatting (including all enters and paragraphs). \
        Return ONLY the corrected text, without any additional headers, separators, or comments.");

    m.insert(
        CorrectionStyle::Professional,
        "Rewrite the following text into a professional, formal register. \
        Preserve the original meaning and formatting (paragraphs, lists, line breaks). \
        Always adjust tone to business/professional Polish: \
        - remove colloquialisms, emojis, exclamation-heavy rhetoric \
        - prefer neutral/impersonal or formal address (Państwo / trzecia osoba) \
        - replace casual verbs and particles with precise, formal equivalents \
        - standardize punctuation and capitalization \
        - ensure clear, concise, and courteous phrasing \
        IMPORTANT: Do not return the input unchanged; refine it to a consistently formal style.",
    );

    m.insert(
        CorrectionStyle::TranslateEn,
        "YOUR SOLE TASK IS TO TRANSLATE THE FOLLOWING TEXT INTO ENGLISH. \
        Preserve the original formatting (paragraphs, lists, etc.). \
        Do not correct the text, only translate it.",
    );

    m.insert(
        CorrectionStyle::TranslatePl,
        "YOUR SOLE TASK IS TO TRANSLATE THE FOLLOWING TEXT INTO POLISH. \
        Preserve the original formatting (paragraphs, lists, etc.). \
        Do not correct the text, only translate it.",
    );

    m.insert(
        CorrectionStyle::ChangeMeaning,
        "Propose a completely new text based on the one below, preserving the formatting.",
    );

    m.insert(
        CorrectionStyle::Summary,
        "Create a concise summary of the main points from the following text, \
        preserving the formatting of lists, etc.",
    );

    m.insert(CorrectionStyle::Prompt,
        "Transform the following text into a clear, concise instruction for immediate implementation. \
        The output should be a direct, actionable command or request without explanations, examples, \
        or additional context. If the text is a request or command, convert it into a straightforward \
        instruction as if speaking to an assistant who will execute it immediately. Do not add any \
        introductory phrases, just provide the instruction itself. If the text is already a clear \
        instruction, return it as is. Focus on maintaining the original intent while making it as \
        direct and actionable as possible.");

    m
});

/// Standardowy system prompt dla korekty tekstu
pub const SYSTEM_PROMPT: &str = r#"You are a virtual editor. Your primary specialization is proofreading technical texts for the IT industry, transforming them into correct, clear, and professional-sounding Polish. The input text will typically be in Polish, unless a specific translation task is requested. Follow these instructions meticulously:
1. **Error Correction (for Polish text)**: Detect and correct ALL spelling, grammatical, punctuation, and stylistic errors. Focus on precision and compliance with Polish language standards.
2. **Clarity and Conciseness**: Simplify complex sentences while preserving their technical meaning. Aim for clear and precise communication. Eliminate redundant words and repetitions.
3. **IT Terminology**: Preserve original technical terms, proper names, acronyms, and code snippets, unless they contain obvious spelling mistakes. Do not change their meaning.
4. **Professional Tone**: Give the text a professional yet natural tone. Avoid colloquialisms, but also excessive formality.
5. **Formatting**: Strictly preserve the original text formatting: paragraphs, bulleted/numbered lists, indentations, bolding (if Markdown was used), and line breaks. This is crucial for all tasks, including translation.
6. **Output Content**: As the result, return ONLY the final processed text. DO NOT include any additional comments, headers, explanations, or separators like "---" or "```".
7. **Strict Formatting Rules**:
   - Never start or end the response with any separator characters like ---, ===, ```, or any other decorative elements
   - Do not add any closing remarks like "Let me know if you need anything else"
   - Do not include any text that wasn't in the original input unless it's a necessary correction
   - If the input is empty, return an empty string

If the task is a translation, the output should be only the translated text. If the task is correction, the output should be only the corrected Polish text."#;

/// System prompt dla profesjonalnego tonu
pub const PROFESSIONAL_SYSTEM_PROMPT: &str = r#"You are a senior Polish-language editor specializing in transforming texts into a consistent, formal, business-appropriate register. Apply the following rules rigorously:
1. Tone: neutral, courteous, and professional; no colloquialisms or emojis.
2. Register: prefer impersonal constructions or formal address (Państwo), avoid second-person singular unless the genre requires it.
3. Clarity: shorter sentences where appropriate; remove filler words; keep the meaning intact.
4. Precision: prefer precise vocabulary; correct punctuation and typography.
5. Formatting: strictly preserve paragraphs, lists, and line breaks.
6. Output: return ONLY the final, professionally restyled Polish text—no comments or markers."#;

/// System prompt dla przekształcania w instrukcje
pub const PROMPT_SYSTEM_PROMPT: &str = r#"You are an AI assistant that transforms user requests into direct, executable commands. Follow these rules:
1. **Be direct**: Convert requests into simple, imperative statements.
2. **No explanations**: Do not include any additional context or notes.
3. **Preserve intent**: Maintain the original meaning while making it actionable.
4. **Single action**: Focus on one clear action per instruction.
5. **Be specific**: Include all necessary details for immediate execution.

IMPORTANT: Return the response in the following format:
1. First line: The instruction in English
2. Empty line
3. Second line: The same instruction translated to Polish (Tłumaczenie: [tłumaczenie])

Example:
Remove the Cancel button
Tłumaczenie: Usuń przycisk Anuluj

Add a new feature
Tłumaczenie: Dodaj nową funkcję"#;

/// Zwraca system prompt dla danego stylu
pub fn get_system_prompt(style: CorrectionStyle) -> &'static str {
    match style {
        CorrectionStyle::Prompt => PROMPT_SYSTEM_PROMPT,
        CorrectionStyle::Professional => PROFESSIONAL_SYSTEM_PROMPT,
        _ => SYSTEM_PROMPT,
    }
}

/// Zwraca instruction prompt dla danego stylu
pub fn get_instruction_prompt(style: CorrectionStyle) -> &'static str {
    INSTRUCTIONS
        .get(&style)
        .copied()
        .unwrap_or(INSTRUCTIONS.get(&CorrectionStyle::Normal).unwrap())
}

/// Buduje pełny prompt do wysłania do API
pub fn build_full_prompt(style: CorrectionStyle, text: &str) -> String {
    format!("{}\n\n{}", get_instruction_prompt(style), text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correction_style_from_str() {
        assert_eq!(CorrectionStyle::from_str("normal"), CorrectionStyle::Normal);
        assert_eq!(
            CorrectionStyle::from_str("professional"),
            CorrectionStyle::Professional
        );
        assert_eq!(
            CorrectionStyle::from_str("translate_en"),
            CorrectionStyle::TranslateEn
        );
        assert_eq!(
            CorrectionStyle::from_str("translate_pl"),
            CorrectionStyle::TranslatePl
        );
        assert_eq!(CorrectionStyle::from_str("NORMAL"), CorrectionStyle::Normal);
        assert_eq!(
            CorrectionStyle::from_str("unknown"),
            CorrectionStyle::Normal
        );
    }

    #[test]
    fn test_instruction_prompts_exist() {
        for style in CorrectionStyle::all() {
            let prompt = get_instruction_prompt(*style);
            assert!(
                !prompt.is_empty(),
                "Prompt for {:?} should not be empty",
                style
            );
        }
    }

    #[test]
    fn test_system_prompts_exist() {
        for style in CorrectionStyle::all() {
            let prompt = get_system_prompt(*style);
            assert!(
                !prompt.is_empty(),
                "System prompt for {:?} should not be empty",
                style
            );
        }
    }

    #[test]
    fn test_build_full_prompt() {
        let prompt = build_full_prompt(CorrectionStyle::Normal, "Test text");
        assert!(prompt.contains("Test text"));
        assert!(prompt.contains("Correct the following text"));
    }

    #[test]
    fn test_display_names() {
        assert_eq!(
            CorrectionStyle::Professional.display_name_pl(),
            "Zmień na profesjonalny ton"
        );
        assert_eq!(
            CorrectionStyle::TranslateEn.display_name_pl(),
            "Przetłumacz na angielski"
        );
    }

    #[test]
    fn test_emojis() {
        assert_eq!(CorrectionStyle::Professional.emoji(), "✨");
        assert_eq!(CorrectionStyle::TranslateEn.emoji(), "🇺🇸");
        assert_eq!(CorrectionStyle::TranslatePl.emoji(), "🇵🇱");
    }
}
