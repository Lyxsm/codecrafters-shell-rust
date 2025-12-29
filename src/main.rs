#![allow(unused_imports)]
use std::{
    io::{self, Write},
    process::{Command, Stdio},
    path::{self, PathBuf},
    fs, env, slice,
    collections::HashMap,
};
#[allow(unused_imports)]
use crossterm::{
    ExecutableCommand, cursor, terminal, execute,
    event::{self, KeyEvent, KeyCode, KeyModifiers},
};

mod cmd;
mod tests;

fn main() {
    loop {
        let input = String::new();
        if !active(input) {
            return;
        }
    }
}

fn active(input: String) -> bool {
    print!("$ ");
    io::stdout().flush().unwrap();
    terminal::enable_raw_mode().unwrap();
    let mut input = input;
    let mut event_handled = false;

    'inner: while !event_handled {
        if event::poll(std::time::Duration::from_millis(100)).unwrap() {
            if let event::Event::Key(KeyEvent { code, modifiers, .. }) = event::read().unwrap() {
                match code {
                    KeyCode::Esc => break,
                    KeyCode::Tab => {
                        terminal::enable_raw_mode().unwrap();
                        io::stdout().flush().unwrap();
                        let mut matches = cmd::get_matches(&input);
                        matches.sort();
                        if matches.len() > 1 {
                            print!("\x07");
                            io::stdout().flush().unwrap();
                        }

                        let (string, event, stay_active) = auto_complete(input.clone(), matches.clone());
                        event_handled = event;
                        io::stdout().flush().unwrap();
                        if !stay_active {
                            return false;
                        }
                        terminal::enable_raw_mode().unwrap();
                        io::stdout().flush().unwrap();
                        input = string;
                    },
                    KeyCode::Enter => {
                        if input.trim().is_empty() {
                            //io::stdout().flush().unwrap();
                            terminal::disable_raw_mode().unwrap();
                            println!();
                            io::stdout().flush().unwrap();
                            event_handled = true;
                        } else if input.trim() == "exit" {
                            terminal::disable_raw_mode().unwrap();
                            println!();
                            io::stdout().flush().unwrap();
                            return false;
                        } else {
                            terminal::disable_raw_mode().unwrap();
                            println!();
                            io::stdout().flush().unwrap();
                            execute_cmd(input.clone());
                            io::stdout().flush().unwrap();
                            input.clear();
                            event_handled = true;
                        }
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
                                io::stdout().flush().unwrap();
                                terminal::disable_raw_mode().unwrap();
                                println!();
                                break 'inner;
                            } else if input.trim() == "exit" {
                                terminal::disable_raw_mode().unwrap();
                                println!();
                                io::stdout().flush().unwrap();
                                return false;
                            } else {
                                terminal::disable_raw_mode().unwrap();
                                println!();
                                io::stdout().flush().unwrap();
                                execute_cmd(input.clone());
                                io::stdout().flush().unwrap();
                                input.clear();
                                io::stdout().flush().unwrap();
                                event_handled = true;
                            }
                        } else if modifiers == KeyModifiers::CONTROL && c == 'w' {
                            if !input.is_empty() {                               
                                while input.chars().last().unwrap().is_whitespace() {
                                    print!("\x08 \x08");
                                    io::stdout().flush().unwrap();
                                    input.pop();
                                }
                                let temp = input.split_whitespace().last().unwrap().to_string();
                                terminal::disable_raw_mode().unwrap();
                                for _i in 0..temp.len() {
                                    print!("\x08 \x08");
                                    io::stdout().flush().unwrap();
                                    input.pop();
                                }
                                
                                io::stdout().flush().unwrap();
                                terminal::enable_raw_mode().unwrap();
                            } else {
                                print!("\x07");
                                io::stdout().flush().unwrap();
                            }
                        } else {
                            input.push(c);
                            print!("{}", c);
                            io::stdout().flush().unwrap();
                        }
                    },
                    KeyCode::Up => {
                    },
                    _ => {

                    },
                }
            }
        }            
        //io::stdout().flush().unwrap();
        //terminal::disable_raw_mode().unwrap();
    }
    return true;
}

fn execute_cmd(input: String) {
    let (cmd_type, command, args, target, pipe) = cmd::parse(&input);
    let command = command.as_str();
    //println!("command: {:?}\targs: {:?}\ttarget: {:?}", command, args, target);
    match cmd_type {
        cmd::Type::BuiltIn => {
            match command {
                "echo"  => {
                    let arguments: String = args.join(" ");
                    if let Some(pipe_info) = pipe {
                        let (_piped_cmd_type, piped_cmd, piped_args, _piped_target) = pipe_info;
                        let output = format!("{}", arguments);
                        
                        if let Some(target) = target {
                            cmd::print_to_file_built_in(output.clone(), &target.0, target.1).unwrap_or_else(|e| println!("{e}"));
                        } else {
                            println!("{}", output);
                        }

                        let mut output_cmd = Command::new(piped_cmd)
                            .args(&piped_args)
                            .stdin(Stdio::piped())
                            .spawn()
                            .expect("Failed to start piped command");

                        if let Some(stdin) = output_cmd.stdin.as_mut() {
                            stdin.write_all(output.as_bytes()).expect("Failed to write to piped command");
                        }

                        output_cmd.wait().expect("Piped command was not running");
                    } else {
                        if target.is_some() {
                            let target = target.unwrap();
                            cmd::print_to_file_built_in(arguments, &target.0, target.1).expect("Failed to print to file");
                        } else {
                            println!("{}", arguments);
                        }
                    }
                },
                "pwd"   => {
                    let current_dir = fs::canonicalize(".").expect("failed to retrieve working directory");
                    let output = format!("{}", current_dir.display());
                    if let Some(pipe_info) = pipe {
                        let (_piped_cmd_type, piped_cmd, piped_args, _piped_target) = pipe_info;

                        let mut output_cmd = Command::new(piped_cmd)
                            .args(&piped_args)
                            .stdin(Stdio::piped())
                            .spawn()
                            .expect("Failed to start piped command");

                        if let Some(stdin) = output_cmd.stdin.as_mut() {
                            stdin.write_all(output.as_bytes()).expect("Failed to write to piped command");
                        }

                        output_cmd.wait().expect("Piped command was not running");
                    } else {
                        println!("{}", current_dir.display());
                    }
                },
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
            if let Some((file, t)) = target {
                file_path = Some(file);
                target_type = t;
            }
            match cmd::find_in_path(command) {
                Some(_path_buf) => {
                    if let Some(ref path) = file_path {
                        let _= cmd::print_to_file(command, args, path, target_type);
                        //match error {
                        //    Ok(_) => {},
                        //    Err(e) => println!("{:?}", e),
                        //}
                    } else { 
                        if let Some(pipe_info) = pipe {
                            let (_piped_cmd_type, piped_cmd, piped_args, _piped_target) = pipe_info;

                            let mut child = Command::new(command)
                                .args(&args)
                                .stdout(Stdio::piped())
                                .spawn()
                                .expect("Failed to execute command");

                            let mut output = Command::new(piped_cmd) 
                                .args(&piped_args)
                                .stdin(child.stdout.take().expect("Failed to fetch stdout"))
                                .spawn()
                                .expect("Failed to execute piped command");

                            child.wait().expect("Command was not running");
                            output.wait().expect("Piped command was not running");

                        } else {
                            Command::new(command)
                                .args(&args)
                                .spawn()
                                .expect("failed to execute")
                                .wait()
                                .expect("failed to wait");
                        }
                    }
            },
                None => println!("{command}: not found"),
            }
        },
        _ => println!("{command}: command not found"),
    }
}

fn auto_complete(mut input: String, matches: Vec<String>) -> (String, bool, bool) {

    terminal::disable_raw_mode().unwrap();
    if matches.is_empty() {
        print!("\x07");
        io::stdout().flush().unwrap();
        terminal::enable_raw_mode().unwrap();
        return (input, false, true);
    } else if matches.len() == 1 {
        for _i in 0..input.len() {
            print!("\x08 \x08");
        }
        io::stdout().flush().unwrap();
        input = matches[0].clone();
        input += " ";
        print!("{}", input);
        io::stdout().flush().unwrap();
        terminal::enable_raw_mode().unwrap();
        return (input, false, true);
    } else if matches.len() > 1 {
        let common = longest_common_prefix(&matches);
        //println!("\n{:?}", matches);
        if common.len() >= 1 {
            let mut common_temp = common.clone();
            let temp = common.join("_");
            let k = input.len();
            if k < temp.len() {
                input = temp.clone();
            }
            common_temp.remove(0);
            if common_temp.is_empty() {
                let temp = matches.join("  ");
                print!("\n{}", temp);
                print!("\n$ {}", input);
                io::stdout().flush().unwrap();
            } else {
                for _i in 0..k {
                    print!("\x08 \x08");
                }
                io::stdout().flush().unwrap();
                input = common.join("_");
                print!("{}", input);
            }
        } else {
            for _i in 0..input.len() {
                print!("\x08 \x08");
            }
            io::stdout().flush().unwrap();
            print!("{}", input);
        }
        //print!("\t\t{:?}\t{:?}", common, old_matches);
        //print!("\n{:?}", common);
        //print!("\n$ {}", input);
        io::stdout().flush().unwrap();
        terminal::enable_raw_mode().unwrap();
    }

    loop {
        if event::poll(std::time::Duration::from_millis(100)).unwrap() {
            if let event::Event::Key(KeyEvent { code, modifiers, .. }) = event::read().unwrap() {
                match code {
                    KeyCode::Tab => { 
                        if matches.len() > 1 {
                            terminal::enable_raw_mode().unwrap();
                            let (result, bool1, bool2) = auto_complete(input, matches);    
                            io::stdout().flush().unwrap();
                            return (result, bool1, bool2);
                        } else {
                            terminal::enable_raw_mode().unwrap();
                            io::stdout().flush().unwrap();
                            return (input, false, true);
                        }
                    },
                    KeyCode::Enter => {
                        if input.trim() == "exit" {
                            terminal::disable_raw_mode().unwrap();
                            println!();
                            io::stdout().flush().unwrap();
                            return (input,true, false);
                        } else {
                            terminal::disable_raw_mode().unwrap();
                            println!();
                            io::stdout().flush().unwrap();
                            execute_cmd(input.clone());
                            io::stdout().flush().unwrap();
                            input.clear();
                            io::stdout().flush().unwrap();
                            terminal::enable_raw_mode().unwrap();
                            return(input,true, true);
                        }
                    },
                    KeyCode::Backspace => {
                        input.pop();
                        print!("\x08 \x08");
                        io::stdout().flush().unwrap();
                        return (input, false, true);
                    },
                    KeyCode::Char(c) => {
                        if modifiers == KeyModifiers::CONTROL && c == 'w' {
                            if !input.is_empty() {                               
                                while input.chars().last().unwrap().is_whitespace() {
                                    print!("\x08 \x08");
                                    io::stdout().flush().unwrap();
                                    input.pop();
                                }
                                let temp = input.split_whitespace().last().unwrap().to_string();
                                terminal::disable_raw_mode().unwrap();
                                for _i in 0..temp.len() {
                                    print!("\x08 \x08");
                                    io::stdout().flush().unwrap();
                                    input.pop();
                                }
                                io::stdout().flush().unwrap();
                                terminal::enable_raw_mode().unwrap();
                            } else {
                                print!("\x07");
                                io::stdout().flush().unwrap();
                            }
                        } else {
                            input.push(c);
                            print!("{}", c);
                            io::stdout().flush().unwrap();
                            terminal::enable_raw_mode().unwrap();
                            return (input, false, true);
                        }
                    },
                    _ => {
                        return (input, false, true);
                    }
                }
            }
        }
    }
}

fn longest_common_prefix(matches: &Vec<String>) -> Vec<String> {
    let mut prefixes = HashMap::new();
    for (m, item) in matches.iter().enumerate() {
        let temp: Vec<String> = item.trim().split('_').map(String::from).collect();
        prefixes.insert(m, temp);
    }
    let common = common_strings(&prefixes);
    common
}

fn common_strings(map: &HashMap<usize, Vec<String>>) -> Vec<String> {
    let mut common: Vec<String> = match map.values().next() {
        Some(first_vec) => first_vec.clone(),
        None => return Vec::new(),
    };

    for vec in map.values() {
        common = common.into_iter()
            .enumerate()
            .filter_map(|(index, string)| {
                if index < vec.len() && &string == &vec[index] {
                    Some(string)
                } else {
                    None
                }
            })
            .collect();
    }
    common
}