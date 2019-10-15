use ggez;
use ggez::event;
use ggez::graphics;
use ggez::nalgebra as na;
use rand;
use rand::Rng;

const UPDATE_STEP: f32 = 5.0;

struct GameWorld {
    players: Vec<Player>,
    main_player: Player,
    critters: Vec<Critter>
}

#[derive(Copy, Clone)]
enum LRDir {
    Left = -1,
    Right = 1,
}

#[derive(Copy, Clone)]
enum UDDir {
    Up = -1,
    Down = 1,
}

struct Critter {
    pos_x: f32,
    pos_y: f32,
    size: u32,
    color: graphics::Color,
}

impl Critter {
    pub fn random_color() -> graphics::Color {
        let mut rng = rand::thread_rng();
        graphics::Color::new(
            rng.gen_range(0.0, 1.0),
            rng.gen_range(0.0, 1.0),
            rng.gen_range(0.0, 1.0),
            1.0,
        )
    }
}

struct Player {
    pos_x: f32,
    pos_y: f32,
    size: u32,
    moving: (Option<LRDir>, Option<UDDir>),
}

impl Default for Player {
    fn default() -> Self {
        Player { pos_x: 0.0, pos_y: 0.0, size: 1, moving: (None, None) }
    }
}

impl GameWorld {
    fn new() -> ggez::GameResult<GameWorld> {
        let mut critters = vec![];
        let mut rng = rand::thread_rng();
        for _ in 0..10 {
            let x = rng.gen_range(0.0, 800.0);
            let y = rng.gen_range(0.0, 800.0);
            let size = rng.gen_range(1, 10);
            critters.push(Critter { pos_x: x, pos_y: y, size: size, color: Critter::random_color() })
        }
        let world = GameWorld { players: vec![], main_player: Default::default(), critters };
        Ok(world)
    }
}

impl event::EventHandler for GameWorld {
    fn update(&mut self, _ctx: &mut ggez::Context) -> ggez::GameResult {
        let mut x: f32 = 0.0;
        let mut y: f32 = 0.0;
        if let Some(xx) = self.main_player.moving.0 {
            x = (xx as i32) as f32;
        } 
        if let Some(yy) = self.main_player.moving.1 {
            y = (yy as i32) as f32;
        } 
        self.main_player.pos_x += x * UPDATE_STEP;
        self.main_player.pos_y += y * UPDATE_STEP;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut ggez::Context,
        keycode: event::KeyCode,
        _keymod: event::KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            event::KeyCode::Space => {
                self.players.push(Default::default());
            }
            event::KeyCode::Up => {
                self.main_player.moving = (self.main_player.moving.0, Some(UDDir::Up));
            }
            event::KeyCode::Down => {
                self.main_player.moving = (self.main_player.moving.0, Some(UDDir::Down));
            }
            event::KeyCode::Left => {
                self.main_player.moving = (Some(LRDir::Left), self.main_player.moving.1);
            }
            event::KeyCode::Right => {
                self.main_player.moving = (Some(LRDir::Right), self.main_player.moving.1);
            }
            _ => ()
        }
    }

    fn key_up_event(&mut self, _ctx: &mut ggez::Context, keycode: event::KeyCode, _keymod: event::KeyMods) {
        match (keycode, self.main_player.moving) {
            (event::KeyCode::Up, (_, Some(UDDir::Up))) => {
                self.main_player.moving = (self.main_player.moving.0, None);
            }
            (event::KeyCode::Down, (_, Some(UDDir::Down))) => {
                self.main_player.moving = (self.main_player.moving.0, None);
            }
            (event::KeyCode::Left, (Some(LRDir::Left), _)) => {
                self.main_player.moving = (None, self.main_player.moving.1);
            }
            (event::KeyCode::Right, (Some(LRDir::Right), _)) => {
                self.main_player.moving = (None, self.main_player.moving.1);
            }
            _ => ()
        }
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        for critter in self.critters.iter() {
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                na::Point2::new(critter.pos_x, critter.pos_y),
                (10 * critter.size) as f32,
                2.0,
                critter.color,
            )?;
            graphics::draw(ctx, &circle, (na::Point2::new(0.0, 0.0),))?;
        }

        for player in self.players.iter() {
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                na::Point2::new(player.pos_x, player.pos_y),
                (100 * player.size) as f32,
                2.0,
                graphics::WHITE,
            )?;
            graphics::draw(ctx, &circle, (na::Point2::new(0.0, 0.0),))?;
        }

        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            na::Point2::new(self.main_player.pos_x, self.main_player.pos_y),
            (100 * self.main_player.size) as f32,
            2.0,
            graphics::WHITE,
        )?;
        graphics::draw(ctx, &circle, (na::Point2::new(0.0, 0.0),))?;
        graphics::present(ctx)?;

        Ok(())
    }
}

pub fn main() -> ggez::GameResult { 
    let cb = ggez::ContextBuilder::new("super_simple", "ggez");
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut GameWorld::new()?;
    event::run(ctx, event_loop, state)
}
