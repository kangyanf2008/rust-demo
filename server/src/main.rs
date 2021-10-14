use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

const LOCAL_HOST : &str = "0.0.0.0:80";
const MESSAGE_SIZE: usize = 32;

fn sleep(){
    thread::sleep(std::time::Duration::from_micros(100))
}
fn main() {
    //启动服务监听
    let listener = TcpListener::bind(LOCAL_HOST).expect("Failed to create TcpListener");
    //设置非阻塞模式
    listener.set_nonblocking(true).expect("Cannot set non-blocking");

    //创建可变客户连接列表
    let mut clients = vec![];

    //创String类型管道
    let (sender, receiver) = mpsc::channel::<String>();

    //循环处理连接请求
    loop {

        //连接处理
        if let Ok((mut socket, address)) = listener.accept() {
            println!("client{}: connected", address);

            //
            let sender = sender.clone();

            //放入连接列表
            clients.push(socket.try_clone().expect("failed to clone client socket"));

            //创建线程片处理客户端发送消息
            thread::spawn(move || loop {
                //定义接收消息缓冲区
                let mut buffer = vec![0; MESSAGE_SIZE];
                //读取消息
                match socket.read_exact(&mut buffer) {
                    Ok(_) => {
                        let message = buffer.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let mut message = String::from_utf8(message).expect("Invalid utf8 message");
                        println!("{}: {:?}", address, message);
                        //sender.send("user:"+address.to_string()+",message:"+message).expect("Failed to send message to receiver");

                        let mut content_string = String::from("user:");
                        content_string.push_str(&mut address.to_string());
                        let mut ip = String::from("message:");
                        content_string.push_str(&mut ip);
                        content_string.push_str(&mut message);
                        sender.send(content_string).expect("Failed to send message to receiver");
                    },
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock =>(),
                    Err(_) => {
                        println!("closing connection whih:{}", address);
                        break;
                    }
                }
                sleep();
            });
        }// end OK

        if let Ok(message) = receiver.try_recv()
        {
            clients = clients.into_iter().filter_map(|mut client|
                {
                    let mut buffer = message.clone().into_bytes();
                    buffer.resize(MESSAGE_SIZE, 0);
                    client.write_all(&buffer).map(|_| client).ok()
                }).collect::<Vec<_>>();
        }
        sleep();
    }
}
