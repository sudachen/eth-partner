# Product Requirements Document: MCP Wallet Server

## 1. Introduction/Overview
The MCP Wallet Server is a command-line utility that provides basic wallet functionality for Ethereum networks. It allows users to manage Ethereum accounts, create and sign transactions, and associate addresses with human-readable aliases. The wallet stores its data in a simple JSON file for easy inspection and modification.

## 2. Goals
1. Provide a simple way to generate and manage Ethereum keypairs
2. Allow creating and signing Ethereum transactions
3. Support address aliasing for better usability
4. Work with both mainnet and devnet
5. Store wallet data in an easily editable JSON format

## 3. User Stories
1. As a user, I want to generate a new Ethereum keypair so that I can receive funds
2. As a user, I want to associate an alias with an address so that I can reference it more easily
3. As a user, I want to create a transaction to send ETH to another address
4. As a user, I want to sign a transaction with a specific account (by address or alias)
5. As a user, I want to view my account balance and transaction history

## 4. Functional Requirements

### 4.1 Account Management
1. The wallet must be able to generate new Ethereum keypairs
2. Private and public keys must be stored as hex strings in the wallet file
3. The wallet must store the nonce for each account

### 4.2 Alias System
1. Users can associate an alias (1-20 alphanumeric chars) with any address
2. Each alias must be unique (one-to-one mapping to addresses)
3. The same address can have multiple aliases

### 4.3 Transaction Handling
1. Support creating raw Ethereum transactions (type 2 - EIP-1559)
2. Support signing transactions with specified account (by address or alias)
3. Support both mainnet and devnet
4. Allow setting gas price, gas limit, and nonce
5. Support creating transactions with zero values for later population

### 4.4 Data Storage
1. Store all wallet data in a JSON file (default: wallet.json in current directory)
2. Support specifying an alternative file path via CLI argument
3. File format must be human-readable and easily editable

### 4.5 MCP Interface
1. Expose functionality through stdio-based API
2. Support the following commands:
   - `new-account`: Generate a new keypair
   - `set-alias <address> <alias>`: Associate an alias with an address
   - `get-address <alias>`: Get address by alias
   - `create-tx <from> <to> <value> [--nonce <n>] [--gas <g>] [--gas-price <gp>]`: Create a new transaction
   - `sign-tx <tx-json> <from>`: Sign a transaction
   - `list-accounts`: List all accounts and their aliases

## 5. Non-Goals (Out of Scope)
1. Hardware wallet integration
2. Support for ERC-20 tokens (future enhancement)
3. Transaction broadcasting (will be handled by external tools)
4. Account balance tracking
5. Transaction history
6. Network connectivity
7. Security features (encryption, password protection)

## 6. Technical Considerations
1. Use Rust for implementation
2. Use `ethers-rs` for Ethereum-specific functionality
3. Store wallet data in a simple JSON structure:
   ```json
   {
     "accounts": {
       "0x123...": {
         "private_key": "0x...",
         "public_key": "0x...",
         "nonce": 0,
         "aliases": ["main", "backup"]
       }
     },
     "aliases": {
       "main": "0x123...",
       "backup": "0x123..."
     }
   }
   ```

## 7. Success Metrics
1. Successfully creates and manages Ethereum keypairs
2. Correctly creates and signs transactions that can be broadcast to the network
3. Maintains data consistency between aliases and addresses
4. Handles concurrent access to the wallet file (if implemented)

## 8. Open Questions
1. Should we implement any form of file locking for the wallet file?
2. Should we add any validation for transaction parameters?
3. Should we include any form of checksum validation for addresses?
