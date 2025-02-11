use core::panic;
use std::path::PathBuf;
use std::{fs, io};

use crossterm::event::{self, Event, KeyEventKind};
use ratatui::layout::Position;
use ratatui::style::Stylize;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Paragraph};
use ratatui::{DefaultTerminal, Frame};

use crate::todo::Todo;
use crate::ui_state::{ListState, UiState};

#[derive(Debug)]
pub struct App {
    ui_state: UiState,
    state: AppState,
    data_path: PathBuf,
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
        static TODO_FILE: &str = "todo.md";
        let mut path = if let Ok(path) = std::env::var("TODUI_DIR") {
            PathBuf::from(path)
        } else if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
            let mut xdg_data_path = PathBuf::from(data_home);
            xdg_data_path.push("todui");
            xdg_data_path
        } else {
            let mut home =
                xdg_home::home_dir().ok_or(anyhow::anyhow!("Could not get home directory"))?;
            home.push(".todui");
            home
        };
        path.push(TODO_FILE);

        let todos: Vec<Todo> = if fs::exists(&path)? {
            fs::read_to_string(&path)?
        } else {
            fs::create_dir_all(&path.parent().unwrap())?;
            String::new()
        }
        .split('\n')
        .filter_map(Todo::deserialize)
        .collect();

        Ok(Self {
            data_path: path,
            ui_state: UiState::List(ListState::new(0)),
            state: AppState {
                todos,
                hide_completed: false,
            },
        })
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        while !matches!(self.ui_state, UiState::Quit) {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_events()?;
        }
        self.write_to_file();
        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        let title = Line::from("todui").bold().blue().centered();
        let block = Block::new().title_bottom(title);

        let mut todo_lines: Vec<Line> = self
            .state
            .todos
            .iter()
            .enumerate()
            .filter_map(|(i, todo)| {
                if self.state.hide_completed && todo.completed {
                    return None;
                }

                let position = match &self.ui_state {
                    UiState::List(state) => state.position,
                    UiState::Delete(state) => state.position,
                    _ => usize::MAX,
                };

                let selected = i == position;

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
            UiState::List(state) => {
                let add_new_selected = state.position == self.state.todos.len();
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
            UiState::Delete(_) => Line::from("  add new +"),
            UiState::Add(state) => {
                Line::from(format!("> {description}", description = state.description)).light_blue()
            }
            UiState::Quit => panic!("Should not hit quit state"),
        };

        todo_lines.push(add_line);

        if let UiState::Add(state) = &self.ui_state {
            let pos = Position {
                x: (state.description.len() + 2) as u16,
                y: (todo_lines.len() - 1) as u16,
            };
            frame.set_cursor_position(pos)
        }

        frame.render_widget(
            Paragraph::new(Text::from(todo_lines)).block(block),
            frame.area(),
        );
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match &mut self.ui_state {
                UiState::List(state) => {
                    if let Some(new_state) = state.handle_key_event(key, &mut self.state) {
                        self.ui_state = new_state
                    }
                }
                UiState::Add(state) => {
                    if let Some(new_state) = state.handle_key_event(key, &mut self.state.todos) {
                        self.ui_state = new_state
                    }
                }
                UiState::Delete(state) => {
                    if let Some(new_state) = state.handle_key_event(key, &mut self.state) {
                        self.ui_state = new_state
                    }
                }
                _ => {}
            },
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    fn write_to_file(&mut self) {
        let mut serilized = String::new();
        for todo in &self.state.todos {
            serilized.push_str(&todo.serialize());
            serilized.push('\n');
        }
        fs::write(&self.data_path, serilized).unwrap();
    }
}
