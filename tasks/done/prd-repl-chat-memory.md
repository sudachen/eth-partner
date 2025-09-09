
# PRD: REPL Chat Memory

## 1. Introduction/Overview

This document outlines the requirements for implementing an in-session chat memory for the REPL AI assistant. The goal is to enable the LLM to maintain conversational context within a single user session. This feature will also include user-facing commands to give the user control over the conversation history.

## 2. Goals

- Enable the AI assistant to remember previous interactions within the same session.
- Provide users with explicit control over the conversation history, including viewing and clearing it.
- Improve the user experience by allowing for more natural, stateful conversations.

## 3. User Stories

- **As a user,** I want the assistant to remember context from earlier in the session so I don't have to repeat myself.
- **As a user,** I want to be able to clear the conversation history with a command to start a new topic without restarting the application.
- **As a user,** I want to be able to view the history to remind myself of what has been discussed in the current session.

## 4. Functional Requirements

1.  The REPL application must maintain an in-memory history of the conversation, including user prompts and AI responses.
2.  The history must be cleared automatically when the REPL application is terminated.
3.  The application must provide a `/clear_history` command that erases the current in-session history.
4.  The application must provide a `/show_history` command that displays the conversation history to the user in a readable format.
5.  For each new user prompt, the entire existing in-memory history must be sent to the LLM along with the new prompt.

## 5. Non-Goals (Out of Scope)

- Persisting the chat history to a file or any other storage medium.
- Loading chat history from previous sessions.
- **Advanced context management.** For this initial version, a summarization technique or a sliding window is not required. The entire conversation history will be sent with each request.

## 6. Design Considerations (Optional)

- The output format for the `/show_history` command needs to be defined. It should be simple and human-readable, for example:
  ```
  > User: [First prompt]
  < Assistant: [First response]
  > User: [Second prompt]
  < Assistant: [Second response]
  ```

## 7. Technical Considerations (Optional)

- The implementation will require adding a command-parsing mechanism to the REPL to handle `/clear_history` and `/show_history`.
- The main interaction loop in the `repl` crate will need to be modified to store and pass the history.
- A simple vector or list structure (e.g., `Vec<ChatMessage>`) should be sufficient to hold the conversation turns in memory.

## 8. Success Metrics

- The assistant can successfully answer questions that rely on context from previous turns in the conversation.
- Executing the `/clear_history` command makes the assistant forget all previous context from the session.
- Executing the `/show_history` command correctly prints the list of prompts and responses from the current session.

## 9. Open Questions

- Should we consider adding a warning to the user if the conversation history grows large, as it may approach LLM context limits and increase costs?
