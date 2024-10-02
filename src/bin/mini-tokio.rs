#[path = "../rust-futures/mod.rs"]
mod rust_futures;

use futures::task;
use log::info;
use rust_futures::delay::Delay;
use simple_logger::SimpleLogger;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use std::time::Duration;
use tokio::time::Instant;

type Task = Pin<Box<dyn Future<Output = ()> + Send>>;
struct MiniTokio {
  tasks: VecDeque<Task>,
}

impl MiniTokio {
  fn new() -> Self {
    Self {
      tasks: VecDeque::new(),
    }
  }

  fn spawn<F>(&mut self, future: F)
  where
    F: Future<Output = ()> + Send + 'static,
  {
    self.tasks.push_back(Box::pin(future));
  }

  fn run(&mut self) {
    let waker = task::noop_waker();
    let mut cx = Context::from_waker(&waker);

    while let Some(mut task) = self.tasks.pop_front() {
      if task.as_mut().poll(&mut cx).is_pending() {
        self.tasks.push_back(task);
      }
    }
  }
}

fn main() {
  let mut mini_tokio = MiniTokio::new();
  SimpleLogger::new().init().unwrap();

  mini_tokio.spawn(async {
    let when = Instant::now() + Duration::from_secs(2);

    let future = Delay { when };

    let out = future.await;
    info!("Completed: {:#?}", out);
  });

  mini_tokio.run();
}
