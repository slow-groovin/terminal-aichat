use std::io::{self, Write};

pub async fn interactive_input() -> Result<String, Box<dyn std::error::Error>> {
    println!("Enter your message (Enter empty lines to send):");

    let mut lines = Vec::new();

    loop {
        io::stdout().flush()?;

        let mut line = String::new();
        io::stdin().read_line(&mut line)?;

        let line = line.trim().to_string();

        // Handle Ctrl+C
        if line.starts_with('\x03') {
            println!("\nCancelled.");
            std::process::exit(0);
        }

        if line.trim().is_empty() {
            break;
        } else {
            lines.push(line);
        }
    }

    Ok(lines.join("\n"))
}
