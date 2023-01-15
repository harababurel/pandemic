use crate::color::Color;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlayerCard {
    CityCard(String),
    EpidemicCard,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlayerClass {
    Dispatcher,
    Generalist,
    Medic,
    Scientist,
    Researcher,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub enum PlayerAction {
    Drive(String),
    DirectFlight(String),
    CharterFlight(String),
    ShuttleFlight(String),
    BuildResearchCenter,
    TreatDisease(Color),
    GiveCard(PlayerCard, Player),
    ReceiveCard(PlayerCard, Player),
    DiscoverCure(Color),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Player {
    pub class: PlayerClass,
    pub location: String,
    pub hand: Vec<PlayerCard>,
}

impl Player {
    pub fn new(class: PlayerClass, location: &str) -> Self {
        Player {
            class,
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

    pub fn cards_needed_for_cure(&self) -> usize {
        match self.class {
            PlayerClass::Scientist => 4,
            _ => 5,
        }
    }

    pub fn can_give_any_card(&self) -> bool {
        match self.class {
            PlayerClass::Researcher => true,
            _ => false,
        }
    }
}
