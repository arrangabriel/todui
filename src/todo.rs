use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug)]
pub struct Todo {
    description: String,
    pub completed: bool,
}

impl Todo {
    pub fn new(description: &str) -> Self {
        Self {
            description: description.trim().to_string(),
            completed: false,
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "[{check}] {description}",
            check = if self.completed { "x" } else { " " },
            description = self.description
        )
    }

    pub fn deserialize(line: &str) -> Option<Self> {
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"- \[([ xX])\] (.*)").unwrap());
        RE.captures(line).map(|caps| Self {
            completed: !caps[1].eq(" "),
            description: caps[2].into(),
        })
    }

    pub fn serialize(&self) -> String {
        format!("- {}", self.to_string())
    }
}
