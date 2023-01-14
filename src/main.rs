#![allow(unused_imports)]
#![allow(unused_variables)]
use pandemic;

fn main() {
    let mut game = pandemic::Game::new();
    game.load_cities("cities.json")
        .expect("Could not parse cities.json");
    game.create_player_cards();

    // println!("world: {:?}", world);
    println!("There are {} cities", game.world.len());
    println!("There are {} player cards", game.player_cards.len());
}
