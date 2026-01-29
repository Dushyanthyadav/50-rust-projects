use std::{
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    path::PathBuf,
};

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    ///Path of the html file
    file_path: std::path::PathBuf,

    ///Network interface to bind to
    #[arg(short = 'i', long, default_value_t = String::from("127.0.0.1"))]
    host: String,

    /// Port number to listen on
    #[arg(short, long, default_value_t = String::from("8080"))]
    port: String,
}

fn main() {
    let args = Args::parse();

    let ip_port = format!("{}:{}", args.host.trim(), args.port.trim());
    let listener = TcpListener::bind(ip_port.as_str()).unwrap();

    // .incoming is iterator which never gives None so the loop never stop similar to tcp accept in loop
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream, args.file_path.clone());
    }
}

fn handle_connection(mut stream: TcpStream, file_path: std::path::PathBuf) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let mut not_found = PathBuf::new();
    not_found.push("404.html");

    let (status_line, file_name) = if request_line == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", file_path)
    } else {
        ("HTTP/1.1 200 OK", not_found)
    };

    let contents = fs::read_to_string(file_name.as_path()).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
