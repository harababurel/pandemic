use crate::color::Color;
use serde::{Deserialize, Deserializer, Serialize};

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
