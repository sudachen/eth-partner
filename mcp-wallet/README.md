# MCP Wallet Server

`mcp-wallet` is a lightweight, command-line Ethereum wallet server that communicates over a simple, JSON-based Message-Oriented Communication Protocol (MCP) via stdio. It provides essential wallet functionalities, including account creation, transaction creation, and signing, all managed through a human-readable JSON file.

This server is designed to be a component in a larger system, where other tools can interact with it programmatically to manage Ethereum wallets without needing direct access to private keys.

## Features

- **Account Management**: Generate new Ethereum accounts and import existing ones from private keys.
- **Alias System**: Assign human-readable aliases to addresses for easier reference.
- **EIP-1559 Transactions**: Create and sign modern, EIP-1559 compliant transactions.
- **JSON-Based Storage**: Wallet data is stored in a simple, human-readable JSON file (`~/.mcp-wallet.json` by default).
- **MCP Interface**: Interact with the wallet server via a simple stdio-based JSON protocol.

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

The server will start and listen for JSON commands on standard input. It will print JSON responses to standard output.

### Wallet File

By default, the wallet data is stored in `~/.mcp-wallet.json`. If the file does not exist, a new one will be created automatically upon the first run.

## MCP Command Reference

All commands and responses are single-line JSON objects.

### `new-account`

Generates a new Ethereum account and optionally assigns an alias.

**Request:**
```json
{"command":"new-account","params":{"alias":"main_account"}}
```

**Success Response:**
```json
{"status":"success","data":{"address":"0x..."}}
```

### `list-accounts`

Lists all accounts in the wallet, along with their nonces and aliases.

**Request:**
```json
{"command":"list-accounts","params":{}}
```

**Success Response:**
```json
{"status":"success","data":[{"address":"0x...","nonce":0,"aliases":["main_account"]}]}
```

### `set-alias`

Associates a new alias with an existing address.

**Request:**
```json
{"command":"set-alias","params":{"address":"0x...","alias":"backup_account"}}
```

**Success Response:**
```json
{"status":"success","data":null}
```

### `create-tx`

Creates an EIP-1559 transaction request. Gas parameters are optional and will use defaults if not provided.

**Request:**
```json
{"command":"create-tx","params":{"from":"main_account","to":"0x...","value":"1000000000000000000","chain_id":1}}
```

**Success Response:**
```json
{"status":"success","data":{"chain_id":1,"to":"0x...","value":"1000000000000000000",...}}
```

### `sign-tx`

Signs a transaction request with a specified account.

**Request:**
```json
{"command":"sign-tx","params":{"from":"main_account","tx_json":{"chain_id":1,...}}}
```

**Success Response:**
```json
{"status":"success","data":{"raw_transaction":"0x...","hash":"0x...",...}}
```

### Error Response

If a command fails, the server will return an error response:

```json
{"status":"error","error":"Error message here"}
```
