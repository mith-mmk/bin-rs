#![cfg(feature = "util")]
//! Compatibility observations, not approval of every legacy API design.

use bin_rs::io;

#[test]
fn ascii_writer_is_utf8_plus_nul_not_the_inverse_byte_mapping() {
  for text in ["", "ASCII", "\u{e9}", "\u{65e5}\u{1f600}", "A\0B", "end\0"] {
    let mut bytes = vec![0xa5, 0x5a];
    io::write_ascii_string(text.to_owned(), &mut bytes);
    let mut expected = vec![0xa5, 0x5a];
    expected.extend_from_slice(text.as_bytes());
    expected.push(0);
    assert_eq!(bytes, expected);
    let mapped: String = text.as_bytes().iter().map(|&b| char::from(b)).collect();
    assert_eq!(io::read_ascii_string(&bytes, 2, text.len()), mapped);
    assert_eq!(
      io::read_ascii_string(&bytes, 2, text.len() + 1),
      mapped + "\0"
    );
    if !text.is_ascii() {
      assert_ne!(io::read_ascii_string(&bytes, 2, text.len()), text);
    }
    let mut unterminated = vec![0xa5, 0x5a];
    io::write_string(text.to_owned(), &mut unterminated);
    assert_eq!(&unterminated[2..], text.as_bytes());
  }
}

#[test]
fn u16_vector_widens_bytes_without_packing_and_drops_odd_tail() {
  let data: Vec<u8> = (0..=255).collect();
  for offset in [0, 1, 3] {
    let mut bytes = vec![0xa5; offset];
    bytes.extend_from_slice(&data);
    bytes.push(0xcc);
    for length in 0..=data.len() {
      let expected: Vec<u16> = data[..length / 2 * 2]
        .iter()
        .map(|&b| u16::from(b))
        .collect();
      assert_eq!(io::read_bytes_as_u16_vec(&bytes, offset, length), expected);
      assert_eq!(
        io::read_bytes_as_u16_vec(&bytes[..offset + length], offset, length),
        expected
      );
    }
    assert!(io::read_bytes_as_u16_vec(&bytes, bytes.len(), 0).is_empty());
  }
  assert!(io::read_bytes_as_u16_vec(&[], 0, 0).is_empty());
  assert_eq!(
    io::read_bytes_as_u16_vec(&[0x12, 0x34, 0xff], 0, 3),
    [0x12, 0x34]
  );
}

#[test]
fn utf8_limits_count_bytes_and_invalid_prefix_returns_empty() {
  let text = "A\u{e9}\u{65e5}\u{1f600}";
  for offset in [0, 1, 3] {
    let mut bytes = vec![0xff; offset];
    bytes.extend_from_slice(text.as_bytes());
    bytes.push(0xff);
    for length in 0..=text.len() {
      let expected = std::str::from_utf8(&text.as_bytes()[..length]).unwrap_or("");
      assert_eq!(io::read_string(&bytes, offset, length), expected);
      assert_eq!(
        io::read_string(&bytes[..offset + length], offset, length),
        expected
      );
    }
    assert_eq!(io::read_string(&bytes, bytes.len(), 0), "");
  }
  assert_eq!(io::read_string(&[], 0, 0), "");
}

#[test]
fn utf8_nul_ends_decoding_but_does_not_repair_invalid_prefix() {
  for (data, expected) in [
    (&b"A\0\xff"[..], "A"),
    (&b"\0\xff"[..], ""),
    (&b"A\xff\0B"[..], ""),
    (&b"\xc3\0\xa9"[..], ""),
    (&b"\xc3\xa9\0\xff"[..], "\u{e9}"),
    (&b"\xc0\xaf"[..], ""),
    (&b"\xed\xa0\x80"[..], ""),
    (&b"\xf4\x90\x80\x80"[..], ""),
  ] {
    let mut bytes = vec![0xff];
    bytes.extend_from_slice(data);
    assert_eq!(io::read_string(&bytes, 1, data.len()), expected);
    bytes.push(b'!');
    assert_eq!(io::read_string(&bytes, 1, data.len()), expected);
  }
  // The same String value represents valid empty text and invalid UTF-8.
  assert_eq!(io::read_string(&[0xff], 0, 1), io::read_string(&[], 0, 0));
}
