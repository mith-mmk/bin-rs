# Known pre-existing regressions

This is an opt-in, non-publishable test crate against the parent source. These
tests assert the desired contract, not the existing incorrect behavior. They
are intentionally separate from the passing suite while the fixes are scoped
and reviewed. A passing root `cargo test` does **not** resolve these findings.
When fixing a finding, move its regression test into the regular suite.

Run from the repository root:

```sh
cargo test --manifest-path regressions/Cargo.toml --no-fail-fast
cargo test --release --manifest-path regressions/Cargo.toml --no-fail-fast
```

After the BytesReader bounds fix: 13 failures remain in both debug and release
(3 async, 10 util). The original 3 BytesReader regressions now run in the regular
suite, along with broader extreme-length and signed-seek checks.

## Fixed: BytesReader overflow before bounds checking

- `skip_ptr(usize::MAX)` after consuming one byte previously panicked in debug.
  In release the addition wrapped, the request succeeded, and the position
  wrapped back to zero.
- `SeekFrom::Current(i64::MAX)` at position one and
  `SeekFrom::End(i64::MAX)` on a nonempty buffer previously panicked in debug.

These inputs require no large allocation. They now return errors and leave the
position unchanged in both profiles. See `tests/bytes_reader_bounds.rs`.

## AsyncBytesReader: peek boundaries and buffer size

- An exact-size peek (`abc`, length 3) incorrectly returns `Data shotage`.
- A zero-length peek on empty input incorrectly returns the same error.
- A valid peek longer than the current buffer window is rejected.

The first two are boundary errors. Supporting larger lookahead needs a separate
API/design decision: the current async peek method does not require `AsyncSeek`,
so the synchronous seek/restore fix cannot simply be copied without affecting
which readers are accepted.

## util: incorrect values and range handling

- `read_i8` casts a byte's address rather than reading its value.
- `read_i16_le` (and its endian-dispatch path) uses big-endian byte order.
- All four signed/unsigned 128-bit readers read the first eight bytes twice.
- `read_ascii_string` ignores its offset and length.
- `read_utf16_string` ignores its length. Even an in-bounds nonzero-offset
  request can then panic by reading beyond the provided slice.

All util repros provide valid input ranges. They are not tests of unsupported
out-of-range calls. None of these source functions is modified by this PR.
