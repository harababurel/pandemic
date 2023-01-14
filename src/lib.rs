#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use rand::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
// use serde_json::Result;
use colored::*;
use std::collections::{HashMap, VecDeque};
use std::fs;
use thiserror::Error;

const INFECTION_RATES: [i32; 7] = [2, 2, 2, 3, 3, 4, 4];

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default)]
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct City {
    name: String,
    color: Color,
    neighbors: Vec<String>,
    has_research_center: bool,
    #[serde(default = "HashMap::new")]
    infections: HashMap<Color, u32>,
}

impl std::fmt::Display for City {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name.color(self.color.clone()))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PlayerCard {
    CityCard(String),
    EpidemicCard,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PlayerClass {
    Dispatcher,
    Generalist,
    Medic,
    Scientist,
    Researcher,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    class: PlayerClass,
    location: String,
    hand: Vec<PlayerCard>,
}

impl Player {
    pub fn new(class: PlayerClass, location: &str) -> Self {
        Player {
            class: class,
            location: location.to_string(),
            hand: Vec::new(),
        }
    }

    pub fn dispatcher(location: &str) -> Self {
        Self::new(PlayerClass::Dispatcher, location)
    }
    pub fn generalist(location: &str) -> Self {
        Self::new(PlayerClass::Generalist, location)
    }
    pub fn medic(location: &str) -> Self {
        Self::new(PlayerClass::Medic, location)
    }
    pub fn scientist(location: &str) -> Self {
        Self::new(PlayerClass::Scientist, location)
    }
    pub fn researcher(location: &str) -> Self {
        Self::new(PlayerClass::Researcher, location)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Game {
    pub world: HashMap<String, City>,
    pub player_cards: VecDeque<PlayerCard>,
    pub players: Vec<Player>,
    pub infection_level: usize,
    pub outbreaks: usize,
    pub diseases: Vec<Disease>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Disease {
    pub color: Color,
    pub cured: bool,
    pub eradicated: bool,
}

impl Disease {
    pub fn new(color: Color) -> Self {
        Disease {
            color,
            cured: false,
            eradicated: false,
        }
    }
}

impl Game {
    pub fn new() -> Self {
        Game {
            world: HashMap::new(),
            player_cards: VecDeque::new(),
            players: Vec::new(),
            infection_level: 0,
            outbreaks: 0,
            diseases: vec![
                Disease::new(Color::Blue),
                Disease::new(Color::Red),
                Disease::new(Color::Yellow),
                Disease::new(Color::Black),
            ],
        }
    }

    pub fn infection_rate(&self) -> i32 {
        INFECTION_RATES[self.infection_level]
    }

    pub fn add_player(&mut self, p: Player) -> Result<(), PandemicError> {
        if self.players.iter().any(|x| x.class == p.class) {
            return Err(PandemicError::PlayerClassConflict);
        }
        if !self.world.contains_key(&p.location) {
            return Err(PandemicError::InvalidPlayerlocation);
        }

        self.players.push(p);
        Ok(())
    }

    pub fn load_cities(&mut self, path: &str) -> Result<(), PandemicError> {
        let contents = fs::read_to_string(path)?;

        let cities: Vec<City> = serde_json::from_str(&contents)?;
        let mut world: HashMap<String, City> = HashMap::new();

        for city in cities {
            world.insert(city.name.clone(), city.clone());
        }

        for (name, city) in &world {
            for n_name in &city.neighbors {
                if name == n_name {
                    return Err(PandemicError::CityGraphError(format!(
                        "Self loop detected: {}",
                        name
                    )));
                }
                if !world.contains_key(n_name) {
                    return Err(PandemicError::CityGraphError(format!(
                        "{} connected to unknown city: {}",
                        name, n_name
                    )));
                }
                if !(world.get(n_name).unwrap().neighbors.contains(name)) {
                    return Err(PandemicError::CityGraphError(format!(
                        "{} -> {} edge is not bidirectional!",
                        name, n_name
                    )));
                }
            }
        }

        self.world = world;
        Ok(())
    }

    pub fn create_player_cards(&mut self) {
        for city_name in self.world.keys() {
            let card = PlayerCard::CityCard(city_name.clone());
            self.player_cards.push_back(card);
        }
        for _ in 0..5 {
            self.player_cards.push_back(PlayerCard::EpidemicCard);
        }
        self.player_cards
            .make_contiguous()
            .shuffle(&mut rand::thread_rng());
    }
}

#[derive(Error, Debug)]
pub enum PandemicError {
    #[error("some I/O error")]
    IoError(#[from] std::io::Error),
    #[error("some serde json error")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("city graph error: {0}")]
    CityGraphError(String),
    #[error("player class conflict")]
    PlayerClassConflict,
    #[error("invalid player location")]
    InvalidPlayerlocation,
}
