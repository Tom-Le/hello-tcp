use std::{
  fs,
  io::{BufRead, BufReader, Write},
  net::{TcpListener, TcpStream},
};

use anyhow::anyhow;
use log::{info, warn};

fn handle_connection(mut stream: TcpStream) -> anyhow::Result<()> {
  let buf_reader = BufReader::new(&mut stream);
  let request_line = buf_reader
    .lines()
    .next()
    .ok_or(anyhow!("request empty"))??;
  info!("Incoming request: {request_line}");

  let (status_line, filename) = match request_line.as_str() {
    "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
    _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
  };
  info!("Response: {status_line}");
  let contents = fs::read_to_string(filename)?;
  let length = contents.len();
  let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
  stream.write_all(response.as_bytes())?;

  Ok(())
}

pub fn run() -> anyhow::Result<()> {
  let listener = TcpListener::bind("127.0.0.1:7878")?;

  for stream in listener.incoming() {
    let result = stream
      .map_err(anyhow::Error::new)
      .and_then(handle_connection);
    if let Err(e) = result {
      warn!("Failed to handle connection: {e}");
    }
  }

  Ok(())
}
