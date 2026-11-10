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

After the BytesReader, util, and async boundary fixes on 2026-11-10, only one
async larger-window API/design limitation remains in both debug and release.
The original 3 BytesReader, 10 util, and 2 async boundary regressions now run
in the regular suite, alongside expanded boundary coverage.



## Fixed: BytesReader overflow before bounds checking

- `skip_ptr(usize::MAX)` after consuming one byte previously panicked in debug.
  In release the addition wrapped, the request succeeded, and the position
  wrapped back to zero.
- `SeekFrom::Current(i64::MAX)` at position one and
  `SeekFrom::End(i64::MAX)` on a nonempty buffer previously panicked in debug.

These inputs require no large allocation. They now return errors and leave the
position unchanged in both profiles. See `tests/bytes_reader_bounds.rs`.

## AsyncBytesReader: peek boundaries and buffer size

- A valid peek longer than the current buffer window is rejected. This remains
  an API/design limit and the opt-in test records the desired larger-window behavior.

Exact-window and empty peeks are now fixed and tested in
`tests/async_reader_edges.rs`; empty peeks perform no I/O.

Supporting larger lookahead needs a separate
API/design decision: the current async peek method does not require `AsyncSeek`,
so the synchronous seek/restore fix cannot simply be copied without affecting
which readers are accepted. Adding owned lookahead state would also affect the
existing `Copy` implementation and require coordinating all reads and seeks.

A possible additive API is `read_bytes_no_move_seekable` on the existing
`AsyncSeek`-bounded impl. It could save the logical position, allocate fallibly,
read exactly, and restore position on success or read failure. It would need an
explicit policy for restore failures (including preserving both errors), and
cancellation cannot guarantee async restoration after the future is dropped.
No such API is added here pending a separate design decision.

## Resolved util regressions

All 10 former `util.rs` repros now live in the regular `tests/util_edges.rs`
suite, alongside expanded boundary coverage. The fixes read signed bytes by
value, use little-endian order for signed 16-bit values, read both halves of
128-bit values, and honor the byte offset and byte length for ASCII and UTF-16.
ASCII continues to map every byte to its same Unicode code point, including
embedded NUL and high bytes. UTF-16 remains lossy, stops at a NUL code unit, and
ignores an odd trailing byte. Its length is in bytes, consistent with the
BinaryReader UTF-16 API and the original full-buffer tests. Slice callers must
provide an in-bounds requested range; no new fallible API is introduced.
