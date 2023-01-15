#![allow(unused_imports)]
#![allow(unused_variables)]
// use pandemic;
// extern crate pancurses;
use bevy::prelude::*;

// use pancurses::{endwin, initscr};

fn hello_world() {
    println!("hello world!");
}

fn main() {
    // let mut game =
    //     pandemic::Game::from_file("cities.json").expect("Could not create game from cities.json");

    // game.add_player(pandemic::Player::medic("Atlanta"))
    //     .expect("Could not create medic in Atlanta");
    // game.add_player(pandemic::Player::scientist("Atlanta"))
    //     .expect("Could not create medic in Atlanta");
    // game.setup();

    // println!("There are {} players", game.players.len());
    // println!("There are {} cities", game.world.len());
    // println!("There are {} player cards", game.player_cards.len());
    // println!(
    //     "There are {} infection cards ({} discarded)",
    //     game.infection_card_pile.len(),
    //     game.infection_discard_pile.len()
    // );
    // println!("Outbreaks: {}", game.outbreaks);
    // println!(
    //     "Infection rate: {} (level={})",
    //     game.infection_rate(),
    //     game.infection_level
    // );

    // for city in game.world.values() {
    //     println!("{}", city);
    // }
    // println!();

    // let actions = game.possible_actions(&game.players[0]);

    // println!(
    //     "{:#?} can perform one of {} actions:",
    //     &game.players[0],
    //     actions.len()
    // );

    // for action in actions {
    //     println!("{:?}", action);
    // }

    // let window = initscr();
    // window.printw("Hello Rust");
    // window.refresh();
    // window.getch();
    // endwin();

    // game.run();
    App::new()
        .add_plugins(DefaultPlugins)
        .add_system(hello_world)
        .run();
}
