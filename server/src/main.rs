use std::net::{TcpListener, TcpStream};
use std::io;
// use std::io::{ Write, BufWriter }; // BufReader, BufRead};
use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use entities;
use serde::{Deserialize}; // Serialize;
use std::sync::{ Arc, Mutex };
// use std::time::Duration;
use std::vec::Vec;

struct Client {
    pub socket: TcpStream,
    pub name: String,
}

struct Application {
    clients: Arc<Mutex<Vec<Client>>>, // arc mutx
    listeners: std::vec::Vec<std::thread::JoinHandle<()>>,
    receiver: Arc<Mutex<Receiver<String>>>,
    sender: Sender<String>,
}

fn start_listening(stream : Receiver<TcpStream>, sender : Sender<String>) {
    let client = stream.recv().expect("Error TcpStream received invalid");
    loop {
        let mut de = serde_json::Deserializer::from_reader(&client);
        let payload1 = entities::Player::deserialize(&mut de).unwrap();
        sender.send(serde_json::to_string(&payload1).unwrap()).unwrap();
    }
}

fn write_to_client(client: &Client, message: &str) {
    let player: entities::Player = serde_json::from_str(message).unwrap();
    if player.name != client.name {
        serde_json::to_writer(&client.socket, &player).unwrap();
    }
}

impl Application {
    fn publish(&self, message: String) {
        for client in &*self.clients.lock().unwrap() {
            write_to_client(client, &message);
        }
    }

    fn add_client(&mut self, client : TcpStream) {
        let stream_clone = client.try_clone().unwrap();

        // Get the client's player name. This is used to prevent broadcasting
        // movement messages to self.
        let mut de = serde_json::Deserializer::from_reader(&client);
        let payload1 = entities::Player::deserialize(&mut de).unwrap();
        let name: String = payload1.name.to_string();

        self.clients.lock().unwrap().push(Client {
            socket: client,
            name
        });

        println!("New client connected {}", payload1.name);

        let (send, rec) = mpsc::channel();
        let sender = self.sender.clone();
        self.listeners.push(thread::spawn(move || start_listening(rec, sender)));
        send.send(stream_clone).unwrap();
    }

    fn process(&self) {
        let cloned_rec = Arc::clone(&self.receiver);
        let cloned_clients = self.clients.clone();
        thread::spawn(move || {
            loop {
                match cloned_rec.lock().unwrap().try_recv() {
                    Ok(d) => {
                        for client in &*cloned_clients.lock().unwrap() {
                            write_to_client(client, &d); 
                        }
                    },
                    Err(_e) => {},
                }
            }
        });
    }

    #[allow(unused)]
    fn on_message_received(&self, message: String) {
        self.publish(message);
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
