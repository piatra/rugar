use rand;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use serde::{Serialize, Deserialize};
use std::f32;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Message {
    pub mtype: MessageType,
    pub world: Option<Vec<Critter>>,
    pub player: Option<Player>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum MessageType {
    PlayerPosition,
    WorldState,
}

impl Message {
    pub fn player_update(p: &Player) -> Message {
        Message {
            mtype: MessageType::PlayerPosition,
            world: None,
            player: Some(Player::copy(p)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameWorld {
    pub players: Vec<Player>,
    pub main_player: Player,
    pub objects: Vec<Critter>
}

impl GameWorld {
    pub fn new() -> ggez::GameResult<GameWorld> {
        let mut critters = vec![];
        let mut rng = rand::thread_rng();
        for _ in 0..10 {
            let x = rng.gen_range(0.0, 800.0);
            let y = rng.gen_range(0.0, 800.0);
            let size = 10 * rng.gen_range(1, 10);
            critters.push(Critter { pos_x: x, pos_y: y, size, color: random_color() })
        }
        let world = GameWorld {
            players: vec![],
            main_player: Player::new(),
            objects: critters
        };
        Ok(world)
    }

    pub fn update_player(&mut self, player: Player) {
        if !self.main_player.same_player(&player) {
            if let Some(index) = self.players.iter().position(|x| x.name == player.name) {
                self.players[index] = player;
            } else {
                self.players.push(player);
            }
        }
    }
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Player {
    pub name: String,
    pub pos: Pos,
    pub size: u32,
    pub moving: (Option<LRDir>, Option<UDDir>),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct Pos {
    pos_x: f32,
    pos_y: f32,
}

impl Pos {
    pub fn new(x: f32, y: f32) -> Self {
        Pos { pos_x: x, pos_y: y }
    }

    pub fn x(&self) -> f32 {
        self.pos_x
    }

    pub fn y(&self) -> f32 {
        self.pos_y
    }

    fn inc_x(&mut self, val: f32) {
        self.pos_x += val;
    }

    fn inc_y(&mut self, val: f32) {
        self.pos_y += val;
    }

    pub fn move_player(&mut self, x: f32, y: f32) {
        self.inc_x(x);
        self.inc_y(y);
    }

    pub fn object_distance(&mut self, pos: Pos) -> f32 {
        ((self.pos_x - pos.x()) * (self.pos_x - pos.x()) +
        (self.pos_y - pos.y()) * (self.pos_y - pos.y())).sqrt()
    }

    pub fn object_distance_2(&mut self, pos: Pos) -> f32 {
        (self.pos_x - pos.x()) * (self.pos_x - pos.x()) +
        (self.pos_y - pos.y()) * (self.pos_y - pos.y())
    }
}

impl Player {
    pub fn random_username() -> String {
        thread_rng().sample_iter(&Alphanumeric)
            .take(30)
            .collect()
    }

    pub fn same_player(&self, p: &Player) -> bool {
        self.name == p.name
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    // (R0 - R1)^2 <= (x0 - x1)^2 + (y0 - y1)^2 <= (R0 + R1)^2
    // but only second part because we want to capture player inside critter
    pub fn intersect(&mut self, pos: Pos, size: u32) -> bool {
        let distance = self.pos.object_distance_2(pos);
        let high_r: f32 = ((self.size + size) * (self.size + size)) as f32;
        distance <= high_r
    }

    pub fn new() -> Player {
        Player {
            name: Player::random_username(),
            pos: Pos { pos_x: 0.0, pos_y: 0.0 },
            size: 10,
            moving: (None, None)
        }
    }

    pub fn copy(p: &Player) -> Player {
        Player {
            name: String::from(&p.name),
            pos: Pos { pos_x: p.pos.pos_x, pos_y: p.pos.pos_y },
            size: p.size,
            moving: (None, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dummy() {
        let mut obj = Pos { pos_x: 10.0, pos_y: 10.0 };
        assert_eq!(0.0, obj.object_distance(Pos { pos_x: 10.0, pos_y: 10.0 }));
    }

    #[test]
    fn test_distance() {
        let mut obj = Pos { pos_x: 10.0, pos_y: 10.0 };
        assert_eq!(5.0, obj.object_distance(Pos { pos_x: 13.0, pos_y: 14.0 }));
    }

    #[test]
    fn test_intersect_dummy() {
        let mut p = Player::new();
        assert_eq!(true, p.intersect(Pos { pos_x: 0.0, pos_y: 0.0 }, 1));
    }

    #[test]
    fn test_intersect_sized() {
        let mut p = Player::new();
        assert_eq!(true, p.intersect(Pos { pos_x: 103.0, pos_y: 104.0 }, 100));
    }
}
