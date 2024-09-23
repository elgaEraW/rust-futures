use core::str;
use std::io;

use log::info;
use simple_logger::SimpleLogger;
use tokio::{
  fs::File,
  io::{AsyncReadExt, AsyncWriteExt},
};

#[tokio::main]
async fn main() -> io::Result<()> {
  SimpleLogger::new().init().unwrap();
  let mut f = File::open("foo.txt").await?;

  let mut buffer = [0; 11];

  let n = f.read(&mut buffer[..]).await?;

  info!("The bytes: {:?}", str::from_utf8(&buffer[..n]));

  let mut file = File::create("bar.txt").await?;

  let n = file.write(b"some bytes").await?;

  info!("wrote {} bytes", n);

  Ok(())
}
