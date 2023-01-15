use colored::Colorize;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default, Copy)]
pub enum Color {
    #[default]
    Blue,
    Yellow,
    Red,
    Black,
}

impl Into<colored::Color> for Color {
    fn into(self) -> colored::Color {
        match self {
            Color::Blue => colored::Color::Blue,
            Color::Red => colored::Color::Red,
            Color::Yellow => colored::Color::Yellow,
            Color::Black => colored::Color::BrightBlack,
        }
    }
}
