use std::usize;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{app::AppState, ui_state::AddState};

use super::{DeleteState, UiState};

#[derive(Debug)]
pub struct ListState {
    pub position: usize,
}

impl ListState {
    pub fn new(position: usize) -> Self {
        ListState { position }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent, app_state: &mut AppState) -> Option<UiState> {
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('h')) => self.toggle_hide_completed(app_state),
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c')) => Some(UiState::Quit),
            (_, KeyCode::Up | KeyCode::Char('k')) => self.move_position(true, app_state),
            (_, KeyCode::Down | KeyCode::Char('j')) => self.move_position(false, app_state),
            (_, KeyCode::Char(' ')) => self.handle_interact(app_state),
            (_, KeyCode::Char('d')) if self.position != app_state.todos.len() => {
                Some(UiState::Delete(DeleteState::new(self.position)))
            }
            _ => None,
        }
    }

    fn toggle_hide_completed(&mut self, app_state: &mut AppState) -> Option<UiState> {
        app_state.hide_completed = !app_state.hide_completed;
        if app_state.hide_completed
            && app_state
                .todos
                .get(self.position)
                .map_or(false, |todo| todo.completed)
        {
            self.position = app_state.get_next_position(false, self.position);
        }
        None
    }

    fn move_position(&mut self, up: bool, app_state: &AppState) -> Option<UiState> {
        self.position = app_state.get_next_position(up, self.position);
        None
    }

    fn handle_interact(&mut self, app_state: &mut AppState) -> Option<UiState> {
        if self.position < app_state.todos.len() {
            app_state.toggle_todo(self.position);
            if app_state.hide_completed {
                self.move_position(false, app_state);
            }
            None
        } else {
            Some(UiState::Add(AddState::new(String::new())))
        }
    }
}
