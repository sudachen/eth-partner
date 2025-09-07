# Task List: REPL AI Assistant

Based on `prd-repl-app.md`.

## Relevant Files
* `repl/Cargo.toml`: Project manifest for the new `repl` crate.
* `repl/src/main.rs`: Main application entry point and REPL loop.
* `repl/src/config.rs`: Configuration loading and management.
* `repl/src/llm/mod.rs`: LLM provider abstraction and implementation.
* `repl/src/agent.rs`: Core agent logic using the `rig` framework.
* `repl/src/tools/mod.rs`: Toolset definition for the agent.
* `repl/src/tools/web_search.rs`: Brave Search API tool implementation.
* `~/.config/eth-partner/config.json` (example): Example configuration file.

---

## 1. Project Setup & Basic REPL
- [x] **T1: Create the `repl` binary crate and implement the basic REPL loop.**
  - [x] **1.1:** Create a new binary crate named `repl` within the workspace.
  - [x] **1.2:** Add necessary dependencies for the REPL loop (`rustyline`).
  - [x] **1.3:** Implement a simple loop in `main.rs` that reads user input from stdin.
  - [x] **1.4:** Implement command handling for `/exit` to terminate the application.
  - [x] **1.5:** Implement command handling for `/help` to display a help message.

## 2. Configuration Management
- [x] **T2: Implement configuration loading.**
  - [x] **2.1:** Create a `config.rs` module.
  - [x] **2.2:** Add dependencies for serialization (`serde`, `serde_json`) and home directory resolution (`dirs`).
  - [x] **2.3:** Define a `Config` struct to hold all settings.
  - [x] **2.4:** Implement a function to load the configuration from `~/.config/eth-partner/config.json`.
  - [x] **2.5:** Integrate the config loading into `main.rs`.

## 3. LLM Provider Abstraction
- [ ] **T3: Implement a flexible LLM provider system.**
  - [ ] **3.1:** Create an `llm` module with a `LlmProvider` trait.
  - [ ] **3.2:** Implement a `GeminiProvider` struct that conforms to the trait.
  - [ ] **3.3:** Implement logic to initialize Gemini using application-default credentials or an API key from the config.

## 4. Agent & Tool Integration
- [ ] **T4: Set up the `rig` agent and integrate tools.**
  - [ ] **4.1:** Add `rig` as a dependency.
  - [ ] **4.2:** Create an `agent.rs` module to initialize the `rig` agent and the Re-Act loop.
  - [ ] **4.3:** Create a `tools` module.
  - [ ] **4.4:** Implement the `web_search` tool in `tools/web_search.rs` using the Brave Search API.
  - [ ] **4.5:** Integrate the `mcp-wallet` crate as a tool.
  - [ ] **4.6:** Connect the REPL input (non-commands) to the agent for processing.

## 5. Finalization & Testing
- [ ] **T5: Polish and finalize the application.**
  - [ ] **5.1:** Write unit and integration tests for key components (config loading, command parsing).
  - [ ] **5.2:** Add comprehensive error handling.
  - [ ] **5.3:** Write a `README.md` for the `repl` crate explaining how to configure and run the application.
