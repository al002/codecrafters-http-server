// Uncomment this block to pass the first stage
use anyhow::Result;
use std::env;
use std::fs;
use std::path::{PathBuf, Path};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn handle_connection(mut stream: TcpStream, file_path: Option<&Path>) -> Result<()> {
    println!("accepted new connection");
    let buf: &mut [u8; 2048] = &mut [0; 2048];
    stream.read(buf).await?;
    let request_str = String::from_utf8(buf.to_vec()).unwrap();

    let lines: Vec<&str> = request_str.split("\r\n").collect();
    let first = lines[0];
    let first_parts: Vec<&str> = first.split_whitespace().collect();
    let method = first_parts[0].to_lowercase();
    let path = first_parts[1];

    let response = match path {
        "/" => {
            "HTTP/1.1 200 OK\r\n\r\n".to_string()
        }
        "/user-agent" => {
            let mut ua = "";
            for l in lines {
                if l.contains("User-Agent: ") {
                    ua = &l[12..];
                    break;
                }
            }

            let len = ua.len();
            format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n\r\n", len, ua)
        }
        p if p.starts_with("/echo") => {
            let s = &p[6..];
            let len = s.len();
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n\r\n",
                len, s
            )
        }
        p if p.starts_with("/files") => {

            let filename = &p[7..];
            let mut p = file_path.unwrap().to_path_buf();
            p.push(filename);
            let res;

            if method == "post" {
                let body: &str = request_str.split("\r\n\r\n").collect::<Vec<&str>>()[1].trim_end_matches("\0");

                if let Ok(_) = fs::write(p, body) {
                    res = "HTTP/1.1 201 OK\r\n\r\n".to_string();
                } else  {
                    res = "HTTP/1.1 500 Error\r\n\r\n".to_string();
                }

            } else {
                match fs::read_to_string(p) {
                    Ok(data) => {
                        res = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}\r\n\r\n",
                            data.len(),
                            data,
                        );
                    }
                    _ => {
                        res = "HTTP/1.1 404 NOT FOUND\r\n\r\n".to_string();
                    }
                }
            } 

            res
        }
        _ => {
            "HTTP/1.1 404 NOT FOUND\r\n\r\n".to_string()
        }
    };

    stream.write_all(response.as_bytes()).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let mut directory: Option<PathBuf> = Option::None;

    let mut args = env::args();
    args.next();

    if let Some("--directory") = args.next().as_deref() {
        directory = Some(args.next().unwrap().into());
    }

    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let dest = directory.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, dest.as_deref()).await {
                eprintln!("{e:?}");
            }
        });
    }
}
