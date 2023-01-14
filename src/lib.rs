#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use rand::prelude::*;
use serde::{Deserialize, Serialize};
// use serde_json::Result;
use std::collections::{HashMap, VecDeque};
use std::fs;
use thiserror::Error;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Color {
    Blue,
    Yellow,
    Red,
    Black,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct City {
    name: String,
    color: Color,
    neighbors: Vec<String>,
    has_research_center: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PlayerCard {
    CityCard(String),
    EpidemicCard,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PlayerClass {
    Dispatcher,
    Generalist,
    Medic,
    Scientist,
    Researcher,
}

#[derive(Serialize, Debug)]
pub struct Player {
    class: PlayerClass,
    location: str,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Game {
    pub world: HashMap<String, City>,
    pub player_cards: VecDeque<PlayerCard>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            world: HashMap::new(),
            player_cards: VecDeque::new(),
        }
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
}
