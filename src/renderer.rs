use crate::tile;
use crate::tilesource::{TileServerSource, TileSource};
use crate::util;
use crate::util::Coords;
use crate::vector_tile;
use image::{GenericImage, GenericImageView, ImageBuffer, Rgb, RgbImage};
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;

const MIN_ZOOM: u32 = 0;
const MAX_ZOOM: u32 = 14;

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

pub struct Renderer<TS: TileSource> {
    width: usize,
    height: usize,
    center: Coords,
    pub zoom: u32,
    tilesource: TS,
    img: RgbImage,
    rel_zoom: f64,
}

impl Renderer<TileServerSource> {
    pub fn new(res: (usize, usize), center: Coords) -> Self {
        Renderer {
            width: res.0,
            height: res.1,
            center,
            zoom: 3,
            tilesource: TileServerSource::new(),
            img: ImageBuffer::new(res.0 as u32, res.1 as u32),
            rel_zoom: 20.,
        }
    }

    pub fn zoom_in(&mut self) {
        self.zoom = std::cmp::min(self.zoom + 1, MAX_ZOOM);
    }

    pub fn zoom_out(&mut self) {
        if self.zoom > std::cmp::max(0, MIN_ZOOM) {
            self.zoom -= 1;
        }
    }

    pub fn clear_img(&mut self) {
        for (_, _, pixel) in self.img.enumerate_pixels_mut() {
            *pixel = image::Rgb([0, 0, 0]);
        }
    }

    pub fn draw(&mut self) {
        self.clear_img();

        let mut tiles: Vec<tile::Tile> = self.visible_tiles();

        tiles.iter_mut().for_each(|ref mut t| {
            t.vtile = self.tilesource.get_tile(t.z(), t.x(), t.y()).unwrap().vtile;
        });

        // for (x, y, pixel) in self.img.enumerate_pixels_mut() {
        //     let r = (100.0 + 0.2 * x as f32) as u8;
        //     let b = (100.0 + 0.2 * y as f32) as u8;
        //     *pixel = image::Rgb([r, 0, b]);
        // }
        for t in &tiles {
            self.draw_tile(&t);
        }
        self.img.save("test.png").unwrap();
    }
    pub fn draw_tile(&mut self, t: &tile::Tile) {
        let tdx = t.x();
        let tdy = t.y();

        let ox = t.offset.unwrap().0;
        let oy = t.offset.unwrap().1;

        debug!("tdx={}, tdy={}", tdx, tdy);

        let layer_colors = hashmap! {
        "aeroway" => Rgb([47, 79, 79]),
        "boundary" => Rgb([107, 142, 35]),
        "building" => Rgb([100, 149, 237]),
        "housenumber" => Rgb([192, 192, 192]),
        "landcover" => Rgb([221, 160, 221]),
        "landuse" => Rgb([106, 90, 205]),
        "mountain_peak" => Rgb([255, 248, 220]),
        "park" => Rgb([255, 228, 196]),
        "place" => Rgb([250, 128, 114]),
        "poi" => Rgb([250, 128, 114]),
        "transportation" => Rgb([250, 128, 114]),
        "transportation_name" => Rgb([250, 128, 114]),
        "water" => Rgb([250, 128, 114]),
        "water_name" => Rgb([250, 128, 114]),
        "waterway" => Rgb([250, 128, 114])
        };

        if let Some(vtile) = t.vtile.as_ref() {
            for layer in &vtile.layers {
                let extent = layer.extent();

                let color = layer_colors.get(layer.name.as_str()).unwrap_or(&Rgb([255,255,255]));

                error!("layer,{}", layer.name);

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
                                        let nc = (cursor.0 + dx, cursor.1 + dy);

                                        if self.point_within_bounds((nc.0 as f32, nc.1 as f32)) {
                                            self.img.put_pixel(
                                                nc.0 as u32,
                                                nc.1 as u32,
                                                Rgb([255, 255, 255]),
                                            );
                                        }
                                        cursor = nc;
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
                                        let nc = (cursor.0 + dx, cursor.1 + dy);
                                        self.draw_line_on_img(
                                            cursor, nc, extent, tdx, tdy, ox, oy, *color,
                                        );
                                        cursor = nc;
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
                                        let nc = (cursor.0 + dx, cursor.1 + dy);

                                        if (nc.0 - cursor.0).abs() > 0
                                            && (nc.1 - cursor.1).abs() > 0
                                        {
                                            self.draw_line_on_img(
                                                cursor, nc, extent, tdx, tdy, ox, oy, *color,
                                            );
                                        }
                                        cursor = nc;
                                    }
                                    tile::GeometryCommand::ClosePath => {
                                        self.draw_line_on_img(
                                            cursor,
                                            polygon_start,
                                            extent,
                                            tdx,
                                            tdy,
                                            ox,
                                            oy,
                                            *color,
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
        extent: u32,
        tdx: i32,
        tdy: i32,
        ox: u32,
        oy: u32,
        color: Rgb<u8>,
    ) {
        let from = (
            ((256 * ox) as f32 + p.0 as f32 * (256. / extent as f32)) * self.rel_zoom as f32,
            ((256 * oy) as f32 + p.1 as f32 * (256. / extent as f32)) * self.rel_zoom as f32,
        );
        let to = (
            ((256 * ox) as f32 + q.0 as f32 * (256. / extent as f32)) * self.rel_zoom as f32,
            ((256 * oy) as f32 + q.1 as f32 * (256. / extent as f32)) * self.rel_zoom as f32,
        );

        if self.point_within_bounds(from) && self.point_within_bounds(to) {
            imageproc::drawing::draw_line_segment_mut(
                &mut self.img,
                from,
                to,
                color, // Rgb([0u8, 0u8, 0u8]), // RGB colors
            );
        }
    }

    pub fn point_within_bounds(&self, p: (f32, f32)) -> bool {
        0. <= p.0 && p.0 < self.width as f32 && 0. <= p.1 && p.1 < self.height as f32
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

    pub fn visible_tiles(&self) -> Vec<tile::Tile> {
        let center_t = util::coords_to_tile(&self.center, self.zoom as f64);

        let tile_size = 256;

        let uncovered_right = ((self.width - tile_size) as f64
            / (2. * tile_size as f64 * self.rel_zoom))
            .ceil() as i32;
        let uncovered_up =
            ((self.height - tile_size) as f64 / 2.0 / (tile_size as f64) / self.rel_zoom).ceil()
                as i32;

        let max_tile = 2i32.pow(self.zoom) - 1;

        let mut tiles: Vec<tile::Tile> = Vec::new();
        for dx in (-uncovered_right)..(uncovered_right + 1) {
            for dy in (-uncovered_up)..(uncovered_up + 1) {
                let tx = center_t.x as i32 + dx;
                let ty = center_t.y as i32 + dy;

                if 0 <= tx && tx <= max_tile && 0 <= ty && ty <= max_tile {
                    tiles.push(tile::Tile {
                        zxy: (self.zoom as usize, tx, ty),
                        offset: Some(((dx + uncovered_right) as u32, (dy + uncovered_up) as u32)),
                        vtile: None,
                    });
                }
            }
        }

        tiles
    }
}
