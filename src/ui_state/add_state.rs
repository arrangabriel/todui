use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::todo::Todo;

use super::{ListState, UiState};

#[derive(Debug)]
pub struct AddState {
    pub description: String,
}

impl AddState {
    pub fn new(description: String) -> Self {
        Self { description }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent, todos: &mut Vec<Todo>) -> Option<UiState> {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc) => Some(Self::get_back_state(todos)),
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => Some(UiState::Quit),
            (_, KeyCode::Char(c)) => self.edit_description(Some(c)),
            (_, KeyCode::Backspace) => self.edit_description(None),
            (_, KeyCode::Enter) => self.save_new(todos),
            _ => None,
        }
    }

    fn edit_description(&mut self, char: Option<char>) -> Option<UiState> {
        if let Some(c) = char {
            self.description.push(c);
        } else {
            self.description.pop();
        };
        None
    }

    fn save_new(&self, todos: &mut Vec<Todo>) -> Option<UiState> {
        // TODO: trim whitespace from end
        if !self.description.is_empty() {
            todos.push(Todo::new(&self.description));
            Some(Self::get_back_state(todos))
        } else {
            None
        }
    }

    fn get_back_state(todos: &[Todo]) -> UiState {
        UiState::List(ListState::new(todos.len()))
    }
}
