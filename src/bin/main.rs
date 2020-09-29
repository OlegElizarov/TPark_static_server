// use chunked_transfer::Encoder;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::str;
use std::io::Write;
use std::fs::File;
use static_server::ThreadPool;
// use snedfile::send_file;
use sendfile::send_file;
use std::future::Future;
use std::pin::Pin;
use futures_test::task::noop_context;
use std::fs;
use mime::Mime;

// use std::thread::sleep;
// use std::time::Duration;


const STATIC_PATH: &str = "static/";
const OK_RESP: &str = "HTTP/1.1 200 OK";
const NOT_FOUND_RESP: &str = "HTTP/1.1 404 NOT FOUND";
const FORBIDDEN: &str = "HTTP/1.1 403 NOT FOUND";
const NOT_ALLOWED: &str = "HTTP/1.1 405 Method Not Allowed";
// const HTML_TYPE: &str = "Content-Type: text/html; charset=utf-8";
// const JPEG_TYPE: &str = "Content-Type: image/jpeg; charset=utf-8";
// const WORKERS_NUM: usize = 2;


fn main() {
    let (threads, cores) = read_conf("conf/conf.conf".parse().unwrap());
    println!("{}!{}!", threads, cores);
    let listener = TcpListener::bind("0.0.0.0:80").unwrap();
    let pool = ThreadPool::new(threads as usize);

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
    let head = b"HEAD"; //* HTTP/1.1\r\n";
    let (mut status_line, mut filename) = if
    buffer.starts_with(get) || buffer.starts_with(head) {
        (OK_RESP, format!("{}{}", STATIC_PATH, &mess[5..(mess.find("HTTP").unwrap() - 1)]))
    } else {
        (NOT_ALLOWED, format!("{}{}", STATIC_PATH, "close.txt"))
    };
    // println!("{}", filename);
    let found = std::path::Path::new(&filename).exists();
    if !found {
        status_line = NOT_FOUND_RESP;
        filename = format!("{}{}", STATIC_PATH, "close.txt".to_string());
    }

    let file = File::open(&filename).unwrap();
    let file_len = format!("{}{}", "Content-Length: ", file.metadata().unwrap().len().to_string());
    let con = &"Connection: close";
    let mime = format!("{}{}", "Content-Type: ", find_mimetype(&filename));
    let headers =
        [status_line, con, &mime, &file_len, "\r\n"];
    let response: Vec<u8> = headers.join("\r\n")
        .to_string()
        .into_bytes();

    match stream.write(&response) {
        Ok(_) => println!("Response sent"),
        Err(e) => println!("Failed sending response: {}", e),
    }
    if buffer.starts_with(get) {
        let mut ctx = noop_context();
        let mut send_file = unsafe { send_file(file, stream) };
        let _result = Pin::new(&mut send_file).poll(&mut ctx);

        // println!("{:?}", send_file);
        // println!("{}", result.is_ready());

        let (_, mut socket) = send_file.into_inner();
        socket.flush().unwrap();
        return;
    }
    stream.flush().unwrap()
}

fn read_conf(filename: String) -> (i32, i32) {
    let core = "cpu_limit";
    let worker = "thread_limit";
    let conf: String = fs::read_to_string(filename).unwrap();
    // println!("{}", conf);
    let a = conf.find(worker).unwrap();
    let b = conf.find(core).unwrap();
    let c = &conf[b + core.len() + 1..a - 1];
    let w = &conf[a + worker.len() + 1..];
    // println!("{}!{}!", c,w);
    return (w.parse().unwrap(), c.parse().unwrap());
}

fn find_mimetype(filename: &String) -> Mime {
    let parts: Vec<&str> = filename.split('.').collect();

    let res = match parts.last() {
        Some(v) =>
            match *v {
                "png" => mime::IMAGE_PNG,
                "jpg" => mime::IMAGE_JPEG,
                "json" => mime::APPLICATION_JSON,
                "html" => mime::TEXT_HTML_UTF_8,
                "css" => mime::TEXT_CSS,
                "js" => mime::APPLICATION_JAVASCRIPT,
                &_ => mime::TEXT_PLAIN,
            },
        None => mime::TEXT_PLAIN,
    };
    return res;
}

// docker run --name nginx -v /Users/elenaelizarova/PycharmProjects/TPark_static_server/conf/serv.conf:/etc/nginx/conf.d/default.conf:ro -p 8080:80 -v /Users/elenaelizarova/PycharmProjects/TPark_static_server/static:/usr/share/nginx/html:ro -v /Users/elenaelizarova/PycharmProjects/TPark_static_server/conf/nginx.conf:/etc/nginx/nginx.conf --cpus=2  -d nginx


