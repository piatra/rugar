use ggez;
use ggez::event;
use ggez::graphics;
use ggez::nalgebra as na;
use rand;
use rand::Rng;
use entities;
use entities::{ UDDir, LRDir };
use ws;

const UPDATE_STEP: f32 = 5.0;

struct LocalGameWorld {
    game: entities::GameWorld,
}

struct Client {
    out: ws::Sender,
}

impl ws::Handler for Client {
    fn on_open(&mut self, _: ws::Handshake) -> ws::Result<()> {
        println!("Connected");
        let sender = self.out.clone();
        std::thread::spawn(move || {
            send_updates(sender);
        });
        Ok(())
    }
} 

fn send_updates(sender: ws::Sender) {}

impl event::EventHandler for LocalGameWorld {
    fn update(&mut self, _ctx: &mut ggez::Context) -> ggez::GameResult {
        let main_player = &mut self.game.main_player;
        let mut x: f32 = 0.0;
        let mut y: f32 = 0.0;
        if let Some(xx) = main_player.moving.0 {
            x = (xx as i32) as f32;
        } 
        if let Some(yy) = main_player.moving.1 {
            y = (yy as i32) as f32;
        } 
        main_player.pos_x += x * UPDATE_STEP;
        main_player.pos_y += y * UPDATE_STEP;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut ggez::Context,
        keycode: event::KeyCode,
        _keymod: event::KeyMods,
        _repeat: bool,
    ) {
        let main_player = self.game.main_player;
        match keycode {
            event::KeyCode::Space => {
                self.game.players.push(Default::default());
            }
            event::KeyCode::Up => {
                main_player.moving = (main_player.moving.0, Some(UDDir::Up));
            }
            event::KeyCode::Down => {
                main_player.moving = (main_player.moving.0, Some(UDDir::Down));
            }
            event::KeyCode::Left => {
                main_player.moving = (Some(LRDir::Left), main_player.moving.1);
            }
            event::KeyCode::Right => {
                main_player.moving = (Some(LRDir::Right), main_player.moving.1);
            }
            _ => ()
        }
    }

    fn key_up_event(&mut self, _ctx: &mut ggez::Context, keycode: event::KeyCode, _keymod: event::KeyMods) {
        let main_player = self.game.main_player;
        match (keycode, main_player.moving) {
            (event::KeyCode::Up, (_, Some(UDDir::Up))) => {
                main_player.moving = (main_player.moving.0, None);
            }
            (event::KeyCode::Down, (_, Some(UDDir::Down))) => {
                main_player.moving = (main_player.moving.0, None);
            }
            (event::KeyCode::Left, (Some(LRDir::Left), _)) => {
                main_player.moving = (None, main_player.moving.1);
            }
            (event::KeyCode::Right, (Some(LRDir::Right), _)) => {
                main_player.moving = (None, main_player.moving.1);
            }
            _ => ()
        }
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        for critter in self.game.critters.iter() {
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                na::Point2::new(critter.pos_x, critter.pos_y),
                (10 * critter.size) as f32,
                2.0,
                entities::random_color(),
            )?;
            graphics::draw(ctx, &circle, (na::Point2::new(0.0, 0.0),))?;
        }

        for player in self.game.players.iter() {
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
            na::Point2::new(self.game.main_player.pos_x, self.game.main_player.pos_y),
            (100 * self.game.main_player.size) as f32,
            2.0,
            graphics::WHITE,
        )?;
        graphics::draw(ctx, &circle, (na::Point2::new(0.0, 0.0),))?;
        graphics::present(ctx)?;

        Ok(())
    }
}

pub fn main() -> ggez::GameResult { 
    ws::connect("ws://127.0.0.1:3012", |out| { Client { out: out } }).unwrap();
    let cb = ggez::ContextBuilder::new("super_simple", "ggez");
    let (ctx, event_loop) = &mut cb.build().unwrap();
    let state = &mut LocalGameWorld { game: entities::GameWorld::new()? };
    event::run(ctx, event_loop, state)
}