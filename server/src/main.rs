use std::collections::hash_map::{Entry, HashMap};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;

const SERVER_ADDR: &str = "127.0.0.1:8000";
const MSG_SIZE: usize = 32;

fn main() {
    thread::spawn(move || accept_connections());
}

fn accept_connections() {
    let listener = TcpListener::bind(SERVER_ADDR).expect("Tcp listener failed to bind");
    let (client_sender, server_receiver) = channel();

    thread::spawn(move || writer_loop(server_receiver));

    for stream in listener.incoming() {
        let s = client_sender.clone();
        match stream {
            Ok(stream) => {
                thread::spawn(move || connection_loop(s, stream));
            }
            Err(_) => {
                println!("Error accepting a connection")
            }
        }
    }
}

enum Event {
    NewPeer {
        name: String,
        stream: Arc<TcpStream>,
    },
    Message {
        from: String,
        msg: String,
    },
}

fn writer_loop(receiver: Receiver<Event>) {
    let mut clients: HashMap<String, Sender<String>> = HashMap::new();

    loop {
        match receiver.recv() {
            Ok(event) => match event {
                Event::NewPeer { name, stream } => match clients.entry(name.clone()) {
                    Entry::Occupied(_) => todo!(),
                    Entry::Vacant(_) => {
                        let (client_sender, client_receiver) = channel();
                        clients.insert(name, client_sender);
                        thread::spawn(move || connection_writer_loop(stream, client_receiver));
                    }
                },
                Event::Message { from, msg } => {
                    for (_name, stream) in &clients {
                        stream
                            .send(format!("{}: {}", from, msg))
                            .expect("Error sending message to other clients.");
                    }
                }
            },
            Err(_) => println!("Error receiving message from a"),
        };
    }
}

fn connection_loop(client_sender: Sender<Event>, stream: TcpStream) {
    let stream = Arc::new(stream); // for reader loop
    let mut de_stream = &*stream;

    let mut buf = vec![0; MSG_SIZE];
    match de_stream.read_exact(&mut buf) {
        Ok(_) => {
            let message = buf.into_iter().take_while(|&x| x != 0).collect();
            match String::from_utf8(message) {
                Ok(message) => {
                    let mut split = message.split(":");
                    client_sender
                        .send(Event::Message {
                            from: split.next().unwrap().to_string(),
                            msg: split.next().unwrap().to_string(),
                        })
                        .unwrap();
                }
                Err(_) => println!("Error converting a message to UTF-8"),
            }
        }
        Err(_) => {}
    }
    loop {
        let mut buf = vec![0; MSG_SIZE];
        match de_stream.read_exact(&mut buf) {
            Ok(_) => {
                let message = buf.into_iter().take_while(|&x| x != 0).collect();
                match String::from_utf8(message) {
                    Ok(init_message) => {
                        client_sender
                            .send(Event::NewPeer {
                                name: init_message,
                                stream: stream.clone(),
                            })
                            .unwrap();
                    }
                    Err(_) => println!("Error converting a message to UTF-8"),
                }
            }
            Err(_) => {}
        }
    }
}

fn connection_writer_loop(stream: Arc<TcpStream>, receiver: Receiver<String>) {
    let mut stream = &*stream;
    loop {
        match receiver.recv() {
            Ok(message) => {
                let mut buf = message.clone().into_bytes();
                buf.resize(MSG_SIZE, 0);
                stream
                    .write(&buf)
                    .expect("Error writing the message to client");
            }
            Err(_) => println!("Erorr getting message from recv_channel"),
        }
    }
}
