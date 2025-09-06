# PRD: Ethereum RPC Client Integration for MCP-Wallet

## 1. Introduction/Overview

This document outlines the requirements for integrating an Ethereum RPC client into the `mcp-wallet`. The goal is to enable the wallet to interact directly with the Ethereum blockchain, providing core functionalities like checking balances, getting block information, and sending transactions. All new functionality will be exposed through MCP (Model Context Protocol) tools.

## 2. Goals

*   Integrate a robust Ethereum client to connect to an RPC endpoint.
*   Expose essential Ethereum functionalities as MCP tools.
*   Enable transaction signing using locally stored private keys.
*   Provide users with the ability to send ETH and monitor transaction status.

## 3. User Stories

*   **As a user**, I want to get the current block number to confirm the wallet is connected to the network and synced.
*   **As a user**, I want to check the ETH balance of any Ethereum address to know its funds.
*   **As a user**, I want to send a signed transaction and get the transaction hash immediately for tracking purposes.
*   **As a user**, I want to get information about a previously sent transaction using its hash.
*   **As a user**, I want a simple way to transfer ETH from my wallet to another address, with gas fees handled automatically.

## 4. Functional Requirements

1.  **RPC Connection:** The wallet must connect to an Ethereum RPC server. The RPC URL should be configurable, with a default value of `http://127.0.0.1:8545`.
2.  **Private Key Storage:** Private keys must be stored as plain hex strings in a local wallet storage file. The exact format and location of this file will be determined during implementation (e.g., a simple JSON file).
3.  **MCP Tool: `eth_getCurrentBlock`**
    *   **Input:** None.
    *   **Action:** Fetches the latest block number from the connected Ethereum node.
    *   **Output:** The current block number (integer).
4.  **MCP Tool: `eth_getBalance`**
    *   **Input:** `address` (string, Ethereum address).
    *   **Action:** Fetches the ETH balance for the given address.
    *   **Output:** The balance in Ether (string or float).
5.  **MCP Tool: `eth_sendSignedTransaction`**
    *   **Input:** `signed_transaction_hex` (string).
    *   **Action:** Broadcasts a pre-signed, hex-encoded transaction to the network.
    *   **Output:** The transaction hash (string).
6.  **MCP Tool: `eth_getTransactionInfo`**
    *   **Input:** `transaction_hash` (string).
    *   **Action:** Fetches the details of a transaction by its hash.
    *   **Output:** Transaction details, including status (e.g., pending, success, failed), block number, gas used, etc.
7.  **MCP Tool: `eth_transferEth`**
    *   **Input:** `to_address` (string), `amount_eth` (float).
    *   **Action:** Creates a transaction, automatically estimates the gas fee, signs it with the stored private key, and sends it.
    *   **Output:** The transaction hash (string).

## 5. Non-Goals (Out of Scope)

*   Management of address aliases.
*   Support for any token standards (e.g., ERC-20, ERC-721). Only ETH transfers are supported.
*   Advanced wallet features like HD wallet derivation, multi-sig, or contract interaction beyond simple transfers.
*   A user interface for managing keys or wallets; interaction is purely through MCP tools.

## 6. Technical Considerations

*   **Library:** The implementation will use the `ethers-rs` crate for all Ethereum-related functionality.
*   **Modularity:** New logic for the RPC client and tools should be organized into appropriate modules within the `mcp-wallet/src` directory.
*   **Error Handling:** The system must handle potential errors gracefully, such as RPC connection failures, invalid private keys, insufficient funds, and transaction failures.
*   **Async:** All network operations must be asynchronous to avoid blocking.

## 7. Success Metrics

*   All specified MCP tools are implemented and available through the wallet's service layer.
*   A user can successfully query the blockchain for block numbers and balances.
*   A user can successfully send an ETH transaction from the wallet to another address and verify its status.

## 8. Open Questions

*   What is the exact format and location of the wallet storage file for the private key?
