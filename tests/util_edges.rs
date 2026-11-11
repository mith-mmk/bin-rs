#![cfg(feature = "util")]

use bin_rs::{io, io::*, Endian};

// Slice utilities require callers to provide a sufficiently large buffer.
// Invalid slices are deliberately not assigned a new error-handling contract here.
macro_rules! integer_writes {
  ($name:ident, $ty:ty, $write:ident, $be:ident, $le:ident, $values:expr) => {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), test)]
    fn $name() {
      for value in $values {
        for endian in [Endian::BigEndian, Endian::LittleEndian] {
          let expected = if endian == Endian::BigEndian {
            value.to_be_bytes()
          } else {
            value.to_le_bytes()
          };
          let mut bytes = vec![0xa5, 0x5a, 0xcc];
          $write(value, &mut bytes, endian);
          assert_eq!(&bytes[..3], &[0xa5, 0x5a, 0xcc]);
          assert_eq!(&bytes[3..], expected.as_slice());
          let mut direct = vec![0xa5, 0x5a, 0xcc];
          if endian == Endian::BigEndian {
            $be(value, &mut direct);
          } else {
            $le(value, &mut direct);
          }
          assert_eq!(direct, bytes);
        }
      }
    }
  };
}
integer_writes!(
  u16_writes,
  u16,
  write_u16,
  write_u16_be,
  write_u16_le,
  [0u16, 1, 0x1234, u16::MAX]
);
integer_writes!(
  u32_writes,
  u32,
  write_u32,
  write_u32_be,
  write_u32_le,
  [0u32, 1, 0x12345678, u32::MAX]
);
integer_writes!(
  u64_writes,
  u64,
  write_u64,
  write_u64_be,
  write_u64_le,
  [0u64, 1, 0x123456789abcdef0, u64::MAX]
);
integer_writes!(
  u128_writes,
  u128,
  write_u128,
  write_u128_be,
  write_u128_le,
  [0u128, 1, 0x0123456789abcdeffedcba9876543210, u128::MAX]
);
integer_writes!(
  i16_writes,
  i16,
  write_i16,
  write_i16_be,
  write_i16_le,
  [i16::MIN, -1, 0, 1, i16::MAX]
);
integer_writes!(
  i32_writes,
  i32,
  write_i32,
  write_i32_be,
  write_i32_le,
  [i32::MIN, -1, 0, 1, i32::MAX]
);
integer_writes!(
  i64_writes,
  i64,
  write_i64,
  write_i64_be,
  write_i64_le,
  [i64::MIN, -1, 0, 1, i64::MAX]
);
integer_writes!(
  i128_writes,
  i128,
  write_i128,
  write_i128_be,
  write_i128_le,
  [i128::MIN, -1, 0, 1, i128::MAX]
);

macro_rules! integer_reads {
  ($name:ident, $read:ident, $be:ident, $le:ident, $values:expr) => {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), test)]
    fn $name() {
      for value in $values {
        for endian in [Endian::BigEndian, Endian::LittleEndian] {
          let encoded = if endian == Endian::BigEndian {
            value.to_be_bytes()
          } else {
            value.to_le_bytes()
          };
          for offset in [0, 1, 3] {
            let mut bytes = vec![0xa5; offset];
            bytes.extend_from_slice(&encoded);
            // Exercise both an exact end boundary and trailing neighbor bytes.
            assert_eq!($read(&bytes, offset, endian), value);
            bytes.extend_from_slice(&[0x5a, 0xcc]);
            assert_eq!($read(&bytes, offset, endian), value);
            let actual = if endian == Endian::BigEndian {
              $be(&bytes, offset)
            } else {
              $le(&bytes, offset)
            };
            assert_eq!(actual, value);
          }
        }
      }
    }
  };
}
integer_reads!(
  u16_reads,
  read_u16,
  read_u16_be,
  read_u16_le,
  [0u16, 1, 0x1234, u16::MAX]
);
integer_reads!(
  u32_reads,
  read_u32,
  read_u32_be,
  read_u32_le,
  [0u32, 1, 0x12345678, u32::MAX]
);
integer_reads!(
  u64_reads,
  read_u64,
  read_u64_be,
  read_u64_le,
  [0u64, 1, 0x123456789abcdef0, u64::MAX]
);
integer_reads!(
  i32_reads,
  read_i32,
  read_i32_be,
  read_i32_le,
  [i32::MIN, -12345, -1, 0, 1, i32::MAX]
);
integer_reads!(
  i64_reads,
  read_i64,
  read_i64_be,
  read_i64_le,
  [i64::MIN, -12345, -1, 0, 1, i64::MAX]
);

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn i16_big_endian_reads() {
  for value in [i16::MIN, -12345, -1, 0, 1, i16::MAX] {
    let mut bytes = vec![0xa5];
    bytes.extend_from_slice(&value.to_be_bytes());
    assert_eq!(read_i16_be(&bytes, 1), value);
    assert_eq!(read_i16(&bytes, 1, Endian::BigEndian), value);
  }
}

macro_rules! float_bits {
  ($name:ident, $ty:ty, $read:ident, $write:ident, $be:ident, $le:ident, $bits:expr) => {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), test)]
    fn $name() {
      for bits in $bits {
        let value = <$ty>::from_bits(bits);
        for endian in [Endian::BigEndian, Endian::LittleEndian] {
          let expected = if endian == Endian::BigEndian {
            bits.to_be_bytes()
          } else {
            bits.to_le_bytes()
          };
          let mut bytes = vec![0xa5];
          $write(value, &mut bytes, endian);
          assert_eq!(bytes[0], 0xa5);
          assert_eq!(&bytes[1..], expected.as_slice());
          assert_eq!($read(&bytes, 1, endian).to_bits(), bits);
          bytes.push(0xcc);
          assert_eq!($read(&bytes, 1, endian).to_bits(), bits);
          let mut direct = Vec::new();
          if endian == Endian::BigEndian {
            $be(value, &mut direct);
          } else {
            $le(value, &mut direct);
          }
          assert_eq!(direct, expected);
        }
      }
    }
  };
}
// Positive/negative zero, subnormals, normal extrema, infinities and NaN payloads.
float_bits!(
  f32_bits,
  f32,
  read_f32,
  write_f32,
  write_f32_be,
  write_f32_le,
  [
    0u32, 0x80000000, 1, 0x80000001, 0x007fffff, 0x00800000, 0x7f7fffff, 0xff7fffff, 0x7f800000,
    0xff800000, 0x7fc01234, 0xffc05678
  ]
);
float_bits!(
  f64_bits,
  f64,
  read_f64,
  write_f64,
  write_f64_be,
  write_f64_le,
  [
    0u64,
    0x8000000000000000,
    1,
    0x8000000000000001,
    0x000fffffffffffff,
    0x0010000000000000,
    0x7fefffffffffffff,
    0xffefffffffffffff,
    0x7ff0000000000000,
    0xfff0000000000000,
    0x7ff8000000001234,
    0xfff8000000005678
  ]
);

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn byte_operations_append_and_respect_ranges() {
  let mut bytes = vec![0xa5];
  for value in [0, 1, 127, 128, 255] {
    write_byte(value, &mut bytes);
  }
  for value in [i8::MIN, -1, 0, 1, i8::MAX] {
    write_i8(value, &mut bytes);
  }
  assert_eq!(bytes, [0xa5, 0, 1, 127, 128, 255, 128, 255, 0, 1, 127]);
  assert_eq!(read_byte(&bytes, bytes.len() - 1), 127);
  assert_eq!(read_bytes_as_vec(&bytes, 2, 3), [1, 127, 128]);
  assert!(read_bytes_as_vec(&bytes, bytes.len(), 0).is_empty());
  assert!(read_bytes_as_vec(&[], 0, 0).is_empty());
  let before = bytes.clone();
  write_bytes(&[], &mut bytes);
  assert_eq!(bytes, before);
  write_bytes(&[0, 255], &mut bytes);
  assert_eq!(&bytes[before.len()..], &[0, 255]);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn utf8_strings_honor_offset_limit_and_nul() {
  assert_eq!(read_string(b"xhello!", 1, 5), "hello");
  assert_eq!(read_string(b"xhi\0tail", 1, 7), "hi");
  assert_eq!(read_string(b"x\0tail", 1, 5), "");
  assert_eq!(read_string(b"x", 1, 0), "");
  assert_eq!(read_string(&[], 0, 0), "");
  let text = "A\u{65e5}\u{1f600}";
  let mut bytes = vec![0xa5];
  write_string(text.to_owned(), &mut bytes);
  assert_eq!(bytes[0], 0xa5);
  assert_eq!(&bytes[1..], text.as_bytes());
  assert_eq!(read_string(&bytes, 1, text.len()), text);
  write_string(String::new(), &mut bytes);
  assert_eq!(bytes.len(), 1 + text.len());
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn ascii_writer_appends_one_terminator() {
  let mut bytes = vec![0xa5];
  write_ascii_string("ABC".to_owned(), &mut bytes);
  write_ascii_string(String::new(), &mut bytes);
  assert_eq!(bytes, [0xa5, b'A', b'B', b'C', 0, 0]);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn utf16_decodes_surrogates_lossily_and_stops_at_nul() {
  for endian in [Endian::BigEndian, Endian::LittleEndian] {
    for (units, expected) in [
      (vec![0x0041, 0x65e5, 0xd83d, 0xde00], "A\u{65e5}\u{1f600}"),
      (vec![0xd800, 0x0041, 0xdc00], "\u{fffd}A\u{fffd}"),
      (vec![0x0041, 0, 0x0042], "A"),
      (vec![], ""),
    ] {
      let mut bytes = Vec::new();
      for unit in units {
        write_u16(unit, &mut bytes, endian);
      }
      assert_eq!(read_utf16_string(&bytes, 0, bytes.len(), endian), expected);
    }
  }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn signed_byte_reads_value_not_address() {
  let bytes = [0x80; 256];
  for offset in 0..bytes.len() {
    assert_eq!(io::read_i8(&bytes, offset), -128, "offset {offset}");
  }
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn signed_little_endian_16_preserves_minimum() {
  assert_eq!(io::read_i16_le(&[0, 0x80], 0), i16::MIN);
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn signed_little_endian_dispatch_preserves_minimum() {
  assert_eq!(io::read_i16(&[0, 0x80], 0, Endian::LittleEndian), i16::MIN);
}
macro_rules! wide_read {
  ($name:ident, $read:ident, $ty:ty, $encoding:ident) => {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), test)]
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn ascii_honors_offset_and_length() {
  assert_eq!(io::read_ascii_string(b"xABCz", 1, 3), "ABC");
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn utf16_honors_zero_length() {
  assert_eq!(
    io::read_utf16_string(&[0, b'A'], 0, 0, Endian::BigEndian),
    ""
  );
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn utf16_valid_nonzero_offset_does_not_overrun() {
  assert_eq!(
    io::read_utf16_string(&[0xff, 0xff, 0, b'A'], 2, 2, Endian::BigEndian),
    "A"
  );
}

integer_reads!(
  i16_reads,
  read_i16,
  read_i16_be,
  read_i16_le,
  [i16::MIN, -12345, -1, 0, 1, i16::MAX]
);
integer_reads!(
  u128_reads,
  read_u128,
  read_u128_be,
  read_u128_le,
  [
    0u128,
    1,
    1 << 64,
    0x0123456789abcdeffedcba9876543210,
    u128::MAX
  ]
);
integer_reads!(
  i128_reads,
  read_i128,
  read_i128_be,
  read_i128_le,
  [
    i128::MIN,
    -0x0123456789abcdeffedcba9876543210,
    -1,
    0,
    1,
    1 << 64,
    i128::MAX
  ]
);

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn signed_byte_reads_all_values_at_boundaries() {
  for byte in 0..=u8::MAX {
    assert_eq!(read_i8(&[byte], 0), byte as i8);
    assert_eq!(read_i8(&[0xa5, byte], 1), byte as i8);
    assert_eq!(read_i8(&[0xa5, byte, 0xcc], 1), byte as i8);
  }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn ascii_preserves_nul_and_high_bytes_within_range() {
  assert_eq!(read_ascii_string(&[], 0, 0), "");
  assert_eq!(read_ascii_string(b"x", 1, 0), "");
  assert_eq!(read_ascii_string(b"xABC", 1, 3), "ABC");
  assert_eq!(read_ascii_string(b"x\0A\x80\xffz", 1, 4), "\0A\u{80}\u{ff}");
  for byte in 0..=u8::MAX {
    assert_eq!(
      read_ascii_string(&[0xa5, byte, 0xcc], 1, 1),
      char::from(byte).to_string()
    );
  }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn utf16_limits_are_bytes_and_ignore_incomplete_units() {
  for endian in [Endian::BigEndian, Endian::LittleEndian] {
    for offset in [0, 1, 2, 3] {
      let mut bytes = vec![0xa5; offset];
      for unit in [0x0041, 0x65e5, 0xd83d, 0xde00, 0, 0x0042] {
        write_u16(unit, &mut bytes, endian);
      }
      for (length, expected) in [
        (0, ""),
        (1, ""),
        (2, "A"),
        (3, "A"),
        (4, "A\u{65e5}"),
        (5, "A\u{65e5}"),
        (6, "A\u{65e5}\u{fffd}"),
        (7, "A\u{65e5}\u{fffd}"),
        (8, "A\u{65e5}\u{1f600}"),
        (9, "A\u{65e5}\u{1f600}"),
        (10, "A\u{65e5}\u{1f600}"),
        (11, "A\u{65e5}\u{1f600}"),
        (12, "A\u{65e5}\u{1f600}"),
      ] {
        assert_eq!(read_utf16_string(&bytes, offset, length, endian), expected);
        // The same range ending at EOF must not inspect trailing neighbors.
        assert_eq!(
          read_utf16_string(&bytes[..offset + length], offset, length, endian),
          expected
        );
      }
      assert_eq!(read_utf16_string(&bytes, bytes.len(), 0, endian), "");
      assert_eq!(read_utf16_string(&bytes, offset + 8, 4, endian), "");
    }
    assert_eq!(read_utf16_string(&[], 0, 0, endian), "");
    assert_eq!(read_utf16_string(&[0xff], 0, 1, endian), "");
  }
}
