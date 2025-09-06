//! Data structures for generating the MCP tool definition.

use serde::Serialize;
use std::collections::HashMap;

/// The root of the tool definition structure.
#[derive(Debug, Serialize)]
pub struct ToolDefinition {
    /// The main tool object.
    pub tool: Tool,
}

/// Describes the tool itself.
#[derive(Debug, Serialize)]
pub struct Tool {
    /// The name of the tool.
    pub name: String,
    /// A high-level description of what the tool does.
    pub description: String,
    /// A list of all functions (commands) the tool provides.
    pub functions: Vec<Function>,
}

/// Describes a single function (command) that the tool can execute.
#[derive(Debug, Serialize)]
pub struct Function {
    /// The name of the function.
    pub name: String,
    /// A natural language description of what the function does.
    pub description: String,
    /// The parameters that the function accepts.
    pub parameters: Parameters,
}

/// Describes the parameters for a function, following an OpenAPI-like schema.
#[derive(Debug, Serialize)]
pub struct Parameters {
    /// The type of the parameter object (always "object").
    #[serde(rename = "type")]
    pub type_: String,
    /// A map of parameter names to their definitions.
    pub properties: HashMap<String, Property>,
    /// A list of parameter names that are required.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
}

/// Describes a single parameter property.
#[derive(Debug, Serialize)]
pub struct Property {
    /// The data type of the parameter (e.g., "string", "integer", "object").
    #[serde(rename = "type")]
    pub type_: String,
    /// A natural language description of the parameter.
    pub description: String,
}

/// Generates the complete tool definition for the MCP wallet server.
pub fn generate_tool_definition() -> ToolDefinition {
    ToolDefinition {
        tool: Tool {
            name: "eth_wallet_manager".to_string(),
            description: "Manages an Ethereum wallet. Used for creating accounts, listing accounts, creating transactions, and signing transactions.".to_string(),
            functions: vec![
                new_account_definition(),
                list_accounts_definition(),
                set_alias_definition(),
                create_tx_definition(),
                sign_tx_definition(),
            ],
        },
    }
}

// --- Function Definitions ---

fn new_account_definition() -> Function {
    Function {
        name: "new-account".to_string(),
        description: "Generates a new Ethereum keypair and optionally assigns an alias to it.".to_string(),
        parameters: Parameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("alias".to_string(), Property {
                    type_: "string".to_string(),
                    description: "A human-readable alias for the new account (e.g., 'main_account').".to_string(),
                }),
            ]),
            required: vec![],
        },
    }
}

fn list_accounts_definition() -> Function {
    Function {
        name: "list-accounts".to_string(),
        description: "Lists all accounts in the wallet, along with their nonces and aliases.".to_string(),
        parameters: Parameters {
            type_: "object".to_string(),
            properties: HashMap::new(),
            required: vec![],
        },
    }
}

fn set_alias_definition() -> Function {
    Function {
        name: "set-alias".to_string(),
        description: "Associates a new alias with an existing address.".to_string(),
        parameters: Parameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("address".to_string(), Property {
                    type_: "string".to_string(),
                    description: "The Ethereum address to associate the alias with.".to_string(),
                }),
                ("alias".to_string(), Property {
                    type_: "string".to_string(),
                    description: "The new alias to assign.".to_string(),
                }),
            ]),
            required: vec!["address".to_string(), "alias".to_string()],
        },
    }
}

fn create_tx_definition() -> Function {
    Function {
        name: "create-tx".to_string(),
        description: "Creates an EIP-1559 transaction request. The transaction is not signed.".to_string(),
        parameters: Parameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("from".to_string(), Property {
                    type_: "string".to_string(),
                    description: "The address or alias of the account that will sign the transaction.".to_string(),
                }),
                ("to".to_string(), Property {
                    type_: "string".to_string(),
                    description: "The recipient's Ethereum address.".to_string(),
                }),
                ("value".to_string(), Property {
                    type_: "string".to_string(),
                    description: "The amount of ETH to send, in wei.".to_string(),
                }),
                ("chain_id".to_string(), Property {
                    type_: "integer".to_string(),
                    description: "The chain ID for the network (e.g., 1 for Mainnet, 11155111 for Sepolia).".to_string(),
                }),
                ("gas".to_string(), Property {
                    type_: "integer".to_string(),
                    description: "(Optional) The gas limit for the transaction.".to_string(),
                }),
                ("max_fee_per_gas".to_string(), Property {
                    type_: "string".to_string(),
                    description: "(Optional) The maximum fee per gas, in wei.".to_string(),
                }),
                ("max_priority_fee_per_gas".to_string(), Property {
                    type_: "string".to_string(),
                    description: "(Optional) The maximum priority fee per gas, in wei.".to_string(),
                }),
            ]),
            required: vec!["from".to_string(), "to".to_string(), "value".to_string(), "chain_id".to_string()],
        },
    }
}

fn sign_tx_definition() -> Function {
    Function {
        name: "sign-tx".to_string(),
        description: "Signs a previously created transaction request using the private key of a specified account.".to_string(),
        parameters: Parameters {
            type_: "object".to_string(),
            properties: HashMap::from([
                ("from".to_string(), Property {
                    type_: "string".to_string(),
                    description: "The address or alias of the account to sign with.".to_string(),
                }),
                ("tx_json".to_string(), Property {
                    type_: "object".to_string(),
                    description: "The full JSON object of the transaction request created by the 'create-tx' function.".to_string(),
                }),
            ]),
            required: vec!["from".to_string(), "tx_json".to_string()],
        },
    }
}
