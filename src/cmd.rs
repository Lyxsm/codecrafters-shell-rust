#![allow(unused)]
use std::{
	os::unix::fs::PermissionsExt,
	path::{Path, PathBuf},
	env,
	fs,
};

pub const BUILT_IN: [&str; 5] = ["echo", "exit", "type", "pwd", "cd"];
//pub const BUILT_IN: [&str; 4] = ["echo", "exit", "type", "pwd"];

#[derive(PartialEq)]
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
	let mut in_single_quotes = false;
	let mut in_double_quotes = false;
	let mut prev_whitespace = false;
	
	let quotes = find_quotes(input);
	let mut quote_indices: Vec<(usize, usize)> = quotes.into_iter().map(|(start, end, _)| (start, end)).collect();

	let mut char_indices = input.char_indices().peekable();

	while let Some((i, ch)) = char_indices.next() {
		if let Some((q_start, q_end)) = quote_indices.first() {
			if i >= *q_start && i <= *q_end {
				if i > *q_start && i < *q_end {
					current.push(ch);
				}
				continue;
			} else if i > *q_end {
				quote_indices.remove(0);
			}
		}

		match ch {
            '\'' | '"' => {
            },
            c if c.is_whitespace() && !prev_whitespace => {
                // Collapse multiple whitespaces into a single one between arguments
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current)); // Push the current argument when whitespace is found
                }
                prev_whitespace = true;
            },
            c => {
                current.push(c);  // Add character to the current argument
                prev_whitespace = false;
            },
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
	for c in input.chars() {
		match c {
			'\'' => {
				single_temp.push(i);
				temp.push(i);
				i += 1;
			},
			'\"' => {
				double_temp.push(i);
				temp.push(i);
				i += 1;
			},
			_ => i += 1,
		}
	};

	let mut open_single = false;
	let mut open_double = false;
	let mut buf = temp.clone();
	let mut to_remove = Vec::new();

	for j in 0..temp.len() {
		if single_temp.contains(&temp[j]) {
			if !open_single && !open_double{
				open_single = true;
			} else {
				open_single = false;
			}
		} else if double_temp.contains(&temp[j]) {
			if !open_double && !open_single {
				open_double = true;
				
			} else {
				open_double = false;
			}
		}
		if j < temp.len() - 1 {
			if double_temp.contains(&temp[j+1]) && open_single {
				let index = buf.iter().position(|x| *x == temp[j+1]).unwrap();
				to_remove.push(index);
			}
			if single_temp.contains(&temp[j+1]) && open_double {
				let index = buf.iter().position(|x| *x == temp[j+1]).unwrap();
				to_remove.push(index);
			}
		}
	}

	if open_double {
		let index = buf.iter().position(|x| *x == double_temp[double_temp.len() - 1]).unwrap();
		to_remove.push(index);
	}
	if open_single {
		let index = buf.iter().position(|x| *x == single_temp[single_temp.len() - 1]).unwrap();
		to_remove.push(index);		
	}


	to_remove.sort_by(|a, b| b.cmp(a));
	for index in to_remove {
		buf.remove(index);
	}

	println!("[{}]: {:?}", temp.len(), temp);
	println!("[{}]: {:?}", buf.len(), buf);

	let mut result: Vec<(usize, usize, &str)> = Vec::new();

	let mut start: usize;
	let mut end: usize;

	for idx in 0..buf.len() {
		if idx == 0 || idx % 2 == 0 {
			start = buf[idx];
			end = buf[idx + 1];

			let string = &input[start..=end];
			result.push((start, end, string));
		}
	}

	println!("{:?}", result);
	result
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_quotes_only() {
        let input = "This is a 'single quote'.";
        find_quotes(input); 
        // Expected output: Should find one single quote at index 15 and 30
    }

    #[test]
    fn test_double_quotes_only() {
        let input = "This is a \"double quote\" test.";
        find_quotes(input); 
        // Expected output: Should find double quotes at index 10 and 26
    }

    #[test]
    fn test_mixed_quotes() {
        let input = "'Single quotes' and \"double quotes\" mixed.";
        find_quotes(input);
        // Expected output: Single quotes between 0-14 and double quotes between 19-35
    }

    #[test]
    fn test_quotes_inside_quotes() {
        let input = "'This \"string\" has nested quotes'.";
        find_quotes(input); 
        // Expected output: Single quotes around the whole string, double quotes inside it.
    }

    #[test]
    fn test_unbalanced_single_quotes() {
        let input = "'This is an unbalanced quote.";
        find_quotes(input); 
        // Expected output: Should find a single quote at index 0 and 28 (open), but no closing quote.
    }

    #[test]
    fn test_unbalanced_double_quotes() {
        let input = "\"This is an unbalanced quote.";
        find_quotes(input); 
        // Expected output: Should find a double quote at index 0 but no closing quote.
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        find_quotes(input);
        // Expected output: No quotes, so no output or changes.
    }

    #[test]
    fn test_no_quotes() {
        let input = "No quotes here!";
        find_quotes(input);
        // Expected output: No quotes found, so no output or changes.
    }

    #[test]
    fn test_quotes_at_beginning_and_end() {
        let input = "'Start and end with quotes'";
        find_quotes(input); 
        // Expected output: Single quotes at index 0 and 27
    }

    #[test]
    fn test_double_quotes_at_end() {
        let input = "Ends with a \"double quote\"";
        find_quotes(input); 
        // Expected output: Double quotes found at indices 14 and 30
    }

    #[test]
    fn test_single_quotes_at_end() {
        let input = "Ends with a 'single quote'";
        find_quotes(input); 
        // Expected output: Single quotes found at indices 13 and 29
    }

    #[test]
    fn test_quotes_inside_string() {
        let input = "The string contains a 'quote' here and \"double quotes\" there.";
        find_quotes(input);
        // Expected output: Single quotes around 'quote', and double quotes around "double quotes"
    }

    #[test]
    fn test_repeated_quotes() {
        let input = "'Repeated' 'quotes' 'in' the 'sentence'.";
        find_quotes(input); 
        // Expected output: Single quotes found around each 'Repeated', 'quotes', 'in', 'sentence'
    }

    #[test]
    fn test_consecutive_quotes() {
        let input = "\"\"\"Triple double quotes\"\"\"";
        find_quotes(input); 
        // Expected output: Triple double quotes found around the text
    }

    #[test]
    fn test_escaped_quotes() {
        let input = "This is an escaped quote: \"\\\"escaped\\\"\"";
        find_quotes(input); 
        // Expected output: This should not remove the escaped quotes and should handle the literal quotes.
        // NOTE: This case will fail because current code doesn't handle escaped quotes.
    }

    #[test]
    fn test_quotes_with_spaces() {
        let input = "This 'is' a 'quote' with spaces inside.";
        find_quotes(input); 
        // Expected output: Should identify single quotes around 'is' and 'quote'
    }

    #[test]
    fn test_nested_quotes_with_spaces() {
        let input = "'This is a \"nested quote\" inside'.";
        find_quotes(input); 
        // Expected output: Single quotes around the whole string, double quotes around "nested quote".
    }

    #[test]
    fn test_quotes_with_newlines() {
        let input = "'Single quote\nTest' and \"double\nquote\" here.";
        find_quotes(input); 
        // Expected output: Should handle the newline within quotes properly.
    }

    #[test]
    fn test_multiple_single_quotes() {
        let input = "Here is 'one' and 'two' single quotes.";
        find_quotes(input); 
        // Expected output: Single quotes around 'one' and 'two'
    }

    #[test]
    fn test_multiple_double_quotes() {
        let input = "Here are \"three\" and \"four\" double quotes.";
        find_quotes(input); 
        // Expected output: Double quotes around "three" and "four"
    }

    #[test]
    fn test_combined_multiple_quotes() {
        let input = "'First single' \"second double\" 'third single'.";
        find_quotes(input); 
        // Expected output: Should find single quotes around 'First single' and 'third single' and double quotes around "second double"
    }

    #[test]
    fn test_single_empty_quote() {
        let input = "An empty quote: ''";
        find_quotes(input); 
        // Expected output: Single quotes around the empty string ''
    }

    #[test]
    fn test_double_empty_quote() {
        let input = "An empty quote: \"\"";
        find_quotes(input); 
        // Expected output: Double quotes around the empty string ""
    }
}
