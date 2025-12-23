#![allow(unused)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, Pathbuf};
use std::env;

pub const BUILT_IN: [&str; 3] = ["echo", "exit", "type"];

#[derive(PartialEq)]
pub enum Type {
	BuiltIn,
	PathExec,
	PathNoExec,
	Invalid,
}


pub fn parse(input: &str) -> (Type, &str, &str) {
	let path = std::env::var("PATH").unwrap();
	let input = input.trim();
	let (cmd, args) = cmd_split(input);

	for arg in args.split_whitespace() {
		if BUILT_IN.contains(&arg) {
			return (Type::BuiltIn, arg, &args);
		} else {
			match find_in_path(arg) {
				Some(path_buf) => return (Type::PathExec, arg, &args),
				None => return (Type::PathNoExec, arg, &args),
			}
			return (Type::Invalid, arg, &args);
		}
	}
	return (Type::Invalid, arg, &args);
}


pub fn cmd_split(input: &str) -> (&str, String) {
	let (cmd, args) = input.split_once(' ').unwrap_or((input, ""));
	(cmd, args.to_string())
}

pub fn is_executable(path: &Path) -> bool {
	path.metadata()
		.map(|m| m.is_file() && (m.permissions().mode() & 0o111 != 0))
		.unwrap_or(false)
}

pub fn cmd_type(input: &str) -> Type {
	let input = input.trim();
	let (cmd, args) = cmd_split(input);

	for arg in args.split_whitespace() {
		if BUILT_IN.contains(&arg) {
			return Type::BuiltIn;
		} else {
			match find_in_path(arg) {
				Some(path_buf) => return Type::PathExec,
				None => return Type::PathNoExec,
			}
			return Type::Invalid;
		}
	}
	return Type::Invalid
;}

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