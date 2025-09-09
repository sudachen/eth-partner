# Tasks: Default File Logging to eth-partner-log.txt

Parent Task T1: Implement default file logging across crates [x]

- [x] T1.1 Audit current logging setup in repository
- [x] T1.2 Define unified approach and dependencies
- [x] T1.3 Implement default file logging in `repl` crate
- [x] T1.4 Implement default file logging in `mcp-wallet` crate
- [x] T1.5 Ensure tests remain stable (single global logger, no double init)
- [x] T1.6 Verify end-to-end: binaries and tests write to `eth-partner-log.txt`
- [x] T1.7 Update READMEs with logging behavior

## Findings for T1.1 (Audit)

- `repl` (`repl/src/lib.rs`): uses `tracing` + `tracing-subscriber::fmt` initialized in
  `run_repl()` with an `EnvFilter` defaulting to `info`. Output currently goes to stderr
  (no file writer configured). It ignores errors if the subscriber is already set.

- `mcp-wallet` (`mcp-wallet/src/main.rs`): uses `env_logger` + `log` macros, targets
  stderr explicitly. No file writer configured.

- Cargo dependencies:
  - `repl` already depends on `tracing` and `tracing-subscriber`.
  - `mcp-wallet` depends on `log` and `env_logger`, not on `tracing`.

Conclusion: two different logging stacks are used. To meet the requirement and keep
call sites unchanged, standardize on `tracing` across both crates and capture `log`
macros via `tracing_log::LogTracer` in `mcp-wallet`.

## Unified Approach (T1.2)

- Standardize on `tracing` across both crates with `tracing-subscriber`.
- Use `tracing-appender` with a non-rolling file appender targeting
  `./eth-partner-log.txt` and a `non_blocking` writer.
- Respect `RUST_LOG` via `EnvFilter::try_from_default_env()` and default to `info`.
- In `mcp-wallet`, capture existing `log` macros via `tracing_log::LogTracer::init()` so
  all `log::info!` etc. are forwarded into `tracing` without changing call sites.
- Initialize the global subscriber once per process and ignore `set_global_default`
  errors (common in tests that pre-init logging).

## Dependencies to Add (T1.2)

- `repl/Cargo.toml`
  - tracing-appender = "0.2"

- `mcp-wallet/Cargo.toml`
  - tracing = "0.1"
  - tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
  - tracing-appender = "0.2"
  - tracing-log = "0.2"
  - (optional cleanup later) remove `env_logger` if unused after migration

## Relevant Files

- `tasks/prd-default-file-logging.md` — PRD describing goals and acceptance criteria
- `tasks/tasks-prd-default-file-logging.md` — Task list tracking progress
- `repl/Cargo.toml` — Added `tracing-appender` dependency
- `repl/src/lib.rs` — Initialize logging to write to `eth-partner-log.txt` by default
- `mcp-wallet/Cargo.toml` — Added tracing, tracing-subscriber, tracing-appender, tracing-log
- `mcp-wallet/src/main.rs` — Initialize tracing-based file logging and forward log macros
- `README.md` — Added Logging section for repo-level documentation
- `repl/README.md` — Added Logging section for REPL usage
- `mcp-wallet/README.md` — Added Logging section for wallet usage

## Notes

- T1.5: Ran `cargo test` for the workspace; all tests passed with no logger
  double-initialization issues.
-  Verified `eth-partner-log.txt` exists at repo root.
