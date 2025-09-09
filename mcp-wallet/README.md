# MCP Wallet Server

`mcp-wallet` is a lightweight, command-line Ethereum wallet server that is compliant with the **Rust Message-Oriented Communication Protocol (rmcp)**. It provides essential wallet functionalities, including account creation, transaction creation, and signing, all managed through a human-readable JSON file.

This server is designed to be a component in a larger system, where other tools or AI agents can interact with it programmatically to manage Ethereum wallets without needing direct access to private keys.

## Features

- **`rmcp` Compliant**: Interacts via a standardized, robust stdio protocol.
- **Account Management**: Generate new Ethereum accounts.
- **Alias System**: Assign human-readable aliases to addresses for easier reference.
- **Watch-only Accounts**: Setting an alias for an unknown address auto-creates a watch-only
  account (no private key stored).
- **Private Key Import**: Import a 32-byte secp256k1 private key (0x or raw hex). If a watch-only
  account with the same address exists, it is upgraded to a signing account.
- **EIP-1559 Transactions**: Create and sign modern, EIP-1559 compliant transactions.
- **JSON-Based Storage**: Wallet data is stored in a simple, human-readable JSON file (`~/.mcp-wallet.json` by default).

## Installation

To build the wallet server, you need to have Rust and Cargo installed. You can find installation instructions at [rust-lang.org](https://www.rust-lang.org/).

Once Rust is installed, clone the repository and build the project:

```sh
git clone <repository-url>
cd mcp-wallet
cargo build --release
```

The compiled binary will be located at `target/release/mcp-wallet`.

## Usage

To run the wallet server, simply execute the compiled binary:

```sh
cargo run
```

The server will start and listen for `rmcp` messages on standard input and send responses to standard output.

## Logging

By default, `mcp-wallet` writes logs to `./eth-partner-log.txt`. Control log levels and
filters via the `RUST_LOG` environment variable (defaults to `info` when not set).

Examples:

```bash
# default (info-level) logs written to ./eth-partner-log.txt
cargo run -p mcp-wallet -- --help

# enable debug logs for all modules
RUST_LOG=debug cargo run -p mcp-wallet -- --help

# fine-grained filters (example)
RUST_LOG=repl=debug,mcp_wallet=info cargo run -p mcp-wallet -- --help
```

### Wallet File

By default, the wallet data is stored in `~/.mcp-wallet.json`. If the file does not exist, a new one will be created automatically when the server first needs to save data.

## Interacting with the Server

The server communicates using the `rmcp` protocol. A client can interact with it by sending `rmcp` request messages and receiving response messages over stdio. The `rmcp` crate provides both server and client implementations.

## Tool Reference

The server exposes the following tools that can be called by an `rmcp` client.

### `new_account`

**Description**: Creates a new Ethereum account.

**Parameters**:
- `alias` (optional, string): A human-readable alias to assign to the new account.

**Example Request**:
```json
{"id":1,"method":"call_tool","params":{"name":"new_account","arguments":{"alias":"main_account"}}}
```

**Example Response**:
```json
{"id":1,"result":{"type":"structured","content":{"address":"0x..."}}}
```

---

### `list_accounts`

**Description**: Lists all Ethereum accounts in the wallet.

**Parameters**: None

**Example Request**:
```json
{"id":2,"method":"call_tool","params":{"name":"list_accounts","arguments":{}}}
```

**Example Response**:
```json
{"id":2,"result":{"type":"structured","content":[{"address":"0x...","aliases":["main_account"],"nonce":0,"is_signing":true}]}}
```

---

### `set_alias`

**Description**: Sets an alias for an Ethereum account. If the address is not present in the
wallet, a watch-only account is created automatically.

**Parameters**:
- `address` (string): The Ethereum address to which the alias will be assigned.
- `alias` (string): The new alias to assign.

**Example Request**:
```json
{"id":3,"method":"call_tool","params":{"name":"set_alias","arguments":{"address":"0x...","alias":"backup_account"}}}
```

**Example Response**:
```json
{"id":3,"result":{"type":"structured","content":null}}
```

---

### `resolve_alias`

**Description**: Resolves a case-insensitive alias to its Ethereum address.
Local-only; no ENS or external lookups. Returns an EIP-55 checksummed address.

**Parameters**:
- `alias` (string): The alias to resolve (case-insensitive).

**Example Request**:
```json
{"id":8,"method":"call_tool","params":{"name":"resolve_alias","arguments":{"alias":"alice"}}}
```

**Example Response**:
```json
{"id":8,"result":{"type":"structured","content":{"address":"0x..."}}}
```

On not found, returns an MCP error indicating the alias is not found.

---

### `import_private_key`

**Description**: Imports a private key to create or upgrade an account.

**Behavior**:
- Accepts `private_key` as 0x-prefixed or raw hex string with exactly 64 hex characters.
- If the derived address is not present, a new signing account is created.
- If a watch-only account with the same address exists, it is upgraded to a signing account.
- If a signing account already exists, the server returns a duplicate account error.

**Parameters**:
- `private_key` (string): 32-byte secp256k1 private key (0x or raw hex).

**Example Request**:
```json
{"id":7,"method":"call_tool","params":{"name":"import_private_key","arguments":{"private_key":"0x..."}}}
```

**Example Response**:
```json
{"id":7,"result":{"type":"structured","content":{"address":"0x..."}}}
```

---

### `create_tx`

**Description**: Creates an EIP-1559 transaction request.

**Parameters**:
- `from` (string): The address or alias of the account that will sign the transaction.
- `to` (string): The recipient's Ethereum address.
- `value` (string): The amount of ETH to send, in wei.
- `chain_id` (integer): The chain ID for the transaction (e.g., `1` for Mainnet).
- `gas` (optional, integer): The gas limit for the transaction.
- `max_fee_per_gas` (optional, string): The maximum fee per gas, in wei.
- `max_priority_fee_per_gas` (optional, string): The maximum priority fee per gas, in wei.

**Example Request**:
```json
{"id":4,"method":"call_tool","params":{"name":"create_tx","arguments":{"from":"main_account","to":"0x...","value":"1000000000000000000","chain_id":1}}}
```

**Example Response**:
```json
{"id":4,"result":{"type":"structured","content":{"chain_id":1,"to":"0x...","value":"1000000000000000000",...}}}
```

---

### `sign_tx`

**Description**: Signs a transaction with a specified account.

**Parameters**:
- `from` (string): The address or alias of the account that will sign the transaction.
- `tx_json` (object): The JSON representation of the transaction request created by `create_tx`.

**Example Request**:
```json
{"id":5,"method":"call_tool","params":{"name":"sign_tx","arguments":{"from":"main_account","tx_json":{"chain_id":1,...}}}}
```

**Example Response**:
```json
{"id":5,"result":{"type":"structured","content":{"hash":"0x...","raw_transaction":"0x...",...}}}
```

## Address Formatting and Validation

- Input addresses are parsed and validated; responses return addresses in EIP-55 checksum format.
- Private key input is minimally validated for correct hex length and non-zero value; full curve
  checks are performed by the signer library.

## Watch-only Accounts

- A watch-only account is stored without a private key and cannot sign transactions.
- They are created automatically when `set_alias` targets an unknown address.
- `list_accounts` includes an `is_signing` boolean to indicate whether a private key is present.
