use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;
use std::str;

const LOCAL_HOST: &str = "127.0.0.1:80";
const MESSAGE_SIZE: usize = 32;

fn main() {
    let mut client = TcpStream::connect(LOCAL_HOST).expect("Failed to connect");

    client.set_nonblocking(true).expect("Failed to initiate non-blocking");

    let (sender, receiver) = mpsc::channel::<String>();

    thread::spawn(move || loop
    {
        let mut buffer = vec![0; MESSAGE_SIZE];
        match client.read_exact(&mut buffer)
        {
            Ok(_) =>
                {
                    let message = buffer.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                    let message = str::from_utf8(&message).unwrap();
                    println!("Message :{:?}", message);
                },

            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) =>
                {
                    println!("Connection with server was severed");
                    break;
                }
        }
        match receiver.try_recv()
        {
            Ok(message) =>
                {
                    let mut buffer = message.clone().into_bytes();
                    buffer.resize(MESSAGE_SIZE, 0);
                    client.write_all(&buffer).expect("Writing to socket failed");

                },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }
        thread::sleep(Duration::from_millis(100));
    });
    // This will show up when the user opens the client
    println!("*********************************");
    println!("************ WELCOME ************");
    println!("*********************************");

    loop
    {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).expect("Reading from stdin failed");
        let message = buffer.trim().to_string();
        if message == "exit" || sender.send(message).is_err() {break}
    }
    // Print out GOOD BYE
    println!("*********************************");
    println!("*********** GOOD BYE ************");
    println!("*********************************");
}
