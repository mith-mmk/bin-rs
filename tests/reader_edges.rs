use bin_rs::reader::{BinaryReader, BytesReader, StreamReader};
use bin_rs::Endian;
use std::io::{self, BufReader, Cursor, SeekFrom};

fn readers(data: &[u8]) -> Vec<Box<dyn BinaryReader>> {
  let mut result: Vec<Box<dyn BinaryReader>> = vec![
    Box::new(BytesReader::new(data)),
    Box::new(StreamReader::new(Cursor::new(data.to_vec()))),
  ];
  for capacity in [0, 1, 7, 8192] {
    result.push(Box::new(StreamReader::new(BufReader::with_capacity(
      capacity,
      Cursor::new(data.to_vec()),
    ))));
  }
  result
}

macro_rules! integer_edges {
  ($name:ident, $ty:ty, $read:ident, $be:ident, $le:ident, [$($value:expr),+]) => {
    #[test]
    fn $name() -> io::Result<()> {
      for value in [$($value as $ty),+] {
        for endian in [Endian::BigEndian, Endian::LittleEndian] {
          let bytes = match endian {
            Endian::BigEndian => value.to_be_bytes(),
            Endian::LittleEndian => value.to_le_bytes(),
          };
          let mut data = vec![0xa5];
          data.extend_from_slice(&bytes);
          data.push(0x5a);
          for mut reader in readers(&data) {
            reader.skip_ptr(1)?;
            reader.set_endian(endian);
            assert_eq!(reader.$read()?, value);
            assert_eq!(reader.offset()?, (1 + bytes.len()) as u64);
            assert_eq!(reader.read_u8()?, 0x5a);
            reader.seek(SeekFrom::Start(1))?;
            // Explicit byte order must not depend on the selected default.
            match endian {
              Endian::BigEndian => {
                reader.set_endian(Endian::LittleEndian);
                assert_eq!(reader.$be()?, value);
              }
              Endian::LittleEndian => {
                reader.set_endian(Endian::BigEndian);
                assert_eq!(reader.$le()?, value);
              }
            }
            assert_eq!(reader.read_u8()?, 0x5a);
          }
          // Every proper prefix is truncated, not just the empty input.
          for len in 0..bytes.len() {
            for mut reader in readers(&bytes[..len]) {
              reader.set_endian(endian);
              assert!(reader.$read().is_err());
              assert!(reader.offset()? <= len as u64);
              reader.seek(SeekFrom::Start(0))?;
              assert_eq!(reader.read_bytes_as_vec(len)?, &bytes[..len]);
            }
          }
        }
      }
      Ok(())
    }
  };
}

integer_edges!(
  u16_boundaries,
  u16,
  read_u16,
  read_u16_be,
  read_u16_le,
  [0, 1, 0x80ff, u16::MAX]
);
integer_edges!(
  u32_boundaries,
  u32,
  read_u32,
  read_u32_be,
  read_u32_le,
  [0, 1, 0x80ff1234, u32::MAX]
);
integer_edges!(
  u64_boundaries,
  u64,
  read_u64,
  read_u64_be,
  read_u64_le,
  [0, 1, 0x80ff123456789abc, u64::MAX]
);
integer_edges!(
  u128_boundaries,
  u128,
  read_u128,
  read_u128_be,
  read_u128_le,
  [0, 1, 0x80ff123456789abc0123456789abcdef, u128::MAX]
);
integer_edges!(
  i16_boundaries,
  i16,
  read_i16,
  read_i16_be,
  read_i16_le,
  [i16::MIN, -257, -1, 0, 1, i16::MAX]
);
integer_edges!(
  i32_boundaries,
  i32,
  read_i32,
  read_i32_be,
  read_i32_le,
  [i32::MIN, -257, -1, 0, 1, i32::MAX]
);
integer_edges!(
  i64_boundaries,
  i64,
  read_i64,
  read_i64_be,
  read_i64_le,
  [i64::MIN, -257, -1, 0, 1, i64::MAX]
);
integer_edges!(
  i128_boundaries,
  i128,
  read_i128,
  read_i128_be,
  read_i128_le,
  [i128::MIN, -257, -1, 0, 1, i128::MAX]
);

macro_rules! float_edges {
  ($name:ident, $ty:ty, $bits:ty, $read:ident, $be:ident, $le:ident, [$($pattern:expr),+]) => {
    #[test]
    fn $name() -> io::Result<()> {
      for bits in [$($pattern as $bits),+] {
        let value = <$ty>::from_bits(bits);
        for endian in [Endian::BigEndian, Endian::LittleEndian] {
          let bytes = match endian {
            Endian::BigEndian => value.to_be_bytes(),
            Endian::LittleEndian => value.to_le_bytes(),
          };
          for mut reader in readers(&bytes) {
            reader.set_endian(endian);
            assert_eq!(reader.$read()?.to_bits(), bits);
            assert_eq!(reader.offset()?, bytes.len() as u64);
            reader.seek(SeekFrom::Start(0))?;
            let actual = match endian {
              Endian::BigEndian => reader.$be()?,
              Endian::LittleEndian => reader.$le()?,
            };
            assert_eq!(actual.to_bits(), bits);
          }
          for len in 0..bytes.len() {
            for mut reader in readers(&bytes[..len]) {
              assert!(reader.$read().is_err());
            }
          }
        }
      }
      Ok(())
    }
  };
}

// Signed zero, subnormal, largest finite, infinities, and NaN payloads.
float_edges!(
  f32_bits_are_preserved,
  f32,
  u32,
  read_f32,
  read_f32_be,
  read_f32_le,
  [0, 0x80000000, 1, 0x7f7fffff, 0x7f800000, 0xff800000, 0x7fc01234]
);
float_edges!(
  f64_bits_are_preserved,
  f64,
  u64,
  read_f64,
  read_f64_be,
  read_f64_le,
  [
    0,
    0x8000000000000000,
    1,
    0x7fefffffffffffff,
    0x7ff0000000000000,
    0xfff0000000000000,
    0x7ff8000012345678
  ]
);

#[test]
fn byte_boundaries_and_empty_operations() -> io::Result<()> {
  for mut reader in readers(&[0, 127, 128, 255]) {
    assert_eq!(reader.read_u8()?, 0);
    assert_eq!(reader.read_i8()?, 127);
    assert_eq!(reader.read_i8()?, -128);
    assert_eq!(reader.read_i8()?, -1);
    assert!(reader.read_byte().is_err());
    assert!(reader.read_u8().is_err());
    assert!(reader.read_i8().is_err());
    assert_eq!(reader.offset()?, 4);
    reader.read_exact(&mut [])?;
    assert!(reader.read_bytes_as_vec(0)?.is_empty());
    assert!(reader.read_bytes_no_move(0)?.is_empty());
    assert_eq!(reader.skip_ptr(0)?, 0);
    assert_eq!(reader.offset()?, 4);
  }
  Ok(())
}

#[test]
fn bytes_reader_failed_reads_preserve_offset_and_output() -> io::Result<()> {
  let mut reader = BytesReader::new(b"abc");
  assert_eq!(reader.read_u8()?, b'a');
  let mut output = [0x5a; 3];
  assert!(reader.read_exact(&mut output).is_err());
  assert_eq!(output, [0x5a; 3]);
  assert!(reader.read_bytes_as_vec(3).is_err());
  assert!(reader.read_bytes_no_move(3).is_err());
  assert!(reader.skip_ptr(3).is_err());
  assert!(reader.read_u32().is_err());
  assert_eq!(reader.offset()?, 1);
  assert_eq!(reader.read_bytes_as_vec(2)?, b"bc");
  Ok(())
}

#[test]
fn seek_boundaries_and_rejected_seeks_preserve_offset() -> io::Result<()> {
  let mut reader = BytesReader::new(b"abc");
  assert_eq!(reader.seek(SeekFrom::End(0))?, 3);
  assert_eq!(reader.seek(SeekFrom::Current(-3))?, 0);
  assert_eq!(reader.seek(SeekFrom::End(-2))?, 1);
  for position in [
    SeekFrom::Start(4),
    SeekFrom::Start(u64::MAX),
    SeekFrom::End(1),
    SeekFrom::End(-4),
    SeekFrom::Current(-2),
    SeekFrom::Current(3),
  ] {
    assert!(reader.seek(position).is_err());
    assert_eq!(reader.offset()?, 1);
  }
  assert_eq!(reader.read_u8()?, b'b');
  let mut empty = BytesReader::new(&[]);
  for position in [SeekFrom::Start(0), SeekFrom::End(0), SeekFrom::Current(0)] {
    assert_eq!(empty.seek(position)?, 0);
  }
  assert!(empty.seek(SeekFrom::Current(-1)).is_err());
  Ok(())
}

#[test]
fn utf8_rejects_invalid_sequences_and_consumes_requested_field() -> io::Result<()> {
  for bytes in [
    &[0xff][..],
    &[0x80],
    &[0xc0, 0x80],
    &[0xed, 0xa0, 0x80],
    &[0xf4, 0x90, 0x80, 0x80],
  ] {
    let mut data = bytes.to_vec();
    data.push(b'!');
    for mut reader in readers(&data) {
      assert!(reader.read_utf8_string(bytes.len()).is_err());
      assert_eq!(reader.offset()?, bytes.len() as u64);
      assert_eq!(reader.read_u8()?, b'!');
    }
  }
  for text in ["", "A\0B", "日本語", "a😀z"] {
    for mut reader in readers(text.as_bytes()) {
      assert_eq!(reader.read_utf8_string(text.len())?, text);
      assert_eq!(reader.offset()?, text.len() as u64);
    }
  }
  Ok(())
}

#[test]
fn ascii_nul_stops_text_but_consumes_the_complete_field() -> io::Result<()> {
  for mut reader in readers(b"ab\0cd!") {
    assert_eq!(reader.read_ascii_string(5)?, "ab");
    assert_eq!(reader.offset()?, 5);
    assert_eq!(reader.read_u8()?, b'!');
    assert_eq!(reader.read_ascii_string(0)?, "");
  }
  Ok(())
}

#[test]
fn utf16_surrogate_pairs_and_endian_restoration() -> io::Result<()> {
  for text in ["", "A\0B", "日本語", "a😀z"] {
    let units: Vec<_> = text.encode_utf16().collect();
    let be: Vec<_> = units.iter().flat_map(|v| v.to_be_bytes()).collect();
    let le: Vec<_> = units.iter().flat_map(|v| v.to_le_bytes()).collect();
    for (bytes, endian) in [(&be, Endian::LittleEndian), (&le, Endian::BigEndian)] {
      for mut reader in readers(bytes) {
        reader.set_endian(endian);
        let actual = match endian {
          Endian::LittleEndian => reader.read_utf16be_string(bytes.len())?,
          Endian::BigEndian => reader.read_utf16le_string(bytes.len())?,
        };
        assert_eq!(actual, text);
        assert_eq!(reader.endian(), endian);
        assert_eq!(reader.offset()?, bytes.len() as u64);
      }
    }
  }
  Ok(())
}

#[test]
fn utf16_invalid_surrogates_and_truncation_restore_endian() -> io::Result<()> {
  for bytes in [
    &[0xd8, 0x00][..],
    &[0xdc, 0x00],
    &[0xd8, 0x00, 0x00, 0x41],
    &[0],
  ] {
    for mut reader in readers(bytes) {
      reader.set_endian(Endian::LittleEndian);
      assert!(reader.read_utf16be_string(2.max(bytes.len())).is_err());
      assert_eq!(reader.endian(), Endian::LittleEndian);
    }
    let le: Vec<_> = bytes
      .chunks(2)
      .flat_map(|pair| pair.iter().rev().copied())
      .collect();
    for mut reader in readers(&le) {
      reader.set_endian(Endian::BigEndian);
      assert!(reader.read_utf16le_string(2.max(le.len())).is_err());
      assert_eq!(reader.endian(), Endian::BigEndian);
    }
  }
  Ok(())
}

#[test]
fn owned_input_and_clones_have_independent_positions() -> io::Result<()> {
  let mut source = vec![1, 2, 3];
  let mut reader = BytesReader::new(&source);
  source.fill(0);
  assert_eq!(reader.read_u8()?, 1);
  let mut clone = reader.clone();
  assert_eq!(reader.read_u8()?, 2);
  assert_eq!(clone.offset()?, 1);
  assert_eq!(clone.read_u16_be()?, 0x0203);
  assert_eq!(reader.offset()?, 2);
  Ok(())
}

#[test]
fn mixed_seek_peek_and_read_sequence_matches_slice_oracle() -> io::Result<()> {
  let data: Vec<u8> = (0u8..64).map(|n| n.wrapping_mul(37)).collect();
  for mut reader in readers(&data) {
    let mut seed = 0x1234abcd_u32;
    let mut position = 0usize;
    for _ in 0..512 {
      seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
      let count = ((seed >> 8) % 17) as usize;
      match seed % 4 {
        0 => {
          position = ((seed >> 16) % 65) as usize;
          reader.seek(SeekFrom::Start(position as u64))?;
        }
        1 => {
          let result = reader.read_bytes_no_move(count);
          if count <= data.len() - position {
            assert_eq!(result?, &data[position..position + count]);
          } else {
            assert!(result.is_err());
          }
        }
        2 => {
          let count = count.min(data.len() - position);
          assert_eq!(
            reader.read_bytes_as_vec(count)?,
            &data[position..position + count]
          );
          position += count;
        }
        _ => {
          let count = count.min(data.len() - position);
          assert_eq!(reader.skip_ptr(count)?, count);
          position += count;
        }
      }
      assert_eq!(reader.offset()?, position as u64);
    }
  }
  Ok(())
}

#[cfg(feature = "serde")]
#[test]
fn bytes_conversion_respects_a_sliced_bytes_view() -> io::Result<()> {
  let bytes = bytes::Bytes::from_static(b"prefixDATAending").slice(6..10);
  let mut reader = BytesReader::from(bytes);
  assert_eq!(reader.read_bytes_as_vec(4)?, b"DATA");
  assert!(reader.read_u8().is_err());
  Ok(())
}
