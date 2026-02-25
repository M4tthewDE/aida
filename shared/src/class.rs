use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ClassIdentifier {
    package: Vec<String>,
    name: String,
}

impl ClassIdentifier {
    pub fn parse(raw: &str) -> Self {
        let raw = raw.replace("/", ".").replace(";", "");
        let raw: String = raw.chars().skip_while(|c| *c == 'L' || *c == '[').collect();

        match raw.as_str() {
            "B" => {
                return Self {
                    package: vec!["java".to_string(), "lang".to_string()],
                    name: "Byte".to_owned(),
                };
            }
            "C" => {
                return Self {
                    package: vec!["java".to_string(), "lang".to_string()],
                    name: "Character".to_owned(),
                };
            }
            "D" => {
                return Self {
                    package: vec!["java".to_string(), "lang".to_string()],
                    name: "Double".to_owned(),
                };
            }
            "F" => {
                return Self {
                    package: vec!["java".to_string(), "lang".to_string()],
                    name: "Float".to_owned(),
                };
            }
            "I" => {
                return Self {
                    package: vec!["java".to_string(), "lang".to_string()],
                    name: "Integer".to_owned(),
                };
            }
            "J" => {
                return Self {
                    package: vec!["java".to_string(), "lang".to_string()],
                    name: "Long".to_owned(),
                };
            }
            "S" => {
                return Self {
                    package: vec!["java".to_string(), "lang".to_string()],
                    name: "Short".to_owned(),
                };
            }
            "Z" => {
                return Self {
                    package: vec!["java".to_string(), "lang".to_string()],
                    name: "Boolean".to_owned(),
                };
            }
            _ => {}
        }

        let mut parts: Vec<&str> = raw.split('.').collect();
        let name = parts.last().unwrap().to_string();
        parts.truncate(parts.len() - 1);
        let package: Vec<String> = parts.iter().map(|p| p.to_string()).collect();

        Self { package, name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Display for ClassIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for package in &self.package {
            write!(f, "{}.", package)?;
        }

        write!(f, "{}", self.name)
    }
}
