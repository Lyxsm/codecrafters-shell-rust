#[allow(unused_imports)]
use std::io::{self, Write};

enum Action {
	Terminate,
	NoOp,
	Print(String),
}

fn main() {
	enter_shell();
}

fn enter_shell() {
	loop {
		print!("& ");
		io::stdout().flush().unwrap();

		let mut buf = String::new();
		io::stdin().read_line(&mut buf).unwrap();
		let buf = buf.trim();

		match eval_command(buf) {
			Action::Terminate => break,
			Action::NoOp => println!(""),
			Action::Print(str) => println!("{str}"),
		}
	}
}

fn eval_command(input: &str) -> Action {
	let mut cmd = input.split(" ");

	let binary = if let Some(binary) = cmd.next() {
		binary
	} else {
		return Action::NoOp;
	};

	match binary {
		"exit" => Action::Terminate,
		"echo" => Action::Print(cmd.collect::<Vec<&str>>().join(" ")),
		_ => Action::Print(format!("{binary}: command not found")),
	}
}