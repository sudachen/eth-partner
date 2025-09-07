# Task List: REPL Testing

This task list tracks the work required to add a comprehensive test suite to the `repl` crate.

## 1. Refactor REPL for Testability
- [x] **T1: Move the core REPL logic into a library.**
  - [x] **1.1:** Create a `repl/src/lib.rs` file.
  - [x] **1.2:** Move the REPL loop and related logic from `main.rs` into a `run_repl` function in `lib.rs`.
  - [x] **1.3:** Update `main.rs` to call the new `run_repl` function.

## 2. Add REPL Tests
- [x] **T2: Implement tests for REPL commands and prompting.**
  - [x] **2.1:** Create a `repl/tests/repl_tests.rs` file.
  - [x] **2.2:** Add a test for the `/help` command.
  - [x] **2.3:** Add a test for the `/exit` command.
  - [x] **2.4:** Add a test for agent prompting, using a mock agent.

## Relevant Files

- `repl/src/lib.rs`: The new library file for the REPL logic.
- `repl/src/main.rs`: The updated main binary file.
- `repl/tests/repl_tests.rs`: The new integration test file.
