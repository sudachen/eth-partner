# PRD: Default File Logging for eth-partner

## Context
The repository contains two Rust crates:
- `repl`: Interactive REPL assistant using `tracing` for logging.
- `mcp-wallet`: Minimal MCP-style wallet server currently using `env_logger` + `log` macros.

Currently, logs are printed to stdout/stderr unless configured manually. We want a
sensible default that writes logs to a file.

## Goal
When nothing is specified by the user (no special env or CLI flags), all logs from
both crates must be written to a single log file named `eth-partner-log.txt` in the
current working directory.

## Requirements
- Default behavior: write all logs to `eth-partner-log.txt`.
- Respect `RUST_LOG` if set for log levels/filters.
- Keep behavior compatible with tests that may already initialize a global logger.
- Do not break existing `log::info!` usage in `mcp-wallet`.
- Keep solution minimal and consistent across crates.

## Non-Goals (for this PRD)
- Rotating logs, compression, or retention policies.
- Configurable log file name or directory (can be added later).
- Changing the existing log call sites.

## Proposed Approach
- Standardize on `tracing`/`tracing-subscriber` across both crates.
- Use `tracing-appender` with a non-rolling file appender pointing to
  `./eth-partner-log.txt` plus `non_blocking` writer for performance.
- In `mcp-wallet`, forward `log` macros to `tracing` via `tracing_log::LogTracer::init()`.
- Initialize the subscriber early in each binary entrypoint (`repl` startup path
  and `mcp-wallet` main) with:
  - `EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info"))`.
  - `fmt().with_target(false).with_writer(non_blocking_writer)`.
  - Ignore `set_global_default` errors if already initialized (e.g., tests).

## Acceptance Criteria
- Running either crate with no env/flags creates or appends to `eth-partner-log.txt`.
- Logs include info-level messages by default; level adjustable via `RUST_LOG`.
- `mcp-wallet` log macros are captured in the file without code changes at call sites.
- Tests still pass.

## Rollback
If issues arise, we can revert to prior behavior: `repl` printing via default
`tracing-subscriber` to stderr/stdout and `mcp-wallet` using `env_logger`.
