#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    repl();
}

fn repl() {
    let mut b = true;
    while b {
        // Prints "$" to terminal
        print!("$ ");
        io::stdout().flush().unwrap();
    
        // Reads user input and stores it in the "command" variable
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();

        command = command.replace("\n", "").replace("\r", "").replace("\r\n", "");

        match command.as_str() {
            "exit" => {
                b = false;
            },
            _ => {
                println!("{}: command not found", command);
            }
        }
    }
}