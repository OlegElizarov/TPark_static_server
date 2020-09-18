use chunked_transfer::Encoder;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::str;
use std::io::Write;
use std::fs::File;
use static_server::ThreadPool;
use snedfile::send_file;
// use sendfile::send_file;
// use sendfile::SendFile;


const STATIC_PATH: &str = "static/";
const OK_RESP: &str = "HTTP/1.1 200 OK";
const NOT_FOUND_RESP: &str = "HTTP/1.1 404 NOT FOUND";
const NOT_ALLOWED: &str = "HTTP/1.1 405 Method Not Allowed";
// const HTML_TYPE: &str = "Content-Type: text/html; charset=utf-8";
// const JPEG_TYPE: &str = "Content-Type: image/jpeg; charset=utf-8";


// const WORKERS_NUM: i32 = 5;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let size = stream.read(&mut buffer).unwrap();
    let mess = (str::from_utf8(&buffer.to_vec()).unwrap())[..size].to_owned();
    let get = b"GET"; //* HTTP/1.1\r\n";
    let (mut status_line, mut filename) = if buffer.starts_with(get) {
        // (OK_RESP, format!("{}{}", STATIC_PATH, &mess[5..(mess.find("HTTP").unwrap() - 1)]))
        (OK_RESP, format!("{}{}", STATIC_PATH, &mess[5..11]))
    } else {
        (NOT_ALLOWED, format!("{}{}", STATIC_PATH, "405.html"))
    };
    println!("{}", filename);
    let found = std::path::Path::new(&filename).exists();
    if !found {
        status_line = NOT_FOUND_RESP;
        filename = format!("{}{}", STATIC_PATH, "404.html".to_string());
    }
    // let mut encoded = read_file(filename);
    let mut file = File::open(&filename).unwrap();
    let file_len = format!("{}{}", "Content-Length: ", file.metadata().unwrap().len().to_string());

    let headers =
        // [status_line, "Transfer-Encoding: chunked", "\r\n"];
        [status_line, &file_len, "Transfer-Encoding: chunked", "\r\n"];
    let mut response: Vec<u8> = headers.join("\r\n")
        .to_string()
        .into_bytes();
    // response.extend(encoded);

    match stream.write(&response) {
        Ok(_) => println!("Response sent"),
        Err(e) => println!("Failed sending response: {}", e),
    }

    // match stream.write(&encoded) {
    //     Ok(_) => println!("Response sent"),
    //     Err(e) => println!("Failed sending response: {}", e),
    // }
    // println!("{}", file_len);
    let res = send_file(&mut file, &mut stream);
    // println!("{:?}", res);
    stream.flush().unwrap();
}

fn read_file(filename: String) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut file = File::open(&filename).unwrap();
    file.read_to_end(&mut buf).unwrap();
    let mut encoded = Vec::new();
    {
        let mut encoder = Encoder::with_chunks_size(&mut encoded, 8);
        encoder.write_all(&buf).unwrap();
    }
    encoded
}