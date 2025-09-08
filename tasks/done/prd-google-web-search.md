# PRD: Replace Brave Search with Google Programmable Search (CSE) for Web Search Tool

## 1. Overview

We will replace the current Brave Search-based `web_search` tool in the `repl` agentic system
with Google Programmable Search Engine (CSE) JSON API. The tool enables the agent to query the
full web and return concise search results suitable for LLM consumption.

## 2. Goals

- Replace Brave Search integration entirely with Google CSE JSON API.
- Provide full-web search capability (no domain restrictions) via a properly configured CSE.
- Keep tool output minimal and LLM-friendly: title, url, snippet.
- Expose configuration via environment variables and skip gracefully when not configured.
- Maintain end-to-end coverage and introduce unit tests with mocked HTTP.
- Update documentation and example environment variables accordingly.

## 3. User Stories

- As an agent developer, I want the `web_search` tool to query the public web so that the agent
  can ground its answers with up-to-date sources.
- As a DevOps/user, I want to enable/disable the search tool via environment variables so that
  I can control usage easily without code changes.
- As a maintainer, I want tests that run when credentials are set and are skipped otherwise,
  plus unit tests that don't require real API calls, so CI remains stable.

## 4. Functional Requirements

1. The system must use Google Programmable Search Engine (CSE) JSON API for web search.
2. The system must query the full web. This requires configuring a CSE with "Search the entire
   web" enabled.
3. The tool must require these environment variables:
   - `GOOGLE_SEARCH_API_KEY` (required)
   - `GOOGLE_SEARCH_ENGINE_ID` (aka `cx`, required by the Google API)
4. If either variable is missing, the `web_search` tool must be unavailable and report a clear
   message (e.g., "Web search is not configured").
5. The tool must return for each result: `title`, `url` (link), and `snippet`.
6. The tool must fetch a single page of results with a default of 5 items; allow a caller option
   to request up to 10 items (Google CSE per-page limit) but keep defaults minimal.
7. The tool must disable SafeSearch (set to off) per user request.
8. The tool must not implement additional client-side rate limiting. It may perform basic retries
   for transient network errors but not for quota (429) by default.
9. When Google responds with 4xx/5xx, the tool must return an error message with status and a
   concise hint.
10. The Brave Search integration and configuration must be removed, including tests and docs that
    reference Brave.
11. End-to-end tests must run only when both Google credentials are present; otherwise they should
    be skipped.
12. Unit tests must be added that mock HTTP responses to validate parsing and error handling.

## 5. Non-Goals

- Implementing domain restrictions or site preferences.
- Extracting advanced metadata such as structured data, favicons, or publish dates.
- Implementing advanced ranking or multi-page aggregation.
- Implementing manual rate limiting caps.

## 6. Design Considerations

- API endpoint: `https://www.googleapis.com/customsearch/v1`
- Required query params: `key`, `cx`, `q`, optional `num` (1..10), `safe` ("off").
- Response mapping: `items[].title` -> title, `items[].link` -> url, `items[].snippet` -> snippet.
- Use `reqwest` + `serde` for HTTP and JSON.
- Configuration read from environment (e.g., via existing config mechanism in `repl`).
- Tests: use `tokio::test` for async, mock HTTP with `httpmock` or similar if not already present.

## 7. Success Metrics

- `web_search` tool returns valid results using Google CSE when configured.
- E2E test passes locally/CI when credentials are present and is skipped otherwise.
- Unit tests pass reliably without network access.
- Documentation and `.env.example` updated to include `GOOGLE_SEARCH_API_KEY` and `GOOGLE_SEARCH_ENGINE_ID`.

## 8. Open Questions

- Do we want to expose an optional `num` param to callers, with default 5 and max 10? (Proposed: yes)
- Do we want minimal retries for transient network errors (not quota)? (Proposed: yes, 1 retry)
