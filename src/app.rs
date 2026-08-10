use core::panic;
use std::path::PathBuf;
use std::{fs, io};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::Position;
use ratatui::style::Stylize;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Paragraph};
use ratatui::{DefaultTerminal, Frame};

use crate::config;
use crate::todo::Todo;
use crate::ui_state::{ListState, UiState};

#[derive(Debug)]
pub struct App {
    ui_state: UiState,
    state: AppState,
    data_path: PathBuf,
    initial_content: String,
    should_write: bool,
}

#[derive(Debug)]
pub struct AppState {
    pub hide_completed: bool,
    pub todos: Vec<Todo>,
}

impl AppState {
    pub fn toggle_todo(&mut self, list_position: usize) {
        let current = self
            .todos
            .get_mut(list_position)
            .expect("Position should be a valid index");
        current.completed = !current.completed;
    }

    pub fn delete_todo(&mut self, list_position: usize) {
        self.todos.remove(list_position);
    }

    pub fn get_next_position(&self, up: bool, current_list_position: usize) -> usize {
        if (up && current_list_position == 0) || (!up && current_list_position == self.todos.len())
        {
            return current_list_position;
        }
        let mut current_list_position = if up {
            current_list_position - 1
        } else {
            current_list_position + 1
        };

        if self.hide_completed {
            while (1..self.todos.len()).contains(&current_list_position)
                && self.todos[current_list_position].completed
            {
                if up {
                    current_list_position -= 1
                } else {
                    current_list_position += 1
                }
            }

            if let Some(todo) = self.todos.get(current_list_position) {
                if current_list_position == 0 && todo.completed {
                    current_list_position = self.get_next_position(false, current_list_position);
                }
            }
        }

        current_list_position
    }
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let xdg_base = xdg::BaseDirectories::new();

        let config_file_path = if let Ok(path) = std::env::var("TODUI_CONFIG_FILE") {
            PathBuf::from(path)
        } else if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
            let mut xdg_data_path = PathBuf::from(data_home);
            xdg_data_path.push("todui");
            xdg_data_path
        } else {
            let mut config_home = xdg_base
                .get_config_home()
                .ok_or(anyhow::anyhow!("Could not get XDG config home directory"))?;
            config_home.push("todui");
            config_home.push("config.toml");
            config_home
        };
        let config_str = if fs::exists(&config_file_path)? {
            Some(fs::read_to_string(&config_file_path)?)
        } else {
            None
        };
        let _config = config::parse_config(config_str.as_deref())?;

        let mut todo_file_path = if let Ok(path) = std::env::var("TODUI_DIR") {
            PathBuf::from(path)
        } else {
            let mut data_home = xdg_base
                .data_home
                .ok_or(anyhow::anyhow!("Could not get XDG data home directory"))?;
            data_home.push(".todui");
            data_home
        };

        static TODO_FILE: &str = "todo.md";
        todo_file_path.push(TODO_FILE);

        let initial_content = if fs::exists(&todo_file_path)? {
            fs::read_to_string(&todo_file_path)?
        } else {
            fs::create_dir_all(&todo_file_path.parent().unwrap())?;
            String::new()
        };

        let todos: Vec<Todo> = initial_content
            .split('\n')
            .filter_map(Todo::deserialize)
            .collect();

        Ok(Self {
            data_path: todo_file_path,
            ui_state: UiState::List(ListState::new(0)),
            state: AppState {
                todos,
                hide_completed: false,
            },
            initial_content,
            should_write: true,
        })
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        while !matches!(self.ui_state, UiState::Quit) {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_events()?;
        }
        if self.should_write {
            self.write_to_file();
        }
        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        let title = Line::from("todui").bold().blue().centered();
        let block = Block::new().title_bottom(title);

        let position = match &self.ui_state {
            UiState::List(state) => state.position,
            UiState::Delete(state) => state.position,
            UiState::Edit(state) => state.position,
            UiState::ConfirmOverwrite(pos) => *pos,
            _ => usize::MAX,
        };

        let mut todo_lines: Vec<Line> = self
            .state
            .todos
            .iter()
            .enumerate()
            .filter_map(|(i, todo)| {
                if self.state.hide_completed && todo.completed {
                    return None;
                }

                let selected = i == position;

                if let UiState::Edit(state) = &self.ui_state {
                    if selected {
                        return Some(
                            Line::from(format!("> {description}", description = state.description))
                                .light_blue()
                                .bold(),
                        );
                    }
                }

                let main_span = {
                    let base = Span::from(format!(
                        "{prefix} {todo_string}",
                        prefix = if selected { ">" } else { " " },
                        todo_string = todo.to_string()
                    ));
                    if selected {
                        base.light_blue().bold()
                    } else if todo.completed {
                        base.dark_gray()
                    } else {
                        base
                    }
                };
                let info = if matches!(self.ui_state, UiState::Delete(_)) && selected {
                    Span::from(" delete? y/n").red()
                } else {
                    Span::from("")
                };
                Some(Line::from(vec![main_span, info]))
            })
            .collect();

        let add_line = match &self.ui_state {
            UiState::Add(state) => {
                Line::from(format!("> {description}", description = state.description)).light_blue()
            }
            UiState::Quit => panic!("Should not hit quit state"),
            _ => {
                let add_new_selected = position == self.state.todos.len();
                let line = Line::from(format!(
                    "{prefix} add new +",
                    prefix = if add_new_selected { ">" } else { " " }
                ))
                .italic();
                if add_new_selected {
                    line.blue().bold()
                } else {
                    line
                }
            }
        };

        todo_lines.push(add_line);

        if matches!(self.ui_state, UiState::ConfirmOverwrite(_)) {
            todo_lines.push(Line::from(""));
            todo_lines.push(Line::from(vec![
                Span::from("File has been modified externally. Overwrite? (y/n) ")
                    .red()
                    .bold(),
                Span::from("(esc to continue editing)").dark_gray().italic(),
            ]));
        }

        match &self.ui_state {
            UiState::Add(state) => {
                let pos = Position {
                    x: (state.description.len() + 2) as u16,
                    y: (todo_lines.len() - 1) as u16,
                };
                frame.set_cursor_position(pos);
            }
            UiState::Edit(state) => {
                let y = if self.state.hide_completed {
                    self.state
                        .todos
                        .iter()
                        .take(state.position)
                        .filter(|t| !t.completed)
                        .count()
                } else {
                    state.position
                };
                let pos = Position {
                    x: (state.cursor_x + 2) as u16,
                    y: y as u16,
                };
                frame.set_cursor_position(pos);
            }
            _ => {}
        }

        frame.render_widget(
            Paragraph::new(Text::from(todo_lines)).block(block),
            frame.area(),
        );
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                if let UiState::ConfirmOverwrite(pos) = self.ui_state {
                    match (key.modifiers, key.code) {
                        (_, KeyCode::Char('y')) => self.ui_state = UiState::Quit,
                        (_, KeyCode::Char('n')) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                            self.should_write = false;
                            self.ui_state = UiState::Quit;
                        }
                        (_, KeyCode::Esc) => {
                            self.ui_state = UiState::List(ListState::new(pos));
                        }
                        _ => {}
                    }
                    return Ok(());
                }

                let new_state = match &mut self.ui_state {
                    UiState::List(state) => state.handle_key_event(key, &mut self.state),
                    UiState::Add(state) => state.handle_key_event(key, &mut self.state.todos),
                    UiState::Edit(state) => state.handle_key_event(key, &mut self.state.todos),
                    UiState::Delete(state) => state.handle_key_event(key, &mut self.state),
                    _ => None,
                };
                if let Some(new_state) = new_state {
                    self.transition_to(new_state);
                }
            }
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    fn serialize_todos(&self) -> String {
        let mut serialized = String::new();
        for todo in &self.state.todos {
            serialized.push_str(&todo.serialize());
            serialized.push('\n');
        }
        serialized
    }

    fn transition_to(&mut self, new_state: UiState) {
        self.ui_state = if matches!(&new_state, UiState::Quit) {
            let pos = match &self.ui_state {
                UiState::List(state) => state.position,
                UiState::Delete(state) => state.position,
                UiState::Edit(state) => state.position,
                UiState::Add(_) => self.state.todos.len(),
                _ => 0,
            };
            let new_content = self.serialize_todos();
            let disk_content = fs::read_to_string(&self.data_path).unwrap_or_default();
            if new_content == disk_content {
                self.should_write = false;
                UiState::Quit
            } else if disk_content != self.initial_content {
                UiState::ConfirmOverwrite(pos)
            } else {
                new_state
            }
        } else {
            new_state
        };
    }

    fn write_to_file(&self) {
        fs::write(&self.data_path, self.serialize_todos()).unwrap();
    }
}
