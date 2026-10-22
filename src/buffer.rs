use encoding_rs::{Encoding, UTF_8};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    Crlf,
}

impl LineEnding {
    #[allow(dead_code)]
    pub fn as_str(self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
        }
    }
}

pub struct FileBuffer {
    pub text: String,
    pub encoding: &'static Encoding,
    pub line_ending: LineEnding,
    pub has_bom: bool,
    undo: Vec<String>,
    redo: Vec<String>,
    pub undo_limit: usize,
}

impl FileBuffer {
    pub fn empty() -> Self {
        Self {
            text: String::new(),
            encoding: UTF_8,
            line_ending: LineEnding::Crlf,
            has_bom: false,
            undo: Vec::new(),
            redo: Vec::new(),
            undo_limit: 256,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let (encoding, bom_len) = detect_encoding(bytes);
        let content = &bytes[bom_len..];
        let (cow, _, _) = encoding.decode(content);
        let mut text = cow.into_owned();
        let line_ending = if text.contains("\r\n") {
            LineEnding::Crlf
        } else {
            LineEnding::Lf
        };
        text = text.replace("\r\n", "\n");
        Self {
            text,
            encoding,
            line_ending,
            has_bom: bom_len > 0,
            undo: Vec::new(),
            redo: Vec::new(),
            undo_limit: 256,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let normalized = if self.line_ending == LineEnding::Crlf {
            self.text.replace('\n', "\r\n")
        } else {
            self.text.clone()
        };
        let (bytes, _, _) = self.encoding.encode(&normalized);
        let mut out = Vec::with_capacity(bytes.len() + 3);
        if self.has_bom {
            let nom = self.encoding.name();
            if self.encoding == UTF_8 {
                out.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
            } else if nom == "UTF-16LE" {
                out.extend_from_slice(&[0xFF, 0xFE]);
            } else if nom == "UTF-16BE" {
                out.extend_from_slice(&[0xFE, 0xFF]);
            }
        }
        out.extend_from_slice(&bytes);
        out
    }

    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.undo.pop() {
            let cur = std::mem::replace(&mut self.text, prev);
            self.redo.push(cur);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.redo.pop() {
            let cur = std::mem::replace(&mut self.text, next);
            self.undo.push(cur);
            true
        } else {
            false
        }
    }

    pub fn push_snapshot_after_change(&mut self, before: String) {
        if before != self.text {
            if self.undo.len() >= self.undo_limit {
                self.undo.remove(0);
            }
            self.undo.push(before);
            self.redo.clear();
        }
    }
}

fn detect_encoding(bytes: &[u8]) -> (&'static Encoding, usize) {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return (UTF_8, 3);
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        if let Some(enc) = Encoding::for_label(b"utf-16le") {
            return (enc, 2);
        }
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        if let Some(enc) = Encoding::for_label(b"utf-16be") {
            return (enc, 2);
        }
    }
    if std::str::from_utf8(bytes).is_ok() {
        return (UTF_8, 0);
    }
    if let Some(enc) = Encoding::for_label(b"windows-1252") {
        return (enc, 0);
    }
    (UTF_8, 0)
}
