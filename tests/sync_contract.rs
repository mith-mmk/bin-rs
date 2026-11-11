//! Characterize the 0.0.11 baseline; policy changes need explicit migration notes.
use bin_rs::reader::{BinaryReader, BytesReader, StreamReader};
use bin_rs::Endian;
use std::io::{self, BufReader, Cursor, ErrorKind, SeekFrom};

fn readers(data: &[u8]) -> Vec<Box<dyn BinaryReader>> {
  vec![
    Box::new(BytesReader::new(data)),
    Box::new(StreamReader::new(Cursor::new(data.to_vec()))),
    Box::new(StreamReader::new(BufReader::with_capacity(
      1,
      Cursor::new(data.to_vec()),
    ))),
  ]
}

#[test]
fn ascii_maps_every_nonzero_byte_without_windows_1252_substitution() -> io::Result<()> {
  let bytes: Vec<u8> = (1..=255).collect();
  let expected: String = bytes.iter().map(|&b| char::from(b)).collect();
  for mut reader in readers(&bytes) {
    assert_eq!(reader.read_ascii_string(bytes.len())?, expected);
    assert_eq!(reader.offset()?, 255);
  }
  Ok(())
}

#[test]
fn ascii_nul_consumes_padding_even_when_text_is_empty() -> io::Result<()> {
  for mut reader in readers(&[0, 0xff, b'!']) {
    assert_eq!(reader.read_ascii_string(2)?, "");
    assert_eq!(reader.offset()?, 2);
    assert_eq!(reader.read_u8()?, b'!');
  }
  Ok(())
}

#[test]
fn odd_utf16_lengths_leave_the_unpaired_byte_unread() -> io::Result<()> {
  for mut reader in readers(&[0, b'A', 0xff]) {
    reader.set_endian(Endian::LittleEndian);
    assert_eq!(reader.read_utf16be_string(1)?, "");
    assert_eq!(reader.offset()?, 0);
    assert_eq!(reader.read_utf16be_string(3)?, "A");
    assert_eq!(reader.offset()?, 2);
    assert_eq!(reader.endian(), Endian::LittleEndian);
    assert_eq!(reader.read_u8()?, 0xff);
  }
  for mut reader in readers(&[b'A', 0, 0xff]) {
    reader.set_endian(Endian::BigEndian);
    assert_eq!(reader.read_utf16le_string(3)?, "A");
    assert_eq!(reader.offset()?, 2);
    assert_eq!(reader.endian(), Endian::BigEndian);
    assert_eq!(reader.read_u8()?, 0xff);
  }
  Ok(())
}

#[test]
fn composite_string_reads_are_not_transactional() -> io::Result<()> {
  for mut reader in readers(b"A") {
    assert!(reader.read_utf8_string(2).is_err());
    assert_eq!(reader.offset()?, 1);
  }
  // One complete code unit is consumed before the missing next unit is read.
  for mut reader in readers(&[0, b'A']) {
    reader.set_endian(Endian::LittleEndian);
    assert!(reader.read_utf16be_string(4).is_err());
    assert_eq!(reader.offset()?, 2);
    assert_eq!(reader.endian(), Endian::LittleEndian);
  }
  Ok(())
}

#[test]
fn invalid_encoding_reports_other_after_consuming_the_field() -> io::Result<()> {
  for mut reader in readers(&[0xff, b'!']) {
    assert_eq!(reader.read_utf8_string(1).unwrap_err().kind(), ErrorKind::Other);
    assert_eq!(reader.read_u8()?, b'!');
  }
  for mut reader in readers(&[0xd8, 0, b'!']) {
    assert_eq!(reader.read_utf16be_string(2).unwrap_err().kind(), ErrorKind::Other);
    assert_eq!(reader.read_u8()?, b'!');
  }
  Ok(())
}

#[test]
fn truncated_reads_keep_backend_specific_error_kinds() -> io::Result<()> {
  let mut bytes = BytesReader::new(&[1]);
  assert_eq!(bytes.read_u16_be().unwrap_err().kind(), ErrorKind::Other);
  assert_eq!(bytes.offset()?, 0);
  let mut stream = StreamReader::new(BufReader::with_capacity(1, Cursor::new(vec![1])));
  assert_eq!(stream.read_u16_be().unwrap_err().kind(), ErrorKind::UnexpectedEof);
  // No assertion about failed stream read destination contents or position:
  // std::io::Read::read_exact does not promise transactional behavior.
  Ok(())
}

#[test]
fn seek_beyond_end_is_backend_specific() -> io::Result<()> {
  let mut bytes = BytesReader::new(&[1]);
  assert_eq!(bytes.seek(SeekFrom::Start(2)).unwrap_err().kind(), ErrorKind::Other);
  assert_eq!(bytes.offset()?, 0);
  let mut stream = StreamReader::new(Cursor::new(vec![1]));
  assert_eq!(stream.seek(SeekFrom::Start(2))?, 2);
  assert_eq!(stream.read_u8().unwrap_err().kind(), ErrorKind::UnexpectedEof);
  assert_eq!(stream.seek(SeekFrom::Start(0))?, 0);
  assert_eq!(stream.read_u8()?, 1);
  Ok(())
}

#[cfg(feature = "util")]
#[test]
fn util_strings_have_distinct_nul_and_lossy_semantics() -> io::Result<()> {
  let data = [b'?', 0x80, 0, 0xff, b'!'];
  assert_eq!(bin_rs::io::read_ascii_string(&data, 1, 3), "\u{80}\0ÿ");
  assert_eq!(BytesReader::new(&data[1..4]).read_ascii_string(3)?, "\u{80}");
  // The util helper stops at NUL, whereas the reader preserves it.
  let utf16 = [0, b'A', 0, 0, 0, b'B'];
  assert_eq!(bin_rs::io::read_utf16_string(&utf16, 0, 6, Endian::BigEndian), "A");
  assert_eq!(BytesReader::new(&utf16).read_utf16be_string(6)?, "A\0B");
  let invalid = [0xd8, 0];
  assert_eq!(bin_rs::io::read_utf16_string(&invalid, 0, 2, Endian::BigEndian), "\u{fffd}");
  assert!(BytesReader::new(&invalid).read_utf16be_string(2).is_err());
  Ok(())
}
