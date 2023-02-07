use crate::vector_tile;
use std::f64::consts::PI;

const PROJECT_SIZE: u32 = 256;

// https://wiki.openstreetmap.org/wiki/Slippy_map_tilenames

#[derive(Debug)]
pub struct Coords {
    pub lon: f64,
    pub lat: f64,
}

pub struct TileCoords {
    pub x: u32,
    pub y: u32,
}

impl Coords {
    pub fn from_deg(lon: f64, lat: f64) -> Self {
        Coords { lon, lat }
    }
}

impl TileCoords {
    pub fn new(x: u32, y: u32) -> Self {
        TileCoords { x, y }
    }
}

pub fn coords_to_tile(c: &Coords, zoom: f64) -> TileCoords {
    let lat_rad = c.lat.to_radians();
    let n = 2f64.powf(zoom);
    let xtile = ((c.lon + 180.) / 360. * n).floor() as u32;
    let ytile = ((1. - lat_rad.tan().asinh() / PI) / 2. * n).floor() as u32;

    TileCoords::new(xtile, ytile)
}

pub fn tile_to_coords(t: &TileCoords, zoom: f64) -> Coords {
    let n = 2f64.powf(zoom);
    let lon_deg = t.x as f64 / n * 360. - 180.;
    let lat_rad = (PI * (1. - 2. * t.y as f64 / n)).sinh().atan();
    let lat_deg = lat_rad.to_degrees();

    Coords::from_deg(lon_deg, lat_deg)
}

// pub fn base_zoom(zoom: f64) -> usize {
//     (zoom.floor() as usize).max(0).min(14) // config.tilerange = 14
// }

// pub fn tile_size_at_zoom(zoom: f64) -> f64 {
//     PROJECT_SIZE as f64 * 2f64.powf(zoom - base_zoom(zoom) as f64)
// }
