use std::f64::consts::PI;

// https://wiki.openstreetmap.org/wiki/Slippy_map_tilenames

pub fn coords_to_tile(lon_deg: f64, lat_deg: f64, zoom: f64) -> (f64, f64) {
    let lat_rad = lat_deg.to_radians();
    let n = 2f64.powf(zoom);
    let xtile = ((lon_deg + 180.) / 360. * n).floor();
    let ytile = ((1. - lat_rad.tan().asinh() / PI) / 2. * n).floor();

    (xtile, ytile)
}
