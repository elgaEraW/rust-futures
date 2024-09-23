use core::str;

use log::info;
use simple_logger::SimpleLogger;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> io::Result<()> {
  SimpleLogger::new().init().unwrap();

  let stream = TcpStream::connect("127.0.0.1:6142").await?;

  let (mut rd, mut wr) = io::split(stream);

  tokio::spawn(async move {
    wr.write_all(b"hello\r\n").await?;
    wr.write_all(b"world\r\n").await?;

    drop(wr);

    Ok::<_, io::Error>(())
  });

  let mut buf = vec![0; 128];

  loop {
    let n = rd.read(&mut buf).await?;

    if n == 0 {
      break;
    }

    info!("GOT {:?}", str::from_utf8(&buf[..n]));
  }

  Ok(())
}
