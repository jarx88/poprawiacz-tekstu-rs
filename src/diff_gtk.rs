use gtk4::prelude::*;
use gtk4::TextBuffer;
use regex::Regex;
use similar::{DiffTag, TextDiff};
use std::sync::LazyLock;

static WORD_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\S+").unwrap());

pub fn set_text_with_diff(buffer: &TextBuffer, original: &str, corrected: &str, highlight: bool) {
    buffer.set_text(corrected);

    if highlight && !original.trim().is_empty() && !corrected.trim().is_empty() {
        apply_diff_highlighting(buffer, original, corrected);
    }
}

fn apply_diff_highlighting(buffer: &TextBuffer, original: &str, corrected: &str) {
    let tag_table = buffer.tag_table();

    if tag_table.lookup("diff_highlight").is_none() {
        let tag = gtk4::TextTag::builder()
            .name("diff_highlight")
            .foreground("#d93025")
            .underline(gtk4::pango::Underline::Single)
            .build();
        tag_table.add(&tag);
    }

    let orig_tokens: Vec<&str> = WORD_PATTERN
        .find_iter(original)
        .map(|m| m.as_str())
        .collect();

    let corr_matches: Vec<_> = WORD_PATTERN.find_iter(corrected).collect();
    if corr_matches.is_empty() {
        return;
    }

    let corr_tokens: Vec<&str> = corr_matches.iter().map(|m| m.as_str()).collect();

    let diff = TextDiff::from_slices(&orig_tokens, &corr_tokens);

    for op in diff.ops() {
        match op.tag() {
            DiffTag::Replace | DiffTag::Insert => {
                let j1 = op.new_range().start;
                let j2 = op.new_range().end;

                if j1 >= corr_matches.len() || j1 == j2 {
                    continue;
                }

                let end_index = (j2 - 1).min(corr_matches.len() - 1);
                let start_char = corr_matches[j1].start() as i32;
                let end_char = corr_matches[end_index].end() as i32;

                let start_iter = buffer.iter_at_offset(start_char);
                let end_iter = buffer.iter_at_offset(end_char);
                buffer.apply_tag_by_name("diff_highlight", &start_iter, &end_iter);
            }
            _ => {}
        }
    }
}
