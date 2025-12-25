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

        if input.split_whitespace().next() == Some("exit") {
            break;
        } else {
            execute_cmd(input);
        }
    }
}


fn execute_cmd(input: String) {
    let (cmd_type, command, args, target) = cmd::parse(&input);
    let command = command.as_str();
    match cmd_type {
        cmd::Type::BuiltIn => {
            match command {
                "echo"  => {
                    let mut arguments = String::new();
                    for arg in &args {
                        arguments.push_str(&arg);
                        arguments.push_str(" ");
                    }
                    if target.is_some() {
                        let target = target.unwrap();
                        if target.1 == cmd::Target::Stdout {
                            let error = cmd::print_to_file(arguments, target.0);
                            match error {
                                Err(e) => println!("{}", e),
                                _ => {}
                            }
                        } else if target.1 == cmd::Target::Stderr {
                            Command::new(command)
                                .args(&args)
                                .stderr(Stdio::from(std::fs::File::create(target.0).expect("failed to create file")))
                                .spawn()
                                .expect("failed to execute")
                                .wait()
                                .expect("failed to wait");
                        }
                    } else {
                        println!("{arguments}");
                    }
                },
                "pwd"   => println!("{}", fs::canonicalize(".").expect("failed to retrieve working directory").display()),
                "type"  => {
                    if args.is_empty() {
                        println!("type: not enough arguments");
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
            let mut file_path: Option<String> = None;
            let mut target_type: cmd::Target = cmd::Target::None;
            if target.is_some() {
                let file: String;
                (file, target_type) = target.unwrap();
                file_path = Some(file);
            }
            match cmd::find_in_path(command) {
                Some(_path_buf) => {
                    if let Some(ref path) = file_path {
                        if target_type == cmd::Target::Stdout {
                            Command::new(command)
                                .args(&args)
                                .stdout(Stdio::from(std::fs::File::create(path).expect("failed to create file")))
                                .spawn()
                                .expect("failed to execute")
                                .wait()
                                .expect("failed to wait");
                            } else if target_type == cmd::Target::Stderr {
                                Command::new(command)
                                    .args(&args)
                                    .stderr(Stdio::from(std::fs::File::create(path).expect("failed to create file")))
                                    .spawn()
                                    .expect("failed to execute")
                                    .wait()
                                    .expect("failed to wait");
                            }
                    } else {
                        Command::new(command)
                            .args(&args)
                            .spawn()
                            .expect("failed to execute")
                            .wait()
                            .expect("failed to wait");
                    }
            },
                None => println!("{command}: not found"),
            }
        },
        _ => println!("{command}: command not found"),
    }
}