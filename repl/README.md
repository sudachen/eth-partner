# REPL AI Assistant

This crate provides a simple REPL (Read-Eval-Print Loop) AI assistant that uses the `rig` framework to interact with large language models and external tools.

## Features

- Interactive REPL interface.
- Extensible toolset for interacting with external services.
- Configuration managed through a simple JSON file.
- Powered by Google's Gemini Pro via the `rig` framework.
- Optional web search tool backed by Google Programmable Search Engine (CSE).

## Configuration

Before running the application, you need to create a configuration file at `~/.config/eth-partner/config.json`. This file should contain the necessary API keys for the services you want to use.

### Example Configuration (optional file)

```json
{
    "llm": {
        "google_api_key": "YOUR_GEMINI_API_KEY"
    }
}
```

> Note: A config file is optional. Environment variables are preferred. Values
> read from environment will override config file values.

### Web Search (Google CSE)

The `web_search` tool uses Google Programmable Search Engine (CSE) JSON API to
query the public web and return concise results for LLM consumption.

#### Setup steps

1. Create a CSE in the [Programmable Search Control Panel](https://programmablesearchengine.google.com/).
2. Configure it to "Search the entire web".
3. Note the Search Engine ID (aka `cx`).
4. Create/obtain a Google API key with access to the Custom Search API.

#### Required environment variables

Add the following to your `.env` (see `.env.example`):

```env
GOOGLE_SEARCH_API_KEY="your-google-cse-api-key"
GOOGLE_SEARCH_ENGINE_ID="your-cse-engine-id"
```

When both variables are set, the REPL will register the `web_search` tool.
If either is missing, the tool will be unavailable.

#### Tool output format

Note: The tool sets Google CSE `safe=off` to maximize recall, per project
requirements.

The tool returns a JSON string with the following structure:

```json
{
  "total": 2,
  "results": [
    { "index": 1, "title": "...", "url": "...", "snippet": "..." },
    { "index": 2, "title": "...", "url": "...", "snippet": "..." }
  ],
  "provider": "google_cse"
}
```

When there are no results, the tool returns:

```json
{ "total": 0, "results": [], "provider": "google_cse" }
```

### Obtaining API Keys

- **Gemini API Key**: You can obtain a Gemini API key from [Google AI Studio](https://aistudio.google.com/app/apikey).
--

### Running the Application

To run the REPL assistant, navigate to the root of the workspace and use the following command:

```bash
cargo run -p repl
```

This will start the interactive REPL, where you can enter commands or prompts for the AI assistant.

### Commands

- `/help`: Displays a list of available commands.
- `/exit`: Exits the application.

### Usage example (web_search)

With `GOOGLE_SEARCH_API_KEY` and `GOOGLE_SEARCH_ENGINE_ID` set, the agent can
use `web_search` autonomously when needed. Example prompt:

```
"List recent announcements from the Rust project website and pick the most relevant."
```

The tool returns JSON like:

```json
{
  "total": 3,
  "results": [
    { "index": 1, "title": "Rust Blog — Announcing ...", "url": "https://blog.rust-lang.org/...", "snippet": "..." },
    { "index": 2, "title": "Rust 1.xx Released", "url": "https://blog.rust-lang.org/...", "snippet": "..." },
    { "index": 3, "title": "RFC updates", "url": "https://blog.rust-lang.org/...", "snippet": "..." }
  ],
  "provider": "google_cse"
}
```

The LLM chooses by index and cites the selected URL.
