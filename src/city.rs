pub use crate::color::*;
use colored::Colorize;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct City {
    pub name: String,
    pub color: Color,
    pub neighbors: Vec<String>,
    pub has_research_center: bool,
    #[serde(default = "HashMap::new")]
    pub infections: HashMap<Color, u32>,
}

impl City {
    pub fn is_infected(&self) -> bool {
        self.infections.values().any(|x| x > &0)
    }
}

impl std::fmt::Display for City {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name.color(self.color.clone()))?;

        if self.is_infected() {
            write!(f, " (")?;
        }
        for (clr, qty) in &self.infections {
            for _ in 0..*qty {
                write!(f, "{}", "*".color(*clr))?;
            }
        }
        if self.is_infected() {
            write!(f, ")")?;
        }
        std::fmt::Result::Ok(())
    }
}
