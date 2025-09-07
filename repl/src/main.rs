use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

fn main() -> rustyline::Result<()> {
    let mut rl = DefaultEditor::new()?;
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let trimmed_line = line.trim();
                if trimmed_line == "/exit" {
                    break;
                } else if trimmed_line == "/help" {
                    println!("Available commands:");
                    println!("  /help - Show this help message");
                    println!("  /exit - Exit the application");
                    continue;
                }
                println!("Line: {}", line);
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
    Ok(())
}
