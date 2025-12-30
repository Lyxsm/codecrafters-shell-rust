//#![allow(unused_imports)]
use std::{
    io::{self, Write, Read},
    process::{Command, Stdio},
    fs,
    collections::HashMap,
    time::Duration,
};
use wait_timeout::ChildExt;
#[allow(unused_imports)]
use crossterm::{
    ExecutableCommand, cursor, terminal, execute,
    event::{self, KeyEvent, KeyCode, KeyModifiers},
};

mod cmd;

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
                            execute_cmd(input.clone());
                            io::stdout().flush().unwrap();
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
                                terminal::disable_raw_mode().unwrap();
                                println!();
                                io::stdout().flush().unwrap();
                                break 'inner;
                            } else if input.trim() == "exit" {
                                terminal::disable_raw_mode().unwrap();
                                println!();
                                io::stdout().flush().unwrap();
                                return false;
                            } else {
                                terminal::disable_raw_mode().unwrap();
                                println!();
                                execute_cmd(input.clone());
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
    //println!("command: {:?}, args: {:?}, target: {:?}, pipe: {:?}", command, args, target, pipe);
    if pipe.is_none() {
        match cmd_type {
            cmd::Type::BuiltIn => {
                match command {
                    "echo" => {
                        let arguments: String = args.join(" ");
                        if let Some((file, t)) = target {
                            cmd::print_to_file_built_in(arguments, &file, t).expect("Failed to print to file");
                        } else {
                            println!("{}", arguments);
                        }
                    },
                    "pwd" => {
                        let current_dir = fs::canonicalize(".").expect("Failed to retrieve working directory");
                        println!("{}", current_dir.display());
                    },
                    "type" => {
                        if args.is_empty() {
                            println!("type: not enough arguments");
                        } else {
                            match cmd::cmd_type(args[0].clone()) {
                                cmd::Type::BuiltIn => println!("{} is a shell builtin", args[0]),
                                cmd::Type::PathExec => println!("{} is {}", args[0], cmd::find_in_path(&args[0]).expect("not found").display()),
                                cmd::Type::Invalid => println!("{}: not found", args[0]),
                            }
                        }
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
                            let _ = cmd::print_to_file(command, args, path, target_type);
                        } else {
                            Command::new(command)
                                .args(&args)
                                .spawn()
                                .expect("failed to execute")
                                .wait()
                                .expect("failed to wait");
                        }
                    },
                    None => println!("{}: not found", command),
                }
            },
            _ => println!("{}: command not found", command)
        }
        return;
    }

    let pipes = pipe.unwrap();

    let mut segments: Vec<(cmd::Type, String, Vec<String>, Option<(String, cmd::Target)>)> = Vec::new();

    for seg in &pipes {
        let parsed = cmd::parse(seg);
        segments.push((parsed.0, parsed.1, parsed.2, parsed.3));
    }

    //println!("{:?}", segments); 

    let initial_output: Option<Vec<u8>> = match cmd_type {
        cmd::Type::BuiltIn => {
            let output = run_builtin(command, &args, &target);
            Some(output.into_bytes())
        },
        cmd::Type::PathExec => {
            let mut child = Command::new(command)
                .args(&args)
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to execute command");

            let mut stdout = child.stdout.take();

            match child.wait_timeout(Duration::from_millis(1750)).expect("wait_timeout failed") {
                Some(_status) => {

                },
                None => {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }

            let mut buf = Vec::new();
            if let Some(mut stdout) = stdout {
                let _ = stdout.read_to_end(&mut buf);
            }
            Some(buf)
        },
        _ => None,
    };

    let mut current_data = initial_output.unwrap_or_default();

    for (idx, (seg_type, seg_cmd, seg_args, seg_target)) in segments.into_iter().enumerate() {
        let seg_cmd_str = seg_cmd.as_str();
        //println!("[{}]: type: {:?}\tcmd: {}\targs: {:?}\ttarget: {:?}", idx, seg_type, seg_cmd, seg_args, seg_target);
        //std::io::stdout().write_all(&current_data);
        match seg_type {
            cmd::Type::BuiltIn => {
                let stdin_string = String::from_utf8_lossy(&current_data).to_string();
                let output = run_builtin_stdin(&seg_cmd, &seg_args, &seg_target, &stdin_string);
                current_data = output.into_bytes();
            },
            cmd::Type::PathExec => {
                let mut child = Command::new(seg_cmd_str)
                    .args(&seg_args)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("Failed to start piped command");

                if let Some(mut stdin) = child.stdin.take() {
                    if let Err(e) = stdin.write_all(&current_data) {
                        eprintln!("Failed to write to piped command stdin: {}", e);
                        let _ = child.kill();
                        return;
                    }
                    drop(stdin);
                }

                let mut stdout = child.stdout.take();

                match child.wait_timeout(Duration::from_millis(1750)).unwrap() {
                    Some(status) => {
                        if !status.success() {
                            eprintln!("Piped command exited with status: {}", status);
                        }
                    },
                    None => {
                        //eprintln!("Command timed out, terminating...");
                        let _ = child.kill();
                        let _ = child.wait();
                    }
                }
                
                let mut buf = Vec::new();
                if let Some(mut stdout) = stdout {
                    let _ = stdout.read_to_end(&mut buf);
                }

                current_data = buf;
            },
            _ => {}
        }

        if idx == pipes.len() - 1 {
            if let Some((ref file, t)) = seg_target {
                match seg_type {
                    cmd::Type::PathExec => {
                        let _ = cmd::print_to_file(&seg_cmd, seg_args.clone(), file, t.clone());
                    },
                    cmd::Type::BuiltIn => {
                        let mut options = std::fs::OpenOptions::new();
                        options.create(true).write(true);

                        if t == cmd::Target::StdoutAppend || t == cmd::Target::StderrAppend {
                            options.append(true); 
                        } else {
                            options.truncate(true);
                        }

                        let mut f = options.open(file).expect("Failed to open target file");
                        f.write_all(&current_data).expect("Failed to write to file");
                    },
                    _ => {
                        let mut f = std::fs::OpenOptions::new()
                            .create(true)
                            .write(true)
                            .truncate(true)
                            .open(file)
                            .expect("Failed to open target file");
                        f.write_all(&current_data).expect("Failed to write to file");
                    }
                }
            } else {
                std::io::stdout().write_all(&current_data).expect("Failed to write to stdout");
            }
        }
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

fn run_builtin(cmd: &str, args: &[String], target: &Option<(String, cmd::Target)>) -> String {
    match cmd {
        "echo" => {
            let string = args.join(" ");
            if let Some((file, t)) = target {
                cmd::print_to_file_built_in(string.clone(), file, t.clone()).ok();
                String::new()
            } else {
                format!("{}\n", string)
            }
        },
        "pwd" => {
            let current_dir = std::fs::canonicalize(".").expect("failed to retrieve working directory");
            let output = format!("{}", current_dir.display());
            if let Some((file, t)) = target {
                cmd::print_to_file_built_in(output.clone(), file, t.clone()).ok();
                String::new()
            } else {
                output
            }
        },
        "type" => {
            if args.is_empty() {
                return "type: not enough arguments\n".to_string();
            }
            let output = match cmd::cmd_type(args[0].clone()) {
                cmd::Type::BuiltIn => format!("{} is a shell builtin\n", args[0]),
                cmd::Type::PathExec => format!("{} is {}\n", args[0], cmd::find_in_path(&args[0]).expect("not found").display()),
                cmd::Type::Invalid => format!("{}: not found\n", args[0]),
            };
            if let Some((file, t)) = target {
                cmd::print_to_file_built_in(output.clone(), file, t.clone()).ok();
                String::new()
            } else {
                output
            }
        },
        _ => String::new(),
    }
}

fn run_builtin_stdin(cmd: &str, args: &[String], target: &Option<(String, cmd::Target)>, stdin: &str) -> String {
    match cmd {
        "echo" => {
            let output = if !args.is_empty() {
            args.join(" ")
            } else {
                if stdin.ends_with('\n') {
                stdin.trim_end_matches('\n').to_string()
                } else {
                   stdin.to_string()
                }
            };
            if let Some((file, t)) = target {
                cmd::print_to_file_built_in(output.clone(), file, t.clone()).ok();
                String::new()
            } else {
                format!("{}\n", output)
            }
        },
        "pwd" => {
            let current_dir = std::fs::canonicalize(".").expect("failed to retrieve working directory");
            let output = format!("{}", current_dir.display());
            if let Some((file, t)) = target {
                cmd::print_to_file_built_in(output.clone(), file, t.clone()).ok();
                String::new()
            } else {
                output
            }
        },
        "type" => {
            if args.is_empty() {
                return "type: not enough arguments\n".to_string();
            }
            let output = match cmd::cmd_type(args[0].clone()) {
                cmd::Type::BuiltIn => format!("{} is a shell builtin\n", args[0]),
                cmd::Type::PathExec => format!("{} is {}\n", args[0], cmd::find_in_path(&args[0]).expect("not found").display()),
                cmd::Type::Invalid => format!("{}: not found\n", args[0]),
            };
            if let Some((file, t)) = target {
                cmd::print_to_file_built_in(output.clone(), file, t.clone()).ok();
                String::new()
            } else {
                output
            }
        },
        _ => String::new(),
    }
}