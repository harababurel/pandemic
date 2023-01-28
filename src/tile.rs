use crate::vector_tile::{self, tile::GeomType};
use image::{GenericImage, GenericImageView, ImageBuffer, Rgb, RgbImage};

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
            let mut img: RgbImage = ImageBuffer::new(10000, 10000);

            for (x, y, pixel) in img.enumerate_pixels_mut() {
                let r = (0.1 * x as f32) as u8;
                let b = (0.1 * y as f32) as u8;
                *pixel = image::Rgb([r, 0, b]);
            }

            for layer in &vtile.layers {
                for feature in &layer.features {
                    let mut cursor = (0, 0);

                    let commands = Tile::parse_geometry(&feature.geometry);
                    println!("Commands: {:?}", commands);

                    match feature.r#type() {
                        GeomType::Unknown => {
                            panic!("Found unknown geometry, don't know how to interpret this");
                        }
                        GeomType::Point => {
                            for c in commands {
                                match c {
                                    GeometryCommand::MoveTo(dx, dy) => {
                                        cursor = (cursor.0 + dx, cursor.1 + dy);

                                        if 0 <= cursor.0 && 0 <= cursor.1 {
                                            img.put_pixel(
                                                cursor.0 as u32,
                                                cursor.1 as u32,
                                                Rgb([255, 255, 255]),
                                            );
                                        }
                                    }
                                    _ => {
                                        panic!("Point geometry can only contain MoveTo commands");
                                    }
                                };
                            }
                        }
                        GeomType::Linestring => {}
                        GeomType::Polygon => {}
                    }
                }
            }
            img.save("test.png").unwrap();
        }
    }

    pub fn parse_command_integer(ci: u32) -> (u32, u32) {
        let c_id = ci & 0x7;
        let count = ci >> 3;
        (c_id, count)
    }
}
