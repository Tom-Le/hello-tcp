use std::{
  io::{BufRead, BufReader, Write},
  net::{TcpListener, TcpStream},
};

fn handle_connection(mut stream: TcpStream) -> anyhow::Result<()> {
  println!("Connection established!");

  let buf_reader = BufReader::new(&mut stream);
  let http_request: Vec<_> = buf_reader
    .lines()
    .map_while(|result| result.ok())
    .take_while(|line| !line.is_empty())
    .collect();

  println!("Request: {http_request:#?}");

  let response = "HTTP/1.1 200 OK\r\n\r\n";
  stream.write_all(response.as_bytes())?;

  Ok(())
}

pub fn run() -> anyhow::Result<()> {
  let listener = TcpListener::bind("127.0.0.1:7878")?;

  for stream in listener.incoming() {
    if let Err(e) = stream.map(handle_connection) {
      println!("Failed to handle connection: {e}");
    }
  }

  Ok(())
}
