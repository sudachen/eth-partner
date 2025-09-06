---
trigger: model_decision
description: Guidelines related to Rust implementation of MCP servers
globs: 
---

## Guidelines for MCP services

- use crate `rmcp` to build MCP services
- service must use async 
- service must be implemented with stdio protocol
- servicemust be CLI application and get required paremetrs in command line
