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

## Integration with AI Agentic Systems

The `mcp-wallet` server is designed to be used as a tool by AI agentic systems (like Google's Gemini, Anthropic's Claude, or others with MCP support) that can execute shell commands and interact with subprocesses. By running the wallet as a persistent stdio server, an AI agent can manage a user's wallet without ever having direct access to the private keys.

### How it Works

1.  **Tool Definition**: The AI agent's system administrator defines a new tool that knows how to start and communicate with the `mcp-wallet` server.
2.  **Start the Server**: When the agent needs to perform a wallet operation, it starts the `mcp-wallet` binary as a background process.
3.  **Send Commands**: The agent sends JSON-formatted commands (as defined in the MCP Command Reference below) to the server's standard input (`stdin`).
4.  **Receive Responses**: The agent listens to the server's standard output (`stdout`) to receive JSON-formatted responses.

### Example Tool Configuration (Conceptual)

Here is a conceptual example of how you might define a tool for an AI agent:

**Tool Name**: `eth_wallet`

**Invocation**:
The tool would start the server and keep the process handle to interact with its `stdin` and `stdout`.

```python
# Conceptual Python code for an agent's tool
import subprocess
import json

class EthWalletTool:
    def __init__(self, wallet_path='target/release/mcp-wallet'):
        self.process = subprocess.Popen(
            [wallet_path],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )

    def execute_command(self, command, params):
        request = json.dumps({'command': command, 'params': params})
        self.process.stdin.write(request + '\n')
        self.process.stdin.flush()
        
        response_line = self.process.stdout.readline()
        return json.loads(response_line)

    def close(self):
        self.process.terminate()

# Usage by the agent
wallet_tool = EthWalletTool()
response = wallet_tool.execute_command('list-accounts', {})
print(response)
wallet_tool.close()
```

This setup allows an AI agent to securely manage an Ethereum wallet by delegating all cryptographic operations to the `mcp-wallet` server, which acts as a trusted execution environment.

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
