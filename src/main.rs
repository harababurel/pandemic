#![allow(unused_imports)]
#![allow(unused_variables)]

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use clap::Parser;
use pancurses;
use pancurses::{endwin, initscr, Input};
use pandemic;
use pandemic::renderer::Renderer;
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

    let center = pandemic::util::Coords::from_deg(args.lat, args.lon);

    // let mut renderer = pandemic::renderer::Renderer::new((281*3, 69*5), center);
    let mut renderer = pandemic::renderer::BrailleRenderer::new((540, 400), center);

    let window = initscr();
    loop {
        window.printw(format!("Center: {:?}\n", renderer.center));
        window.printw(format!("Zoom: {}\n", renderer.zoom));
        window.printw(format!("Simplify: {}\n", renderer.simplify));
        if renderer.simplify {
            window.printw(format!("Tolerance: {:.2}\n", renderer.tolerance));
            window.printw(format!(
                "High Quality Simplification: {}\n",
                renderer.high_quality
            ));
        }
        renderer.draw();
        for line in renderer.to_braille() {
            window.printw(format!("{}\n", line));
        }

        match window.getch() {
            Some(Input::Character('a')) => {
                renderer.zoom_in();
            }
            Some(Input::Character('z')) => {
                renderer.zoom_out();
            }
            Some(Input::Character('+')) => {
                renderer.tolerance *= 1.5;
            }
            Some(Input::Character('-')) => {
                renderer.tolerance /= 1.5;
            }
            Some(Input::Character('g')) => {
                renderer.high_quality = !renderer.high_quality;
            }
            Some(Input::Character('s')) => {
                renderer.simplify = !renderer.simplify;
            }
            Some(Input::Character('l')) => {
                renderer.pan(pandemic::renderer::Direction::RIGHT);
            }
            Some(Input::Character('h')) => {
                renderer.pan(pandemic::renderer::Direction::LEFT);
            }
            Some(Input::Character('j')) => {
                renderer.pan(pandemic::renderer::Direction::DOWN);
            }
            Some(Input::Character('k')) => {
                renderer.pan(pandemic::renderer::Direction::UP);
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
