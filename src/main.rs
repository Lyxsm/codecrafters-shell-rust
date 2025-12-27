#![allow(unused_imports)]
use std::{
    io::{self, Write},
    process::{Command, Stdio},
    path::{self, PathBuf},
    fs, env,
};
#[allow(unused_imports)]
use crossterm::{
    ExecutableCommand, cursor, terminal, execute,
    event::{self, KeyEvent, KeyCode, KeyModifiers},
};

mod cmd;
mod tests;

fn main() {
    terminal::enable_raw_mode().unwrap();
    loop {
        terminal::disable_raw_mode().unwrap();
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();

        let mut event_handled = false;
        terminal::enable_raw_mode().unwrap();

        'inner: while !event_handled {
            if event::poll(std::time::Duration::from_millis(100)).unwrap() {
                if let event::Event::Key(KeyEvent { code, modifiers, .. }) = event::read().unwrap() {
                    match code {
                        KeyCode::Esc => break,
                        KeyCode::Tab => {
                            let matches: Vec<String> = cmd::BUILT_IN 
                                .iter()
                                .filter(|&cmd| cmd.starts_with(&input))
                                .map(|&cmd| cmd.to_string())
                                .collect();

                            if matches.len() == 1 {
                                for _ch in input.chars() {
                                    print!("\x08 \x08");
                                    io::stdout().flush().unwrap();
                                }
                                input.pop();
                                input = matches[0].clone();
                                input += " ";
                                print!("{}", input);
                                io::stdout().flush().unwrap();
                            }
                        },
                        KeyCode::Enter => {
                            if input.trim().is_empty() {
                                //io::stdout().flush().unwrap();
                                terminal::disable_raw_mode().unwrap();
                                println!();
                                break 'inner;
                            } else if input.trim() == "exit" {
                                //io::stdout().flush().unwrap();
                                terminal::disable_raw_mode().unwrap();
                                println!();
                                return;
                            } else {
                                terminal::disable_raw_mode().unwrap();
                                println!();
                                //println!("{}\n", input);
                                //io::stdout().flush().unwrap();
                                execute_cmd(input.clone());
                                //io::stdout().flush().unwrap();
                                terminal::enable_raw_mode().unwrap();
                            }
                            input.clear();
                            //io::stdout().flush().unwrap();
                            terminal::disable_raw_mode().unwrap();
                            event_handled = true;
                        },
                        KeyCode::Backspace => {
                            if !input.is_empty() {
                                input.pop();
                                print!("\x08 \x08");
                                io::stdout().flush().unwrap();  
                            }
                        },
                        KeyCode::Char(c) => {
                            if modifiers == KeyModifiers::CONTROL && c == 'j' {
                                if input.trim().is_empty() {
                                    //io::stdout().flush().unwrap();
                                    //terminal::disable_raw_mode().unwrap();
                                    println!();
                                    break 'inner;
                                } else if input.trim() == "exit" {
                                    //io::stdout().flush().unwrap();
                                    //terminal::disable_raw_mode().unwrap();
                                    println!();
                                    return;
                                } else {
                                    terminal::disable_raw_mode().unwrap();
                                    println!();
                                    //println!("{}\n", input);
                                    //io::stdout().flush().unwrap();
                                    execute_cmd(input.clone());
                                    //io::stdout().flush().unwrap();
                                    terminal::enable_raw_mode().unwrap();
                                }
                                input.clear();
                                //io::stdout().flush().unwrap();
                                //terminal::disable_raw_mode().unwrap();
                                event_handled = true;
                            } else {
                                input.push(c);
                                print!("{}", c);
                                io::stdout().flush().unwrap();
                            }
                        },
                        KeyCode::Up => {

                        },
                        _ => {},
                    }
                }
            }
            //io::stdout().flush().unwrap();
            //terminal::disable_raw_mode().unwrap();
        }
        //terminal::disable_raw_mode().unwrap();
    }
    //terminal::disable_raw_mode().unwrap();
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
                        let error = cmd::print_to_file_built_in(arguments, &target.0, target.1);
                        match error {
                            Err(e) => println!("{e}"),
                            _ => {},
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
                        let _error = cmd::print_to_file(command, args, path, target_type);
                        //match error {
                        //    Ok(_) => {},
                        //    Err(e) => println!("{:?}", e),
                        //}
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