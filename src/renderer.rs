use crate::tilesource::{TileServerSource, TileSource};
use crate::util;
use crate::util::{Coords, Tile};
use crate::vector_tile;
use std::f64::consts::PI;

pub struct Renderer<TS: TileSource> {
    width: usize,
    height: usize,
    tilesource: TS,
}

impl Renderer<TileServerSource> {
    pub fn new() -> Self {
        Renderer {
            width: 1280,
            height: 720,
            tilesource: TileServerSource::new(),
        }
    }

    pub fn draw(&self, center: &util::Coords, zoom: f64) {
        let mut tiles = self.visible_tiles(center, zoom);

        tiles.iter_mut().for_each(|ref mut t| {
            t.vtile = self
                .tilesource
                .get_tile(t.xyz.2, t.xyz.0, t.xyz.1)
                .unwrap()
                .vtile;
        });

        for t in &tiles {
            self.get_tile_features(&t, zoom);
        }
    }

    pub fn get_tile_features(&self, tile: &util::Tile, zoom: f64) {
        let draw_order = Renderer::generate_draw_order(zoom);

        let vtile = tile.vtile.as_ref().unwrap();

        draw_order.iter().for_each(|layer_id| {
            if let Some(layer) = vtile.layers.iter().find(|l| &l.name == layer_id) {
                let scale = layer.extent() as f64 / util::tile_size_at_zoom(zoom);

                println!("layer: {:#?}", layer);

                // layer.
            }

            // tile.
        });
    }
    // _getTileFeatures(tile, zoom) {
    //     const position = tile.position;
    //     const layers = {};
    //     const drawOrder = this._generateDrawOrder(zoom);
    //     for (const layerId of drawOrder) {
    //         const layer = (tile.data.layers || {})[layerId];
    //         if (!layer) {
    //             continue;
    //         }

    //         const scale = layer.extent / utils.tilesizeAtZoom(zoom);
    //         layers[layerId] = {
    //             scale: scale,
    //             features: layer.tree.search({
    //                 minX: -position.x * scale,
    //                 minY: -position.y * scale,
    //                 maxX: (this.width - position.x) * scale,
    //                 maxY: (this.height - position.y) * scale
    //             }),
    //         };
    //     }
    //     tile.layers = layers;
    //     return tile;
    // }

    pub fn generate_draw_order(zoom: f64) -> Vec<String> {
        let features = if zoom < 2. {
            vec!["admin", "water", "country_label", "marine_label"]
        } else {
            vec![
                "landuse",
                "water",
                "marine_label",
                "building",
                "road",
                "admin",
                "country_label",
                "state_label",
                "water_label",
                "place_label",
                "rail_station_label",
                "poi_label",
                "road_label",
                "housenum_label",
            ]
        };
        features.into_iter().map(|s| s.to_string()).collect()
    }

    // async draw(center, zoom) {
    //     if (this.isDrawing) return Promise.reject();
    //     this.isDrawing = true;

    //     this.labelBuffer.clear();
    //     this._seen = {};

    //     let ref;
    //     const color = ((ref = this.styler.styleById['background']) !== null ?
    //         ref.paint['background-color'] :
    //         void 0
    //     );
    //     if (color) {
    //         this.canvas.setBackground(x256(utils.hex2rgb(color)));
    //     }

    //     this.canvas.clear();

    //     try {
    //         let tiles = this._visibleTiles(center, zoom);
    //         await Promise.all(tiles.map(async (tile) => {
    //             await this._getTile(tile);
    //             this._getTileFeatures(tile, zoom);
    //         }));
    //         await this._renderTiles(tiles);
    //         return this._getFrame();
    //     } catch (e) {
    //         console.error(e);
    //     } finally {
    //         this.isDrawing = false;
    //         this.lastDrawAt = Date.now();
    //     }
    // }

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
                    vtile: None,
                });
            }
        }
        tiles
    }
}
