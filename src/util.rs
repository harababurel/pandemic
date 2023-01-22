use std::f64::consts::PI;

// https://wiki.openstreetmap.org/wiki/Slippy_map_tilenames

pub struct Coords {
    lon: f64,
    lat: f64,
}

impl Coords {
    pub fn from_deg(lon: f64, lat: f64) -> Self {
        Coords { lon, lat }
    }
}

pub fn coords_to_tile(c: &Coords, zoom: f64) -> (f64, f64) {
    let lat_rad = c.lat.to_radians();
    let n = 2f64.powf(zoom);
    let xtile = ((c.lon + 180.) / 360. * n).floor();
    let ytile = ((1. - lat_rad.tan().asinh() / PI) / 2. * n).floor();

    (xtile, ytile)
}

pub fn tile_to_coords(xtile: i32, ytile: i32, zoom: f64) -> Coords {
    let n = 2f64.powf(zoom);
    let lon_deg = xtile as f64 / n * 360. - 180.;
    let lat_rad = (PI * (1. - 2. * ytile as f64 / n)).sinh().atan();
    let lat_deg = lat_rad.to_degrees();

    Coords::from_deg(lon_deg, lat_deg)
}
