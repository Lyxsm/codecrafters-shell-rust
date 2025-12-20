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

        if command.starts_with("echo") {
            let input = command.strip_prefix("echo ").unwrap().trim();
            println!("{}", input);
        } else {
            match command.trim() {
                "exit" => {
                    b = false;
                },
                cmd => {
                    println!("{cmd}: command not found");
                }
            }
        }
    }
}