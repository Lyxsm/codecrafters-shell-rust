#![allow(unused)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::env;

pub const BUILT_IN: [&str; 3] = ["echo", "exit", "type"];

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