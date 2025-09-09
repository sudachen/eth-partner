
## Relevant Files

- `repl/src/agent.rs` - Will be modified to hold the history and pass it to the LLM.
- `repl/src/main.rs` - The main REPL loop; will be modified to manage history and handle new user commands.
- `tests/repl_tests.rs` - New unit tests will be added to verify the chat memory functionality.
 - `repl/src/lib.rs` - Current location of `run_repl`, `handle_line`, and the new `handle_command` accepting mutable history.

### Notes

- Unit tests should be added to `repl/tests/repl_tests.rs` to cover the new command handling and history management logic.
- Run tests from the workspace root using `cargo test`.

## Tasks

- [x] 1.0 Define data structures for managing chat history.
  - [x] 1.1 In `repl/src/agent.rs`, define a public struct `ChatMessage` that can be serialized, with `role` (String) and `content` (String) fields.
  - [x] 1.2 In `repl/src/agent.rs`, add a `history: Vec<ChatMessage>` field to the `ReplAgent` struct.

- [ ] 2.0 Update the main REPL loop to store the conversation history.
  - [x] 2.1 In `main.rs`, inside `run_repl`, pass a mutable reference of the agent's history to the loop.
  - [x] 2.2 In `main.rs`, after getting a line from the user, add the user's prompt to the history as a `ChatMessage`.
  - [x] 2.3 In `main.rs`, after getting a response from the agent, add the assistant's response to the history.

- [ ] 3.0 Implement the `/show_history` and `/clear_history` user commands.
  - [x] 3.1 In `main.rs`, modify the `handle_command` function to accept the history as a mutable argument.
    - Note: Implemented in `repl/src/lib.rs` as `handle_command(&str, &mut Vec<ChatMessage>)` to match the current architecture.
  - [x] 3.2 In `handle_command`, add a match arm for `/show_history` that iterates through the history and prints it to the console.
  - [x] 3.3 In `handle_command`, add a match arm for `/clear_history` that clears the history vector.
  - [x] 3.4 In `run_repl`, update the call to `handle_command` to pass the history.

- [ ] 4.0 Modify the `ReplAgent` to use the history when prompting the LLM.
  - [x] 4.1 In `agent.rs`, modify the `ReplAgent::run` method to prepend the existing history to the new user prompt before sending it to the model.
  - [x] 4.2 In `main.rs`, ensure the `agent.run()` call is updated if its signature changed.

## Associated PRD

Use @tasks/prd-repl-chat-memory.md
