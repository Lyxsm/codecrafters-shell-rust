#[allow(unused_imports)]
use std::{
    io::{self, Write},
    process::Command,
    path::{self, PathBuf},
    fs,
};

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
        let args = args.as_str();

        match cmd_type {
            cmd::Type::BuiltIn => {
                match command {
                    "echo"  => println!("{args}"),
                    "exit"  => break,
                    "pwd"   => println!("{}", fs::canonicalize(".").expect("failed to retrieve working directory").display()),
                    "type"  => {
                        match cmd::cmd_type(args) {
                            cmd::Type::BuiltIn  => println!("{} is a shell builtin", args),
                            cmd::Type::PathExec => println!("{} is {}", args, cmd::find_in_path(args).expect("not found").display()),
                            cmd::Type::Invalid  => println!("{}: not found", args),
                        };
                    },
                    "cd" => cmd::change_dir(args),
                    _ => println!("Something went wrong!"),
                }
            },
            cmd::Type::PathExec => {
                match cmd::find_in_path(command) {
                    Some(_path_buf) => {
                        let mut arguments = Vec::new();
                        for arg in args.split_whitespace() {
                            arguments.push(arg);
                        }
                        Command::new(command)
                            .args(&arguments)
                            .spawn()
                            .expect("failed to execute")
                            .wait()
                            .expect("failed to wait");
                },
                    None => println!("{command}: not found"),
                }
            },
            _ => println!("{command}: command not found"),
        }
    }
}