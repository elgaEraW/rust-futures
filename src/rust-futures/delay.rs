use std::{future::Future, task::Poll, time::Duration};

use log::info;
use simple_logger::SimpleLogger;
use tokio::time::Instant;

pub struct Delay {
  pub when: Instant,
}

impl Future for Delay {
  type Output = &'static str;

  fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
    if Instant::now() >= self.when {
      info!("Done");
      Poll::Ready("done")
    } else {
      cx.waker().wake_by_ref();
      Poll::Pending
    }
  }
}

#[tokio::main]
async fn main() {
  SimpleLogger::new().init().unwrap();
  let delay = Delay {
    when: Instant::now() + Duration::from_secs(2),
  };
  let res = delay.await;

  info!("After future: {:#?}", res);
}
