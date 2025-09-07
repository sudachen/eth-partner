# REPL AI Assistant

This crate provides a simple REPL (Read-Eval-Print Loop) AI assistant that uses the `rig` framework to interact with large language models and external tools.

## Features

- Interactive REPL interface.
- Extensible toolset for interacting with external services.
- Configuration managed through a simple JSON file.
- Powered by Google's Gemini Pro via the `rig` framework.

## Configuration

Before running the application, you need to create a configuration file at `~/.config/eth-partner/config.json`. This file should contain the necessary API keys for the services you want to use.

### Example Configuration

```json
{
    "llm": {
        "google_api_key": "YOUR_GEMINI_API_KEY"
    },
    "tools": {
        "brave_api_key": "YOUR_BRAVE_SEARCH_API_KEY"
    }
}
```

### Obtaining API Keys

- **Gemini API Key**: You can obtain a Gemini API key from [Google AI Studio](https://aistudio.google.com/app/apikey).
- **Brave Search API Key**: You can obtain a Brave Search API key from the [Brave Search API website](https://brave.com/search/api/).

## Running the Application

To run the REPL assistant, navigate to the root of the workspace and use the following command:

```bash
cargo run -p repl
```

This will start the interactive REPL, where you can enter commands or prompts for the AI assistant.

### Commands

- `/help`: Displays a list of available commands.
- `/exit`: Exits the application.
