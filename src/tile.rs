use crate::vector_tile::{self, tile::GeomType};
use std::f64::consts::PI;

#[derive(Debug)]
pub struct Tile {
    pub zxy: (usize, i32, i32),
    // Coordinates in screen space. Top-left is tile (0, 0)
    pub offset: Option<(u32, u32)>,
    pub vtile: Option<vector_tile::Tile>,
}

#[derive(Debug)]
pub struct BoundingBox {
    north: f64,
    east: f64,
    south: f64,
    west: f64,
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
            offset: None,
            vtile: Some(vtile),
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
            north: Tile::tile2lat(self.y(), self.z()),
            south: Tile::tile2lat(self.y() + 1, self.z()),
            west: Tile::tile2lon(self.x(), self.z()),
            east: Tile::tile2lon(self.x() + 1, self.z()),
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
}
