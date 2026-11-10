# Test suites

Run the default suite and the supported optional-feature suite from the repository root:

```sh
cargo test
cargo test --features stream,async,serde,util
cargo test --release --features stream,async,serde,util
```

- `reader_edges.rs` compares BytesReader and buffered/unbuffered StreamReader
  results against standard integer encodings, floating-point bits, and slice
  oracles. It covers every truncated numeric prefix, empty input, UTF-8/UTF-16,
  seek boundaries, error recovery, independent clones, and a deterministic
  512-step mixed seek/peek/read sequence. The serde-feature test checks conversion
  from a sliced `bytes::Bytes` value.
- `stream_peek.rs` checks small/zero-capacity buffers, partially consumed buffers,
  repeated peeks, EOF, short/interrupted reads, failed reads and seeks, impossible
  allocations, and no-I/O empty peeks.
- `bytes_reader_bounds.rs` covers overflowing lengths and extreme signed seeks,
  ensuring rejection without moving the position in both debug and release.
- `async_reader_edges.rs` is enabled by `async`. It covers numeric boundaries,
  floating-point bit patterns, short buffers, offsets, EOF, string errors, and
  exact-window and no-I/O empty peeks, error propagation, unchanged peek
  offsets, and compatibility with non-seekable `Copy` readers.
- `util_edges.rs` is enabled by `util`. It covers endian conversion, numeric
  writers, valid ranges, floating-point bits, strings, and surrounding bytes.
  It also includes the 10 resolved util regressions and verifies signed minima,
  distinct 128-bit halves, byte-based string limits, NULs, and lossy UTF-16.
  Slice utility functions do not return `Result`; callers must supply valid
  ranges. Tests do not invent an error contract for invalid slices.

## Known failures are not passing coverage

The edge-case audit still has one unresolved async larger-window API/design
limitation after the StreamReader, BytesReader, util, and async boundary fixes.
Its executable assertion lives in the separate
[known-regression suite](../regressions/README.md). It deliberately fails when
run and is not included by the root package's ordinary `cargo test`. No tests
are marked `ignore` or `should_panic` to make these bugs look successful.

`cargo test --all-features` also fails on the existing `codec` implementation:
missing `CodeType` imports, a missing async `.await`, and a missing return
expression in BytesReader. Excluding `codec` from the supported-feature command
above does not imply that the all-features build passes.

The new synchronous integration tests can be cross-compiled separately:

```sh
cargo test --no-run --target wasm32-unknown-unknown --features stream,util,serde --test reader_edges --test stream_peek --test util_edges
```

This is compile coverage, not Wasm execution. The existing full unit-test harness
does not compile for that target because its file test unconditionally calls the
native-only `StreamReader::from_file` method.
