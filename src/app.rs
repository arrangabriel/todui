use std::path::PathBuf;
use std::{fs, io};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use once_cell::sync::Lazy;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui::DefaultTerminal;
use regex::Regex;

#[derive(Debug)]
struct Todo {
    description: String,
    completed: bool,
}

impl Todo {
    fn to_string(&self) -> String {
        format!(
            "[{check}] {description}",
            check = if self.completed { "x" } else { " " },
            description = self.description
        )
    }

    fn deserialize(line: &str) -> Option<Self> {
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"- \[([ xX])\] (.*)").unwrap());
        RE.captures(line).map(|caps| Self {
            completed: !caps[1].eq(" "),
            description: caps[2].into(),
        })
    }

    fn serialize(&self) -> String {
        format!("- {}", self.to_string())
    }
}

#[derive(Debug)]
pub struct App {
    exit: bool,
    position: usize,
    data_path: PathBuf,
    todos: Vec<Todo>,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        static TODO_FILE: &str = "todo.md";
        let mut path = if let Ok(path) = std::env::var("TODUI_DIR") {
            PathBuf::from(path)
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
            exit: false,
            position: 0,
            data_path: path,
            todos,
        })
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.handle_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c')) => self.quit(),
            (_, KeyCode::Up | KeyCode::Char('k')) => self.move_position(true),
            (_, KeyCode::Down | KeyCode::Char('j')) => self.move_position(false),
            (_, KeyCode::Char(' ')) => self.handle_interact(),
            _ => {}
        }
    }

    fn move_position(&mut self, up: bool) {
        if up && self.position > 0 {
            self.position -= 1;
        } else if !up && self.position < self.todos.len() {
            self.position += 1;
        }
    }

    fn handle_interact(&mut self) {
        if self.position < self.todos.len() {
            let current = &mut self.todos[self.position];
            current.completed = !current.completed
        } else {
            todo!("Create new todo")
        }
    }

    fn write_to_file(&mut self) {
        let mut serilized = String::new();
        for todo in &self.todos {
            serilized.push_str(&todo.serialize());
            serilized.push('\n');
        }
        fs::write(&self.data_path, serilized).unwrap();
    }

    fn quit(&mut self) {
        self.write_to_file();
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from("todui").bold().blue();
        let helper_text = Line::from("Press 'q' or 'Esc' to quit.").italic();
        let block = Block::new().title(title).title_bottom(helper_text);

        let mut todo_lines: Vec<Line> = self
            .todos
            .iter()
            .enumerate()
            .map(|(i, todo)| {
                let selected = i == self.position;
                let line = Line::from(format!(
                    "{prefix} {todo_string}",
                    prefix = if selected { ">" } else { " " },
                    todo_string = todo.to_string()
                ));
                if selected {
                    line.light_blue()
                } else if todo.completed {
                    line.dark_gray()
                } else {
                    line
                }
            })
            .collect();

        let add_new = {
            let add_new_selected = self.position == self.todos.len();
            let line = Line::from(format!(
                "{prefix} add new +",
                prefix = if add_new_selected { ">" } else { " " }
            ))
            .italic();
            if add_new_selected {
                line.blue()
            } else {
                line
            }
        };

        todo_lines.push(add_new);

        Paragraph::new(Text::from(todo_lines))
            .block(block)
            .render(area, buf);
    }
}
