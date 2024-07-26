use std::{
  io::{BufRead, BufReader},
  net::{TcpListener, TcpStream},
};

fn handle_connection(mut stream: TcpStream) {
  println!("Connection established!");

  let buf_reader = BufReader::new(&mut stream);
  let http_request: Vec<_> = buf_reader
    .lines()
    .map_while(|result| result.ok())
    .take_while(|line| !line.is_empty())
    .collect();

  println!("Request: {http_request:#?}");
}

pub fn run() -> anyhow::Result<()> {
  let listener = TcpListener::bind("127.0.0.1:7878")?;

  for stream in listener.incoming() {
    match stream {
      Ok(stream) => handle_connection(stream),
      Err(e) => println!("Connection failed: {e}"),
    }
  }

  Ok(())
}
