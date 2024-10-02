#[path = "../rust-futures/mod.rs"]
mod rust_futures;

use log::info;
use rust_futures::{delay::Delay, mini_tokio_lib::Task};
use simple_logger::SimpleLogger;
use std::future::Future;
use std::sync::{mpsc, Arc};
use std::time::Duration;
use tokio::time::Instant;

struct MiniTokio {  
    scheduled: mpsc::Receiver<Arc<Task>>,
    sender: mpsc::Sender<Arc<Task>>,
}

impl MiniTokio {
    fn new() -> Self {
        let (sender, scheduled) = mpsc::channel();
        Self { scheduled, sender }
    }

    fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Task::spawn(future, &self.sender);
    }

    fn run(&mut self) {
        while let Ok(task) = self.scheduled.recv() {
            task.poll();
        }
    }
}

fn main() {
    let mut mini_tokio = MiniTokio::new();
    SimpleLogger::new().init().unwrap();

    async fn complete_with_delay(time: u64) {
        let when = Instant::now() + Duration::from_secs(time);

        let future = Delay { when, waker: None };

        let out = future.await;
        info!("Completed: {:#?}", out);
    }

    for i in 1..10 {
        mini_tokio.spawn(complete_with_delay(i * 5));
    }

    mini_tokio.run();
}
