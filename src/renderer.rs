use crate::tile::{self, BoundingBox, GeometryCommand, Tile};
use crate::tilesource::{CachedTileSource, TileServerSource, TileSource};
use crate::util;
use crate::util::Coords;
use crate::vector_tile;
use crate::vector_tile::tile::GeomType;
use braille::BRAILLE;
use bresenham;
use image::{GenericImage, GenericImageView, ImageBuffer, Rgb, RgbImage};
use rand::{thread_rng, Rng};
use simplify_polyline as sp;
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

pub struct ImageRenderer {
    width: usize,
    height: usize,
    pub center: Coords,
    pub zoom: u32,
    tilesource: Box<dyn TileSource>,
    img: RgbImage,
    rel_zoom: f64,
    pub simplify: bool,
    pub tolerance: f64,
    pub high_quality: bool,
}

pub struct BrailleRenderer {
    width: usize,
    height: usize,
    pub center: Coords,
    pub zoom: u32,
    tilesource: Box<dyn TileSource>,
    buf: Vec<Vec<bool>>,
    rel_zoom: f64,
    pub simplify: bool,
    pub tolerance: f64,
    pub high_quality: bool,
}

pub enum Direction {
    UP = 0,
    DOWN,
    LEFT,
    RIGHT,
}

pub trait Renderer {
    fn new(res: (usize, usize), center: Coords) -> Self;
    // fn width(&self) -> i32;
    // fn height(&self) -> i32;

    fn zoom_in(&mut self);
    fn zoom_out(&mut self);
    fn pan(&mut self, d: Direction);

    fn clear_buf(&mut self);

    fn draw(&mut self);
    fn draw_tile(&mut self, t: &tile::Tile);
    fn draw_line(&mut self, t: &Tile, p: (i32, i32), q: (i32, i32), extent: u32, color: Rgb<u8>);
    fn visible_tiles(&mut self) -> Vec<tile::Tile>;
}

impl Renderer for BrailleRenderer {
    fn new(res: (usize, usize), center: Coords) -> Self {
        BrailleRenderer {
            width: res.0,
            height: res.1,
            center,
            zoom: 0,
            tilesource: Box::new(CachedTileSource::unbounded(TileServerSource::new())),
            buf: vec![vec![false; res.1]; res.0],
            rel_zoom: 2.,
            simplify: false,
            tolerance: 1.,
            high_quality: false,
        }
    }
    fn zoom_in(&mut self) {
        // self.zoom = std::cmp::min(self.zoom + 1, MAX_ZOOM);

        if (self.rel_zoom + 0.2).floor() != self.rel_zoom.floor() {
            self.rel_zoom = self.rel_zoom.floor();
            self.zoom += 1;
        } else {
        self.rel_zoom += 0.2;
        }
    }
    fn zoom_out(&mut self) {
        // if self.zoom > std::cmp::max(0, MIN_ZOOM) {
        //     self.zoom -= 1;
        // }

        // self.rel_zoom -= 0.2;
    }
    fn pan(&mut self, d: Direction) {
        let scaler =  2f64.powf(self.zoom as f64);
        match d {
            Direction::RIGHT => {
                self.center.lon += 5. / scaler;
            }
            Direction::LEFT => {
                self.center.lon -= 5. / scaler;
            }
            Direction::UP => {
                self.center.lat = (self.center.lat + 5.  / scaler ).min(80.);
            }
            Direction::DOWN => {
                self.center.lat = (self.center.lat - 5.  / scaler).max(-80.);
            }
        };
    }

    fn clear_buf(&mut self) {
        for line in &mut self.buf {
            line.fill(false);
        }
    }

    fn draw(&mut self) {
        self.clear_buf();

        let mut tiles: Vec<tile::Tile> = self.visible_tiles();
        info!("There are {} visible tiles", tiles.len());

        for t in &tiles {
            self.draw_tile(&t);
            // info!("Screen position of tile: {:?}", self.screen_position(&t));
        }
    }
    fn draw_tile(&mut self, t: &tile::Tile) {
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
                // if self.point_within_bounds((x, y)) {
                //     self.buf[y as usize][x as usize] = false;
                // }
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
                    let commands = tile::Tile::parse_geometry(&feature.geometry);

                    match feature.r#type() {
                        GeomType::Unknown => {
                            panic!("Found unknown geometry, don't know how to interpret this");
                        }
                        GeomType::Point => {
                            let mut cursor = (0, 0);
                            for c in commands {
                                match c {
                                    tile::GeometryCommand::MoveTo(dx, dy) => {
                                        let nc = (cursor.0 + dx, cursor.1 + dy);

                                        let p = self.tile_point_to_screen_space(t, nc, extent);
                                        let p = (p.x.round() as i32, p.y.round() as i32);

                                        if self.point_within_bounds(p) {
                                            self.buf[p.0 as usize][p.1 as usize] = true;
                                        }
                                        cursor = nc;
                                    }
                                    _ => {
                                        panic!("Point geometry can only contain MoveTo commands");
                                    }
                                };
                            }
                        }
                        GeomType::Linestring | GeomType::Polygon => {
                            let mut lines: Vec<Vec<sp::Point<f32>>> = self
                                .commands_to_polylines(&commands)
                                .into_iter()
                                .map(|line| {
                                    line.into_iter()
                                        .map(|p| self.tile_point_to_screen_space(t, p, extent))
                                        .collect()
                                })
                                .collect();

                            if self.simplify {
                                for i in 0..lines.len() {
                                    let before = lines[i].len();
                                    lines[i] = sp::simplify(&lines[i], self.tolerance, false);
                                    let after = lines[i].len();

                                    info!("Simplified from {} points to {} points", before, after);
                                }
                            }

                            lines.iter().for_each(|line| {
                                line.iter().zip(line.iter().skip(1)).for_each(|(p, q)| {
                                    let p = (p.x.round() as i32, p.y.round() as i32);
                                    let q = (q.x.round() as i32, q.y.round() as i32);
                                    self.draw_line(t, p, q, extent, *color);
                                });
                            });
                        }
                    }
                }
            }
        }

        // Draw screen center
        // let radius = 5i32;
        // for i in -radius..radius {
        //     for j in -radius..radius {
        //         if i.abs() + j.abs() <= radius {
        //             self.img.put_pixel(
        //                 self.width as u32 / 2 + i as u32,
        //                 self.height as u32 / 2 + j as u32,
        //                 Rgb([0, 255, 0]),
        //             );
        //         }
        //     }
        // }
    }
    fn draw_line(&mut self, t: &Tile, p: (i32, i32), q: (i32, i32), extent: u32, color: Rgb<u8>) {
        if self.point_within_bounds(p)
            && self.point_within_bounds(q)
            && (self.point_within_tile_bounds(t, p) || self.point_within_tile_bounds(t, q))
        {
            for x in bresenham::Bresenham::new(
                (p.0 as isize, p.1 as isize),
                (q.0 as isize, q.1 as isize),
            ) {
                self.buf[x.0 as usize][x.1 as usize] = true;
            }
        }
    }
    fn visible_tiles(&mut self) -> Vec<tile::Tile> {
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
                let mut x = (j + center.x as i32) % modulo;
                let mut y = (i + center.y as i32) % modulo;

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

                let top_l = t.screenpos;
                let bot_r = (
                    top_l.0 + tile_screen_size.round() as i32,
                    top_l.1 + tile_screen_size.round() as i32,
                );

                if util::rectangles_intersect(
                    (top_l, bot_r),
                    ((0, 0), (self.width as i32, self.height as i32)),
                ) {
                    tiles.push(t);
                }
            }
        }

        tiles.iter_mut().for_each(|ref mut t| {
            match self.tilesource.get_tile(t.z(), t.x(), t.y()) {
                Ok(tile) => {
                    t.vtile = tile.vtile;
                }
                Err(e) => {
                    error!("Could not get vector tile: {}", e);
                }
            }
        });

        tiles
    }
}

impl BrailleRenderer {
    pub fn point_within_bounds(&self, p: (i32, i32)) -> bool {
        let (x, y) = p;
        0 <= x && x < self.width as i32 && 0 <= y && y < self.height as i32
    }

    pub fn point_within_tile_bounds(&self, t: &Tile, p: (i32, i32)) -> bool {
        let (x, y) = p;
        let tile_screen_size = (256. * self.rel_zoom).round() as i32;
        t.screenpos.0 < x
            && x < t.screenpos.0 + tile_screen_size
            && t.screenpos.1 < y
            && y < t.screenpos.1 + tile_screen_size
    }
    // Each vec of points represents a polyline. There are potentially multiple polylines.
    pub fn commands_to_polylines(&self, commands: &Vec<GeometryCommand>) -> Vec<Vec<(i32, i32)>> {
        let mut lines = Vec::new();
        let mut line = Vec::new();

        let mut cursor = (0, 0);
        for c in commands {
            match c {
                tile::GeometryCommand::MoveTo(dx, dy) => {
                    cursor = (cursor.0 + dx, cursor.1 + dy);

                    if !line.is_empty() {
                        lines.push(line);
                    }
                    line = Vec::new();
                }
                tile::GeometryCommand::LineTo(dx, dy) => {
                    if line.is_empty() || line.last().unwrap() != &cursor {
                        line.push(cursor);
                    }
                    cursor = (cursor.0 + dx, cursor.1 + dy);
                    line.push(cursor);
                }
                tile::GeometryCommand::ClosePath => {
                    // line.push(line[0]);
                }
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }

        lines
    }
    pub fn tile_point_to_screen_space(
        &mut self,
        t: &Tile,
        p: (i32, i32),
        extent: u32,
    ) -> sp::Point<f32> {
        let base_size = 256.;
        sp::Point {
            x: t.screenpos.0 as f32 + p.0 as f32 * base_size / extent as f32 * self.rel_zoom as f32,
            y: t.screenpos.1 as f32 + p.1 as f32 * base_size / extent as f32 * self.rel_zoom as f32,
        }
    }

    pub fn to_braille(&mut self) -> Vec<String> {
        let mut lines = Vec::new();
        for y in 0..(self.height/4) {
            let mut s = String::with_capacity(self.width/2);
            for x in 0..(self.width/2) {
                s.push(
                    BRAILLE
                    [self.buf[2*x][4 * y] as usize][self.buf[2 * x+1][4 * y] as usize]
                    [self.buf[2*x][4 * y+1] as usize][self.buf[2 * x+1][4 * y + 1] as usize]
                    [self.buf[2*x][4 * y+2] as usize][self.buf[2 * x+1][4 * y + 2] as usize]
                    [self.buf[2*x][4 * y+3] as usize][self.buf[2 * x+1][4 * y + 3] as usize]
                );
            }
            lines.push(s);
        }
        lines
    }
}

impl Renderer for ImageRenderer {
    fn new(res: (usize, usize), center: Coords) -> Self {
        ImageRenderer {
            width: res.0,
            height: res.1,
            center,
            zoom: 0,
            tilesource: Box::new(CachedTileSource::unbounded(TileServerSource::new())),
            img: ImageBuffer::new(res.0 as u32, res.1 as u32),
            rel_zoom: 10.,
            simplify: false,
            tolerance: 1.,
            high_quality: false,
        }
    }
    fn zoom_in(&mut self) {
        self.zoom = std::cmp::min(self.zoom + 1, MAX_ZOOM);
    }
    fn zoom_out(&mut self) {
        if self.zoom > std::cmp::max(0, MIN_ZOOM) {
            self.zoom -= 1;
        }
    }
    fn pan(&mut self, d: Direction) {
        match d {
            Direction::RIGHT => {
                self.center.lon += 10.;
            }
            Direction::LEFT => {
                self.center.lon -= 10.;
            }
            Direction::UP => {
                self.center.lat = (self.center.lat + 10.).min(80.);
            }
            Direction::DOWN => {
                self.center.lat = (self.center.lat - 10.).max(-80.);
            }
        };
    }

    fn clear_buf(&mut self) {
        for (_, _, pixel) in self.img.enumerate_pixels_mut() {
            *pixel = image::Rgb([255, 255, 255]);
        }
    }

    fn draw(&mut self) {
        self.clear_buf();

        let mut tiles: Vec<tile::Tile> = self.visible_tiles();
        info!("There are {} visible tiles", tiles.len());

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
    fn draw_tile(&mut self, t: &tile::Tile) {
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
                if self.point_within_bounds((x, y)) {
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
                    let commands = tile::Tile::parse_geometry(&feature.geometry);

                    match feature.r#type() {
                        GeomType::Unknown => {
                            panic!("Found unknown geometry, don't know how to interpret this");
                        }
                        GeomType::Point => {
                            let mut cursor = (0, 0);
                            for c in commands {
                                match c {
                                    tile::GeometryCommand::MoveTo(dx, dy) => {
                                        let nc = (cursor.0 + dx, cursor.1 + dy);

                                        let p = self.tile_point_to_screen_space(t, nc, extent);
                                        let p = (p.x.round() as i32, p.y.round() as i32);

                                        if self.point_within_bounds(p) {
                                            self.img.put_pixel(
                                                p.0 as u32,
                                                p.1 as u32,
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
                        GeomType::Linestring | GeomType::Polygon => {
                            let mut lines: Vec<Vec<sp::Point<f32>>> = self
                                .commands_to_polylines(&commands)
                                .into_iter()
                                .map(|line| {
                                    line.into_iter()
                                        .map(|p| self.tile_point_to_screen_space(t, p, extent))
                                        .collect()
                                })
                                .collect();

                            if self.simplify {
                                for i in 0..lines.len() {
                                    let before = lines[i].len();
                                    lines[i] = sp::simplify(&lines[i], self.tolerance, false);
                                    let after = lines[i].len();

                                    info!("Simplified from {} points to {} points", before, after);
                                }
                            }

                            lines.iter().for_each(|line| {
                                line.iter().zip(line.iter().skip(1)).for_each(|(p, q)| {
                                    let p = (p.x.round() as i32, p.y.round() as i32);
                                    let q = (q.x.round() as i32, q.y.round() as i32);
                                    self.draw_line(t, p, q, extent, *color);
                                });
                            });
                        }
                    }
                }
            }
        }

        // Draw screen center
        // let radius = 5i32;
        // for i in -radius..radius {
        //     for j in -radius..radius {
        //         if i.abs() + j.abs() <= radius {
        //             self.img.put_pixel(
        //                 self.width as u32 / 2 + i as u32,
        //                 self.height as u32 / 2 + j as u32,
        //                 Rgb([0, 255, 0]),
        //             );
        //         }
        //     }
        // }
    }
    fn draw_line(&mut self, t: &Tile, p: (i32, i32), q: (i32, i32), extent: u32, color: Rgb<u8>) {
        let fp = (p.0 as f32, p.1 as f32);
        let fq = (q.0 as f32, q.1 as f32);
        if self.point_within_bounds(p)
            && self.point_within_bounds(q)
            && (self.point_within_tile_bounds(t, p) || self.point_within_tile_bounds(t, q))
        {
            imageproc::drawing::draw_line_segment_mut(&mut self.img, fp, fq, color);
        }
    }
    fn visible_tiles(&mut self) -> Vec<tile::Tile> {
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
                let mut x = (j + center.x as i32) % modulo;
                let mut y = (i + center.y as i32) % modulo;

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

                let top_l = t.screenpos;
                let bot_r = (
                    top_l.0 + tile_screen_size.round() as i32,
                    top_l.1 + tile_screen_size.round() as i32,
                );

                if ImageRenderer::rectangles_intersect(
                    (top_l, bot_r),
                    ((0, 0), (self.width as i32, self.height as i32)),
                ) {
                    tiles.push(t);
                }
            }
        }

        tiles.iter_mut().for_each(|ref mut t| {
            match self.tilesource.get_tile(t.z(), t.x(), t.y()) {
                Ok(tile) => {
                    t.vtile = tile.vtile;
                }
                Err(e) => {
                    error!("Could not get vector tile: {}", e);
                }
            }
        });

        tiles
    }
}

impl ImageRenderer {
    // Each vec of points represents a polyline. There are potentially multiple polylines.
    pub fn commands_to_polylines(&self, commands: &Vec<GeometryCommand>) -> Vec<Vec<(i32, i32)>> {
        let mut lines = Vec::new();
        let mut line = Vec::new();

        let mut cursor = (0, 0);
        for c in commands {
            match c {
                tile::GeometryCommand::MoveTo(dx, dy) => {
                    cursor = (cursor.0 + dx, cursor.1 + dy);

                    if !line.is_empty() {
                        lines.push(line);
                    }
                    line = Vec::new();
                }
                tile::GeometryCommand::LineTo(dx, dy) => {
                    if line.is_empty() || line.last().unwrap() != &cursor {
                        line.push(cursor);
                    }
                    cursor = (cursor.0 + dx, cursor.1 + dy);
                    line.push(cursor);
                }
                tile::GeometryCommand::ClosePath => {
                    // line.push(line[0]);
                }
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }

        lines
    }

    pub fn get_tile_features(&self, tile: &tile::Tile, zoom: f64) {
        let draw_order = ImageRenderer::generate_draw_order(zoom);
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

    pub fn tile_point_to_screen_space(
        &mut self,
        t: &Tile,
        p: (i32, i32),
        extent: u32,
    ) -> sp::Point<f32> {
        let base_size = 256.;
        sp::Point {
            x: t.screenpos.0 as f32 + p.0 as f32 * base_size / extent as f32 * self.rel_zoom as f32,
            y: t.screenpos.1 as f32 + p.1 as f32 * base_size / extent as f32 * self.rel_zoom as f32,
        }
    }

    pub fn point_within_bounds(&self, p: (i32, i32)) -> bool {
        let (x, y) = p;
        0 <= x && x < self.width as i32 && 0 <= y && y < self.height as i32
    }

    pub fn point_within_tile_bounds(&self, t: &Tile, p: (i32, i32)) -> bool {
        let (x, y) = p;
        let tile_screen_size = (256. * self.rel_zoom).round() as i32;
        t.screenpos.0 < x
            && x < t.screenpos.0 + tile_screen_size
            && t.screenpos.1 < y
            && y < t.screenpos.1 + tile_screen_size
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

    fn rectangles_intersect(r1: ((i32, i32), (i32, i32)), r2: ((i32, i32), (i32, i32))) -> bool {
        (r1.0 .0 < r2.1 .0) && (r1.1 .0 > r2.0 .0) && (r1.0 .1 < r2.1 .1) && (r1.1 .1 > r2.0 .1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_pos() {

        // let r = ImageRenderer::new((1920,1080),
        // for t in tests {
        //     let (ci, expected) = t;
        //     assert_eq!(Tile::parse_command_integer(ci), expected);
        // }
    }
}
