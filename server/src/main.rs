use std::net::{TcpListener, TcpStream};
use std::io;
use std::io::{ Write, BufWriter }; // BufReader, BufRead};
use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use entities;
use serde::{Deserialize}; // Serialize;
use std::sync::{ Arc, Mutex };
use std::time::Duration;
use std::vec::Vec;

struct Application {
    clients: Vec<TcpStream>, // arc mutx
    listeners: std::vec::Vec<std::thread::JoinHandle<()>>,
    receiver: Arc<Mutex<Receiver<String>>>,
    sender: Sender<String>,
    players: Arc<Mutex<Vec<entities::Player>>>,
}

fn start_listening(stream : Receiver<TcpStream>, sender : Sender<String>) {
    let client = stream.recv().expect("Error TcpStream received invalid");
    loop {
        let mut de = serde_json::Deserializer::from_reader(&client);
        let payload1 = entities::Player::deserialize(&mut de).unwrap();
        println!("{:?}", payload1);
        sender.send(serde_json::to_string(&payload1).unwrap()).unwrap();
    }
}

impl Application {
    fn publish(&self, message: String) {
        for client in &self.clients {
            let mut buffer = BufWriter::new(client);
            buffer.write_all(&serde_json::to_string(&message).unwrap().as_bytes()).unwrap();
            buffer.flush().expect("Error while writing to TCP");
        }
        println!("wrote to all {}", self.clients.len());
    }

    fn add_client(&mut self, client : TcpStream) {
        self.clients.push(client);
        println!("New client connected");
        // self.publish(String::from_str("Welcome"));
        let stream_clone = self.clients.last().unwrap().try_clone().unwrap();
        let (send, rec) = mpsc::channel();
        let sender = self.sender.clone();
        self.listeners.push(thread::spawn(move || start_listening(rec, sender)));
        send.send(stream_clone).unwrap();
    }

    fn process(&self) {
        let cloned_rec = Arc::clone(&self.receiver);
        let cloned_players = self.players.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(2));
                match cloned_rec.lock().unwrap().try_recv() {
                    Ok(d) => {
                        println!("Server recv");
                        cloned_players.lock().unwrap().push(Default::default());
                    },
                    Err(e) => println!("Server recv err {}", e),
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
        clients: Vec::new(),
        listeners: Vec::new(),
        receiver: Arc::new(Mutex::new(rec)),
        sender: send,
        players: Default::default(),
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
