# bin-rs
 bin-rs is Binary Reader/Writer crate

- 0.0.2 ... BinaryReaderSeek and typo fix
- 0.0.7 ... io/writer*
- 0.0.8 ... yark
- 0.0.9 ... Stream supoort/ read utf16 support
- 0.0.10 ... Some Bug fix, tested async on tokio
- 0.0.11 ... Buffered lookahead, reader overflow, numeric/string utility and async boundary fixes. See [release notes](CHANGELOG.md).

## Tests

See [test coverage and commands](tests/README.md), including the explicitly failing known-regression suite.

## Stabilization

See the [synchronous contract baseline and 0.1.x release gates](docs/synchronous-contract.md). Async stable-contract work targets 0.2.0.

The unreleased 0.1.0 target removes unfinished codec support as a breaking
change, not a 0.0.x patch. See [migration notes](docs/synchronous-contract.md#breaking-migration-for-the-010-target).
