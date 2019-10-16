use ggez;
use ggez::event;
use ggez::graphics;
use ggez::nalgebra as na;
use entities;
use entities::{ UDDir, LRDir };
use serde_json;
use serde::{Deserialize}; // Serialize;
// use std::io;
use std::io::{Write, Read}; // Read;
use ggez::{GameResult, Context};
use std::net::TcpStream;
// use std::io::{BufReader, BufRead};
use std::time::Duration;

const UPDATE_STEP: f32 = 5.0;

type NetToken = u32;

// if connection is not established player will be at   players[0]
// else controllable player will be at                  players[connection.token]
struct MainState {
    game: entities::GameWorld,
    connection: Option<Connection>
}

struct Connection {
    socket: TcpStream,
    token: NetToken
}

impl Connection {
    fn new(socket: TcpStream) -> Result<Connection, String> {
        let mut de = serde_json::Deserializer::from_reader(&socket);
        let payload1 = entities::Player::deserialize(&mut de).unwrap();
        println!("{:?}", payload1);
        Ok(Connection {
            socket,
            token: 4,
        })
    }

    fn send(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        self.socket.write_all(data)?;
        Ok(())
    }

    fn get_new_players(&mut self) -> Option<entities::Player> {
        let mut buf = [0u8; 64];
        self.socket.set_read_timeout(Some(Duration::from_millis(10))).unwrap();

        match self.socket.read(&mut buf) {
            Ok(size) => {
                let player: entities::Player = serde_json::from_slice(
                    &buf[0..(size)]
                ).unwrap();
                println!("{:?}", player);
                Some(player)
            },
            Err(e) => { println!("Err {}", e); None }
        }
    }
}

impl MainState {
    fn new() -> GameResult<MainState> {
        let s = MainState {
            game: entities::GameWorld::new()?,
            connection: None
        };

        Ok(s)
    }

    fn connect(&mut self, host: String) -> Result<(), String> {
        match TcpStream::connect(host) {
            Ok(stream) => match Connection::new(stream) {
                Ok(connection) => {
                    println!("connection established, net_token= {}", connection.token);
                    self.connection = Some(connection);
                    Ok(())
                },
                Err(err) => Err(err)
            },
            Err(e) => Err(format!("{:?}", e.kind()))
        }
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        let mut main_player = &mut self.game.main_player;
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

        if let Some(ref mut connection) = self.connection {
            connection.send(&serde_json::to_string(&main_player).unwrap().into_bytes())?;
            if let Some(player) = connection.get_new_players() {
                self.game.players.push(player.clone());
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        for critter in self.game.objects.iter() {
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                na::Point2::new(critter.pos_x, critter.pos_y),
                (10 * critter.size) as f32,
                2.0,
                graphics::Color::new(critter.color.0, critter.color.1, critter.color.2, critter.color.3)
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

    fn key_down_event(
        &mut self,
        _ctx: &mut ggez::Context,
        keycode: event::KeyCode,
        _keymod: event::KeyMods,
        _repeat: bool,
    ) {
        let mut main_player = &mut self.game.main_player;
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
        let mut main_player = &mut self.game.main_player;
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
}

pub fn main() -> ggez::GameResult {
    let state = &mut MainState::new().unwrap();
    let cb = ggez::ContextBuilder::new("super_simple", "ggez");
    let (ctx, event_loop) = &mut cb.build()?;

    match state.connect("127.0.0.1:3012".to_string()) {
        Ok(_) => println!("connected"),
        Err(e) => println!("Failed to connect: {}", e)
    };

    event::run(ctx, event_loop, state)
}
