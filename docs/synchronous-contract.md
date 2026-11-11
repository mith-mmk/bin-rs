# Synchronous contract baseline for 0.1.x

This document records the behavior of 0.0.11 before preparing 0.1.x. It is a
reviewable compatibility baseline, **not** an announcement that 0.1.0 is ready
or a promise to preserve every legacy behavior forever. Changes to this
baseline need explicit migration notes and regression tests. This PR makes no
runtime, version, feature, dependency, or downstream requirement changes.

## Scope and release gates

The 0.1.x stabilization work prioritizes `BinaryReader`, `BytesReader`,
`StreamReader`, and the optional `util` helpers. Async's stable contract targets
0.2.0; the existing `async` feature and tests remain, but cancellation, lookahead
larger than the buffer, and ownership design are not settled here. The current
opt-in failing larger-window regression remains visible in `regressions/`.

The `codec` feature is not part of the validated baseline: it currently does
not compile. Do not call this an all-features-clean release. Before 0.1.0,
resolve its retain/repair versus deprecate/remove policy explicitly and verify
the selected feature matrix. Also resolve the native-only file test in the
full Wasm harness and validate the declared MSRV with reproducible dependency
resolution. See [test instructions](../tests/README.md).

## Successful reads and byte order

- Constructors use native system byte order. Explicit `_be` / `_le` numeric
  methods ignore the selected order without changing it. Unsuffixed numeric
  methods use the selected order; float reads preserve IEEE bit patterns.
- Fixed-width numeric reads consume their width on success. `read_exact`,
  `read_bytes_as_vec`, and `skip_ptr` consume the requested byte count on
  success. These are exact operations, not short-read APIs.
- Empty byte reads and peeks return empty results without advancing the
  logical position. Empty stream peeks perform no underlying I/O.
- `BytesReader` owns its bytes. Constructing from a slice copies them;
  conversion from a Vec takes ownership. Clones have independent positions
  and byte storage. The `serde` feature adds conversion from `bytes::Bytes`
  (copying its visible slice), not serialization of the reader itself.

## Failure, position, and output

There is **no trait-wide transactional-read guarantee**. These rules describe
the bundled implementations, not arbitrary downstream implementations.

- A BytesReader bounds failure in a primitive numeric/byte read, `read_exact`,
  `read_bytes_as_vec`, `skip_ptr`, or peek returns `ErrorKind::Other`, leaves
  its position unchanged, and (for `read_exact`) leaves the destination
  unchanged. Overflow is rejected before mutation.
- StreamReader exact reads delegate to `std::io::Read::read_exact`; on failure
  the stream may have consumed a prefix and the destination contents are
  unspecified. Truncated input normally returns `UnexpectedEof`. Do not retry
  assuming the original position or destination was preserved.
- Composite UTF-8/UTF-16 reads can consume a prefix even on BytesReader. A
  decoding error occurs after the complete encoded field has been consumed.
  They currently report invalid encoding as `Other`, not `InvalidData`.
- Allocation exhaustion is not a generally fallible API contract. Only the
  synchronous stream peek fallback uses fallible reservation. Callers must
  bound lengths derived from untrusted input before requesting allocations.
- Error message wording is diagnostic and is not a stable matching API.

## Lookahead and seeking

`read_bytes_no_move(n)` returns exactly n bytes without changing the logical
position on success. BytesReader bounds failures preserve position.
StreamReader uses buffered bytes when possible, otherwise reads and restores
its saved position, including after a read failure. If restoration fails,
that error takes precedence and position is unknown. Buffer/cache state and
external side effects are not rolled back. See `tests/stream_peek.rs` for the
injected I/O and restoration-failure cases.

BytesReader permits seeks only within 0..=length and rejects invalid or
unrepresentable positions without mutation. StreamReader delegates seek to
its underlying `Seek`; for example Cursor permits positions beyond EOF.
There is no common clamp-to-file-length policy or rollback promise for a
failed underlying seek.

## Strings: preserve distinctions before choosing new APIs

All sizes below are byte counts, not character counts.

- `BinaryReader::read_ascii_string(size)` reads the full field, then ends the
  returned text at its first NUL. Each preceding byte maps directly to the
  same Unicode code point (including 0x80..0xFF). Despite the name it is not
  strict ASCII or Windows-1252. This mapping resembles ISO-8859-1, but is not
  a general-purpose encoding detector or an ISO date/time parser.
- `read_utf8_string(size)` is strict UTF-8 and preserves embedded NULs.
- `read_utf16_string(size)` is strict UTF-16 in the selected byte order and
  preserves embedded NULs. It reads `2 * floor(size / 2)` bytes: an odd trailing
  byte is left unread. The explicit BE/LE wrappers restore the previous order
  on both success and returned errors (not on arbitrary panics).
- `io::read_ascii_string(bytes, offset, length)` maps **all** selected bytes to
  code points, including NULs. It does not share the reader's NUL termination.
- The util UTF-16 helpers are lossy, stop at a NUL code unit, and ignore an odd
  trailing byte. This differs from the strict reader UTF-16 methods.
- Util callers must supply valid ranges; invalid slices can panic. No fallible
  checked-slice utility API is introduced here.

Changing these semantics under the same name can silently corrupt existing
parsers. Prefer explicitly named additions plus migration tests where a new
strict ASCII, Latin-1, or checked-range API is desired; decide separately
whether an existing spelling should be deprecated.

## Codec and downstream audit (2026-11-11)

The existing CodeType enum lists multiple encodings, but is not implemented
as a working decoder: BytesReader's method returns an unfinished error;
StreamReader ignores CodeType and attempts NUL-terminated UTF-8; the async
method also has missing-await issues. Missing imports and a missing return
expression prevent compilation. Simply fixing imports would not provide the
advertised decoding behavior. `encoding_rs` is only an optional codec
dependency; none of these working string helpers needs it.

Source and Cargo manifests were checked at these consumer revisions:

- [FontReader e8b925d](https://github.com/mith-mmk/FontReader/tree/e8b925db12da24e060f14c02493f9dd56966efd6): bin-rs 0.0.11, default features.
- [wml2-on-rust 13afa2a](https://github.com/mith-mmk/wml2-on-rust/tree/13afa2aeb4669f7c0f6e580cdc42514c24f4fc11): bin-rs 0.0.11 plus util.
- [wasm-paint 68b0e2d](https://github.com/mith-mmk/wasm-paint/tree/68b0e2dc6bda34e7b92ab49bb559e5c87184058e): paintcore uses bin-rs 0.0.11, default features.

No codec/async feature request or CodeType/read_local_string/AsyncBytesReader
use was found in these checked manifests and Rust sources. WML2 does use both
reader and util ASCII helpers in JPEG/GIF/PNG paths, so their different NUL
semantics matter. This is a bounded audit, not proof that no external consumer
uses the optional APIs. Removing codec would remove public symbols/features;
repairing it requires an explicit encoding/error policy. Neither is done here.

## Regression map

`tests/sync_contract.rs` adds full-byte-range string mapping, util/reader NUL
contrasts, odd UTF-16 lengths, composite-read failure positions, legacy error
kinds, and backend-specific seek behavior. Existing `reader_edges`,
`bytes_reader_bounds`, and `stream_peek` cover numeric extrema, endian/float
bits, overflow, seek rejection, and injected peek failures. Together these
make future policy changes visible rather than silently normalizing behavior.
