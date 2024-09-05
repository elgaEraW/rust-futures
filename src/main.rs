use std::{
  sync::{
    mpsc::{sync_channel, Receiver, SyncSender},
    Arc, Mutex,
  },
  task::Context,
  time::Duration,
};

use futures::{
  future::BoxFuture,
  task::{waker_ref, ArcWake},
  Future, FutureExt,
};
use timer::TimerFuture;

struct Executor {
  ready_queue: Receiver<Arc<Task>>,
}

impl Executor {
  fn run(&self) {
    while let Ok(task) = self.ready_queue.recv() {
      let mut future_slot = task.future.lock().unwrap();
      if let Some(mut future) = future_slot.take() {
        let waker = waker_ref(&task);
        let context = &mut Context::from_waker(&waker);

        if future.as_mut().poll(context).is_pending() {
          *future_slot = Some(future);
        }
      }
    }
  }
}

#[derive(Clone)]
struct Spawner {
  task_sender: SyncSender<Arc<Task>>,
}

impl Spawner {
  fn spawn(&self, future: impl Future<Output=()> + 'static + Send) {
    let future = future.boxed();
    let task = Arc::new(Task {
      future: Mutex::new(Some(future)),
      task_sender: self.task_sender.clone(),
    });
    let res = self.task_sender.send(task);
    if res.is_err() {
      println!("too many tasks queued");
    }
  }
}

struct Task {
  future: Mutex<Option<BoxFuture<'static, ()>>>,
  task_sender: SyncSender<Arc<Task>>,
}

impl ArcWake for Task {
  fn wake_by_ref(arc_self: &Arc<Self>) {
    let cloned = arc_self.clone();
    if arc_self.task_sender.send(cloned).is_err() {
      println!("too many tasks queued");
    }
  }
}

fn new_executor_and_spawner() -> (Executor, Spawner) {
  const MAX_QUEUED_TASKS: usize = 10_000;

  let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
  (Executor { ready_queue }, Spawner { task_sender })
}

fn main() {
  let (executor, spawner) = new_executor_and_spawner();

  for _ in 0..5_000 {
    spawner.spawn(async {
      println!("aa");

      TimerFuture::new(Duration::from_secs(2)).await;

      println!("bb");
    });
  }

  drop(spawner);

  executor.run();
}