use std::ops::Range;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SearchMode {
    #[default]
    Find,
    Replace,
}

pub struct SearchState {
    pub query: String,
    pub replacement: String,
    pub case_sensitive: bool,
    pub match_index: usize,
    pub mode: SearchMode,
    pub open: bool,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            query: String::new(),
            replacement: String::new(),
            case_sensitive: false,
            match_index: usize::MAX,
            mode: SearchMode::Find,
            open: false,
        }
    }
}

pub fn find_all(text: &str, query: &str, case_sensitive: bool) -> Vec<Range<usize>> {
    if query.is_empty() {
        return Vec::new();
    }

    if case_sensitive {
        return text
            .match_indices(query)
            .map(|(start, value)| start..start + value.len())
            .collect();
    }

    let mut folded_text = String::with_capacity(text.len());
    let mut original_offsets = Vec::with_capacity(text.len() + 1);
    original_offsets.push(0);

    for (original_start, character) in text.char_indices() {
        let original_end = original_start + character.len_utf8();
        let folded = character.to_lowercase().to_string();
        for folded_character in folded.chars() {
            let mut encoded = [0; 4];
            let encoded = folded_character.encode_utf8(&mut encoded);
            folded_text.push_str(encoded);
            for _ in 0..encoded.len() {
                original_offsets.push(original_start);
            }
        }
        if let Some(last) = original_offsets.last_mut() {
            *last = original_end;
        }
    }

    let folded_query = query.to_lowercase();
    if folded_query.is_empty() {
        return Vec::new();
    }

    let mut matches = Vec::new();
    let mut search_from = 0;
    while let Some(relative_start) = folded_text[search_from..].find(&folded_query) {
        let folded_start = search_from + relative_start;
        let folded_end = folded_start + folded_query.len();
        if folded_end < original_offsets.len() {
            let start = original_offsets[folded_start];
            let end = original_offsets[folded_end];
            if start < end && text.is_char_boundary(start) && text.is_char_boundary(end) {
                matches.push(start..end);
            }
        }
        search_from = folded_end;
        if search_from >= folded_text.len() {
            break;
        }
    }
    matches
}
