#![allow(unused_imports)]
#![allow(unused_variables)]

use clap::Parser;
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
    #[clap(long, default_value_t = 8.55)]
    lon: f64,
    #[clap(long, default_value_t = 47.3667)]
    lat: f64,
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

    let center = pandemic::util::Coords::from_deg(args.lon, args.lat);
    let zoom = 0.;

    let t = pandemic::util::coords_to_tile(&center, zoom);
    let (x, y) = (t.x, t.y);
    println!("Initial tile: z={zoom}, x={x}, y={y}");

    let res =
        reqwest::blocking::get(format!("{}/data/v3/{zoom}/{x}/{y}.pbf", args.tileserver)).unwrap();

    let buf = res.bytes().unwrap();
    println!("Raw size: {}", buf.len());

    let x = pandemic::vector_tile::Tile::decode(&mut std::io::Cursor::new(buf.clone())).unwrap();

    println!("Decoded x = {:#?}", &x);
    println!("Raw size: {}", buf.len());

    let renderer = pandemic::renderer::Renderer::new();
    println!(
        "Visible tiles: {:?}",
        renderer.visible_tiles(&center, zoom).len()
    );

    renderer.draw(&center, zoom);

    game.run();
}
