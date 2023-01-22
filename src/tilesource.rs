use crate::util;
use crate::vector_tile;
use prost::Message;
use thiserror::Error;

pub trait TileSource {
    fn get_tile(&self, z: usize, x: i32, y: i32) -> Result<util::Tile, TileSourceError>;
}

pub struct TileServerSource {
    tileserver: String,
}

pub struct DummyTileSource {}

impl TileServerSource {
    pub fn new() -> Self {
        TileServerSource {
            tileserver: String::from("http://harababurel.com:8080"),
        }
    }
}

impl TileSource for TileServerSource {
    fn get_tile(&self, z: usize, x: i32, y: i32) -> Result<util::Tile, TileSourceError> {
        let endpoint = format!("{}/data/v3/{z}/{x}/{y}.pbf", self.tileserver);
        println!("Fetching tile from {}", endpoint);
        let res = reqwest::blocking::get(endpoint)?;
        let buf = res.bytes()?;
        let vtile = vector_tile::Tile::decode(&mut std::io::Cursor::new(buf.clone()))?;

        let tile = util::Tile::from_proto(x, y, z, vtile);

        Ok(tile)
    }
}
impl TileSource for DummyTileSource {
    fn get_tile(&self, z: usize, x: i32, y: i32) -> Result<util::Tile, TileSourceError> {
        Ok(util::Tile::from_proto(
            x,
            y,
            z,
            vector_tile::Tile::default(),
        ))
    }
}

#[derive(Error, Debug)]
pub enum TileSourceError {
    #[error("some network error")]
    NetworkError(#[from] reqwest::Error),
    #[error("could not decode proto")]
    ProtoDecodeError(#[from] prost::DecodeError),
}
