use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;

const SERVER_ADDR: &str = "127.0.0.1:8000";
const MSG_SIZE: usize = 32;
fn main() {
    let mut reader_stream =
        TcpStream::connect(SERVER_ADDR).expect("Could not connect to the server.");
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

    loop {
        let mut message = String::new();
        println!("Enter a message: ");
        let stdin = io::stdin();
        stdin.read_line(&mut message).unwrap();
        message = message.trim().to_string();
        if message == "quit" {
            break;
        }
        //println!("The message is: {}", message);
        tx.send(message).expect("Error sending message to channel");
    }
    println!("Goodbye!");
}
