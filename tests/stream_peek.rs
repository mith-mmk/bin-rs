use bin_rs::reader::{BinaryReader, StreamReader};
use std::io::{self, BufRead, BufReader, Cursor, Read, Seek, SeekFrom};

#[test]
fn peek_crosses_small_buffer_boundaries_without_consuming() -> io::Result<()> {
  let data: Vec<u8> = (0..=255).cycle().take(600).collect();
  for capacity in [0, 1, 2, 8, 16, 255, 256, 8192] {
    let mut reader = StreamReader::new(BufReader::with_capacity(capacity, Cursor::new(&data)));
    for start in [0, 1, 7, 255] {
      reader.seek(SeekFrom::Start(start))?;
      let expected = &data[start as usize..start as usize + 256];
      assert_eq!(reader.read_bytes_no_move(256)?, expected);
      assert_eq!(reader.offset()?, start);
      assert_eq!(reader.read_bytes_no_move(256)?, expected);
      assert_eq!(reader.read_bytes_as_vec(256)?, expected);
    }
  }
  Ok(())
}

#[test]
fn peek_refills_a_partly_consumed_buffer() -> io::Result<()> {
  let mut reader = StreamReader::new(BufReader::with_capacity(
    8,
    Cursor::new(b"abcdefghijklmnop"),
  ));
  assert_eq!(reader.read_u8()?, b'a');
  assert_eq!(reader.read_bytes_no_move(8)?, b"bcdefghi");
  assert_eq!(reader.offset()?, 1);
  assert_eq!(reader.read_bytes_as_vec(8)?, b"bcdefghi");
  Ok(())
}

#[test]
fn truncated_peek_restores_position_and_returns_unexpected_eof() -> io::Result<()> {
  for capacity in [1, 8, 8192] {
    let mut reader = StreamReader::new(BufReader::with_capacity(capacity, Cursor::new(b"abc")));
    assert_eq!(reader.read_u8()?, b'a');
    let error = reader.read_bytes_no_move(3).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
    assert_eq!(reader.offset()?, 1);
    assert_eq!(reader.read_bytes_no_move(2)?, b"bc");
    assert_eq!(reader.read_bytes_as_vec(2)?, b"bc");
    assert!(reader.read_bytes_no_move(0)?.is_empty());
    assert_eq!(
      reader.read_bytes_no_move(1).unwrap_err().kind(),
      io::ErrorKind::UnexpectedEof
    );
    assert_eq!(reader.offset()?, 3);
  }
  Ok(())
}

struct ShortReader {
  cursor: Cursor<&'static [u8]>,
  fail_read_at: Option<u64>,
  interrupt_read: bool,
  fail_fill: bool,
  fail_position: bool,
  fail_restore: bool,
  seeks: usize,
}

impl ShortReader {
  fn new() -> Self {
    Self {
      cursor: Cursor::new(b"abcdefghijklmnop"),
      fail_read_at: None,
      interrupt_read: false,
      fail_fill: false,
      fail_position: false,
      fail_restore: false,
      seeks: 0,
    }
  }
}

impl Read for ShortReader {
  fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
    if self.interrupt_read {
      self.interrupt_read = false;
      return Err(io::Error::new(io::ErrorKind::Interrupted, "interrupted"));
    }
    if self.fail_read_at == Some(self.cursor.position()) {
      return Err(io::Error::new(io::ErrorKind::BrokenPipe, "read failed"));
    }
    let len = buffer.len().min(1);
    self.cursor.read(&mut buffer[..len])
  }
}

impl BufRead for ShortReader {
  fn fill_buf(&mut self) -> io::Result<&[u8]> {
    if self.fail_fill {
      return Err(io::Error::new(
        io::ErrorKind::PermissionDenied,
        "fill failed",
      ));
    }
    let bytes = self.cursor.fill_buf()?;
    Ok(&bytes[..bytes.len().min(1)])
  }

  fn consume(&mut self, len: usize) {
    self.cursor.consume(len);
  }
}

impl Seek for ShortReader {
  fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
    self.seeks += 1;
    if self.fail_restore {
      return Err(io::Error::new(io::ErrorKind::Other, "restore failed"));
    }
    self.cursor.seek(position)
  }

  fn stream_position(&mut self) -> io::Result<u64> {
    if self.fail_position {
      return Err(io::Error::new(io::ErrorKind::Other, "position failed"));
    }
    Ok(self.cursor.position())
  }
}

#[test]
fn peek_handles_short_and_interrupted_reads() -> io::Result<()> {
  let mut source = ShortReader::new();
  source.cursor.set_position(2);
  source.interrupt_read = true;
  let mut reader = StreamReader::new(&mut source);
  assert_eq!(reader.read_bytes_no_move(8)?, b"cdefghij");
  assert_eq!(reader.offset()?, 2);
  assert_eq!(reader.read_bytes_as_vec(8)?, b"cdefghij");
  Ok(())
}

#[test]
fn read_error_after_partial_progress_restores_position() -> io::Result<()> {
  let mut source = ShortReader::new();
  source.cursor.set_position(2);
  source.fail_read_at = Some(5);
  let mut reader = StreamReader::new(&mut source);
  let error = reader.read_bytes_no_move(8).unwrap_err();
  assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
  assert_eq!(reader.offset()?, 2);
  assert_eq!(reader.read_bytes_as_vec(3)?, b"cde");
  Ok(())
}

#[test]
fn restore_failure_is_reported_even_after_read_failure() {
  for fail_read_at in [None, Some(3)] {
    let mut source = ShortReader::new();
    source.fail_read_at = fail_read_at;
    source.fail_restore = true;
    let mut reader = StreamReader::new(&mut source);
    let error = reader.read_bytes_no_move(8).unwrap_err();
    assert_eq!(error.to_string(), "restore failed");
    assert_eq!(source.seeks, 1);
  }
}

#[test]
fn position_failure_does_not_consume_input() {
  let mut source = ShortReader::new();
  source.fail_position = true;
  let mut reader = StreamReader::new(&mut source);
  assert_eq!(
    reader.read_bytes_no_move(8).unwrap_err().to_string(),
    "position failed"
  );
  assert_eq!(source.cursor.position(), 0);
  assert_eq!(source.seeks, 0);
}

#[test]
fn buffered_peek_does_not_seek() -> io::Result<()> {
  let mut source = ShortReader::new();
  source.fail_position = true;
  source.fail_restore = true;
  let mut reader = StreamReader::new(&mut source);
  assert_eq!(reader.read_bytes_no_move(1)?, b"a");
  assert_eq!(source.cursor.position(), 0);
  assert_eq!(source.seeks, 0);
  Ok(())
}

#[test]
fn empty_peek_performs_no_io_and_fill_errors_propagate() -> io::Result<()> {
  let mut source = ShortReader::new();
  source.fail_fill = true;
  source.fail_position = true;
  source.fail_restore = true;
  let mut reader = StreamReader::new(&mut source);
  assert!(reader.read_bytes_no_move(0)?.is_empty());
  assert_eq!(
    reader.read_bytes_no_move(1).unwrap_err().kind(),
    io::ErrorKind::PermissionDenied
  );
  assert_eq!(source.cursor.position(), 0);
  assert_eq!(source.seeks, 0);
  Ok(())
}
