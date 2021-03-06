use ggez;
use ggez::event;
use ggez::graphics;
use ggez::nalgebra as na;
use entities;
use entities::{ UDDir, LRDir };
use serde_json;
use serde::{Deserialize};
use ggez::{GameResult, Context};
use std::net::TcpStream;
use std::time::Duration;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

const UPDATE_STEP: f32 = 4.0;

struct MainState {
    game: entities::GameWorld,
    connection: Option<Connection>
}

struct Connection {
    socket: TcpStream,
    receiver: Receiver<entities::Message>,
    sender: Sender<entities::Message>,
}

fn get_players(sender: Sender<entities::Message>, socket: TcpStream) {
    loop {
        thread::sleep(Duration::from_millis(15));
        let mut de = serde_json::Deserializer::from_reader(&socket);
        if let Ok(payload) = entities::Message::deserialize(&mut de) {
            sender.send(payload).unwrap();
        }
    }
}

impl Connection {
    fn new(socket: TcpStream) -> Result<Connection, String> {
        let (sender, receiver) = mpsc::channel();
        Ok(Connection {
            socket,
            sender,
            receiver
        })
    }

    fn send(&self, player: &entities::Player) -> Result<(), serde_json::error::Error> {
        let message = entities::Message::player_update(player);
        serde_json::to_writer(&self.socket, &message)
    }

    fn listen(&self) {
        let sender_clone = self.sender.clone();
        let socket_clone = self.socket.try_clone().unwrap();
        thread::spawn(move || get_players(sender_clone, socket_clone));
    }
}

impl MainState {
    fn new() -> GameResult<MainState> {
        let s = MainState {
            game: entities::GameWorld::ggez_new()?,
            connection: None
        };

        Ok(s)
    }

    fn connect(&mut self, host: String, player: &entities::Player) -> Result<(), String> {
        match TcpStream::connect(host) {
            Ok(stream) => match Connection::new(stream) {
                Ok(connection) => {
                    connection.listen();
                    connection.send(player).unwrap();
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
        let main_player = &mut self.game.main_player;
        let mut x: f32 = 0.0;
        let mut y: f32 = 0.0;
        if let Some(xx) = main_player.moving.0 {
            x = (xx as i32) as f32;
        }
        if let Some(yy) = main_player.moving.1 {
            y = (yy as i32) as f32;
        }

        let no_intersect = self.game.objects.iter().all(|critter| {
            !main_player.intersect(
                entities::Pos::new(critter.pos_x, critter.pos_y),
                critter.size
            )}
        );
        let mut new_pos_player = entities::Player::new();
        new_pos_player.pos.move_player(
            main_player.pos.x() + x * UPDATE_STEP,
            main_player.pos.y() + y * UPDATE_STEP,
        );
        let no_future_intersect = self.game.objects.iter().all(|critter| {
            !new_pos_player.intersect(
                entities::Pos::new(critter.pos_x, critter.pos_y),
                critter.size
            )}
        );
        let can_move = no_intersect || no_future_intersect;
        if can_move {
            main_player.pos.move_player(
                x * UPDATE_STEP,
                y * UPDATE_STEP
            );
        }

        if let Some(ref mut connection) = self.connection {
            if can_move && (x != 0.0 || y != 0.0) {
                // TODO: this fails if the server shuts down
                connection.send(main_player).unwrap();
            }
            match connection.receiver.recv_timeout(Duration::from_millis(15)) {
                Ok(message) => {
                    match message.mtype {
                        entities::MessageType::PlayerPosition =>
                            self.game.update_player(message.player.unwrap()),
                        entities::MessageType::WorldState =>
                            self.game.update_world(message.world.unwrap()),
                    }
                },
                _ => {  }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        for critter in self.game.objects.iter() {
            let intersect = self.game.main_player.intersect(
                entities::Pos::new(critter.pos_x, critter.pos_y),
                critter.size
                );
            let color = if intersect {
                graphics::Color::new(
                    255.0,
                    255.0,
                    255.0,
                    critter.color.3
                    )
            } else {
                graphics::Color::new(
                    critter.color.0,
                    critter.color.1,
                    critter.color.2,
                    critter.color.3
                    )
            };
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                na::Point2::new(critter.pos_x, critter.pos_y),
                critter.size as f32,
                2.0,
                color
            )?;
            graphics::draw(ctx, &circle, (na::Point2::new(0.0, 0.0),))?;
        }

        for player in self.game.players.iter() {
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                na::Point2::new(player.pos.x(), player.pos.y()),
                player.size as f32,
                2.0,
                graphics::WHITE,
            )?;
            graphics::draw(ctx, &circle, (na::Point2::new(0.0, 0.0),))?;
        }

        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            na::Point2::new(self.game.main_player.pos.x(), self.game.main_player.pos.y()),
            self.game.main_player.size as f32,
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
        main_player.save_prev_move();
        match keycode {
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

    println!("connect attempt");
    match state.connect("127.0.0.1:3012".to_string(), &state.game.main_player.clone()) {
        Ok(_) => println!("connected"),
        Err(e) => println!("Failed to connect: {}", e)
    };

    event::run(ctx, event_loop, state)
}
