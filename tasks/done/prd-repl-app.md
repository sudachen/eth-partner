# PRD: REPL AI Assistant

## 1. Introduction/Overview

This document outlines the requirements for a REPL (Read-Eval-Print Loop) application that functions as an AI assistant for interacting with the Ethereum blockchain. The application will be a binary crate named `repl`. It will use the `rig` agentic AI framework to implement a Re-Act loop, allowing an LLM to use a set of tools to fulfill user requests. Initial tools will include web search capabilities via the Brave Search API and an embedded `mcp-wallet` server for blockchain interactions.

## 2. Goals

*   **G1:** Create a command-line AI assistant that can understand natural language prompts related to the Ethereum blockchain.
*   **G2:** Implement a flexible tool system that can be expanded in the future.
*   **G3:** Provide a clear and simple configuration method for users to add API keys and connect to external services.
*   **G4:** Build a modular architecture, particularly for LLM providers, to allow for future expansion.

## 3. User Stories

*   **As a user,** I want to ask questions about the Ethereum network (e.g., "what's the latest block number?") in plain English and get a correct answer.
*   **As a user,** I want the assistant to be able to search the internet if it doesn't know the answer to a general question.
*   **As a user,** I want to be able to exit the application cleanly using a simple command like `/exit`.
*   **As a developer,** I want to be able to configure the application with my own API keys and add new MCP servers without recompiling the code.

## 4. Functional Requirements

### REPL & Commands
*   **FR1:** The application MUST be a binary crate named `repl`.
*   **FR2:** The REPL MUST read and process user input from the command line.
*   **FR3:** Input starting with `/` MUST be interpreted as a command. The following commands must be supported:
    *   `/exit`: Terminates the application.
    *   `/help`: Displays a list of available commands and a brief description of the assistant.
*   **FR4:** Any input not starting with `/` MUST be passed to the AI agent for processing.

### AI Agent & Tools
*   **FR5:** The AI agent MUST be implemented using the `rig` framework, following a Re-Act loop pattern.
*   **FR6:** The agent's internal monologue (reasoning steps) will NOT be visible to the user by default.
*   **FR7:** The agent MUST have access to a `websearch` tool that uses the Brave Search API.
*   **FR8:** The application MUST include an embedded `mcp-wallet` server for Ethereum-related tasks.

### LLM & Providers
*   **FR9:** The application MUST feature an LLM provider abstraction to allow for different language models.
*   **FR10:** The initial implementation MUST support Google Gemini.
*   **FR11:** The application should default to using Google application-default credentials for Gemini.
*   **FR12:** If a `GEMINI_APIKEY` is specified in the configuration file, it MUST be used for authentication instead.

### Configuration
*   **FR13:** The application MUST load its configuration from `~/.config/api-partner/config.json`.
*   **FR14:** The configuration file MUST support:
    *   The Brave Search API key.
    *   The RPC URL (`rpc_url`) for the embedded `mcp-wallet`.
    *   An optional `GEMINI_APIKEY`.
    *   A list of external MCP servers, including all necessary connection details (e.g., address, authentication tokens).

## 5. Non-Goals (Out of Scope)

*   Persistent conversation history across sessions.
*   A graphical user interface (GUI).
*   Support for blockchains other than Ethereum in the initial version.
*   Advanced features like user profiles or multi-turn conversation memory beyond the immediate session.

## 6. Design Considerations (Optional)

*   The command-line interface should be clean and simple, clearly distinguishing between user input, assistant responses, and system messages.

## 7. Technical Considerations

*   **Primary Crate:** `repl` (binary)
*   **Agent Framework:** `rig`
*   **Initial LLM:** Google Gemini
*   **Initial Tools:** Brave Search, `mcp-wallet`
*   **Configuration:** JSON format

## 8. Success Metrics

*   A user can successfully start the REPL, ask a question that requires web access, and receive a valid response.
*   A user can successfully ask a question about the Ethereum network and receive a valid response from the `mcp-wallet` tool.
*   The application correctly loads and applies configuration from the specified JSON file.

## 9. Open Questions

*   None at this time.
