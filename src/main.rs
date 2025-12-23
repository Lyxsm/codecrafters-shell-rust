#[allow(unused_imports)]
use std::io::{self, Write};

mod cmd;

fn main() {
    repl();
}

fn repl() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
    
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let (cmd_type, command, args) = cmd::parse(&input);

        match cmd_type {
            cmd::Type::BuiltIn => {
                match command {
                    "echo" => println!("{args}"),
                    "exit" => break,
                    "type" => {
                        match cmd::cmd_type(args) {
                            cmd::Type::BuiltIn => println!("{} is a shell builtin", args),
                            cmd::Type::PathExec => println!("{} is {}", args, cmd::find_in_path(args).expect("not found").display()),
                            cmd::Type::Invalid => println!("{}: not found", args),
                        };
                    },
                    _ => println!("Something went wrong!"),
                }
            },
            //cmd::Type::PathExec => {
            //    println!("{:?}", cmd::find_in_path(command));
            //    match cmd::find_in_path(command) {
            //        Some(path_buf) => println!("{command} is {}", path_buf.display()),
            //        None => println!("{command}: not found"),
            //    }
            //},
            _ => println!("{command}: command not found"),
        }
    }
}