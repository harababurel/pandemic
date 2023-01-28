use crate::vector_tile::{self, tile::GeomType};

#[derive(Debug)]
pub struct Tile {
    pub xyz: (i32, i32, usize),
    pub zoom: f64,
    pub position: (f64, f64),
    pub size: f64,
    pub vtile: Option<vector_tile::Tile>,
}

#[derive(Debug)]
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
                    let dx = (geometry[i] as i32 >> 1) ^ (-(geometry[i] as i32 & 1));
                    let dy = (geometry[i + 1] as i32 >> 1) ^ (-(geometry[i + 1] as i32 & 1));
                    ret.push(GeometryCommand::MoveTo(dx, dy));
                    i += 2;
                } else if c_id == 2 {
                    let dx = (geometry[i] as i32 >> 1) ^ (-(geometry[i] as i32 & 1));
                    let dy = (geometry[i + 1] as i32 >> 1) ^ (-(geometry[i + 1] as i32 & 1));
                    ret.push(GeometryCommand::LineTo(dx, dy));
                    i += 2;
                } else if c_id == 7 {
                    ret.push(GeometryCommand::ClosePath);
                }
            }
        }
        ret
    }

    pub fn process(&self) {
        if let Some(vtile) = self.vtile.as_ref() {
            for layer in &vtile.layers {
                for feature in &layer.features {
                    let mut cursor = (0, 0);

                    let commands = Tile::parse_geometry(&feature.geometry);
                    println!("Commands: {:?}", commands);

                    match feature.r#type() {
                        GeomType::Unknown => {
                            panic!("Found unknown geometry, don't know how to interpret this");
                        }
                        GeomType::Point => {}
                        GeomType::Linestring => {}
                        GeomType::Polygon => {}
                    }
                }
            }
        }
    }

    pub fn parse_command_integer(ci: u32) -> (u32, u32) {
        let c_id = ci & 0x7;
        let count = ci >> 3;
        (c_id, count)
    }
}
