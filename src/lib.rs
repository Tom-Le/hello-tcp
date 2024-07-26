use std::{
  fs,
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

  let status_line = "HTTP/1.1 200 OK";
  let contents = fs::read_to_string("hello.html")?;
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
      println!("Failed to handle connection: {e}");
    }
  }

  Ok(())
}
