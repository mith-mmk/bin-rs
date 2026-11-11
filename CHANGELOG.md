# Changelog

## 0.1.0

The manifest and committed lockfile identify version 0.1.0.

### Breaking change: remove unfinished codec support

- Removed the `codec` feature, optional `encoding_rs` dependency and its implicit
  feature, `reader::CodeType`, and all `read_local_string` methods.
- This removal is a breaking 0.1.0 change and must not ship as a 0.0.x patch.
  All non-codec legacy APIs remain, including deprecated `BinaryReader::read_bytes`
  and `BytesReader::from_vec`.
- Decode legacy encodings in format-layer code after reading raw bytes.
  bin-rs provides no replacement codec. See the
  [migration notes](docs/synchronous-contract.md#breaking-migration-for-the-010-target).
- Existing ASCII/UTF-8/UTF-16 helpers and async behavior are unchanged.
  Async's stable contract still targets 0.2.0; the larger-peek regression
  remains unresolved in the separate regression suite.

## 0.0.11

### Fixes

- `StreamReader::read_bytes_no_move` now supports lookahead spanning small or
  partially consumed buffers. Its fast path does not seek. The fallback restores
  the logical position after a successful read or a read error, and allocation
  capacity errors are returned without consuming input. Empty peeks perform no I/O.
- `BytesReader` rejects overflowing lengths and extreme signed seeks without
  changing position, including optimized builds where addition previously wrapped.
- The `util` helpers correctly read signed bytes, little-endian signed 16-bit
  integers, and both halves of 128-bit integers. ASCII and UTF-16 reads honor byte
  offset and byte length.
- Async peeks now accept an exact buffered window, and empty peeks perform no I/O.
  Existing `Copy` support and non-seekable reader bounds are unchanged.
- Expanded boundary, error and recovery tests. No unsafe code was introduced.

### Compatibility notes

- Truncated synchronous stream lookahead now returns `UnexpectedEof` rather than
  the previous custom `Other` / `Data shotage` error. If restoring the stream
  position fails, that seek error takes precedence; position is then unknown.
- Util string results now respect the supplied range. Callers must pass in-bounds
  ranges; formerly ignored invalid ASCII ranges can now panic. ASCII still maps
  each byte directly, including NUL and high bytes. UTF-16 lengths are bytes,
  decoding is lossy, NUL terminates the string, and an odd final byte is ignored.
- No public signatures, trait bounds, or struct layouts were changed.

### Validation and existing limitations

- The default suite has 40 passing tests including doctests. The
  `stream,async,serde,util` suite has 100 passing tests including doctests, in
  debug and release on Rust 1.71 and 1.91. Rust 1.71 uses a compatible dependency
  lock, including Tokio 1.53.2; the manifest's dependency ranges are unchanged.
- WML2 0.0.43 passed its default, minimal and extended suites with both versions.
  All 634 comparisons across 317 files and memory/default-file readers matched
  (518 successful decodes, 116 controlled errors). Seven real image files also
  passed at buffer capacities 1, 8 and 256 with the fix; the old version failed
  at capacities 1 and 8.
- A matched Windows release benchmark used three warmups and 20 alternating
  pairs. Aggregate paired median time changed by +0.52% (quartiles -0.51% to
  +1.57%). Some short individual cases were slower; this is not a performance
  guarantee or a claim of equivalence for every input.
- Async lookahead larger than the current buffer window remains unsupported.
  Supporting it without changing existing reader requirements needs a separate
  API decision. Its explicit regression case remains in the repository's
  non-published `regressions` crate.
- The existing `codec` feature still has compilation errors. This release is
  not an all-features-clean release. The legacy complete Wasm unit harness also
  calls a native-only file method; separate integration tests were cross-compiled,
  not executed as Wasm tests.
- The manifest retains Rust 1.71 as its declared minimum. Newer optional
  dependency versions can require a newer compiler: Tokio 1.54.1 requires Rust
  1.85. Older toolchains need a compatible lockfile rather than unrestricted
  latest dependencies. See the test documentation for the validated resolution.
