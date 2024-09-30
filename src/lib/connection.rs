use bytes::{Buf, BytesMut};
use mini_redis::{frame::Error::Incomplete, Frame, Result};
use std::io::Cursor;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

struct Connection {
  stream: BufWriter<TcpStream>,
  buffer: BytesMut,
  cursor: usize,
}

impl Connection {
  pub fn new(stream: TcpStream) -> Self {
    Self {
      stream: BufWriter::new(stream),
      buffer: BytesMut::with_capacity(4096),
      cursor: 0,
    }
  }

  pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
    loop {
      if let Some(frame) = self.parse_frame()? {
        return Ok(Some(frame));
      }

      if self.buffer.len() == self.cursor {
        self.buffer.resize(self.cursor * 2, 0);
      }

      let n = self.stream.read(&mut self.buffer[self.cursor..]).await?;

      if 0 == n {
        return if self.cursor == 0 {
          Ok(None)
        } else {
          Err("Connection reset by peer".into())
        };
      } else {
        self.cursor += n;
      }
    }
  }

  pub async fn write_frame(&mut self, frame: &Frame) -> Result<()> {
    match frame {
      Frame::Simple(val) => {
        self.stream.write_u8(b'+').await?;
        self.stream.write_all(val.as_bytes()).await?;
        self.stream.write_all(b"\r\n").await?;
      }
      Frame::Error(val) => {
        self.stream.write_u8(b'-').await?;
        self.stream.write_all(val.as_bytes()).await?;
        self.stream.write_all(b"\r\n").await?;
      }
      Frame::Integer(val) => {
        self.stream.write_u8(b':').await?;
        self.write_decimal(*val).await?;
      }
      Frame::Bulk(val) => {
        let len = val.len();

        self.stream.write_u8(b'$').await?;
        self.write_decimal(len as u64).await?;
        self.stream.write_all(val).await?;
        self.stream.write_all(b"\r\n").await?;
      }
      Frame::Null => {
        self.stream.write_all(b"$-1\r\n").await?;
      }
      Frame::Array(_) => {
        unimplemented!()
      }
    }
    self.stream.flush().await?;

    Ok(())
  }

  async fn write_decimal(&mut self, val: u64) -> io::Result<()> {
    use std::io::Write;

    // Convert the value to a string
    let mut buf = [0u8; 12];
    let mut buf = Cursor::new(&mut buf[..]);
    write!(&mut buf, "{}", val)?;

    let pos = buf.position() as usize;
    self.stream.write_all(&buf.get_ref()[..pos]).await?;
    self.stream.write_all(b"\r\n").await?;

    Ok(())
  }

  fn parse_frame(&mut self) -> Result<Option<Frame>> {
    let mut buf = Cursor::new(&self.buffer[..]);

    match Frame::check(&mut buf) {
      Ok(_) => {
        let len = buf.position() as usize;

        buf.set_position(0);

        let frame = Frame::parse(&mut buf)?;

        self.buffer.advance(len);

        Ok(Some(frame))
      }
      Err(Incomplete) => Ok(None),
      Err(e) => Err(e.into()),
    }
  }
}
