use bin_rs::async_reader::AsyncBytesReader;
use std::io::Cursor;
use tokio::io::BufReader;

#[tokio::test]
async fn exact_boundary_peek_should_succeed() {
  let mut reader = AsyncBytesReader::new(BufReader::new(Cursor::new(b"abc")));
  assert_eq!(reader.read_bytes_no_move(3).await.unwrap(), b"abc");
  assert_eq!(reader.offset().await.unwrap(), 0);
}

#[tokio::test]
async fn zero_byte_peek_at_eof_should_succeed() {
  let mut reader = AsyncBytesReader::new(BufReader::new(Cursor::new(b"")));
  assert!(reader.read_bytes_no_move(0).await.unwrap().is_empty());
}

#[tokio::test]
async fn peek_larger_than_buffer_should_succeed_when_input_exists() {
  let mut reader = AsyncBytesReader::new(BufReader::with_capacity(2, Cursor::new(b"abcdef")));
  assert_eq!(reader.read_bytes_no_move(3).await.unwrap(), b"abc");
  assert_eq!(reader.offset().await.unwrap(), 0);
}
