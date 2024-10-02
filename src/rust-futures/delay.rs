use std::{future::Future, task::Poll, thread, time::Duration};
use std::sync::{Arc, Mutex};
use std::task::Waker;
use log::info;
use simple_logger::SimpleLogger;
use std::time::Instant;

pub struct Delay {
  pub when: Instant,
  pub waker: Option<Arc<Mutex<Waker>>>,
}

impl Future for Delay {
  type Output = &'static str;

  fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
    info!("Polling");
    if Instant::now() >= self.when {
      info!("Done");
      return Poll::Ready("done");
    }

    if let Some(waker) = &self.waker {
      let mut waker = waker.lock().unwrap();

      if !waker.will_wake(cx.waker()) {
        *waker = cx.waker().clone();
      }
    } else {
      let waker = Arc::new(Mutex::new(cx.waker().clone()));
      let when = self.when;
      self.waker = Some(waker.clone());

      thread::spawn(move || {

        let now = Instant::now();

        if now < when {
          thread::sleep(when - now);
        }

        let waker = waker.lock().unwrap();
        waker.wake_by_ref();
      });
    }

      Poll::Pending
    }

}

#[tokio::main]
async fn main() {
  SimpleLogger::new().init().unwrap();
  let delay = Delay {
    when: Instant::now() + Duration::from_secs(2),
    waker: None
  };
  let res = delay.await;

  info!("After future: {:#?}", res);
}
