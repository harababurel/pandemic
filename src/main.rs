#![allow(unused_imports)]
#![allow(unused_variables)]

use clap::Parser;
use osmpbf::{Element, ElementReader};
use osmpbfreader;
use pancurses;
use pandemic;
use prost::Message;
use reqwest;
use std::collections::HashMap;
use std::io::prelude::*;
// use pancurses::{endwin, initscr};

#[derive(Parser)]
struct Cli {
    #[clap(long, default_value_t = String::from("http://harababurel.com:8080"))]
    tileserver: String,
}

pub mod OSMPBF {
    pub mod fileformat {
        include!(concat!(env!("OUT_DIR"), "/osmpbf.rs"));
    }
}
pub mod vector_tile {
    include!(concat!(env!("OUT_DIR"), "/vector_tile.rs"));
}

fn main() {
    let args = Cli::parse();

    let mut game =
        pandemic::Game::from_file("cities.json").expect("Could not create game from cities.json");

    game.add_player(pandemic::Player::medic("Atlanta"))
        .expect("Could not create medic in Atlanta");
    game.add_player(pandemic::Player::scientist("Atlanta"))
        .expect("Could not create medic in Atlanta");
    game.setup();

    println!("There are {} players", game.players.len());
    println!("There are {} cities", game.world.len());
    println!("There are {} player cards", game.player_cards.len());
    println!(
        "There are {} infection cards ({} discarded)",
        game.infection_card_pile.len(),
        game.infection_discard_pile.len()
    );
    println!("Outbreaks: {}", game.outbreaks);
    println!(
        "Infection rate: {} (level={})",
        game.infection_rate(),
        game.infection_level
    );

    for city in game.world.values() {
        println!("{}", city);
    }
    println!();

    let actions = game.possible_actions(&game.players[0]);

    println!(
        "{:#?} can perform one of {} actions:",
        &game.players[0],
        actions.len()
    );

    for action in actions {
        println!("{:?}", action);
    }

    // let window = initscr();
    // window.printw("Hello Rust");
    // window.refresh();
    // window.getch();
    // endwin();
    //
    let lat_deg = 47.36667;
    let lon_deg = 8.55;
    let zoom = 8.;

    let (x, y) = pandemic::util::coords_to_tile(lon_deg, lat_deg, zoom);
    println!("Initial tile: z={zoom}, x={x}, y={y}");

    let res =
        reqwest::blocking::get(format!("{}/data/v3/{zoom}/{x}/{y}.pbf", args.tileserver)).unwrap();

    // pub fn deserialize_shirt(buf: &[u8]) -> Result<items::Shirt, prost::DecodeError> {

    let buf = res.bytes().unwrap();
    println!("Raw size: {}", buf.len());

    let x = vector_tile::Tile::decode(&mut std::io::Cursor::new(buf.clone())).unwrap();

    println!("Decoded x = {:#?}", &x);
    println!("Raw size: {}", buf.len());
    // println!("Decoded x.zlib_data = {:?}", &x.zlib_data().len());
    // println!("Decoded x.lzma_data = {:?}", &x.lzma_data().len());
    // println!(
    //     "Decoded x.OBSOLETE_bzip2_data = {:?}",
    //     &x.obsolete_bzip2_data().len()
    // );
    // OSMPBF::fileformat::decode

    // println!("text: {:?}", res.text().unwrap());
    // println!("res: {:#?}", &res);
    // println!("bytes: {:#?}", &res.bytes());

    // let mut f = std::fs::File::create("/tmp/file-modified.txt").unwrap();
    // f.write(&res.bytes().unwrap());

    // let reader = ElementReader::new(res);
    // reader
    //     .for_each(|element| {
    //         println!("Found element: {:?}", element);
    //         // if let Element::Way(_) = element {
    //         //     ways += 1;
    //         // }
    //     })
    //     .expect("reader error");
    //
    // let mut pbf = osmpbfreader::OsmPbfReader::new(res);
    // let mut nb = 0;
    // for obj in pbf.iter() {
    //     println!("object: {:?}", obj);
    //     nb += 1;
    // }
    // println!("{} objects", nb);

    game.run();
}
