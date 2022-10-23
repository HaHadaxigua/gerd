use std::io;
use std::io::Cursor;
use crate::frame::{self, Frame};

use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

/// Send and receive `Frame` values from a remote peer.
///
/// When implementing networking protocols, a message on that protocol is
/// often composed of several smaller messages known as frames. The purpose of
/// `Connection` is to read and write frames on the underlying `TcpStream`.
///
/// To read frames, the `Connection` uses an internal buffer, which is filled
/// up until there are enough bytes to create a full frame. Once this happens,
/// the `Connection` creates the frame and returns it to the caller.
///
#[derive(Debug)]
pub struct Connection {
	// The `TcpStream`. It is decorated with a `BufWriter`, which provides write
	// level buffering. The `BufWriter` implementation provided by Tokio is
	// sufficient for our needs.
	stream: BufWriter<TcpStream>,

	// The buffer for reading frames.
	buffer: BytesMut,
}

impl Connection {
	pub fn new(socket: TcpStream) -> Connection {
		Connection {
			stream: BufWriter::new(socket),
			buffer: BytesMut::with_capacity(4 * 1024),
		}
	}

	pub async fn read_frame(&mut self) -> crate::Result<Option<Frame>> {
		loop {
			// Attempt to parse a frame from the buffered data. If enough data
			// has been buffered, the frame is returned.
			if let Some(frame) = self.parse_frame()? {
				return Ok(Some(frame));
			}

			// There is not enough buffered data to read a frame. Attempt to
			// read more data from the socket.
			//
			// On success, the number of bytes is returned. `0` indicates "end
			// of stream".
			if 0 == self.stream.read_buf(&mut self.buffer).await? {
				// The remote closed the connection. For this to be a clean
				// shutdown, there should be no data in the read buffer. If
				// there is, this means that the peer closed the socket while
				// sending a frame.
				if self.buffer.is_empty() {
					return Ok(None);
				} else {
					return Err("connection reset by peer".into());
				}
			}
		}
	}

	fn parse_frame(&mut self) -> crate::Result<Option<Frame>> {
		use frame::Error::Incomplete;

		let mut buf = Cursor::new(&self.buffer[..]);

		// The first step is to check if enough data has been buffered to parse
		// a single frame.
		match Frame::check(&mut buf) {
			Ok(_) => {
				// The `check` function will have advanced the cursor until the
				// end of the frame. Since the cursor had position set to zero
				// before `Frame::check` was called, we obtain the length of the
				// frame by checking the cursor position.
				let len = buf.position() as usize;

				// Reset the position to zero before passing the cursor to
				// `Frame::parse`.
				buf.set_position(0);

				// Parse the frame from the buffer. This allocates the necessary
				// structures to represent the frame and returns the frame
				// value.
				//
				// If the encoded frame representation is invalid, an error is
				// returned. This should terminate the **current** connection
				// but should not impact any other connected client.
				let frame = Frame::parse(&mut buf)?;

				// Discard the parsed data from the read buffer.
				//
				// When `advance` is called on the read buffer, all of the data
				// up to `len` is discarded. The details of how this works is
				// left to `BytesMut`. This is often done by moving an internal
				// cursor, but it may be done by reallocating and copying data.
				self.buffer.advance(len);

				// Return the parsed frame to the caller.
				Ok(Some(frame))
			}
			// There is not enough data present in the read buffer to parse a
			// single frame. We must wait for more data to be received from the
			// socket. Reading from the socket will be done in the statement
			// after this `match`.
			//
			// We do not want to return `Err` from here as this "error" is an
			// expected runtime condition.
			Err(Incomplete) => Ok(None),
			// An error was encountered while parsing the frame. The connection
			// is now in an invalid state. Returning `Err` from here will result
			// in the connection being closed.
			Err(e) => Err(e.into()),
		}
	}

	/// Write a single `Frame` value to the underlying stream.
	pub async fn write_frame(&mut self, frame: &Frame) -> io::Result<()> {
		match frame {
			Frame::Array(val) => {
				self.stream.write_u8(b'*').await?;

				self.write_decimal(val.len() as u64).await?;

				for entry in &**val {
					self.write_value(entry).await?;
				}
			}
			// The frame type is a literal. Encode the value directly.
			_ => self.write_value(frame).await?,
		}

		self.stream.flush().await
	}

	async fn write_value(&mut self, frame: &Frame) -> io::Result<()> {
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
			Frame::Null => {
				self.stream.write_all(b"$-1\r\n").await?;
			}
			Frame::Bulk(val) => {
				let len = val.len();

				self.stream.write_u8(b'$').await?;
				self.write_decimal(len as u64).await?;
				self.stream.write_all(val).await?;
				self.stream.write_all(b"\r\n").await?;
			}
			// Encoding an `Array` from within a value cannot be done using a
			// recursive strategy. In general, async fns do not support
			// recursion. Mini-redis has not needed to encode nested arrays yet,
			// so for now it is skipped.
			Frame::Array(_val) => unreachable!(),
		}

		Ok(())
	}

	async fn write_decimal(&mut self, val: u64) -> io::Result<()> {
		use std::io::Write;

		let mut buf = [0u8; 20];
		let mut buf = Cursor::new(&mut buf[..]);
		write!(&mut buf, "{}", val)?;

		let pos = buf.position() as usize;
		self.stream.write_all(&buf.get_ref()[..pos]).await?;
		self.stream.write_all(b"\r\n").await?;

		Ok(())
	}
}


