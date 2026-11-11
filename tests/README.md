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

## Executable Wasm coverage

The same memory-backed unit and integration tests run in Node using
`wasm-bindgen-test`, including the async in-memory cases. Native filesystem
coverage remains in `check_native_files`; memory fixture checks use
`include_bytes!` and do not require a filesystem. No test is ignored to make
Wasm pass. This is Node coverage, not browser or Wasm filesystem support.

Install the matching runner and target, then execute (not just compile):

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.100 --locked
export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-bindgen-test-runner
cargo test --locked --target wasm32-unknown-unknown --lib --tests
cargo test --locked --target wasm32-unknown-unknown --lib --tests --all-features
cargo test --locked --target wasm32-unknown-unknown --lib --tests --release
cargo test --locked --target wasm32-unknown-unknown --lib --tests --release --all-features
```

PowerShell uses `$env:CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER =
'wasm-bindgen-test-runner'` instead of `export`. Put the matching runner on PATH,
or set the variable to its full path. The CLI version must match the pinned
`wasm-bindgen` version. CI uses Node 24. Wasm commands explicitly select unit
and integration tests; doctests run in the native jobs.

## Reproducible Rust 1.71 validation

The root version-3 `Cargo.lock` is committed for CI and contributor tests.
Every CI test uses `--locked`. Normal dependency ranges and the declared
Rust 1.71 MSRV remain unchanged; downstream library consumers resolve their
own dependencies. Wasm test dependencies are target-specific dev dependencies,
so native builds do not compile the Wasm harness.

The lock retains Tokio 1.53.3, tokio-macros 2.7.2, bytes 1.12.1, and serde
1.0.229. The Wasm pair is wasm-bindgen 0.2.100 and wasm-bindgen-test 0.3.50
(both declare Rust 1.57). Their transitive versions are locked too, including
bumpalo 3.14.0 rather than a version requiring Rust 1.71.1.

```sh
cargo +1.71.0 test --locked
cargo +1.71.0 test --locked --all-features
cargo +1.71.0 test --locked --release
cargo +1.71.0 test --locked --release --all-features
```

Dependency updates are deliberate maintenance: resolve with modern Cargo's
`resolver.incompatible-rust-versions="fallback"` setting, keep lock format 3,
and rerun all native MSRV and executable Wasm commands before committing the
updated lock. Do not regenerate a floating lock inside CI. The runner has its
own historical installation lock, separate from this project's graph; its
0.2.100 installation currently warns about yanked spin 0.9.8 and future
compatibility in buf_redux/multipart. Installation succeeds, but runner updates
must be coordinated with the pinned Wasm test dependencies.

## Local validation of the CI/Wasm change

Windows validation with Rust 1.99.0 and 1.71.0 passed 48 default and 113
all-feature native tests (including three doctests) in both profiles. Node
24.22.1 executed 44 default and 109 all-feature Wasm tests in both profiles.
All 16 feature subsets passed native debug and release tests.

Use a unique explicit `CARGO_TARGET_DIR` for each independent worktree when
validating locally. A shared global target directory reused old test binaries
in preliminary runs; the results above come from a complete rerun with isolated
target directories. Existing repository formatting differences make
`cargo fmt --all -- --check` fail on the base as well; this change does not
perform an unrelated repository-wide reformat.

## CI layout

The workflow tests stable and Rust 1.71 on Linux and Windows with default and
all features, in debug and release. A stable Linux job tests all 16 subsets of
stream/async/serde/util in both profiles. A separate Node job executes the Wasm
suite with default and all features in both profiles. The intentionally
failing `regressions/` package remains separate; passing CI does not resolve
its larger-window async limitation.

The workflow uses read-only repository permission, pinned checkout/setup-node
actions, and no persisted checkout credentials, secrets, or privileged
`pull_request_target` trigger.

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
