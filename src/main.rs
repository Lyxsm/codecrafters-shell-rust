#![allow(unused_imports, unused_mut, unused)]
use std::{
    io::{self, Write, Read, BufRead},
    process::{Command, Stdio},
    fs::{self, File}, 
    fmt,
    str::FromStr,
    collections::HashMap, 
    time::Duration,
};
use wait_timeout::ChildExt;
use serde::{Serialize, Deserialize};
#[allow(unused_imports)]
use crossterm::{
    ExecutableCommand, cursor, terminal, execute,
    event::{self, KeyEvent, KeyCode, KeyModifiers},
};

mod cmd;

const HISTORY: &str = "history";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CmdHistory {
    history: Vec<(usize, String)>,
    length: usize,
}

impl CmdHistory {
    fn push(&mut self, string: String) {
        self.length += 1;
        let tuple = (self.length, string);
        self.history.push(tuple);
    }
    fn new() -> Self {
        Self { history: Vec::<(usize, String)>::new(), length: 0 }
    }
    fn last_entry(&self) -> (usize, String) {
        if self.history.len() > 1 {
            self.history.last().unwrap().clone()
        } else if self.history.is_empty() {
            (0, String::new())
        } else {
            self.history[0].clone()
        }
    }
    fn from_vec(vec: &Vec<(usize, String)>) -> Self {
        Self { history: vec.clone(), length: vec.len() }
    }
}

impl fmt::Display for CmdHistory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = String::new();
        for (count, command) in &self.history {
            buf += &format!("{}\n", command);
        }
        write!(f, "{}", buf)
    }
}

impl FromStr for CmdHistory {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        let mut parts: Vec<&str> = s.lines().map(|line| line.trim()).filter(|line| !line.is_empty()).collect();
        let mut history = CmdHistory::new();
        if parts.len() > 0 {
            let mut history = CmdHistory::new();
            for part in parts {
                history.push(part.to_string());
            }
            //println!("{}", history);
            Ok(history)
        } else {
            Err(())
        }
    }
}

fn main() {
    let mut history = CmdHistory::from_vec(&get_history());
    //let mut temp_history = history.clone();
    loop {
        let input = String::new();
        if !active(input, &mut history) {
            return;
        }
    }
}

fn active(input: String, mut history: &mut CmdHistory) -> bool {
    let mut history_offset = 0;
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
                        let count = 0;
                        let (string, event, stay_active, _count) = auto_complete(input.clone(), matches.clone(), count, &mut history);
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
                            add_to_history(input.clone(), &mut history);
                            terminal::disable_raw_mode().unwrap();
                            println!();
                            io::stdout().flush().unwrap();
                            return false;
                        } else {
                            terminal::disable_raw_mode().unwrap();
                            println!();
                            execute_cmd(input.clone(), &mut history);
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
                                add_to_history(input.clone(), &mut history);
                                terminal::disable_raw_mode().unwrap();
                                println!();
                                io::stdout().flush().unwrap();
                                return false;
                            } else {
                                terminal::disable_raw_mode().unwrap();
                                println!();
                                execute_cmd(input.clone(), &mut history);
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
                        if !history.history.is_empty() {
                            terminal::disable_raw_mode().unwrap();
                            for _i in 0..input.len() {
                                print!("\x08 \x08");
                            }
                            input.clear();
                            io::stdout().flush().unwrap();
                            let mut temp = history.history.clone();
                            temp.push((history.length, String::new()));
                            let len = temp.len() - 1;
                            history_offset += 1;
                            if history_offset > len {
                                input = String::new();
                                history_offset = 0;
                            } else {
                                input = temp[len - history_offset].1.trim().to_string();
                            }
                            print!("{input}");
                            io::stdout().flush().unwrap();
                            terminal::enable_raw_mode().unwrap();
                        }
                    },
                    KeyCode::Down => {
                        if !history.history.is_empty() {
                            terminal::disable_raw_mode().unwrap();
                            for _i in 0..input.len() {
                                print!("\x08 \x08");
                            }
                            input.clear();
                            io::stdout().flush().unwrap();
                            let mut temp = history.history.clone();
                            temp.push((history.length, String::new()));
                            let len = temp.len() - 1;
                            if history_offset == 0 {
                                input = String::new();
                            } else {
                                if history_offset > len {
                                    history_offset = len;
                                } else {
                                    history_offset -= 1;
                                }
                                input = temp[len - history_offset].1.trim().to_string();
                            }
                            print!("{input}");
                            io::stdout().flush().unwrap();
                            terminal::enable_raw_mode().unwrap();
                        }
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

fn execute_cmd(input: String, mut history: &mut CmdHistory) {
    add_to_history(input.clone(), &mut history);
    //let mut entry = format!("{}\n", input.clone());
    //history.push(entry);
    let (cmd_type, command, args, target, pipe) = cmd::parse(&input);
    let command = command.as_str();

    //println!("command: {:?}, args: {:?}, target: {:?}, pipe: {:?}", command, args, target, pipe);
    if pipe.is_none() {
        match cmd_type {
            cmd::Type::BuiltIn => {
                match command {
                    "echo" | "pwd" | "type" | "history" => {
                        print!("{}", run_builtin(command, &args, &target, &mut history));
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
            let output = run_builtin(command, &args, &target, &mut history);
            Some(output.into_bytes())
        },
        cmd::Type::PathExec => {
            match command {
                "tail" => {
                    let mut arg_count: usize = 99999;
                    if segments[0].1.contains("head") {
                        let num = first_number(&segments[0].2);
                        arg_count = num.map(|n| n as usize).unwrap_or(5);
                    }
                    let mut child = Command::new(command)
                        .args(&args)
                        .stdout(Stdio::piped())
                        .spawn()
                        .expect("Failed to execute command");

                    let mut stdout = child.stdout.take();
                    let mut buf = Vec::new();
                    let mut line_count = 0;

                    let mut reader = io::BufReader::new(stdout.as_mut().expect("Failed to read stdout"));
                    let mut stdout_handle = io::stdout();

                    for line in reader.lines() {
                        match line {
                            Ok(line_content) => {
                                stdout_handle.write_all(line_content.as_bytes()).expect("Failed to write to stdout");
                                stdout_handle.write_all(b"\n").expect("Failed to write newline to stdout");
                                buf.extend_from_slice(line_content.as_bytes());
                                buf.push(b'\n');
                                line_count += 1;
                                if line_count >= arg_count {
                                    break;
                                }
                            },
                            Err(e) => {
                                eprintln!("Error reading output: {}", e);
                                break;
                            }
                        }
                    }

                    stdout_handle.flush().expect("Failed to flush stdout");

                    let _ = child.kill();
                    let _ = child.wait().expect("Failed to wait on child process");
                    None
                },
                _ => {
                    let mut child = Command::new(command)
                        .args(&args)
                        .stdout(Stdio::piped())
                        .spawn()
                        .expect("Failed to execute command");

                    let mut stdout = child.stdout.take();

                    match child.wait_timeout(Duration::from_millis(100)).expect("wait_timeout failed") {
                        Some(_status) => {},
                        None => {
                            let _ = child.kill();
                            let _ = child.wait();
                        },
                    }

                    let mut buf = Vec::new();
                    if let Some(mut stdout) = stdout {
                        let _ = stdout.read_to_end(&mut buf);
                    }

                    Some(buf)
                }
            }
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
                let output = run_builtin_stdin(&seg_cmd, &seg_args, &seg_target, &stdin_string, &mut history);
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

fn auto_complete(mut input: String, matches: Vec<String>, mut count: usize, mut history: &mut CmdHistory) -> (String, bool, bool, usize) {
    terminal::disable_raw_mode().unwrap();
    //let mut temp: Vec<String>;
    if matches.is_empty() {
        print!("\x07");
        io::stdout().flush().unwrap();
        terminal::enable_raw_mode().unwrap();
        count += 1;
        return (input, false, true, count);
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
        count += 1;
        return (input, false, true, count);
    } else if matches.len() > 1 {
        let common = longest_common_prefix(&matches);
        if common.len() == 1 && count == 0 /*&& input.ends_with('_')*/ {
            print!("\x07");
            io::stdout().flush().unwrap();
            terminal::enable_raw_mode().unwrap();
        } else if common.len() >= 1 {
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
        count += 1;
    }

    loop {
        if event::poll(std::time::Duration::from_millis(100)).unwrap() {
            if let event::Event::Key(KeyEvent { code, modifiers, .. }) = event::read().unwrap() {
                match code {
                    KeyCode::Tab => { 
                        if matches.len() > 1 {
                            terminal::enable_raw_mode().unwrap();
                            let (result, bool1, bool2, count) = auto_complete(input, matches, count, &mut history);    
                            io::stdout().flush().unwrap();
                            return (result, bool1, bool2, count);
                        } else {
                            terminal::enable_raw_mode().unwrap();
                            io::stdout().flush().unwrap();
                            count += 1;
                            return (input, false, true, count);
                        }
                    },
                    KeyCode::Enter => {
                        if input.trim() == "exit" {
                            add_to_history(input.clone(), &mut history);
                            terminal::disable_raw_mode().unwrap();
                            println!();
                            io::stdout().flush().unwrap();
                            return (input,true, false, count);
                        } else {
                            terminal::disable_raw_mode().unwrap();
                            println!();
                            io::stdout().flush().unwrap();
                            execute_cmd(input.clone(), &mut history);
                            io::stdout().flush().unwrap();
                            input.clear();
                            io::stdout().flush().unwrap();
                            terminal::enable_raw_mode().unwrap();
                            return(input,true, true, count);
                        }
                    },
                    KeyCode::Backspace => {
                        input.pop();
                        print!("\x08 \x08");
                        io::stdout().flush().unwrap();
                        return (input, false, true, count);
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
                            return (input, false, true, count);
                        }
                    },
                    _ => {
                        return (input, false, true, count);
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

fn run_builtin(cmd: &str, args: &[String], target: &Option<(String, cmd::Target)>, mut history: &mut CmdHistory) -> String {
    match cmd {
        "echo" => {
            let output = args.join(" ");
            if let Some((file, t)) = target {
                cmd::print_to_file_built_in(output.clone(), file, t.clone()).ok();
                String::new()
            } else {
                format!("{}\n", output)
            }
        },
        "pwd" => {
            let current_dir = std::fs::canonicalize(".").expect("failed to retrieve working directory");
            let output = format!("{}\n", current_dir.display());
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
        "history" => {
            let mut output = String::new();
            let entries = history.history.clone();
            //println!("{:?}", args);
            if !args.is_empty() {
                if let Ok(number) = args[0].parse::<usize>() {
                    let mut count: usize = number;
                    if count > entries.len() {
                        count = entries.len();
                    }
                    let mut output_vec = Vec::new();
                    for i in 1..=count {
                        let idx = entries.len() - i;
                        let entry = (entries[idx].0, entries[idx].1.clone());
                        output_vec.push(entry);
                    }    
                    output_vec.reverse();
                    for entry in output_vec {
                        output.push_str(&format!("    {}  {}\n", entry.0, entry.1));
                    }    
                } else if args[0].trim() == "-r" {
                    if args.len() > 1 {
                        //execute_cmd(format!("cat {}", args[1]), history);
                        add_history_file(&args[1].clone(), &mut history);
                    }
                    return String::new();
                } else {
                    for (count, cmd) in entries {
                        output.push_str(&format!("    {}  {}\n", count, cmd));
                    }
                }
            } else {
                for (count, cmd) in entries {
                    output.push_str(&format!("    {}  {}\n", count, cmd));
                }
            }
            output
        },
        _ => String::new(),
    }
}

fn run_builtin_stdin(cmd: &str, args: &[String], target: &Option<(String, cmd::Target)>, stdin: &str, mut history: &mut CmdHistory) -> String {
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
        "history" => {
            let mut output = String::new();
            let entries = history.history.clone();
            //println!("{:?}", args);
            if !args.is_empty() {
                if let Ok(number) = args[0].parse::<usize>() {
                    let mut count: usize = number;
                    if count > entries.len() {
                        count = entries.len();
                    }
                    let mut output_vec = Vec::new();
                    for i in 1..=count {
                        let idx = entries.len() - i;
                        let entry = (entries[idx].0, entries[idx].1.clone());
                        output_vec.push(entry);
                    }    
                    output_vec.reverse();
                    for entry in output_vec {
                        output.push_str(&format!("    {}  {}\n", entry.0, entry.1));
                    }    
                } else if args[0].trim() == "-r" {
                    if args.len() > 1 {
                        //execute_cmd(format!("cat {}", args[1]), history);
                        add_history_file(&args[1].clone(), &mut history);
                    }
                    return String::new();
                } else {
                    for (count, cmd) in entries {
                        output.push_str(&format!("    {}  {}\n", count, cmd));
                    }
                }
            } else {
                for (count, cmd) in entries {
                    output.push_str(&format!("    {}  {}\n", count, cmd));
                }
            }
            output
        },
        _ => String::new(),
    }
}

fn first_number(strings: &Vec<String>) -> Option<f64> {
    for s in strings {
        if let Ok(number) = s.parse::<f64>() {
            return Some(number);
        }
    }
    None
}

fn get_history() -> Vec<(usize, String)> {
    if let Ok(history) = load_from_file(HISTORY) {
        history.history
    } else {
        CmdHistory::new().history
    }
}

fn add_history_file(file: &str, history: &mut CmdHistory) {
    let mut entries: Vec<(usize, String)>;
    if let Ok(hist) = load_from_file(file) {
        entries = hist.history;
    } else {
        entries = CmdHistory::new().history;
    }
    for entry in entries {
        print!("{:?}", entry);
        history.push(entry.1);
    }
}

fn add_to_history(input: String, history: &mut CmdHistory) {
    if input.is_empty() {
        return
    } else {
        //let input = format!("{}", input);
        history.push(input);
        //print!("{}", history);
        //save_to_json(HISTORY, &history);
        save_to_txt(HISTORY, &history).expect("Failed to save history to file");
    }
}

fn save_to_txt(filename: &str, content: &CmdHistory) -> io::Result<()> {
    let mut file = File::create(filename)?;
    write!(file, "{}\n", content)?;
    //print!("{}", content);
    Ok(())
}

fn load_from_file(filename: &str) -> io::Result<CmdHistory> {
    let content = std::fs::read_to_string(filename)?;
    CmdHistory::from_str(&content).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to parse data"))
}
