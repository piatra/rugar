use std::net::{TcpListener, TcpStream};
use std::io;
use std::io::{ Write, BufWriter }; // BufReader, BufRead};
use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use entities;
use serde::{Deserialize}; // Serialize;

struct Application {
    clients : std::vec::Vec<TcpStream>,
    listeners : std::vec::Vec<std::thread::JoinHandle<()>>,
    #[allow(unused)]
    receiver : Receiver<String>,
    sender : Sender<String>,
}

fn start_listening(stream : Receiver<TcpStream>, _sender : Sender<String>) {
    let client = stream.recv().expect("Error TcpStream received invalid");
    loop {
        let mut de = serde_json::Deserializer::from_reader(&client);
        let payload1 = entities::Player::deserialize(&mut de).unwrap();
        println!("{:?}", payload1);
    }

    // sender.send(s).unwrap();
}

impl Application {
    fn publish(&self, message: entities::Player) {
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
        self.publish(entities::Player { ..Default::default() });
        let stream_clone = self.clients.last().unwrap().try_clone().unwrap();
        let (send, rec) = mpsc::channel();
        let sender = self.sender.clone();
        self.listeners.push(thread::spawn(move || start_listening(rec, sender)));
        send.send(stream_clone).unwrap();
    }

    #[allow(unused)]
    fn on_message_received(&self, mess: String) {
        let message: entities::Player = serde_json::from_str(&*mess).unwrap();
        self.publish(message);
    }
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3012")?;
    let (send, rec) : (Sender<String>, Receiver<String>) = mpsc::channel();
    let mut app = Application{clients :Vec::new(), listeners : Vec::new(), receiver : rec, sender : send};

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
