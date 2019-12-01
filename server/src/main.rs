use std::net::{TcpListener, TcpStream};
use std::io;
use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use entities;
use serde::{Deserialize};
use std::sync::{ Arc, Mutex };
use std::vec::Vec;
use std::collections::HashSet;
use std::time::Duration;

struct Client {
    pub socket: TcpStream,
    pub name: String,
}

struct Application {
    clients: Arc<Mutex<Vec<Client>>>, // arc mutx
    listeners: std::vec::Vec<std::thread::JoinHandle<()>>,
    receiver: Arc<Mutex<Receiver<String>>>,
    sender: Sender<String>,
    game_world: entities::GameWorld,
}

fn start_listening(stream : Receiver<TcpStream>, sender : Sender<String>) {
    let client = stream.recv().expect("Error TcpStream received invalid");
    loop {
        let mut de = serde_json::Deserializer::from_reader(&client);
        if let Ok(payload) = entities::Message::deserialize(&mut de) {
            sender.send(serde_json::to_string(&payload).unwrap()).unwrap();
        }
    }
}

fn write_to_client(client: &Client, message: &str) -> bool {
    let message: entities::Message = serde_json::from_str(message).unwrap();
    println!("message {:?}", message);
    match message.event {
        entities::MessageType::UpdatePositions => {
            let player = message.player.unwrap();
            if player.name != client.name {
                println!("{} write to {}", player.name, client.name);
                if let Err(_) = serde_json::to_writer(&client.socket, &player) {
                    println!("Could not write to {}", client.name);
                    return false
                }
            }
        }
    }
    return true
}

impl Application {
    fn add_client(&mut self, client : TcpStream) {
        let stream_clone = client.try_clone().unwrap();

        // Get the client's player name. This is used to prevent broadcasting
        // movement messages to self.
        let mut de = serde_json::Deserializer::from_reader(&client);
        let payload = entities::Message::deserialize(&mut de).unwrap();

        match payload.event {
            entities::MessageType::UpdatePositions => {
                let player = payload.player.unwrap();
                let name: String = player.name.to_string();

                self.clients.lock().unwrap().push(Client {
                    socket: client,
                    name
                });

                println!("New client connected {}", player.name);

                let (send, rec) = mpsc::channel();
                let sender = self.sender.clone();
                self.listeners.push(thread::spawn(move || start_listening(rec, sender)));
                send.send(stream_clone).unwrap();
            }
        }
    }

    fn process(&self) {
        let cloned_rec = Arc::clone(&self.receiver);
        let cloned_clients = self.clients.clone();
        thread::spawn(move || {
            let mut dropouts = HashSet::new();
            loop {
                match cloned_rec.lock().unwrap().try_recv() {
                    Ok(d) => {
                        for client in &*cloned_clients.lock().unwrap() {
                            if !dropouts.contains(&client.name) &&
                               !write_to_client(client, &d) {
                                dropouts.insert(client.name.clone());
                            }
                        }
                    },
                    Err(_e) => {},
                }
            }
        });
    }
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3012")?;
    let (send, rec) : (Sender<String>, Receiver<String>) = mpsc::channel();
    let mut app = Application {
        clients: Arc::new(Mutex::new(Vec::new())),
        listeners: Vec::new(),
        receiver: Arc::new(Mutex::new(rec)),
        sender: send,
        game_world: entities::GameWorld::new(),
    };

    app.process();

    // accept connections and process them serially
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                app.add_client(stream);
                println!("added client");
            }
            Err(e) => { println!("{}", e) }
        }
    }
    println!("done");
    Ok(())
}
