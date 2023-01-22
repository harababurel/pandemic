use crate::util;
use crate::util::{Coords, Tile};
use std::f64::consts::PI;

pub struct Renderer {
    width: usize,
    height: usize,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            width: 1280,
            height: 720,
        }
    }

    pub fn visible_tiles(&self, center: &Coords, zoom: f64) -> Vec<Tile> {
        let z = util::base_zoom(zoom);

        let center = util::coords_to_tile(center, z as f64);
        let tile_size = util::tile_size_at_zoom(zoom);

        let mut tiles: Vec<Tile> = Vec::new();

        for dy in [-1, 0, 1] {
            for dx in [-1, 0, 1] {
                let mut tx = center.x.floor() as i32 + dx;
                let mut ty = center.y.floor() as i32 + dy;

                let pos_x = self.width as f64 / 2. - (center.x - tx as f64) * tile_size;
                let pos_y = self.height as f64 / 2. - (center.y - ty as f64) * tile_size;
                let grid_size = 2i32.pow(z as u32);

                tx %= grid_size;
                if tx < 0 {
                    tx = if z == 0 { 0 } else { tx + grid_size };
                }

                if ty < 0
                    || ty >= grid_size
                    || pos_x + tile_size < 0.
                    || pos_y + tile_size < 0.
                    || pos_x > self.width as f64
                    || pos_y > self.height as f64
                {
                    continue;
                }

                tiles.push(Tile {
                    xyz: (tx, ty, z),
                    zoom,
                    position: (pos_x, pos_y),
                    size: tile_size,
                });
            }
        }
        tiles
    }
}
