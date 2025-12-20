#[allow(unused_imports)]
use std::io::{self, Write};

mod cmds;

fn main() {
    repl();
}

fn repl() {
    loop {
        // Prints "$" to terminal
        print!("$ ");
        io::stdout().flush().unwrap();
    
        // Reads user input and stores it in the "command" variable
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        let command = command.trim();

        let (action, cmd_type, args) = cmds::eval_cmd(command);
        match action {
            cmds::CmdAction::Terminate => break,
            cmds::CmdAction::Print(str) => println!("{str}"),
            cmds::CmdAction::Type(str) => {
                let arg_type = cmds::cmd_type(str.as_str());
                if arg_type == cmds::CmdType::Invalid {
                    println!("{str}: not found");               
                } else {
                    println!("{str} is a shell {}", if cmd_type == cmds::CmdType::Builtin {"builtin"} else {"poop"});
                }
            }
            cmds::CmdAction::None => println!("{command}: command not found"),
        }
    }
}
