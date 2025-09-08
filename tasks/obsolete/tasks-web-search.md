# Task List: Web Search Tool Integration

This task list tracks the work required to enable and test the `WebSearchTool` in the `repl` AI assistant.

## 1. Configuration
- [x] **T1: Update configuration files.**
  - [x] **1.1:** Add `BRAVE_API_KEY` to the `.env.example` file.

## 2. Testing
- [x] **T2: Add an end-to-end test for the web search tool.**
  - [x] **2.1:** Create a new test file `repl/tests/e2e_web_search_tests.rs`.
  - [x] **2.2:** Write a test that provides a `BRAVE_API_KEY`, sends a prompt requiring a web search, and verifies the response.

## Relevant Files

- `.env.example`: Example environment file.
- `repl/src/lib.rs`: Core REPL logic where the tool is initialized.
- `repl/tests/e2e_web_search_tests.rs`: End-to-end tests for web search integration.
