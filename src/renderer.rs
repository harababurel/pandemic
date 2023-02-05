use crate::tile;
use crate::tilesource::{TileServerSource, TileSource};
use crate::util;
use crate::util::Coords;
use crate::vector_tile;
use image::{GenericImage, GenericImageView, ImageBuffer, Rgb, RgbImage};
use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;

pub struct Renderer<TS: TileSource> {
    width: usize,
    height: usize,
    tilesource: TS,
    img: RgbImage,
}

impl Renderer<TileServerSource> {
    pub fn new(width: usize, height: usize) -> Self {
        Renderer {
            width,
            height,
            tilesource: TileServerSource::new(),
            img: ImageBuffer::new(width as u32, height as u32),
        }
    }

    pub fn draw(&mut self, center: &util::Coords, zoom: u32) {
        let mut tiles: Vec<tile::Tile> = self.visible_tiles(center, zoom);

        tiles.iter_mut().for_each(|ref mut t| {
            t.vtile = self
                .tilesource
                .get_tile(t.z(), t.x(), t.y())
                .unwrap()
                .vtile;
        });

        for (x, y, pixel) in self.img.enumerate_pixels_mut() {
            let r = (100.0 + 0.2 * x as f32) as u8;
            let b = (100.0 + 0.2 * y as f32) as u8;
            *pixel = image::Rgb([r, 0, b]);
        }
        for t in &tiles {
            self.draw_tile(&t, zoom);
        }
        self.img.save("test.png").unwrap();
    }
    pub fn draw_tile(&mut self, t: &tile::Tile, zoom: u32) {
        let scale = 0.5;
        let tdx = t.x();
        let tdy = t.y();

        println!("tdx={}, tdy={}", tdx, tdy);

        let colors = vec![
            Rgb([47, 79, 79]),
            // Rgb([107, 142, 35]),
            // Rgb([100, 149, 237]),
            // Rgb([192, 192, 192]),
            // Rgb([221, 160, 221]),
            // Rgb([106, 90, 205]),
            // Rgb([255, 248, 220]),
            // Rgb([255, 228, 196]),
            // Rgb([250, 128, 114]),
        ];

        let color = colors[((tdx * 3 + tdy) as usize) % colors.len()];

        if let Some(vtile) = t.vtile.as_ref() {
            for layer in &vtile.layers {
                let extent = layer.extent();
                println!("layer extent is {}", extent);

                for feature in &layer.features {
                    let mut cursor = (0, 0);
                    let commands = tile::Tile::parse_geometry(&feature.geometry);
                    // println!("Commands: {:?}", commands);

                    match feature.r#type() {
                        vector_tile::tile::GeomType::Unknown => {
                            panic!("Found unknown geometry, don't know how to interpret this");
                        }
                        vector_tile::tile::GeomType::Point => {
                            for c in commands {
                                match c {
                                    tile::GeometryCommand::MoveTo(dx, dy) => {
                                        let ncursor = (cursor.0 + dx, cursor.1 + dy);

                                        if 0 <= ncursor.0 && 0 <= ncursor.1 {
                                            self.img.put_pixel(
                                                cursor.0 as u32,
                                                cursor.1 as u32,
                                                Rgb([255, 255, 255]),
                                            );
                                        }
                                        cursor = ncursor;
                                    }
                                    _ => {
                                        panic!("Point geometry can only contain MoveTo commands");
                                    }
                                };
                            }
                        }
                        vector_tile::tile::GeomType::Linestring => {
                            for c in commands {
                                match c {
                                    tile::GeometryCommand::MoveTo(dx, dy) => {
                                        cursor = (cursor.0 + dx, cursor.1 + dy);
                                    }
                                    tile::GeometryCommand::LineTo(dx, dy) => {
                                        let ncursor = (cursor.0 + dx, cursor.1 + dy);
                                        // if 0 <= ncursor.0 && 0 <= ncursor.1 {
                                        self.draw_line_on_img(
                                            cursor, ncursor, scale, extent, tdx, tdy, color,
                                        );
                                        // }
                                        cursor = ncursor;
                                    }
                                    _ => {
                                        panic!("LineString geometry can only contain MoveTo or LineTo commands");
                                    }
                                }
                            }
                        }
                        vector_tile::tile::GeomType::Polygon => {
                            let mut polygon_start = (0, 0);
                            for c in commands {
                                match c {
                                    tile::GeometryCommand::MoveTo(dx, dy) => {
                                        cursor = (cursor.0 + dx, cursor.1 + dy);
                                        polygon_start = cursor;
                                    }
                                    tile::GeometryCommand::LineTo(dx, dy) => {
                                        let ncursor = (cursor.0 + dx, cursor.1 + dy);
                                        // if 0 <= ncursor.0 && 0 <= ncursor.1 {
                                        self.draw_line_on_img(
                                            cursor, ncursor, scale, extent, tdx, tdy, color,
                                        );
                                        // }
                                        cursor = ncursor;
                                    }
                                    tile::GeometryCommand::ClosePath => {
                                        self.draw_line_on_img(
                                            cursor,
                                            polygon_start,
                                            scale,
                                            extent,
                                            tdx,
                                            tdy,
                                            color,
                                        );
                                        // unimplemented!("moveto");
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn get_tile_features(&self, tile: &tile::Tile, zoom: f64) {
        let draw_order = Renderer::generate_draw_order(zoom);
        println!("draw order is {:?}", draw_order);

        let vtile = tile.vtile.as_ref().unwrap();

        vtile.layers.iter().for_each(|l| {
            println!(
                "layer: {} (version={}, {} features, {} keys, {} values)",
                l.name,
                l.version,
                l.features.len(),
                l.keys.len(),
                l.values.len()
            );
            // println!("\tkeys: {:?}", l.keys);
        });
    }

    pub fn draw_line_on_img(
        &mut self,
        p: (i32, i32),
        q: (i32, i32),
        scale: f32,
        extent: u32,
        tdx: i32,
        tdy: i32,
        color: Rgb<u8>,
    ) {
        imageproc::drawing::draw_line_segment_mut(
            &mut self.img,
            (
                p.0 as f32 * scale + tdx as f32 * extent as f32 * scale,
                p.1 as f32 * scale + tdy as f32 * extent as f32 * scale,
            ), // start point
            (
                q.0 as f32 * scale + tdx as f32 * extent as f32 * scale,
                q.1 as f32 * scale + tdy as f32 * extent as f32 * scale,
            ), // end point
            color, // Rgb([0u8, 0u8, 0u8]), // RGB colors
        );
    }

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

    pub fn visible_tiles(&self, center: &Coords, zoom: u32) -> Vec<tile::Tile> {
        let center_t = util::coords_to_tile(center, zoom as f64);

        let tile_size = 256;

        let uncovered_right =
            ((self.width - tile_size) as f64 / 2.0 / (tile_size as f64)).ceil() as u32;
        let uncovered_up =
            ((self.height - tile_size) as f64 / 2.0 / (tile_size as f64)).ceil() as u32;

        let mut tiles: HashMap<(i32, i32, usize), tile::Tile> = HashMap::new();
        for dx in 0..(uncovered_right + 1) {
            for dy in 0..(uncovered_up + 1) {
                        tiles.insert(
                            (center_t.x() - dx , center_t.y(), z),
                            tile::Tile {
                                zxy: (z, tx, ty),
                                vtile: None,
                            },
                        );
            }
        }

        tiles.into_values().collect()

        // let mut tiles: HashMap<(i32, i32, usize), tile::Tile> = HashMap::new();

        // for dy in [-1, 0, 1] {
        //     for dx in [-1, 0, 1] {
        //         let mut tx = center.x.floor() as i32 + dx;
        //         let mut ty = center.y.floor() as i32 + dy;

        //         let pos_x = self.width as f64 / 2. - (center.x - tx as f64) * tile_size;
        //         let pos_y = self.height as f64 / 2. - (center.y - ty as f64) * tile_size;
        //         let grid_size = 2i32.pow(z as u32);

        //         tx %= grid_size;
        //         if tx < 0 {
        //             tx = if z == 0 { 0 } else { tx + grid_size };
        //         }

        //         if ty < 0
        //             || ty >= grid_size
        //             || pos_x + tile_size < 0.
        //             || pos_y + tile_size < 0.
        //             || pos_x > self.width as f64
        //             || pos_y > self.height as f64
        //         {
        //             continue;
        //         }

        //         tiles.insert(
        //             (tx, ty, z),
        //             tile::Tile {
        //                 xyz: (tx, ty, z),
        //                 zoom,
        //                 position: (pos_x, pos_y),
        //                 size: tile_size,
        //                 vtile: None,
        //             },
        //         );
        //     }
        // }
        // tiles.into_values().collect()
    }
}
