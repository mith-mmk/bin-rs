use bin_rs::reader::{BinaryReader, BytesReader};
use std::io::SeekFrom;

#[test]
fn oversized_skip_is_an_error_without_moving() {
  let mut reader = BytesReader::new(b"ab");
  reader.skip_ptr(1).unwrap();
  assert!(reader.skip_ptr(usize::MAX).is_err());
  assert_eq!(reader.offset().unwrap(), 1);
}

#[test]
fn extreme_current_seek_is_an_error_without_moving() {
  let mut reader = BytesReader::new(b"ab");
  reader.skip_ptr(1).unwrap();
  assert!(reader.seek(SeekFrom::Current(i64::MAX)).is_err());
  assert_eq!(reader.offset().unwrap(), 1);
}

#[test]
fn extreme_end_seek_is_an_error_without_moving() {
  let mut reader = BytesReader::new(b"ab");
  assert!(reader.seek(SeekFrom::End(i64::MAX)).is_err());
  assert_eq!(reader.offset().unwrap(), 0);
}
