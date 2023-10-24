// Uncomment this block to pass the first stage
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn handle_connection(mut stream: TcpStream) {
    println!("accepted new connection");
    let buf: &mut [u8; 128] = &mut [0; 128];
    stream.read(buf);
    let request_str = String::from_utf8(buf.to_vec()).unwrap();

    let lines: Vec<&str> = request_str.split("\r\n").collect();
    let first = lines[0];
    let first_parts: Vec<&str> = first.split_whitespace().collect();
    let path = first_parts[1];

    match path {
        "/" => {
            stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n");
        },
        "/user-agent" =>  {
            let mut ua = "";
            for l in lines {
                if l.contains("User-Agent") {
                    ua = &l[12..];
                    break;
                }
            }

            let len = ua.len();
            let res = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n\r\n", len, ua);
            println!("{}",  res);
            stream.write_all(res.as_bytes());
        },
        p if p.contains("/echo") => {
            let s = &p[6..];
            let len = s.len();
            let res = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n\r\n",
            len, s
        );
            stream.write_all(res.as_bytes());
        },
        _ => {
            stream.write_all(b"HTTP/1.1 404 NOT FOUND\r\n\r\n");
        }
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => handle_connection(_stream),
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
