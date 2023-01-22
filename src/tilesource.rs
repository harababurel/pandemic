use crate::vector_tile;
use prost::Message;
use thiserror::Error;

trait TileSource {
    fn get_tile(&self, z: usize, x: i32, y: i32) -> Result<vector_tile::Tile, TileSourceError>;
}

struct TileServerSource {
    tileserver: String,
}

struct DummyTileSource {}

impl TileServerSource {
    pub fn new() -> Self {
        TileServerSource {
            tileserver: String::from("http://harababurel.com:8080"),
        }
    }
}

impl TileSource for TileServerSource {
    fn get_tile(&self, z: usize, x: i32, y: i32) -> Result<vector_tile::Tile, TileSourceError> {
        let res = reqwest::blocking::get(format!("{}/data/v3/{z}/{x}/{y}.pbf", self.tileserver))?;
        let buf = res.bytes()?;
        let tile = vector_tile::Tile::decode(&mut std::io::Cursor::new(buf.clone()))?;

        Ok(tile)
    }
}
impl TileSource for DummyTileSource {
    fn get_tile(&self, z: usize, x: i32, y: i32) -> Result<vector_tile::Tile, TileSourceError> {
        Ok(vector_tile::Tile::default())
    }
}

#[derive(Error, Debug)]
pub enum TileSourceError {
    #[error("some network error")]
    NetworkError(#[from] reqwest::Error),
    #[error("could not decode proto")]
    ProtoDecodeError(#[from] prost::DecodeError),
}
