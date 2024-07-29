use std::{
  io,
  sync::{mpsc, Arc, Mutex},
  thread,
};

use anyhow::bail;
use log::{error, info};

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
  id: usize,
  handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
  fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> io::Result<Worker> {
    let builder = thread::Builder::new();
    Ok(Worker {
      id,
      handle: Some(builder.spawn(move || loop {
        let message = receiver.lock().expect("mutex poisoned").recv();
        match message {
          Ok(job) => {
            info!("Worker {id} received a new job");
            job();
          }
          Err(_) => {
            info!("Worker {id} disconnected");
            break;
          }
        }
      })?),
    })
  }
}

#[derive(thiserror::Error, Debug)]
pub enum ThreadPoolBuildError {
  #[error("invalid thread pool size {size}")]
  InvalidSize { size: usize },
}

pub struct ThreadPool {
  sender: Option<mpsc::Sender<Job>>,
  workers: Vec<Worker>,
}

impl ThreadPool {
  pub fn build(size: usize) -> anyhow::Result<ThreadPool> {
    if size == 0 {
      bail!(ThreadPoolBuildError::InvalidSize { size });
    }
    let (sender, receiver) = mpsc::channel();
    let receiver = Arc::new(Mutex::new(receiver));
    let mut workers = Vec::with_capacity(size);
    for id in 0..size {
      workers.push(Worker::new(id, Arc::clone(&receiver))?);
    }
    Ok(ThreadPool {
      sender: Some(sender),
      workers,
    })
  }

  pub fn execute<F>(&self, f: F)
  where
    F: FnOnce() + Send + 'static,
  {
    let Some(sender) = self.sender.as_ref() else {
      return;
    };
    sender.send(Box::new(f)).expect("no receivers left");
  }
}

impl Drop for ThreadPool {
  fn drop(&mut self) {
    drop(self.sender.take());

    for worker in &mut self.workers {
      info!("Shutting down worker {}", worker.id);
      if let Some(handle) = worker.handle.take() {
        handle.join().expect("couldn't join worker thread");
      }
    }
  }
}
