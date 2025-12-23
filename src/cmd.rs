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

pub fn parse(input: &str) -> (Type, &str, &str) {
	let (cmd, args) = cmd_split(&input.trim());

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
