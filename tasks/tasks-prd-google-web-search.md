## Relevant Files

- `repl/src/tools/web_search.rs` - Main web search tool implementation to switch provider to Google CSE.
- `repl/src/config.rs` - Configuration loader; add Google API env variables.
- `repl/src/lib.rs` - Tool registration/wiring; ensure tool enabled only when configured.
- `repl/tests/e2e_web_search_tests.rs` - E2E tests; update to Google and skip when creds missing.
- `repl/tests/repl_tests.rs` - Integration tests that may reference tool availability.
- `repl/tests/e2e_tests.rs` - Ensure any agent flows invoking web_search are adjusted.
- `.env.example` - Add GOOGLE_SEARCH_API_KEY and GOOGLE_SEARCH_ENGINE_ID.
- `repl/README.md` - Document configuration and usage.

### Notes

- Use Google Programmable Search Engine (CSE) JSON API at `https://www.googleapis.com/customsearch/v1`.
- Required params: `key`, `cx`, `q`; optional `num` (1..10), `safe=off`.
- Return minimal LLM-friendly fields: title, url, snippet.
- Remove Brave-specific code and docs.
- E2E tests should run only when both credentials are present; otherwise skip.

## Tasks

- [x] 1.0 Replace Brave integration with Google CSE client
  - [x] 1.1 Audit `repl/src/tools/web_search.rs` for Brave-specific code (endpoints, params, models).
  - [x] 1.2 Remove Brave-specific request/response models and constants.
  - [x] 1.3 Introduce a Google CSE client layer (request builder + response DTOs with `serde`).
  - [x] 1.4 Map Google results to tool output fields: `title`, `url`, `snippet`.
  - [x] 1.5 Implement clear error types/messages for 4xx/5xx and parsing failures.

- [x] 2.0 Add configuration for Google CSE (env vars) and wiring
  - [x] 2.1 Add `GOOGLE_SEARCH_API_KEY` and `GOOGLE_SEARCH_ENGINE_ID` to `repl/src/config.rs`.
  - [x] 2.2 Validate presence of both variables; surface a clear "tool unavailable" state if missing.
  - [x] 2.3 Wire tool registration in `repl/src/lib.rs` to enable only when both vars are set.
  - [x] 2.4 Update `.env.example` to include both variables with brief descriptions.

- [ ] 3.0 Update `web_search` tool implementation to call Google and map results
  - [x] 3.1 Implement call to `https://www.googleapis.com/customsearch/v1` with params `key`, `cx`, `q`.
  - [x] 3.2 Support optional `num` (default 5, max 10) and set `safe=off` per requirements.
  - [x] 3.3 Parse JSON response and transform to the tool's output structure.
  - [x] 3.4 Handle empty results gracefully (return empty list, not error).
  - [x] 3.5 Add concise logging around queries and response sizes (no sensitive data).

- [ ] 4.0 Testing: add unit tests with mocked HTTP and update e2e tests
  - [x] 4.1 Unit test: success scenario parses `title`, `link`, `snippet` correctly.
  - [x] 4.2 Unit test: empty `items` returns empty results.
  - [x] 4.3 Unit test: 4xx/5xx errors return readable error messages with status.
  - [x] 4.4 E2E test: execute search through agent when creds present; skip when missing.
  - [x] 4.5 Ensure CI respects skip behavior and tests are stable without network.

- [x] 5.0 Documentation and examples: README and .env.example updates
  - [x] 5.1 Update `repl/README.md` with setup steps to create a CSE and obtain `cx`.
  - [x] 5.2 Document required env vars and SafeSearch off behavior.
  - [x] 5.3 Add usage examples showing expected output fields.

- [x] 6.0 Cleanup: remove Brave code paths, tests, and references
  - [x] 6.1 Remove Brave-specific code and constants from the codebase.
  - [x] 6.2 Remove Brave-related tests and test utilities.
  - [x] 6.3 Update or remove Brave mentions from docs and `.env.example`.
  - [x] 6.4 Run `cargo fmt --all` and `cargo clippy` to ensure cleanliness.

## Associated PRD

Use @tasks/prd-google-web-search.md
