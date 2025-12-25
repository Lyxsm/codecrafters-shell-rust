#![allow(unused)]
use std::{
	os::unix::fs::PermissionsExt,
	path::{Path, PathBuf},
	env,
	fs,
};

pub const BUILT_IN: [&str; 5] = ["echo", "exit", "type", "pwd", "cd"];
//pub const BUILT_IN: [&str; 4] = ["echo", "exit", "type", "pwd"];

#[derive(PartialEq, Debug)]
pub enum Type {
	BuiltIn,
	PathExec,
	Invalid,
}

pub fn parse(input: &str) -> (Type, &str, Vec<String>) {
	let (cmd, args) = cmd_split(input.trim());

	let args = parse_args(args);

	return (cmd_type(cmd), cmd, args);
}

pub fn cmd_split(input: &str) -> (&str, &str) {
	let (cmd, args) = input.split_once(' ').unwrap_or((input, ""));
	(cmd, args)
}

pub fn is_executable(path: &Path) -> bool {
	path.metadata()
		.map(|m| m.is_file() && (m.permissions().mode() & 0o111 != 0))
		.unwrap_or(false)
}

pub fn cmd_type(input: &str) -> Type {
	let cmd = input.trim();

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

pub fn handle_error<T, E>(dir: &str, result: Result<T, E>, error_message: &str) -> Option<T> {
	match result {
		Ok(value) => Some(value),
		Err(_) => {
			println!("cd: {}: {}", dir, error_message);
			None
		}
	}
}

pub fn parse_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();

    let quotes = find_quotes(input);
    let mut quote_indices: Vec<(usize, usize)> = quotes.into_iter().map(|(start, end, _)| (start, end)).collect();

    let mut char_indices = input.char_indices().peekable();
    let mut escape = false;

    while let Some((i, ch)) = char_indices.next() {
        if let Some((q_start, q_end)) = quote_indices.first() {
            if i >= *q_start && i <= *q_end {
                if i > *q_start && i < *q_end {
                    current.push(ch);
                }
                if i == *q_end {
                    quote_indices.remove(0);
                }
                continue;
            }
        }
		
		if escape {
            // If the previous character was an escape, add this character literally
            current.push(ch);
            escape = false;
            continue;
        }

        if ch == '\\' {
            // Handle escape character
            escape = true;
            continue;
        }

        match ch {
            '\'' | '"' => {
                current.push(ch);
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

pub fn find_quotes(input: &str) -> Vec<(usize, usize, &str)> {
    let mut single_temp: Vec<usize> = Vec::new();
    let mut double_temp: Vec<usize> = Vec::new();
    let mut temp: Vec<usize> = Vec::new();

    let mut i = 0;
    let mut escape = false;
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    for c in input.chars() {
        if escape {
            // Skip this character as it is escaped
            escape = false;
            i += 1;
            continue;
        }

        if in_single_quote {
            if c == '\'' {
                in_single_quote = false;
                single_temp.push(i);
                temp.push(i);
            }
            i += 1;
            continue;
        }

        if in_double_quote {
            if c == '"' {
                in_double_quote = false;
                double_temp.push(i);
                temp.push(i);
            }
            i += 1;
            continue;
        }

        if c == '\\' {
            // Set escape flag for the next character
            escape = true;
            i += 1;
            continue;
        }

        match c {
            '\'' => {
                in_single_quote = true;
                single_temp.push(i);
                temp.push(i);
            }
            '"' => {
                in_double_quote = true;
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

    // Ensure unmatched quotes are not included
    if open_single {
        buf.pop(); // Remove the last unmatched single quote
    }
    if open_double {
        buf.pop(); // Remove the last unmatched double quote
    }

    let mut result: Vec<(usize, usize, &str)> = Vec::new();

    for idx in (0..buf.len()).step_by(2) {
        if idx + 1 < buf.len() {
            let start = buf[idx];
            let end = buf[idx + 1];
            let string = &input[start..=end];
            result.push((start, end, string));
        }
    }

    result
}
