use bin_rs::{io, Endian};

#[test]
fn signed_byte_reads_value_not_address() {
  let bytes = [0x80; 256];
  for offset in 0..bytes.len() {
    assert_eq!(io::read_i8(&bytes, offset), -128, "offset {offset}");
  }
}
#[test]
fn signed_little_endian_16_preserves_minimum() {
  assert_eq!(io::read_i16_le(&[0, 0x80], 0), i16::MIN);
}
#[test]
fn signed_little_endian_dispatch_preserves_minimum() {
  assert_eq!(io::read_i16(&[0, 0x80], 0, Endian::LittleEndian), i16::MIN);
}
macro_rules! wide_read {
  ($name:ident, $read:ident, $ty:ty, $encoding:ident) => {
    #[test]
    fn $name() {
      let value: $ty = 0x0123456789abcdeffedcba9876543210;
      let mut bytes = vec![0xa5];
      bytes.extend_from_slice(&value.$encoding());
      assert_eq!(io::$read(&bytes, 1), value);
    }
  };
}
wide_read!(u128_be_distinct_halves, read_u128_be, u128, to_be_bytes);
wide_read!(u128_le_distinct_halves, read_u128_le, u128, to_le_bytes);
wide_read!(i128_be_distinct_halves, read_i128_be, i128, to_be_bytes);
wide_read!(i128_le_distinct_halves, read_i128_le, i128, to_le_bytes);
#[test]
fn ascii_honors_offset_and_length() {
  assert_eq!(io::read_ascii_string(b"xABCz", 1, 3), "ABC");
}
#[test]
fn utf16_honors_zero_length() {
  assert_eq!(
    io::read_utf16_string(&[0, b'A'], 0, 0, Endian::BigEndian),
    ""
  );
}
#[test]
fn utf16_valid_nonzero_offset_does_not_overrun() {
  assert_eq!(
    io::read_utf16_string(&[0xff, 0xff, 0, b'A'], 2, 2, Endian::BigEndian),
    "A"
  );
}
