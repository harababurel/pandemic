#![allow(unused_imports)]
#![allow(unused_variables)]
use pandemic;

fn main() {
    let mut game = pandemic::Game::new();
    game.load_cities("cities.json")
        .expect("Could not parse cities.json");
    game.create_player_cards();

    game.add_player(pandemic::Player::medic("Atlanta"))
        .expect("Could not create medic in Atlanta");

    game.add_player(pandemic::Player::scientist("Atlanta"))
        .expect("Could not create medic in Atlanta");

    println!("There are {} players", game.players.len());
    println!("There are {} cities", game.world.len());
    println!("There are {} player cards", game.player_cards.len());

    for city in game.world.values() {
        println!("{}", city);
    }
}
