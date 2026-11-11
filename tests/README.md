# Test suites

Run the default suite and the supported optional-feature suite from the repository root:

```sh
cargo test
cargo test --all-features
cargo test --release --all-features
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
[known-regression suite](https://github.com/mith-mmk/bin-rs/blob/b2934121238df21206749ae943518ce59be9e606/regressions/README.md). It deliberately fails when
run and is not included by the root package's ordinary `cargo test`. No tests
are marked `ignore` or `should_panic` to make these bugs look successful.

The former `codec` implementation prevented all-features compilation (missing
imports, an async await, and a BytesReader return expression). This is now
historical: the unfinished feature, dependency, and APIs were removed for the
breaking 0.1.0 target, not a 0.0.x patch. The all-features commands above cover
all remaining features. See [migration notes](../docs/synchronous-contract.md#breaking-migration-for-the-010-target).
Async's stable contract still targets 0.2.0; passing this suite does not resolve
the separate larger-window regression.

The new synchronous integration tests can be cross-compiled separately:

```sh
cargo test --no-run --target wasm32-unknown-unknown --features stream,util,serde --test reader_edges --test stream_peek --test util_edges
```

This is compile coverage, not Wasm execution. The existing full unit-test harness
does not compile for that target because its file test unconditionally calls the
native-only `StreamReader::from_file` method.

## Rust 1.71 validation

Release 0.0.11 was tested with Rust 1.71.0 and a compatible dependency lock:
Tokio 1.53.2, tokio-macros 2.7.2, bytes 1.12.1, serde 1.0.229, and
encoding_rs 0.8.35. The default suite passed 40 tests; the `stream,async,serde,util`
suite passed 100 tests in both debug and release, including doctests.

The manifest keeps its existing dependency ranges. Latest dependency versions
are not guaranteed to support older Rust: Tokio 1.54.1 requires Rust 1.85.
Cargo 1.71 does not automatically select a compatible resolution. To reproduce
the strategy in an isolated checkout, generate a lockfile with modern Cargo's
`resolver.incompatible-rust-versions="fallback"` setting, then test that lock
using `cargo +1.71.0 test --locked --features stream,async,serde,util` and its
`--release` variant. Keep the generated compatible lock for repeatable results;
future registry changes may produce a different resolution.

## Synchronous contract baseline

`sync_contract.rs` characterizes legacy string mappings, odd UTF-16 sizes,
composite-read error positions, and backend-specific error/seek behavior.
See [the contract baseline](../docs/synchronous-contract.md); these tests do
not imply that unresolved release gates have been satisfied.

```sh
cargo test --test sync_contract
cargo test --features util --test sync_contract
```

## Codec-removal validation (0.1.0 target)

On Windows with Rust/Cargo 1.99.0, the default suite passed 47 tests and
all-features passed 108 tests in both debug and release, including doctests.
All 16 combinations of stream/util/serde/async passed. Focused contract tests
passed with default (7) and util (8); all-features all-target checks and
documentation generation passed. Cargo metadata and the all-features dependency
tree contain neither a codec/encoding_rs feature nor an encoding_rs dependency.

Rust 1.71.0 also passed the 108-test all-features suite in debug and release
using a locally generated compatible lock (Tokio 1.53.3, tokio-macros 2.7.2,
bytes 1.12.1, serde 1.0.229). The lock is not published or committed; dependency
ranges and the declared MSRV are unchanged. The separate async larger-window
regression was rerun and still fails; no Wasm execution was performed here.

## Util contract characterization

`util_contract.rs` is enabled by `util` and records the existing UTF-8/NUL
writer behavior, invalid-UTF-8 empty result, and byte-widening/odd-tail behavior
of `read_bytes_as_u16_vec`. It does not fix these quirks or declare them final
stable API decisions. Valid offset/range boundaries are tested without assigning
an error contract to invalid ranges. See the util section of the contract baseline.

```sh
cargo test --features util --test util_contract
cargo test --release --features util --test util_contract
```

Validation on Windows with Rust 1.99.0: default debug/release passed 47 tests
and all-features debug/release passed 112, including doctests. The focused util
suite passed four tests in both profiles. Rust 1.71.0 all-features also passed
112 tests in both profiles using the locally generated compatible lock described
above. Changed Rust files pass rustfmt checks and all-features rustdoc builds.
Consumer suites, Wasm execution, and the separate known-failing async regression
were not rerun for this documentation-and-test-only change.
