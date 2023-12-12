use std::{
    fs::File,
    io::{prelude::BufRead, prelude::Write, BufReader},
    net::{TcpListener, TcpStream},
    thread,
};

use tail::tail::{follow_file, get_last_lines};

mod tail;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:9999").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_stream(stream));
            }
            Err(err) => {
                panic!("{err}")
            }
        }
    }
}

fn format_sse_data(data: &mut Vec<u8>) -> Vec<u8> {
    let mut prefix = b"data: ".to_vec();
    let mut suffix = b"\n\n".to_vec();
    prefix.append(data);
    prefix.append(&mut suffix);
    return prefix;
}

fn handle_stream(mut stream: TcpStream) {
    println!("Handling stream");
    let lines = BufReader::new(&mut stream).lines();

    let mut path: String = String::from("");
    for line in lines {
        match line {
            Ok(line) => {
                if line.starts_with("GET") {
                    println!("{line}");
                    let v: Vec<_> = line.split(" ").collect();
                    let p = *v.get(1).unwrap();
                    if p == "/logs" {
                        path = "logs".to_string();
                        break;
                    }
                    if p == "/index" {
                        path = "index".to_string();
                        break;
                    }
                    if p.starts_with("/") {
                        path = "".to_string();
                        break;
                    }
                }
            }
            Err(err) => {
                panic!("{err}")
            }
        }
    }

    if path == "index" {
        let response = "HTTP/1.1 200 OK\n\nHello world";
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
        return;
    }

    if path == "" {
        println!("Not found path");
        let response = "HTTP/1.1 404 Not Found\n\n";
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
        return;
    }

    let response = "HTTP/1.1 200 OK\nContent-Type: text/event-stream\n\n";
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap(); // Make sure to flush the response

    let mut file = File::open("./out.log").unwrap();

    let mut buf = get_last_lines(&mut file, 100);
    let mut buf = format_sse_data(&mut buf);

    stream.write(&mut buf).unwrap();
    stream.flush().unwrap();

    follow_file(&mut file, &mut stream, format_sse_data);
}
