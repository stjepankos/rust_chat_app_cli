use crossbeam_channel::unbounded;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener};
use std::thread;

const SERVER_ADDR: &str = "127.0.0.1:8000";
const MSG_SIZE: usize = 32;

fn main() {
    let listener = TcpListener::bind(SERVER_ADDR).expect("Tcp listener failed to bind");

    let (s, r) = unbounded::<String>();

    loop {
        match listener.accept() {
            Ok((mut socket, addr)) => {
                let sender = s.clone();
                let receiver = r.clone();
                thread::spawn(move || loop {
                    let mut buf = vec![0; MSG_SIZE];
                    //send incoming messages from the channel to the client
                    while !receiver.is_empty() {
                        match receiver.recv() {
                            Ok(message) => {
                                buf = message.clone().into_bytes();
                                buf.resize(MSG_SIZE, 0);
                                socket
                                    .write(&buf)
                                    .expect("Error writing the message to addr: {addr}");
                            }
                            Err(_) => println!("Error getting the message from the channel"),
                        }
                    }
                    //receive message from client and send to the channel
                    match socket.read_exact(&mut buf) {
                        Ok(_) => {
                            let message = buf.into_iter().take_while(|&x| x != 0).collect();
                            match String::from_utf8(message) {
                                Ok(message) => {
                                    let full_message = format!("{}: {}", addr, message);
                                    println!("{full_message}");
                                    sender.send(full_message).expect("Error sending message")
                                }
                                Err(_) => println!("Error converting a message to UTF-8"),
                            }
                        }
                        Err(_) => {
                            println!("Error reading from client with addr {}", addr);
                            socket.shutdown(Shutdown::Both).unwrap();
                        }
                    };
                })
            }
            Err(_) => todo!(),
        };
    }
}
