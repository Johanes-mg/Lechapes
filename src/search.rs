#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Find,
    Replace,
}

#[derive(Default)]
pub struct SearchState {
    pub open: bool,
    pub mode: SearchMode,
    pub query: String,
    pub replacement: String,
    pub case_sensitive: bool,
    pub match_index: usize,
}

impl Default for SearchMode {
    fn default() -> Self {
        SearchMode::Find
    }
}

pub struct Match {
    pub start: usize,
    pub end: usize,
}

pub fn find_all(haystack: &str, needle: &str, case_sensitive: bool) -> Vec<Match> {
    if needle.is_empty() {
        return Vec::new();
    }
    let (h, n) = if case_sensitive {
        (haystack.to_string(), needle.to_string())
    } else {
        (haystack.to_lowercase(), needle.to_lowercase())
    };
    let mut out = Vec::new();
    let mut start = 0;
    while let Some(pos) = h[start..].find(&n) {
        let abs = start + pos;
        out.push(Match {
            start: abs,
            end: abs + n.len(),
        });
        start = abs + n.len().max(1);
    }
    out
}
