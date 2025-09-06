# Task List: Ethereum RPC Client Integration

This task list is based on the `prd-eth-rpc-client.md` document.

## Relevant Files

*   `mcp-wallet/src/eth_client.rs` - New module for Ethereum RPC client logic.
*   `mcp-wallet/src/service/mod.rs` - To integrate the new MCP tools.
*   `mcp-wallet/Cargo.toml` - To add new dependencies like `ethers-rs`.
*   `mcp-wallet/src/main.rs` - To handle the optional RPC URL argument.
*   `mcp-wallet/src/wallet_storage.rs` - New module for managing the wallet file.

---

### Task 1: Project Setup & Dependencies

- [x] **Sub-task 1.1:** Add `ethers-rs` and other required dependencies (`serde`, `tokio`, `thiserror`) to `mcp-wallet/Cargo.toml`.
- [x] **Sub-task 1.2:** Create a new module `mcp-wallet/src/eth_client.rs` to encapsulate all Ethereum-related logic.
- [x] **Sub-task 1.3:** Create a new module `mcp-wallet/src/wallet_storage.rs` for loading and saving the private key.

### Task 2: Implement RPC Client & Basic Tools

- [x] **Sub-task 2.1:** Implement the basic structure of the `EthClient` in `eth_client.rs`, including a constructor that takes an RPC URL.
- [x] **Sub-task 2.2:** Add a `get_current_block` method to `EthClient`.
- [x] **Sub-task 2.3:** Add a `get_balance` method to `EthClient`.
- [x] **Sub-task 2.4:** Update `main.rs` to accept an optional `--rpc-url` command-line argument, using `http://127.0.0.1:8545` as the default.

### Task 3: Implement Wallet Storage

- [ ] **Sub-task 3.1:** In `wallet_storage.rs`, implement a function to load a private key from a simple JSON file (e.g., `~/.mcp-wallet/key.json`).
- [ ] **Sub-task 3.2:** The wallet should be initialized with the private key upon startup.

### Task 4: Implement Transaction-related Tools

- [ ] **Sub-task 4.1:** Add a `send_signed_transaction` method to `EthClient`.
- [ ] **Sub-task 4.2:** Add a `get_transaction_info` method to `EthClient`.
- [ ] **Sub-task 4.3:** Implement the `transfer_eth` method in `EthClient`. This method will:
    - Create a transaction request.
    - Estimate the gas fee.
    - Sign the transaction using the loaded private key.
    - Send the transaction using the `send_signed_transaction` method.

### Task 5: Integrate into MCP Service

- [ ] **Sub-task 5.1:** In `mcp-wallet/src/service/mod.rs`, integrate the `EthClient`.
- [ ] **Sub-task 5.2:** Expose the `eth_getCurrentBlock` functionality as an MCP tool.
- [ ] **Sub-task 5.3:** Expose the `eth_getBalance` functionality as an MCP tool.
- [ ] **Sub-task 5.4:** Expose the `eth_sendSignedTransaction` functionality as an MCP tool.
- [ ] **Sub-task 5.5:** Expose the `eth_getTransactionInfo` functionality as an MCP tool.
- [ ] **Sub-task 5.6:** Expose the `eth_transferEth` functionality as an MCP tool.

### Task 6: Testing and Finalization

- [ ] **Sub-task 6.1:** Add unit tests for the new functionality.
- [ ] **Sub-task 6.2:** Manually test the MCP tools against a local testnet (e.g., Anvil).
- [ ] **Sub-task 6.3:** Update `README.md` to document the new Ethereum tools and the `--rpc-url` argument.
