#[allow(unused_imports)]
use std::{
    io::{self, Write},
    process::{Command, Stdio},
    path::{self, PathBuf},
    fs,
    env,
};

mod cmd;
mod tests;

fn main() {
    repl();
}

fn repl() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
    
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let (cmd_type, command, args, target) = cmd::parse(&input);
        let command = command.as_str();

        //eprintln!("{:?}", args);

        match cmd_type {
            cmd::Type::BuiltIn => {
                match command {
                    "echo"  => {
                        let mut arguments = String::new();
                        for arg in args {
                            arguments.push_str(&arg);
                            arguments.push_str(" ");
                        }

                        if target.is_some() {
                            let error = cmd::print_to_file(arguments, target.unwrap());
                            match error {
                                Err(e) => println!("{}", e),
                                _ => {}
                            }
                        } else {
                            println!("{arguments}");
                        }
                    },
                    "exit"  => break,
                    "pwd"   => println!("{}", fs::canonicalize(".").expect("failed to retrieve working directory").display()),
                    "type"  => {
                        if args.is_empty() {
                            println!("type: not enough arguments");
                            continue;
                        }
                        match cmd::cmd_type(args[0].clone()) {
                            cmd::Type::BuiltIn  => println!("{} is a shell builtin", args[0]),
                            cmd::Type::PathExec => println!("{} is {}", args[0], cmd::find_in_path(&args[0]).expect("not found").display()),
                            cmd::Type::Invalid  => println!("{}: not found", args[0]),
                        };
                    },
                    "cd" => {
                        if args.is_empty() {
                            cmd::change_dir("~");
                        } else {
                            cmd::change_dir(&args[0]);
                        }
                    },
                    _ => println!("Something went wrong!"),
                }
            },
            cmd::Type::PathExec => {
                let file_path: Option<String> = target;
                match cmd::find_in_path(command) {
                    Some(_path_buf) => {
                        if let Some(ref path) = file_path {
                            Command::new(command)
                                .args(&args)
                                .stdout(Stdio::from(std::fs::File::create(path).expect("failed to create file")))
                                .spawn()
                                .expect("failed to execute")
                                .wait()
                                .expect("failed to wait");
                        } else {
                            Command::new(command)
                                .args(&args)
                                .spawn()
                                .expect("failed to execute")
                                .wait()
                                .expect("failed to wait");
                            println!();
                        }
                },
                    None => println!("{command}: not found"),
                }
            },
            _ => println!("{command}: command not found"),
        }
    }
}