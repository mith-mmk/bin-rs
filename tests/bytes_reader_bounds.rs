use bin_rs::reader::{BinaryReader, BytesReader};
use std::io::{self, SeekFrom};

#[test]
fn oversized_skip_is_an_error_without_moving() -> io::Result<()> {
  let mut reader = BytesReader::new(b"ab");
  reader.skip_ptr(1)?;
  assert!(reader.skip_ptr(usize::MAX).is_err());
  assert_eq!(reader.offset()?, 1);
  assert_eq!(reader.read_u8()?, b'b');
  Ok(())
}

#[test]
fn extreme_current_seek_is_an_error_without_moving() -> io::Result<()> {
  let mut reader = BytesReader::new(b"ab");
  reader.skip_ptr(1)?;
  assert!(reader.seek(SeekFrom::Current(i64::MAX)).is_err());
  assert_eq!(reader.offset()?, 1);
  assert_eq!(reader.read_u8()?, b'b');
  Ok(())
}

#[test]
fn extreme_end_seek_is_an_error_without_moving() -> io::Result<()> {
  let mut reader = BytesReader::new(b"ab");
  assert!(reader.seek(SeekFrom::End(i64::MAX)).is_err());
  assert_eq!(reader.offset()?, 0);
  assert_eq!(reader.read_u8()?, b'a');
  Ok(())
}

#[test]
fn extreme_lengths_are_rejected_by_each_bounded_operation() -> io::Result<()> {
  for length in [usize::MAX, usize::MAX - 1, usize::MAX - 2] {
    for data in [&b""[..], &b"abc"[..]] {
      for offset in 0..=data.len() {
        let mut reader = BytesReader::new(data);
        reader.seek(SeekFrom::Start(offset as u64))?;
        assert!(reader.read_bytes_as_vec(length).is_err());
        assert!(reader.read_bytes_no_move(length).is_err());
        assert!(reader.skip_ptr(length).is_err());
        assert_eq!(reader.offset()?, offset as u64);
        assert_eq!(
          reader.read_bytes_as_vec(data.len() - offset)?,
          &data[offset..]
        );
      }
    }
  }
  Ok(())
}

#[test]
fn signed_seek_extremes_and_valid_boundaries_work_in_all_profiles() -> io::Result<()> {
  for data in [&b""[..], &b"abc"[..]] {
    for offset in 0..=data.len() {
      let mut reader = BytesReader::new(data);
      reader.seek(SeekFrom::Start(offset as u64))?;
      for delta in [i64::MIN, i64::MIN + 1, i64::MAX - 1, i64::MAX] {
        assert!(reader.seek(SeekFrom::Current(delta)).is_err());
        assert!(reader.seek(SeekFrom::End(delta)).is_err());
        assert_eq!(reader.offset()?, offset as u64);
      }
      assert_eq!(reader.seek(SeekFrom::End(0))?, data.len() as u64);
      assert_eq!(reader.seek(SeekFrom::End(-(data.len() as i64)))?, 0);
      assert_eq!(
        reader.seek(SeekFrom::Current(data.len() as i64))?,
        data.len() as u64
      );
      assert_eq!(reader.seek(SeekFrom::Current(-(data.len() as i64)))?, 0);
      assert_eq!(reader.read_bytes_as_vec(data.len())?, data);
    }
  }
  Ok(())
}
