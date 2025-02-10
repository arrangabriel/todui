use crossterm::event::{KeyCode, KeyEvent};

use crate::app::AppState;

use super::{ListState, UiState};

#[derive(Debug)]
pub struct DeleteState {
    pub position: usize,
}

impl DeleteState {
    pub fn new(position: usize) -> Self {
        Self { position }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent, app_state: &mut AppState) -> Option<UiState> {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('n')) => {
                Some(UiState::List(ListState::new(self.position)))
            }
            (_, KeyCode::Char('y')) => self.delete_todo(app_state),
            _ => None,
        }
    }

    fn delete_todo(&self, app_state: &mut AppState) -> Option<UiState> {
        app_state.delete_todo(self.position);
        let new_position = app_state.get_next_position(true, self.position);

        Some(UiState::List(ListState::new(new_position)))
    }
}
