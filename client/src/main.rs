use std::io::{self, Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

//const SERVER_ADDR: &str = "127.0.0.1:8000";
const MSG_SIZE: usize = 32;

fn main() {
    let mut reader_stream = match TcpStream::connect_timeout(
        &SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000),
        Duration::from_secs(5),
    ) {
        Ok(stream) => stream,
        Err(_) => {
            println!("Error connecting to the server");
            return;
        }
    };
    let mut writer_stream = reader_stream.try_clone().unwrap();

    let (tx, rx) = mpsc::channel::<String>();

    let _receiver = thread::spawn(move || loop {
        let mut buf = vec![0; MSG_SIZE];
        match reader_stream.read_exact(&mut buf) {
            Ok(_) => {
                let message = buf.into_iter().take_while(|&x| x != 0).collect();
                match String::from_utf8(message) {
                    Ok(message) => println!("{}", message),
                    Err(_) => println!("Error converting the message to utf-8"),
                }
            }
            Err(_) => println!("Error reading from stream"),
        }
    });

    let _sender = thread::spawn(move || loop {
        let message = rx.recv().unwrap();
        let mut buff = message.clone().into_bytes();
        buff.resize(MSG_SIZE, 0);
        writer_stream
            .write(&buff)
            .expect("Error sending the message to the server");
    });

    println!("Enter your name:");
    let stdin = io::stdin();
    let mut name = String::new();
    stdin.read_line(&mut name).unwrap();
    name = name.trim().to_string();
    tx.send(name.clone()).unwrap();

    println!("Welcome to the chat!");
    loop {
        let name = name.clone();
        let mut message = String::new();
        let stdin = io::stdin();
        stdin.read_line(&mut message).unwrap();
        message = message.trim().to_string();
        let full_message = name + ":" + &message;
        if message == "quit" {
            break;
        }
        //println!("The message is: {}", message);
        tx.send(full_message)
            .expect("Error sending message to channel");
    }
    println!("Goodbye!");
}
