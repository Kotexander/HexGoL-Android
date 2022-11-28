pub mod hex;
pub use hex::*;

const NEIGHBORS: [HexInt; 6] = [
    HexInt::new(1, 0),
    HexInt::new(0, 1),
    HexInt::new(-1, 0),
    HexInt::new(0, -1),
    HexInt::new(1, -1),
    HexInt::new(-1, 1),
];

use std::collections::HashMap;
type GameState = HashMap<HexInt, bool>;
pub struct HexGOL {
    // size: i32,
    game: GameState,
    game_back: GameState,
}
impl HexGOL {
    pub fn new(size: i32) -> Self {
        let mut game = HashMap::new();
        for q in -size..=size {
            for r in (-size).max(-q - size)..=size.min(-q + size) {
                game.insert(HexInt::new(q, r), false);
            }
        }
        let game_back = game.clone();

        Self {
            // size,
            game,
            game_back,
        }
    }
    pub fn update(&mut self) {
        for (hex, _state) in &self.game {
            *self.game_back.get_mut(hex).unwrap() = self.get_num_neighbors(hex) == 2;
        }

        std::mem::swap(&mut self.game, &mut self.game_back);
    }
    pub fn get(&self, hex: &HexInt) -> Option<&bool> {
        self.game.get(hex)
    }
    pub fn get_num_neighbors(&self, hex: &HexInt) -> i32 {
        if let Some(_) = self.get(&hex) {
            let mut num_neighbors = 0;
            for neighbor_hex in &NEIGHBORS {
                if let Some(true) = self.get(&(*hex + *neighbor_hex)) {
                    num_neighbors += 1;
                }
            }
            num_neighbors
        } else {
            0
        }
    }
    pub fn iter(&self) -> std::collections::hash_map::Iter<HexInt, bool> {
        self.game.iter()
    }
}

impl HexGOL {
    pub fn randomize(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for (_hex, cell) in &mut self.game {
            *cell = rng.gen() && rng.gen() && rng.gen();
        }
    }
}
