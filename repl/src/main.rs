mod config;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use rustyline::history::DefaultHistory;
use anyhow::Result;

fn main() -> Result<()> {
    let config = config::load()?;
    println!("Loaded config: {:?}", config);

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
                let trimmed_line = line.trim();
                if trimmed_line == "/exit" {
                    break;
                } else if trimmed_line == "/help" {
                    println!("Available commands:");
                    println!("  /help - Show this help message");
                    println!("  /exit - Exit the application");
                    continue;
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
