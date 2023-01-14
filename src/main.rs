#![allow(unused_imports)]
#![allow(unused_variables)]
use pandemic;

fn main() {
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

    for city in game.world.values() {
        println!("{}", city);
    }
    println!();

    game.run();
}
