use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::todo::Todo;

use super::{ListState, UiState};

#[derive(Debug)]
pub struct EditState {
    pub position: usize,
    pub description: String,
    pub cursor_x: usize,
}

impl EditState {
    pub fn new(position: usize, description: String) -> Self {
        let cursor_x = description.chars().count();
        Self {
            position,
            description,
            cursor_x,
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent, todos: &mut [Todo]) -> Option<UiState> {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc) => Some(UiState::List(ListState::new(self.position))),
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => Some(UiState::Quit),
            (_, KeyCode::Left) => self.move_cursor(-1),
            (_, KeyCode::Right) => self.move_cursor(1),
            (_, KeyCode::Char(c)) => self.insert(c),
            (_, KeyCode::Backspace) => self.backspace(),
            (_, KeyCode::Enter) => self.save(todos),
            _ => None,
        }
    }

    fn move_cursor(&mut self, delta: isize) -> Option<UiState> {
        let len = self.description.chars().count();
        let new = self.cursor_x as isize + delta;
        self.cursor_x = new.clamp(0, len as isize) as usize;
        None
    }

    fn byte_offset(&self, char_idx: usize) -> usize {
        self.description
            .char_indices()
            .nth(char_idx)
            .map(|(b, _)| b)
            .unwrap_or(self.description.len())
    }

    fn insert(&mut self, c: char) -> Option<UiState> {
        let byte = self.byte_offset(self.cursor_x);
        self.description.insert(byte, c);
        self.cursor_x += 1;
        None
    }

    fn backspace(&mut self) -> Option<UiState> {
        if self.cursor_x == 0 {
            return None;
        }
        let byte = self.byte_offset(self.cursor_x - 1);
        self.description.remove(byte);
        self.cursor_x -= 1;
        None
    }

    fn save(&self, todos: &mut [Todo]) -> Option<UiState> {
        let trimmed = self.description.trim();
        if trimmed.is_empty() {
            return None;
        }
        todos[self.position].description = trimmed.to_string();
        Some(UiState::List(ListState::new(self.position)))
    }
}
