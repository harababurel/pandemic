use crate::util;
use crate::vector_tile::{self, tile::GeomType};
use std::collections::HashMap;
use std::f64::consts::PI;

#[derive(Debug, Default, Clone)]
pub struct Tile {
    pub zxy: (usize, i32, i32),
    pub offset: Option<(f64, f64)>,
    pub row: Option<i32>,
    pub col: Option<i32>,
    pub screenpos: (i32, i32),
    pub vtile: Option<vector_tile::Tile>,
}

#[derive(Debug)]
pub struct BoundingBox {
    pub n: f64,
    pub s: f64,
    pub e: f64,
    pub w: f64,
}

impl BoundingBox {
    pub fn new(n: f64, s: f64, e: f64, w: f64) -> Self {
        BoundingBox { n, s, e, w }
    }
    pub fn contains(&self, p: util::Coords) -> bool {
        self.w <= p.lon && p.lon <= self.e && self.s <= p.lat && p.lat <= self.n
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum GeometryCommand {
    MoveTo(i32, i32), // (dx, dy)
    LineTo(i32, i32), // (dx, dy)
    ClosePath,
}

impl Tile {
    pub fn from_proto(x: i32, y: i32, z: usize, vtile: vector_tile::Tile) -> Self {
        Tile {
            zxy: (z, x, y),
            vtile: Some(vtile),
            ..Default::default()
        }
    }

    pub fn x(&self) -> i32 {
        self.zxy.1
    }
    pub fn y(&self) -> i32 {
        self.zxy.2
    }
    pub fn z(&self) -> usize {
        self.zxy.0
    }

    pub fn parse_geometry(geometry: &Vec<u32>) -> Vec<GeometryCommand> {
        let mut ret: Vec<GeometryCommand> = Vec::new();
        let mut i = 0;
        while i < geometry.len() {
            let (c_id, count) = Tile::parse_command_integer(geometry[i]);
            i += 1;

            assert!(c_id == 1 || c_id == 2 || c_id == 7);
            let param_count = if c_id == 7 { 0 } else { 2 };

            for _ in 0..count {
                if c_id == 1 {
                    let dx = Tile::decode_parameter_integer(geometry[i]);
                    let dy = Tile::decode_parameter_integer(geometry[i + 1]);
                    ret.push(GeometryCommand::MoveTo(dx, dy));
                    i += 2;
                } else if c_id == 2 {
                    let dx = Tile::decode_parameter_integer(geometry[i]);
                    let dy = Tile::decode_parameter_integer(geometry[i + 1]);
                    ret.push(GeometryCommand::LineTo(dx, dy));
                    i += 2;
                } else if c_id == 7 {
                    ret.push(GeometryCommand::ClosePath);
                }
            }
        }
        ret
    }

    pub fn parse_command_integer(ci: u32) -> (u32, u32) {
        let c_id = ci & 0x7;
        let count = ci >> 3;
        (c_id, count)
    }
    pub fn decode_parameter_integer(pi: u32) -> i32 {
        (pi as i32 >> 1) ^ (-(pi as i32 & 1))
    }

    pub fn bounds(&self) -> BoundingBox {
        BoundingBox {
            n: Tile::tile2lat(self.y(), self.z()),
            s: Tile::tile2lat(self.y() + 1, self.z()),
            e: Tile::tile2lon(self.x() + 1, self.z()),
            w: Tile::tile2lon(self.x(), self.z()),
        }
    }

    fn tile2lon(x: i32, z: usize) -> f64 {
        x as f64 / 2f64.powf(z as f64) * 360. - 180.
    }

    fn tile2lat(y: i32, z: usize) -> f64 {
        let n = PI - (2. * PI * y as f64) / 2f64.powf(z as f64);
        n.sinh().atan().to_degrees()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_integer() {
        let tests = vec![
            (9, (1, 1)),
            (961, (1, 120)),
            (10, (2, 1)),
            (26, (2, 3)),
            (15, (7, 1)),
        ];
        for t in tests {
            let (ci, expected) = t;
            assert_eq!(Tile::parse_command_integer(ci), expected);
        }
    }

    #[test]
    fn test_parse_geometry() {
        let geometry: Vec<u32> = vec![9, 6, 12, 18, 10, 12, 24, 44, 15];
        let expected_commands = vec![
            GeometryCommand::MoveTo(3, 6),
            GeometryCommand::LineTo(5, 6),
            GeometryCommand::LineTo(12, 22),
            GeometryCommand::ClosePath,
        ];
        assert_eq!(Tile::parse_geometry(&geometry), expected_commands);
    }

    #[test]
    fn test_tile_bounds() {
        let eps = 0.1;
        let tests = vec![((0, 0, 0), BoundingBox::new(85., -85., 180., -180.))];

        for (zxy, b) in tests {
            let t = Tile {
                zxy,
                ..Default::default()
            };

            for (exp, got, dir) in vec![
                (b.n, t.bounds().n, "north"),
                (b.s, t.bounds().s, "south"),
                (b.e, t.bounds().e, "east"),
                (b.w, t.bounds().w, "west"),
            ] {
                assert!(
                    (exp - got).abs() < eps,
                    "Checking {} bound (got: {}, expected: {})",
                    dir,
                    got,
                    exp
                );
            }
        }
    }

    #[test]
    fn test_tile_bounds_cities() {
        use map_macro::map;
        let cities = map! {
            "Paris" => (48.8566, 2.349014),
            "Lyon" => (45.763420,4.834277),
            "Barcelona" => (41.3874, 2.1686),
            "Milan" => (45.4642, 9.1900),
            "Zurich" => (47.3769, 8.5417),
            "London" => (51.5072, -0.1276),
            "New York" => (40.7128, -74.0060),
            "Sofia" => (42.6977, 23.3219),
            "Valencia" => (39.4699, 0.3763),
            "Rome" => (41.9028, 12.4964),
            "Tokyo" => (35.6762, 139.6503),
            "Kyoto" =>(35.0116, 135.7681),
            "Osaka" =>(34.6937, 135.5023),
            "Nagasaki" =>(32.7503, 129.8779),
            "Asahi" =>(35.7205, 140.6466),
            "Fuji" =>(35.3606, 138.7274),
            "Maebashi" =>(36.3895, 139.0634),
        };

        let tests = vec![
            (
                (5, 16, 11), // covers most of western Europe
                vec!["Paris", "Lyon", "Barcelona", "Milan", "Zurich"],
                vec!["London", "New York", "Sofia", "Valencia", "Rome", "Tokyo"],
            ),
            (
                (8, 227, 100), // centered on Tokyo
                vec!["Tokyo"],
                vec![
                    "London", "New York", "Sofia", "Rome", "Paris", "Zurich", "Kyoto", "Osaka",
                    "Nagasaki", "Fuji", "Maebashi", "Asahi",
                ],
            ),
        ];

        for (zxy, includes, excludes) in tests {
            let t = Tile {
                zxy,
                ..Default::default()
            };

            assert!(
                includes.iter().all(|name| t
                    .bounds()
                    .contains(util::Coords::from_deg(cities[name].0, cities[name].1))),
                "Tile {:?} must contain all of these cities within its bounds: {:?}",
                zxy,
                includes
            );
            assert!(
                excludes.iter().all(|name| !t
                    .bounds()
                    .contains(util::Coords::from_deg(cities[name].0, cities[name].1))),
                "Tile {:?} must NOT contain any of these cities within its bounds: {:?}",
                zxy,
                excludes
            );
        }
    }
}
