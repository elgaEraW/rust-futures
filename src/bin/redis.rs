use bytes::Bytes;
use mini_redis::{Command, Connection, Frame};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

struct SharedState {
  db: HashMap<String, Bytes>,
  counter: usize,
}

type SharedStateType = Arc<Mutex<SharedState>>;

#[tokio::main]
async fn main() {
  let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
  let db = HashMap::new();
  let shared_state = Arc::new(Mutex::new(SharedState { db, counter: 0 }));

  loop {
    let (socket, _) = listener.accept().await.unwrap();
    let shared_state = shared_state.clone();

    println!(
      "Accepted, Counter: {:#?}",
      shared_state.lock().await.counter
    );
    tokio::spawn(async move {
      process(socket, shared_state).await;
    });
  }
}

async fn process(socket: TcpStream, shared_state: SharedStateType) {
  let mut connection = Connection::new(socket);

  while let Some(frame) = connection.read_frame().await.unwrap() {
    let response = match Command::from_frame(frame).unwrap() {
      Command::Set(cmd) => {
        let mut shared_state = shared_state.lock().await;
        shared_state.counter += 1;
        shared_state
          .db
          .insert(cmd.key().to_string(), cmd.value().clone());
        Frame::Simple("OK".into())
      }
      Command::Get(cmd) => {
        let shared_state = shared_state.lock().await;

        let should_block = shared_state.counter % 2 == 0;

        // if shared_state.counter % 2 == 0 {
        //   tokio::time::sleep(Duration::from_secs(15)).await;
        // }

        let frame = if let Some(value) = shared_state.db.get(cmd.key()) {
          Frame::Bulk(value.clone())
        } else {
          Frame::Null
        };

        drop(shared_state);

        if should_block {
          tokio::time::sleep(Duration::from_secs(15)).await;
        }

        frame
      }
      cmd => {
        panic!("unimplemented {:#?}", cmd)
      }
    };

    connection.write_frame(&response).await.unwrap();
  }
}
