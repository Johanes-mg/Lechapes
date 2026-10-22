use crate::buffer::FileBuffer;
use std::path::PathBuf;

pub struct Tab {
    pub id: u64,
    pub buffer: FileBuffer,
    pub path: Option<PathBuf>,
    pub dirty: bool,
    pub cursor: usize,
    pub scroll_to: Option<usize>,
}

impl Tab {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            buffer: FileBuffer::empty(),
            path: None,
            dirty: false,
            cursor: 0,
            scroll_to: None,
        }
    }

    pub fn title(&self, untitled: &str) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| untitled.to_string())
    }
}

pub struct TabsState {
    pub tabs: Vec<Tab>,
    pub active: usize,
    next_id: u64,
}

impl TabsState {
    pub fn new() -> Self {
        let mut s = Self {
            tabs: Vec::new(),
            active: 0,
            next_id: 1,
        };
        s.new_tab();
        s
    }

    pub fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn new_tab(&mut self) {
        let id = self.alloc_id();
        self.tabs.push(Tab::new(id));
        self.active = self.tabs.len() - 1;
    }

    pub fn close_tab(&mut self, index: usize) {
        if index >= self.tabs.len() {
            return;
        }
        self.tabs.remove(index);
        if self.tabs.is_empty() {
            self.new_tab();
        } else if self.active >= self.tabs.len() {
            self.active = self.tabs.len() - 1;
        } else if self.active > index {
            self.active -= 1;
        }
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active)
    }

    pub fn any_dirty(&self) -> bool {
        self.tabs.iter().any(|t| t.dirty)
    }
}
