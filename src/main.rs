use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::fs;
use std::str;

const STATIC_PATH: &str = "static/";
// const WORKERS_NUM: i32 = 5;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        thread::spawn(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let size = stream.read(&mut buffer).unwrap();
    let mut mess = (str::from_utf8(&buffer.to_vec()).unwrap())[..size].to_owned();
    // println!("{:?}", &(str::from_utf8(&*buffer.to_vec()).unwrap())[..size]);
    println!("{}", &mess[4..mess.find("HTTP").unwrap()]);
    let get = b"GET / HTTP/1.1\r\n";
    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", format!("{}{}", STATIC_PATH, "index.html"))
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", format!("{}{}", STATIC_PATH, "404.html"))
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}