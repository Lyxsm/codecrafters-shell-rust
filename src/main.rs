#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // Prints "$" to terminal
    print!("$ ");
    io::stdout().flush().unwrap();

    // Reads user input and stores it in the "command" variable
    let mut command = String::new();
    io::stdin().read_line(&mut command).unwrap();

    match command {
        _ => {
            println!("{}: command not found", command.replace("\n", "").replace("\r", "").replace("\r\n", ""));
        }
    }
}
