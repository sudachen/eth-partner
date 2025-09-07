mod config;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use rustyline::history::DefaultHistory;
use rig::client::ProviderClient;
use rig::providers::gemini;

fn main() -> anyhow::Result<()> {
    let config = config::load()?;
    println!("Loaded config: {:?}", config);

    // Initialize the Gemini client if an API key is available.
    // The `rig` Gemini client is initialized from the environment, so we set the env var if the key is in our config.
    if let Some(api_key) = config.llm.google_api_key {
        std::env::set_var("GEMINI_API_KEY", api_key);
        let _client = gemini::Client::from_env();
        println!("Gemini client initialized.");
    } else {
        println!("Warning: GEMINI_API_KEY not found in config. LLM functionality will be disabled.");
    }

    let mut rl = Editor::<(), DefaultHistory>::new()?;
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                println!("Line: {}", line);
                if line == "/exit" {
                    break;
                } else if line == "/help" {
                    println!("Commands: /exit, /help");
                } else {
                    // TODO: Process input with the agent
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    rl.save_history("history.txt")?;

    Ok(())
}
