#![allow(unused)]
#[derive(PartialEq)]
pub enum CmdType {
	Builtin,
	Valid,
	Invalid,
}
#[derive(PartialEq)]
pub enum CmdAction {
	Print(String),
	Terminate,
	Type(String),
	None,
}

#[derive(PartialEq)]
pub enum Eval {
	CmdType,
	CmdAction,
}

pub fn eval_cmd(input: &str) -> (CmdAction, CmdType, String) {
	let (cmd, args) = cmd_split(input);
	let cmd_action = cmd_action(cmd, args.clone());
	let cmd_type = cmd_type(cmd);

	(cmd_action, cmd_type, args)
}

pub fn cmd_type(cmd: &str) -> CmdType {
	match cmd {
		"echo" => CmdType::Builtin,
		"exit" => CmdType::Builtin,
		"type" => CmdType::Builtin,
		"none" => CmdType::Invalid,
		_ => CmdType::Invalid,
	}
}

pub fn cmd_action(cmd: &str, args: String) -> CmdAction {
	match cmd {
		"echo" => CmdAction::Print(args),
		"exit" => CmdAction::Terminate,
		"type" => CmdAction::Type(args),
		"none" => CmdAction::None,
		_ => CmdAction::Print(format!("{cmd}: command not found")),
	}
}

pub fn cmd_split(input: &str) -> (&str, String) {
	let mut split = input.split(" ");

	let cmd = if let Some(cmd) = split.next() {
		cmd
	} else {
		return ("none", "none".to_string());
	};

	let args = split.collect::<Vec<&str>>().join(" ");
	//println!("cmd: {}; args: {}", cmd, args);
	(cmd, args)
}