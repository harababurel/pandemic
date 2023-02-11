#![allow(unused_imports)]
#![allow(unused_variables)]

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use clap::Parser;
use pancurses;
use pancurses::{endwin, initscr, Input};
use pandemic;
use prost::Message;
use reqwest;
use std::collections::HashMap;
use std::io::prelude::*;

#[derive(Parser)]
struct Cli {
    #[clap(long, default_value_t = String::from("http://harababurel.com:8080"))]
    tileserver: String,
    // #[clap(long, default_value_t = 8.55)]
    // lon: f64,
    // #[clap(long, default_value_t = 47.3667)]
    // lat: f64,
    #[clap(long, default_value_t = -74.)]
    lon: f64,
    #[clap(long, default_value_t = 40.71)]
    lat: f64,
}

fn main() {
    let args = Cli::parse();
    pretty_env_logger::init();

    let mut game =
        pandemic::Game::from_file("cities.json").expect("Could not create game from cities.json");

    game.add_player(pandemic::Player::medic("Atlanta"))
        .expect("Could not create medic in Atlanta");
    game.add_player(pandemic::Player::scientist("Atlanta"))
        .expect("Could not create medic in Atlanta");
    game.setup();

    info!("There are {} players", game.players.len());
    info!("There are {} cities", game.world.len());
    info!("There are {} player cards", game.player_cards.len());
    info!(
        "There are {} infection cards ({} discarded)",
        game.infection_card_pile.len(),
        game.infection_discard_pile.len()
    );
    info!("Outbreaks: {}", game.outbreaks);
    info!(
        "Infection rate: {} (level={})",
        game.infection_rate(),
        game.infection_level
    );

    for city in game.world.values() {
        info!("{}", city);
    }

    let actions = game.possible_actions(&game.players[0]);

    info!(
        "{:#?} can perform one of {} actions:",
        &game.players[0],
        actions.len()
    );

    for action in actions {
        info!("{:?}", action);
    }

    // let t = pandemic::util::coords_to_tile(&center, zoom as f64);
    // let (x, y) = (t.x, t.y);
    // info!("Initial tile: z={zoom}, x={x}, y={y}");
    // let x = pandemic::vector_tile::Tile::decode(&mut std::io::Cursor::new(buf.clone())).unwrap();
    // info!("Decoded x = {:#?}", &x);
    // info!("Raw size: {}", buf.len());

    let center = pandemic::util::Coords::from_deg(args.lat, args.lon);

    let mut renderer = pandemic::renderer::Renderer::new((1920, 1080), center);

    let window = initscr();
    loop {
        window.printw(format!("Zoom: {}\n", renderer.zoom));
        renderer.draw();

        match window.getch() {
            Some(Input::Character('a')) => {
                renderer.zoom_in();
            }
            Some(Input::Character('z')) => {
                renderer.zoom_out();
            }
            Some(Input::Character('q')) => {
                endwin();
                break;
            }
            _ => {}
        }
        window.clear();
        window.refresh();
    }

    // game.run();
}
