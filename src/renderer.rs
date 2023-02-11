use crate::tile::{self, BoundingBox, Tile};
use crate::tilesource::{CachedTileSource, TileServerSource, TileSource};
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

pub struct Renderer {
    width: usize,
    height: usize,
    center: Coords,
    pub zoom: u32,
    tilesource: Box<dyn TileSource>,
    img: RgbImage,
    rel_zoom: f64,
}

impl Renderer {
    pub fn new(res: (usize, usize), center: Coords) -> Self {
        Renderer {
            width: res.0,
            height: res.1,
            center,
            zoom: 0,
            tilesource: Box::new(CachedTileSource::unbounded(TileServerSource::new())),
            img: ImageBuffer::new(res.0 as u32, res.1 as u32),
            rel_zoom: 2.,
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
            *pixel = image::Rgb([255, 255, 255]);
        }
    }

    pub fn draw(&mut self) {
        self.clear_img();

        let mut tiles: Vec<tile::Tile> = self.visible_tiles();
        info!("There are {} visible tiles", tiles.len());

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
            // info!("Screen position of tile: {:?}", self.screen_position(&t));
        }
        self.img.save("test.png").unwrap();
    }
    pub fn draw_tile(&mut self, t: &tile::Tile) {
        info!("bounding box: {:?}", t.bounds());
        let layer_colors = hashmap! {
            "aeroway" => Rgb([47, 79, 79]),
            "boundary" => Rgb([107, 142, 35]),
            "building" => Rgb([100, 149, 237]),
            "housenumber" => Rgb([192, 192, 192]),
            "landcover" => Rgb([221, 160, 221]),
            "landuse" => Rgb([106, 90, 205]),
            "mountain_peak" => Rgb([255, 248, 220]),
            "park" => Rgb([255, 228, 196]),
            "place" => Rgb([253, 150, 147]),
            "poi" => Rgb([4, 88, 187]),
            "transportation" => Rgb([241, 5, 37]),
            "transportation_name" => Rgb([19, 79, 94]),
            "water" => Rgb([176, 1, 20]),
            "water_name" => Rgb([254, 70, 216]),
            "waterway" => Rgb([107, 41, 12])
        };

        let tile_screen_size = (256. * self.rel_zoom).round() as i32;
        for i in 0..tile_screen_size {
            for j in 0..tile_screen_size {
                let x = t.screenpos.0 + i;
                let y = t.screenpos.1 + j;
                if self.point_within_bounds((x as f32, y as f32)) {
                    self.img.put_pixel(x as u32, y as u32, Rgb([0, 0, 0]));
                }
            }
        }

        if let Some(vtile) = t.vtile.as_ref() {
            for layer in &vtile.layers {
                let extent = layer.extent();

                let color = layer_colors
                    .get(layer.name.as_str())
                    .unwrap_or(&Rgb([255, 255, 255]));

                info!("layer,{}", layer.name);

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
                                        self.draw_line_on_img(t, cursor, nc, extent, *color);
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
                                            self.draw_line_on_img(t, cursor, nc, extent, *color);
                                        }
                                        cursor = nc;
                                    }
                                    tile::GeometryCommand::ClosePath => {
                                        self.draw_line_on_img(
                                            t,
                                            cursor,
                                            polygon_start,
                                            extent,
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

        let radius = 5i32;
        for i in -radius..radius {
            for j in -radius..radius {
                if i.abs() + j.abs() <= radius {
                    self.img.put_pixel(
                        self.width as u32 / 2 + i as u32,
                        self.height as u32 / 2 + j as u32,
                        Rgb([0, 255, 0]),
                    );
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
        t: &Tile,
        p: (i32, i32),
        q: (i32, i32),
        extent: u32,
        color: Rgb<u8>,
    ) {
        let base_size = 256.;

        let from = (
            t.screenpos.0 as f32 + p.0 as f32 * base_size / extent as f32 * self.rel_zoom as f32,
            t.screenpos.1 as f32 + p.1 as f32 * base_size / extent as f32 * self.rel_zoom as f32,
        );

        let to = (
            t.screenpos.0 as f32 + q.0 as f32 * base_size / extent as f32 * self.rel_zoom as f32,
            t.screenpos.1 as f32 + q.1 as f32 * base_size / extent as f32 * self.rel_zoom as f32,
        );

        // let from = (
        //     ((256 * t.row.unwrap_or_default()) as f32 + p.0 as f32 * (256. / extent as f32))
        //         * self.rel_zoom as f32,
        //     ((256 * t.col.unwrap_or_default()) as f32 + p.1 as f32 * (256. / extent as f32))
        //         * self.rel_zoom as f32,
        // );
        // let to = (
        //     ((256 * t.row.unwrap_or_default()) as f32 + q.0 as f32 * (256. / extent as f32))
        //         * self.rel_zoom as f32,
        //     ((256 * t.col.unwrap_or_default()) as f32 + q.1 as f32 * (256. / extent as f32))
        //         * self.rel_zoom as f32,
        // );

        if self.point_within_bounds(from) && self.point_within_bounds(to) {
            imageproc::drawing::draw_line_segment_mut(
                &mut self.img,
                from,
                to,
                color, // Rgb([0u8, 0u8, 0u8]), // RGB colors
            );
        }
    }

    // Pixel coordinates of the top-left corner of a given tile, such that self.center is rendered
    // exactly at the center of the canvas.
    // pub fn screen_position(&self, t: &Tile) -> (u32, u32) {
    //     let b = t.bounds();

    //     info!(
    //         "Tile bounds are {:?}, renderer is centered on {:?}",
    //         b, self.center
    //     );

    //     let tile_size = 256.0 * self.rel_zoom;

    //     let (mut tx, mut ty) = (t.x(), t.y());

    //     let dx = (&self.center.lat - b.w) / (b.e - b.w);
    //     let dy = (&self.center.lon - b.s) / (b.n - b.s);

    //     let px = (self.width as f64 * dx).round() as u32;
    //     let py = (self.height as f64 * dy).round() as u32;

    //     (px, py)
    // }

    pub fn point_within_bounds(&self, p: (f32, f32)) -> bool {
        let x = p.0.round() as i32;
        let y = p.1.round() as i32;
        0 <= x && x < self.width as i32 && 0 <= y && y < self.height as i32
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
        let center = util::coords_to_tile(&self.center, self.zoom as f64);

        let tile_screen_size = 256.0 * self.rel_zoom;
        let lon = self.center.lon;
        let lat = self.center.lat;

        // center tile
        let mut ct = tile::Tile {
            zxy: (self.zoom as usize, center.x as i32, center.y as i32),
            ..Default::default()
        };

        let dx = (lon - ct.bounds().w) / (ct.bounds().e - ct.bounds().w);
        let dy = ((1. - ((lat * PI / 180.).tan() + 1. / (lat * PI / 180.).cos()).ln() / PI) / 2.
            * 2f64.powf(self.zoom as f64))
        .fract();
        ct.screenpos = (
            (self.width as f64 / 2. - tile_screen_size * dx).round() as i32,
            (self.height as f64 / 2. - tile_screen_size * dy).round() as i32,
        );

        let hcnt = 1 + (self.width as f64 / tile_screen_size).ceil() as i32;
        let vcnt = 1 + (self.height as f64 / tile_screen_size).ceil() as i32;
        info!("dx = {:.2}, dy = {:.2}", dx, dy);
        info!("hcnt = {}, vcnt = {}", hcnt, vcnt);

        let modulo = 2i32.pow(self.zoom);

        let mut tiles: Vec<tile::Tile> = Vec::new();
        for i in -vcnt..vcnt + 1 {
            for j in -hcnt..hcnt + 1 {
                let mut x = (j + center.x as i32 + 100 * modulo) % modulo;
                let mut y = (i + center.y as i32 + 100 * modulo) % modulo;

                let mut t = tile::Tile {
                    zxy: (self.zoom as usize, x, y),
                    screenpos: (
                        (self.width as f64 / 2. - tile_screen_size * dx
                            + j as f64 * tile_screen_size)
                            .round() as i32,
                        (self.height as f64 / 2. - tile_screen_size * dy
                            + i as f64 * tile_screen_size)
                            .round() as i32,
                    ),
                    ..Default::default()
                };

                let top_l = (t.screenpos.0 as f32, t.screenpos.1 as f32);
                if [(0, 0), (0, 1), (1, 0), (1, 1)]
                    .into_iter()
                    .map(|(i, j)| {
                        (
                            top_l.0 + i as f32 * tile_screen_size as f32,
                            top_l.1 + j as f32 * tile_screen_size as f32,
                        )
                    })
                    .any(|p| self.point_within_bounds(p))
                {
                    tiles.push(t);
                }
            }
        }

        tiles
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_pos() {

        // let r = Renderer::new((1920,1080),
        // for t in tests {
        //     let (ci, expected) = t;
        //     assert_eq!(Tile::parse_command_integer(ci), expected);
        // }
    }
}
