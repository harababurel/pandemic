use crate::vector_tile::{self, tile::GeomType};

#[derive(Debug)]
pub struct Tile {
    pub xyz: (i32, i32, usize),
    pub zoom: f64,
    pub position: (f64, f64),
    pub size: f64,
    pub vtile: Option<vector_tile::Tile>,
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
            xyz: (x, y, z),
            position: (0., 0.),
            size: 0.,
            zoom: 0.,
            vtile: Some(vtile),
        }
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
