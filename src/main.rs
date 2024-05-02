use std::{fs, time::Duration};

use async_std::io::WriteExt;
use async_std::net::TcpStream;
use async_std::task;
use async_std::{io::ReadExt, net::TcpListener};
use futures::stream::StreamExt;

#[async_std::main]
async fn main() {
    // Recall previous experience in Computer Networks course,
    // especially about socket programming and binding to a port.
    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();

    listener
        .incoming()
        .for_each_concurrent(/* limit */ None, |tcpstream| async move {
            let stream = tcpstream.unwrap();
            handle_connection(stream).await;
        })
        .await;

    println!("Shutting down.");
}

async fn handle_connection(mut stream: TcpStream) {
    // TODO: Is it possible to use BufReader instead of manually-allocated buffer?
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).await.unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /SLEEP HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else if buffer.starts_with(sleep) {
        task::sleep(Duration::from_secs(5)).await;
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write(response.as_bytes()).await.unwrap();
    stream.flush();
}
