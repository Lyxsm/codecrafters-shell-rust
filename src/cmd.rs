#![allow(unused)]
use std::{
	os::unix::fs::PermissionsExt,
	path::{Path, PathBuf},
	env,
	fs::{self, OpenOptions},
    io::{Write, Error},
    process::{Stdio, Command},
};

pub const BUILT_IN: [&str; 5] = ["echo", "exit", "type", "pwd", "cd"];
//pub const BUILT_IN: [&str; 4] = ["echo", "exit", "type", "pwd"];

#[derive(PartialEq, Debug)]
pub enum Type {
	BuiltIn,
	PathExec,
	Invalid,
}

#[derive(PartialEq, Debug)]
pub enum QuoteType {
	Single,
	Double,
}

#[derive(PartialEq, Debug)]
pub enum Target {
    Stdout,
    Stderr,
    StdoutAppend,
    StderrAppend,
    None,
}

pub fn parse(input: &str) -> (Type, String, Vec<String>, Option<(String, Target)>) {
	let (cmd, mut args, target) = cmd_split(input.trim());

	return (cmd_type(cmd.clone()), cmd, args, target);
}

pub fn cmd_split(input: &str) -> (String, Vec<String>, Option<(String, Target)>) {
	let mut temp = parse_args(input.trim().to_string());
    let mut j: usize = 0;
    let mut cmd = String::new();
    let mut args = Vec::new(); 
    let mut target: Option<(String, Target)> = None;

    if temp.contains(&String::from(">")) || temp.contains(&String::from("1>")) || temp.contains(&String::from("2>")) || temp.contains(&String::from(">>")) || temp.contains(&String::from("1>>")) || temp.contains(&String::from("2>>")) {
        for i in 0..temp.len() {
            if temp[i] == ">" || temp[i] == "1>" {
                cmd = temp[0].clone();
                target = Some((temp[i+1..].join(""), Target::Stdout));
                for j in 1..i {
                    args.push(temp[j].clone());
                }
            } else if temp[i] == "2>" {
                cmd = temp[0].clone();
                target = Some((temp[i+1..].join(""), Target::Stderr));
                for j in 1..i {
                    args.push(temp[j].clone());
                }
            } else if temp[i] == ">>" || temp[i] == "1>>" {
                cmd = temp[0].clone();
                target = Some((temp[i+1..].join(""), Target::StdoutAppend));
                for j in 1..i {
                    args.push(temp[j].clone());
                }
            } else if temp[i] == "2>>" {
                cmd = temp[0].clone();
                target = Some((temp[i+1..].join(""), Target::StderrAppend));
                for j in 1..i {
                    args.push(temp[j].clone());
                }
            }
        }
    } else {
        cmd = temp[0].clone();
        target = None;
        for j in 1..temp.len() {
            args.push(temp[j].clone());
        }
        //println!("\ncmd: {}\nargs: {:?}", cmd, args);
    }

    //println!("command: {}", cmd);
    return (cmd, args, target);
}

pub fn is_executable(path: &Path) -> bool {
	path.metadata()
		.map(|m| m.is_file() && (m.permissions().mode() & 0o111 != 0))
		.unwrap_or(false)
}

pub fn cmd_type(input: String) -> Type {
	//let cmd = input.trim();
    let cmd = input.as_str();

	if BUILT_IN.contains(&cmd) {
		return (Type::BuiltIn);
	} else {
		match find_in_path(cmd) {
			Some(_) => return (Type::PathExec),
			None => return (Type::Invalid),
		}
	}
}

pub fn find_in_path(binary: &str) -> Option<PathBuf> {
	for dir in get_path_entries() {
		let candidate = dir.join(binary);

		if is_executable(&candidate) {
			return Some(candidate);
		}
	}

	None
}

pub fn get_path_entries() -> Vec<PathBuf> {
	env::var_os("PATH")
		.map(|paths| env::split_paths(&paths).collect())
		.unwrap()
}

pub fn handle_error<T, E>(dir: &str, result: Result<T, E>, error_message: &str) -> Option<T> {
	match result {
		Ok(value) => Some(value),
		Err(_) => {
			println!("{}: {}", dir, error_message);
			None
		}
	}
}

pub fn parse_args(input: String) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut some_string: String = input.clone();
	let input = input.as_str();

    let quotes = find_quotes(input);
    let quote_ranges: Vec<(usize, usize, QuoteType)> = quotes.into_iter().map(|(start, end, _, q_type)| (start, end, q_type)).collect();

    let mut string: &str = "";
    let other_string = input.split_once(" ");
    if other_string.is_some() {
        string = other_string.unwrap().1;
        let mut cmd = other_string.unwrap().0.to_string();
        cmd.push_str(" ");

        let quotes_temp: Vec<(usize, usize, QuoteType)> = find_quotes(string).into_iter().map(|(start, end, _, q_type)| (start, end, q_type)).collect();
        
        let mut quoted: Vec<usize> = Vec::new();
        for quote in &quotes_temp {
            quoted.push(quote.0);
            quoted.push(quote.1);
        }
        

        let mut in_quote = false;
        let mut arguments: Vec<String> = Vec::new();
        let mut argument = String::new();
        arguments.push(cmd);
        for i in 0..string.len() {
            let c = string.chars().nth(i).unwrap();
            if c.is_whitespace() && !in_quote {
                arguments.push(argument.clone());
                argument = String::from(" ");
                continue;
            }
            if quoted.contains(&i) {
                in_quote = !in_quote;
                if !in_quote {
                    argument.push(c);
                    //if !argument.is_empty() { arguments.push(argument) };
                    arguments.push(argument);
                    argument = String::new();
                    continue;
                } else {
                    //if !argument.is_empty() { arguments.push(argument) };
                    arguments.push(argument);
                    argument = String::from(c);
                    continue;
                }
            }
            argument.push(c);

            if i == string.len() - 1 {
                arguments.push(argument.clone());
            }
        }
        let mut some_vec = Vec::new();

        for mut arg in arguments {
            if is_directory(&arg) {
                arg = to_directory(&arg).unwrap_or(arg);
            }
            some_vec.push(arg);
        }
        some_string = some_vec.join("");
    }

    let mut char_indices = some_string.char_indices().peekable();
    let mut escape = false;

    while let Some((i, ch)) = char_indices.next() {
        // Check if inside a quote
        let inside_quote = quote_ranges.iter().any(|(start, end, _)| i > *start && i < *end);
        let quote_type = quote_ranges.iter().find(|(start, end, _)| i > *start && i < *end).map(|(_, _, qt)| qt);

        if escape {
            if let Some(QuoteType::Double) = quote_type {
                match ch {
                    'n' => current.push('\n'),
                    't' => current.push('\t'),
                    'r' => current.push('\r'),
                    '\\' => current.push('\\'),
                    '"' => current.push('"'),
                    '$' => current.push('$'),
                    _ => {
                        current.push('\\');
                        current.push(ch);
                    }
                }
            } else if let Some(QuoteType::Single) = quote_type {
                // Outside or single quote, \ is literal
                current.push('\\');
                current.push(ch);
            } else {
				// Outside quotes, treat normally
                match ch {
                    'n' => current.push('\n'),
                    't' => current.push('\t'),
                    'r' => current.push('\r'),
                    _ => {
                        current.push(ch);
                    }
                }
			}
            escape = false;
            continue;
        }

        if ch == '\\' {
            escape = true;
            continue;
        }

        if inside_quote {
            current.push(ch);
            continue;
        }

        match ch {
            '\'' | '"' => {
                let is_delimiter = quote_ranges.iter().any(|(start, end, _)| i == *start || i == *end);
                if !is_delimiter {
                    current.push(ch);
                }
            }
            c if c.is_whitespace() => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            c => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}

pub fn find_quotes(input: &str) -> Vec<(usize, usize, &str, QuoteType)> {
    let mut single_temp: Vec<usize> = Vec::new();
    let mut double_temp: Vec<usize> = Vec::new();
    let mut temp: Vec<usize> = Vec::new();

    let mut i = 0;
    let mut escape = false;

    for c in input.chars() {
        if escape {
            escape = false;
            i += 1;
            continue;
        }

        if c == '\\' {
            escape = true;
            i += 1;
            continue;
        }

        match c {
            '\'' => {
                single_temp.push(i);
                temp.push(i);
            }
            '"' => {
                double_temp.push(i);
                temp.push(i);
            }
            _ => {}
        }
        i += 1;
    }

    let mut open_single = false;
    let mut open_double = false;
    let mut buf = Vec::new();

    for &pos in &temp {
        if single_temp.contains(&pos) {
            if !open_single && !open_double {
                open_single = true;
                buf.push(pos);
            } else if open_single {
                open_single = false;
                buf.push(pos);
            }
        } else if double_temp.contains(&pos) {
            if !open_double && !open_single {
                open_double = true;
                buf.push(pos);
            } else if open_double {
                open_double = false;
                buf.push(pos);
            }
        }
    }

    if open_single {
        buf.pop(); 
    }
    if open_double {
        buf.pop(); 
    }

    let mut result: Vec<(usize, usize, &str, QuoteType)> = Vec::new();

    for idx in (0..buf.len()).step_by(2) {
        if idx + 1 < buf.len() {
            let start = buf[idx];
            let end = buf[idx + 1];
            let string = &input[start..=end];
            let quote_type = if &string[0..1] == "'" { QuoteType::Single } else { QuoteType::Double };
            result.push((start, end, string, quote_type));
        }
    }

    result
}

pub fn print_to_file_built_in(args: String, path: &String, target_type: Target) -> std::io::Result<()> {
    let mut file = match target_type {
        Target::Stdout => OpenOptions::new() 
                .write(true)
                .create(true)
                .open(path)?, 
        Target::StdoutAppend => OpenOptions::new() 
                .write(true)
                .create(true)
                .append(true)
                .open(path)?,
        Target::Stderr => {
            OpenOptions::new()
                .write(true)
                .create(true)
                .open(path)?;
            println!("{}", args);
            return Ok(());
        }, 
        Target::StderrAppend => {
            OpenOptions::new()
                .write(true)
                .create(true)
                .open(path)?;
            println!("{}", args);
            return Ok(());
        },
        _ => return Ok(()),
    };

    file.write_all(args.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}

pub fn print_to_file(cmd: &str, args: Vec<String>, path: &String, target_type: Target) -> Result <(), Error> {
    match target_type {
        Target::Stdout => {
            Command::new(cmd)
                .args(&args)
                .stdout(Stdio::from(OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(path)?
                ))
                .spawn()?
                .wait()?;

                Ok(())
        },
        Target::Stderr => {
            Command::new(cmd)
                .args(&args)
                .stderr(Stdio::from(OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(path)?
                ))
                .spawn()?
                .wait()?;

                Ok(())
        },
        Target::StdoutAppend => {
            Command::new(cmd)
                .args(&args)
                .stdout(Stdio::from(OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create(true)
                    .open(&path)?
                ))
                .spawn()?
                .wait()?;

                Ok(())
        },
        Target::StderrAppend => {
            Command::new(cmd)
                .args(&args)
                .stderr(Stdio::from(OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create(true)
                    .open(&path)?
                ))
                .spawn()?
                .wait()?;

                Ok(())
        },
        _ => {
            Ok(())
        }
    }
}


pub fn change_dir(input: &str) {
    let mut path: PathBuf;
    let mut dir = input.trim();
    let mut new_dir = String::new();

    if dir.is_empty() || dir == "~" {
        path = env::home_dir().expect("you are homeless");
        env::set_current_dir(&path);
        return;
    } else if dir.split("/").next() == Some("~") {
        new_dir = env::home_dir().expect("you are homeless").to_string_lossy().to_string();
        let temp = dir.split("/").skip(1);
        for i in temp {
            new_dir = new_dir + "/" + i;
            println!("{}", new_dir);
        }
        env::set_current_dir(env::home_dir().expect("you are homeless"));
        if new_dir.is_empty() {
            return;
        } else {
            change_dir(&new_dir);
            return; 
        }
    } else {
        path = handle_error(dir, fs::canonicalize(dir), "No such file or directory").unwrap_or(".".into());
    }

    env::set_current_dir(&path);
}

pub fn is_directory(input: &str) -> bool {
    let temp = input.trim();

    match temp.chars().next() {
        Some('~') | Some('/') => return true,
        Some('.') => {
            println!("{:?}", temp.chars().nth(1));
            match temp.chars().nth(1) {
                Some('/') | Some('.') | None => return true,
                _ => return false,
            }
        },
        _ => return false,
    }
}

pub fn to_directory(input: &str) -> Option<String> {
    let mut dir = input.trim();
    
    if dir.is_empty() {
        return None;
    }

    let mut split: Vec<&str> = dir.split("/").collect();
    let home = env::home_dir().expect("you are homeless").to_string_lossy().to_string();

    if split[0] == "~" {
        split[0] = home.as_str();
    }

    let split = split.join("/");

    let path = match fs::canonicalize(split.as_str()) {
        Ok(result) => Some(result.to_string_lossy().to_string()),
        Err(e) => None, 
    };

    return path;
}

/*

    for i in 0..args.len() {
        println!("{}", args[i]);
        match is_directory(&args[i]) {
            Some(arg) => args[i] = arg,
            None => {},
        }
        println!("{}", args[i]);
    }
*/