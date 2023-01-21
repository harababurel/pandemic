#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use colored::Colorize;
use rand::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs;
use thiserror::Error;

pub mod city;
pub mod color;
pub mod disease;
pub mod player;
pub mod util;
pub use crate::city::City;
pub use crate::color::Color;
pub use crate::disease::Disease;
pub use crate::player::*;

const INFECTION_RATES: [i32; 7] = [2, 2, 2, 3, 3, 4, 4];
const MIN_PLAYERS: usize = 2;
const MAX_PLAYERS: usize = 4;

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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Game {
    pub world: HashMap<String, City>,
    pub player_cards: VecDeque<PlayerCard>,
    pub infection_card_pile: VecDeque<String>,
    pub infection_discard_pile: VecDeque<String>,
    pub players: Vec<Player>,
    pub infection_level: usize,
    pub outbreaks: usize,
    pub diseases: Vec<Disease>,
}

impl Game {
    pub fn from_file(cities_file: &str) -> Result<Self, PandemicError> {
        let mut game = Game {
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
            ..Default::default()
        };
        game.load_cities(cities_file)?;
        Ok(game)
    }

    pub fn infection_rate(&self) -> i32 {
        INFECTION_RATES[self.infection_level]
    }

    pub fn add_player(&mut self, p: Player) -> Result<(), PandemicError> {
        if self.players.len() == MAX_PLAYERS {
            return Err(PandemicError::TooManyPlayers);
        }

        if self.players.iter().any(|x| x.class == p.class) {
            return Err(PandemicError::PlayerClassConflict);
        }
        if !self.world.contains_key(&p.location) {
            return Err(PandemicError::InvalidPlayerlocation);
        }

        self.players.push(p);
        Ok(())
    }

    fn load_cities(&mut self, path: &str) -> Result<(), PandemicError> {
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

    fn create_player_cards(&mut self) {
        for city_name in self.world.keys() {
            let card = PlayerCard::CityCard(city_name.clone());
            self.player_cards.push_back(card);
        }
        self.player_cards
            .make_contiguous()
            .shuffle(&mut rand::thread_rng());
    }

    fn add_epidemic_cards(&mut self) {
        for _ in 0..5 {
            self.player_cards.push_back(PlayerCard::EpidemicCard);
        }
        self.player_cards
            .make_contiguous()
            .shuffle(&mut rand::thread_rng());
    }

    fn create_infection_cards(&mut self) {
        for city_name in self.world.keys() {
            self.infection_card_pile.push_back(city_name.clone());
        }
        self.infection_card_pile
            .make_contiguous()
            .shuffle(&mut thread_rng());
    }

    fn deal_player_cards(&mut self) {
        let cards_per_player = match self.players.len() {
            2 => 4,
            3 => 3,
            4 => 2,
            _ => panic!("Invalid number of players, don't know how many cards to deal"),
        };

        for p in &mut self.players {
            for _ in 0..cards_per_player {
                let card = self.player_cards.pop_front().expect(
                    "Tried to deal initial cards but there aren't enough in the player card deck",
                );
                p.hand.push(card);
            }
        }
    }

    pub fn infect_initial_cities(&mut self) {
        for severity in [3, 2, 1] {
            for _ in 0..3 {
                let city_name = self.infection_card_pile.pop_front().expect("Tried to infect initial cities but there aren't enough infection cards in the deck");
                self.world.entry(city_name.clone()).and_modify(|city| {
                    *city.infections.entry(city.color).or_insert(0) += severity;
                });
                self.infection_discard_pile.push_back(city_name);
            }
        }
    }

    pub fn setup(&mut self) {
        self.create_player_cards();
        self.deal_player_cards();
        self.add_epidemic_cards();
        self.create_infection_cards();
        self.infect_initial_cities();
    }

    pub fn possible_actions(&self, p: &Player) -> Vec<PlayerAction> {
        let mut actions = Vec::new();
        let city = self.world.get(&p.location).unwrap();

        // Drive
        for dest in &city.neighbors {
            actions.push(PlayerAction::Drive(dest.clone()));
        }

        // DirectFlight / CharterFlight
        for card in &p.hand {
            if let PlayerCard::CityCard(name) = card {
                if name != &p.location {
                    actions.push(PlayerAction::DirectFlight(name.clone()));
                } else {
                    for dest in self.world.keys() {
                        if dest != name {
                            actions.push(PlayerAction::CharterFlight(dest.clone()));
                        }
                    }
                }
            }
        }

        // ShuttleFlight
        if city.has_research_center {
            for dest in self.world.values() {
                if dest.name != city.name && dest.has_research_center {
                    actions.push(PlayerAction::ShuttleFlight(dest.name.clone()));
                }
            }
        }

        // BuildResearchCenter
        if !city.has_research_center
            && p.location == city.name
            && p.hand
                .iter()
                .any(|card| *card == PlayerCard::CityCard(city.name.clone()))
        {
            actions.push(PlayerAction::BuildResearchCenter);
        }

        // TreatDisease
        city.infections
            .iter()
            .filter(|(clr, qty)| qty > &&0)
            .for_each(|(clr, qty)| {
                actions.push(PlayerAction::TreatDisease(*clr));
            });

        // DiscoverCure
        for disease in &self.diseases {
            if !disease.cured {
                let mut cnt = 0;
                for card in &p.hand {
                    if let PlayerCard::CityCard(name) = card {
                        let color = self.world.get(name).unwrap().color;
                        if color == disease.color {
                            cnt += 1;
                        }
                    }
                }
                if cnt >= p.cards_needed_for_cure() {
                    actions.push(PlayerAction::DiscoverCure(disease.color));
                }
            }
        }

        // GiveCard / ReceiveCard
        for q in &self.players {
            if p == q || p.location != q.location {
                continue;
            }
            for card in &p.hand {
                if let PlayerCard::CityCard(name) = card {
                    if name == &p.location || p.can_give_any_card() {
                        actions.push(PlayerAction::GiveCard(card.clone(), q.clone()));
                    }
                }
            }
            for card in &q.hand {
                if let PlayerCard::CityCard(name) = card {
                    if name == &q.location || q.can_give_any_card() {
                        actions.push(PlayerAction::ReceiveCard(card.clone(), q.clone()));
                    }
                }
            }
        }

        actions
    }

    pub fn run(&mut self) {}
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
    #[error("too many players")]
    TooManyPlayers,
}
