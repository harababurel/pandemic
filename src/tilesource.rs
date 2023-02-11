use std::num::NonZeroUsize;

use crate::tile;
use crate::util;
use crate::vector_tile;
use lru;
use prost::Message;
use thiserror::Error;

pub trait TileSource {
    fn get_tile(&mut self, z: usize, x: i32, y: i32) -> Result<tile::Tile, TileSourceError>;
}

pub struct TileServerSource {
    tileserver: String,
}

pub struct CachedTileSource<TS: TileSource> {
    ts: TS,
    cache: lru::LruCache<(usize, i32, i32), Result<tile::Tile, TileSourceError>>,
}

pub struct DummyTileSource {}

impl TileServerSource {
    pub fn new() -> Self {
        TileServerSource {
            tileserver: String::from("http://harababurel.com:8080"),
        }
    }
}

impl<TS: TileSource> CachedTileSource<TS> {
    pub fn with_cap(ts: TS, cap: NonZeroUsize) -> Self {
        CachedTileSource {
            ts,
            cache: lru::LruCache::new(cap),
        }
    }
    pub fn unbounded(ts: TS) -> Self {
        CachedTileSource {
            ts,
            cache: lru::LruCache::unbounded(),
        }
    }
}

impl<TS: TileSource> TileSource for CachedTileSource<TS> {
    fn get_tile(&mut self, z: usize, x: i32, y: i32) -> Result<tile::Tile, TileSourceError> {
        let k = (z, x, y);

        if !self.cache.contains(&k) {
            self.cache.push(k, self.ts.get_tile(z, x, y));
        }

        // Workaround: reqwest::Error can't be cloned because of the streamy nature of it.
        // When we have to return a fresh clone of a tile source error, we just return a new
        // UndefinedError
        match self.cache.get(&k).unwrap() {
            Ok(tile) => Ok(tile.to_owned()),
            Err(err) => Err(TileSourceError::UnknownError),
        }
    }
}

impl TileSource for TileServerSource {
    fn get_tile(&mut self, z: usize, x: i32, y: i32) -> Result<tile::Tile, TileSourceError> {
        let endpoint = format!("{}/data/v3/{z}/{x}/{y}.pbf", self.tileserver);
        info!("Fetching tile from {}", endpoint);
        let res = reqwest::blocking::get(endpoint)?;
        let buf = res.bytes()?;
        let vtile = vector_tile::Tile::decode(&mut std::io::Cursor::new(buf.clone()))?;

        let tile = tile::Tile::from_proto(x, y, z, vtile);

        Ok(tile)
    }
}
impl TileSource for DummyTileSource {
    fn get_tile(&mut self, z: usize, x: i32, y: i32) -> Result<tile::Tile, TileSourceError> {
        Ok(tile::Tile::from_proto(
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
    #[error("unknown error")]
    UnknownError,
}
