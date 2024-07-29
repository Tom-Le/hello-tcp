mod threading;

use core::time::Duration;
use std::{
  fs,
  io::{self, BufRead, BufReader, Write},
  net::{TcpListener, TcpStream},
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
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
  let running = Arc::new(AtomicBool::new(true));

  {
    let running = running.clone();
    ctrlc::set_handler(move || {
      running.store(false, Ordering::SeqCst);
    })?;
  }

  let listener = TcpListener::bind("127.0.0.1:7878")?;
  listener.set_nonblocking(true)?;

  let pool = ThreadPool::build(4)?;

  while running.load(Ordering::SeqCst) {
    let stream = match listener.accept() {
      Ok((stream, _)) => stream,
      Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
      Err(e) => {
        warn!("Listener encountered error: {e}");
        break;
      }
    };
    pool.execute(move || {
      if let Err(e) = handle_connection(stream) {
        warn!("Failed to handle connection: {e}");
      }
    });
  }

  info!("Server shutting down");
  Ok(())
}
