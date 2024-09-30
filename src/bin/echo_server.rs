use log::info;
use simple_logger::SimpleLogger;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
  SimpleLogger::new().init().unwrap();

  let listener = TcpListener::bind("127.0.0.1:6142").await?;

  loop {
    let (mut socket, _) = listener.accept().await?;

    tokio::spawn(async move {
      let mut buf = vec![0; 1024];

      loop {
        let size = socket.read(&mut buf).await;
        info!("Request to write: {:#?}", size);
        match size {
          Ok(0) => {
            return;
          }
          Ok(n) => {
            if socket.write_all(&buf[..n]).await.is_err() {
              return;
            }
          }
          Err(_) => {
            return;
          }
        }
      }
    });
  }
}
