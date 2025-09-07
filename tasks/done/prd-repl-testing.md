# PRD: REPL Testing

## 1. Overview

This document outlines the requirements for adding a comprehensive test suite to the `repl` crate. The goal is to ensure the reliability and correctness of the REPL's command handling and agent interaction.

## 2. Goals

- Add unit and integration tests for the REPL's command parsing logic.
- Add tests for the agent prompting workflow.
- Refactor the REPL loop to improve testability.

## 3. Requirements

- The test suite should cover all REPL commands (`/help`, `/exit`).
- The tests should not make real API calls to the LLM or other external services.
- The tests should be runnable with `cargo test`.
