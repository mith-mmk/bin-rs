#![cfg(feature = "async")]

use bin_rs::{async_reader::AsyncBytesReader, Endian};
use std::io::{Cursor, ErrorKind, SeekFrom};
use tokio::io::BufReader;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn reader(data: &[u8], capacity: usize) -> AsyncBytesReader<BufReader<Cursor<Vec<u8>>>> {
  AsyncBytesReader::new(BufReader::with_capacity(
    capacity,
    Cursor::new(data.to_vec()),
  ))
}

macro_rules! integer_edges {
    ($name:ident, $ty:ty, $read:ident, $be:ident, $le:ident, [$($value:expr),+]) => {
        #[tokio::test]
        async fn $name() -> TestResult {
            for value in [$($value),+] {
                for endian in [Endian::BigEndian, Endian::LittleEndian] {
                    let bytes = match endian {
                        Endian::BigEndian => <$ty>::to_be_bytes(value),
                        Endian::LittleEndian => <$ty>::to_le_bytes(value),
                    };
                    // Numeric reads must work across a one-byte buffer.
                    let mut input = reader(&bytes, 1);
                    input.set_endian(endian);
                    assert_eq!(input.$read().await?, value);
                    assert_eq!(input.offset().await?, bytes.len() as u64);
                    let error = input.read_u8().await.unwrap_err();
                    assert_eq!(error.downcast_ref::<std::io::Error>().unwrap().kind(), ErrorKind::UnexpectedEof);

                    // Explicit endian methods must not depend on the configured endian.
                    let mut input = reader(&<$ty>::to_be_bytes(value), 1);
                    input.set_endian(Endian::LittleEndian);
                    assert_eq!(input.$be().await?, value);
                    let mut input = reader(&<$ty>::to_le_bytes(value), 1);
                    input.set_endian(Endian::BigEndian);
                    assert_eq!(input.$le().await?, value);
                }
            }
            Ok(())
        }
    };
}

integer_edges!(
  u16_edges,
  u16,
  read_u16,
  read_u16_be,
  read_u16_le,
  [0, 1, 0x1234, u16::MAX]
);
integer_edges!(
  u32_edges,
  u32,
  read_u32,
  read_u32_be,
  read_u32_le,
  [0, 1, 0x12345678, u32::MAX]
);
integer_edges!(
  u64_edges,
  u64,
  read_u64,
  read_u64_be,
  read_u64_le,
  [0, 1, 0x0123456789abcdef, u64::MAX]
);
integer_edges!(
  u128_edges,
  u128,
  read_u128,
  read_u128_be,
  read_u128_le,
  [0, 1, 0x0123456789abcdef_fedcba9876543210, u128::MAX]
);
integer_edges!(
  i16_edges,
  i16,
  read_i16,
  read_i16_be,
  read_i16_le,
  [i16::MIN, -1, 0, 1, 0x1234, i16::MAX]
);
integer_edges!(
  i32_edges,
  i32,
  read_i32,
  read_i32_be,
  read_i32_le,
  [i32::MIN, -1, 0, 1, 0x12345678, i32::MAX]
);
integer_edges!(
  i64_edges,
  i64,
  read_i64,
  read_i64_be,
  read_i64_le,
  [i64::MIN, -1, 0, 1, 0x0123456789abcdef, i64::MAX]
);
integer_edges!(
  i128_edges,
  i128,
  read_i128,
  read_i128_be,
  read_i128_le,
  [
    i128::MIN,
    -1,
    0,
    1,
    0x0123456789abcdef_fedcba9876543210,
    i128::MAX
  ]
);

#[tokio::test]
async fn byte_signed_and_unsigned_boundaries() -> TestResult {
  let mut input = reader(&[0, 127, 128, 255, 0, 127, 128, 255], 1);
  for expected in [0, 127, 128, 255] {
    assert_eq!(input.read_u8().await?, expected);
  }
  for expected in [0, 127, -128, -1] {
    assert_eq!(input.read_i8().await?, expected);
  }
  assert_eq!(input.offset().await?, 8);
  Ok(())
}

#[tokio::test]
async fn partial_eof_reports_unexpected_eof_and_consumed_offset() -> TestResult {
  for capacity in [1, 2, 8] {
    let mut input = reader(b"abc", capacity);
    let error = input.read_u32_be().await.unwrap_err();
    assert_eq!(
      error.downcast_ref::<std::io::Error>().unwrap().kind(),
      ErrorKind::UnexpectedEof
    );
    assert_eq!(input.offset().await?, 3);
    assert_eq!(input.seek(SeekFrom::Start(0)).await?, 0);
    let error = input.read_bytes_as_vec(4).await.unwrap_err();
    assert_eq!(
      error.downcast_ref::<std::io::Error>().unwrap().kind(),
      ErrorKind::UnexpectedEof
    );
    assert_eq!(input.offset().await?, 3);
    input.seek(SeekFrom::Start(0)).await?;
    let error = input.skip_ptr(4).await.unwrap_err();
    assert_eq!(
      error.downcast_ref::<std::io::Error>().unwrap().kind(),
      ErrorKind::UnexpectedEof
    );
    assert_eq!(input.offset().await?, 3);
  }
  Ok(())
}

#[tokio::test]
async fn buffered_peek_repeats_without_consuming() -> TestResult {
  let mut input = reader(b"abcdef", 6);
  assert_eq!(input.read_u8().await?, b'a');
  assert!(input.read_bytes_no_move(0).await?.is_empty());
  assert_eq!(input.read_bytes_no_move(3).await?, b"bcd");
  assert_eq!(input.read_bytes_no_move(3).await?, b"bcd");
  assert_eq!(input.offset().await?, 1);
  assert_eq!(input.read_bytes_as_vec(3).await?, b"bcd");
  assert_eq!(input.offset().await?, 4);
  Ok(())
}

#[tokio::test]
async fn seek_and_skip_track_logical_offsets() -> TestResult {
  let mut input = reader(b"abcdefgh", 3);
  assert_eq!(input.skip_ptr(2).await?, 2);
  assert_eq!(input.offset().await?, 2);
  assert_eq!(input.seek(SeekFrom::Current(2)).await?, 4);
  assert_eq!(input.read_u8().await?, b'e');
  assert_eq!(input.seek(SeekFrom::Current(-3)).await?, 2);
  assert_eq!(input.read_u8().await?, b'c');
  assert_eq!(input.seek(SeekFrom::End(-1)).await?, 7);
  assert_eq!(input.read_u8().await?, b'h');
  assert_eq!(input.seek(SeekFrom::Start(0)).await?, 0);
  assert_eq!(input.read_bytes_as_vec(8).await?, b"abcdefgh");
  assert_eq!(input.seek(SeekFrom::Start(12)).await?, 12);
  assert!(input.read_u8().await.is_err());
  assert_eq!(input.offset().await?, 12);
  Ok(())
}

#[tokio::test]
async fn empty_reads_and_skips_succeed_at_eof() -> TestResult {
  let mut input = reader(b"", 1);
  assert!(input.read_bytes_as_vec(0).await?.is_empty());
  assert_eq!(input.skip_ptr(0).await?, 0);
  assert_eq!(input.read_ascii_string(0).await?, "");
  assert_eq!(input.read_utf8_string(0).await?, "");
  assert_eq!(input.offset().await?, 0);
  Ok(())
}

#[tokio::test]
async fn strings_preserve_utf8_nul_but_ascii_stops_at_nul() -> TestResult {
  let mut input = reader("é\0猫!".as_bytes(), 1);
  assert_eq!(input.read_utf8_string(6).await?, "é\0猫");
  assert_eq!(input.offset().await?, 6);
  assert_eq!(input.read_u8().await?, b'!');
  // The documented ASCII field consumes its full width, even after NUL.
  let mut input = reader(&[b'a', 0, 0xff, b'b', b'!'], 1);
  assert_eq!(input.read_ascii_string(4).await?, "a");
  assert_eq!(input.offset().await?, 4);
  assert_eq!(input.read_u8().await?, b'!');
  Ok(())
}

#[tokio::test]
async fn malformed_utf8_errors_after_consuming_the_field() -> TestResult {
  for data in [&[0xff, b'!'][..], &[0xc3, b'!'][..], &[0x80, b'!'][..]] {
    let mut input = reader(data, 1);
    assert!(input.read_utf8_string(1).await.is_err());
    assert_eq!(input.offset().await?, 1);
    assert_eq!(input.read_u8().await?, b'!');
    let mut input = reader(data, 1);
    assert!(input.read_ascii_string(1).await.is_err());
    assert_eq!(input.offset().await?, 1);
    assert_eq!(input.read_u8().await?, b'!');
  }
  Ok(())
}

#[tokio::test]
async fn truncated_strings_report_unexpected_eof() -> TestResult {
  let mut input = reader(b"ab", 1);
  let error = input.read_utf8_string(3).await.unwrap_err();
  assert_eq!(
    error.downcast_ref::<std::io::Error>().unwrap().kind(),
    ErrorKind::UnexpectedEof
  );
  assert_eq!(input.offset().await?, 2);
  input.seek(SeekFrom::Start(0)).await?;
  let error = input.read_ascii_string(3).await.unwrap_err();
  assert_eq!(
    error.downcast_ref::<std::io::Error>().unwrap().kind(),
    ErrorKind::UnexpectedEof
  );
  assert_eq!(input.offset().await?, 2);
  Ok(())
}

macro_rules! float_bits_edges {
    ($name:ident, $ty:ty, $read:ident, $be:ident, $le:ident, [$($bits:expr),+]) => {
        #[tokio::test]
        async fn $name() -> TestResult {
            for bits in [$($bits),+] {
                let value = <$ty>::from_bits(bits);
                for capacity in [1, 2, 3] {
                    for endian in [Endian::BigEndian, Endian::LittleEndian] {
                        let bytes = match endian {
                            Endian::BigEndian => value.to_be_bytes(),
                            Endian::LittleEndian => value.to_le_bytes(),
                        };
                        let mut input = reader(&bytes, capacity);
                        input.set_endian(endian);
                        assert_eq!(input.$read().await?.to_bits(), bits);
                        assert_eq!(input.offset().await?, bytes.len() as u64);
                    }
                    // Compare bits, not floating-point equality: preserve negative
                    // zero and NaN payloads as well as finite values and infinities.
                    let mut input = reader(&value.to_be_bytes(), capacity);
                    input.set_endian(Endian::LittleEndian);
                    assert_eq!(input.$be().await?.to_bits(), bits);
                    let mut input = reader(&value.to_le_bytes(), capacity);
                    input.set_endian(Endian::BigEndian);
                    assert_eq!(input.$le().await?.to_bits(), bits);
                }
            }
            Ok(())
        }
    };
}

float_bits_edges!(
  f32_bits_edges,
  f32,
  read_f32,
  read_f32_be,
  read_f32_le,
  [0x00000000, 0x80000000, 0x00000001, 0x7f7fffff, 0x7f800000, 0xff800000, 0x7fc12345, 0xffc54321]
);
float_bits_edges!(
  f64_bits_edges,
  f64,
  read_f64,
  read_f64_be,
  read_f64_le,
  [
    0x0000000000000000,
    0x8000000000000000,
    0x0000000000000001,
    0x7fefffffffffffff,
    0x7ff0000000000000,
    0xfff0000000000000,
    0x7ff8123456789abc,
    0xfff8abcdef012345
  ]
);
