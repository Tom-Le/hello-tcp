mod threading;

use core::time::Duration;
use std::{
  fs,
  io::{BufRead, BufReader, Write},
  net::{TcpListener, TcpStream},
  thread,
};

use anyhow::{anyhow, bail};
use log::{info, warn};
use threading::ThreadPool;

fn handle_connection(mut stream: TcpStream) -> anyhow::Result<()> {
  let buf_reader = BufReader::new(&mut stream);
  let request_line = buf_reader
    .lines()
    .next()
    .ok_or(anyhow!("request empty"))??;
  info!("Incoming request: {request_line}");

  let (status_line, filename) = match request_line.as_str() {
    "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
    "GET /error HTTP/1.1" => bail!("oh no"),
    "GET /sleep HTTP/1.1" => {
      thread::sleep(Duration::from_secs(5));
      ("HTTP/1.1 200 OK", "hello.html")
    }
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
  let pool = ThreadPool::build(4)?;

  for stream in listener.incoming() {
    pool.execute(move || {
      if let Err(e) = stream
        .map_err(anyhow::Error::new)
        .and_then(handle_connection)
      {
        warn!("Failed to handle connection: {e}");
      }
    });
  }

  Ok(())
}
