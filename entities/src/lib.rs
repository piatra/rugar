use rand;
use rand::Rng;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameWorld {
    pub players: Vec<Player>,
    pub main_player: Player,
    pub objects: Vec<Critter>
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum LRDir {
    Left = -1,
    Right = 1,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum UDDir {
    Up = -1,
    Down = 1,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Critter {
    pub pos_x: f32,
    pub pos_y: f32,
    pub size: u32,
    pub color: (f32, f32, f32, f32),
}

pub fn random_color() -> (f32, f32, f32, f32) {
    let mut rng = rand::thread_rng();
    (rng.gen_range(0.0, 1.0),
    rng.gen_range(0.0, 1.0),
    rng.gen_range(0.0, 1.0),
    1.0)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    pub pos_x: f32,
    pub pos_y: f32,
    pub size: u32,
    pub moving: (Option<LRDir>, Option<UDDir>),
}

impl Default for Player {
    fn default() -> Self {
        Player { pos_x: 0.0, pos_y: 0.0, size: 1, moving: (None, None) }
    }
}

impl GameWorld {
    pub fn new() -> ggez::GameResult<GameWorld> {
        let mut critters = vec![];
        let mut rng = rand::thread_rng();
        for _ in 0..10 {
            let x = rng.gen_range(0.0, 800.0);
            let y = rng.gen_range(0.0, 800.0);
            let size = rng.gen_range(1, 10);
            critters.push(Critter { pos_x: x, pos_y: y, size, color: random_color() })
        }
        let world = GameWorld { players: vec![], main_player: Default::default(), objects: critters };
        Ok(world)
    }
}
