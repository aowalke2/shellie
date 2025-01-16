use std::io::{self, Write};
use std::process::Command;
use std::{env, fs, process};

const BUILTINS: &'static [&'static str] = &["exit", "echo", "type", "pwd"];

#[derive(Debug, Clone)]
pub enum CommandType {
    Exit,
    Echo,
    Type,
    Pwd,
    External(String),
}

impl From<&str> for CommandType {
    fn from(value: &str) -> Self {
        match value {
            "exit" => CommandType::Exit,
            "echo" => CommandType::Echo,
            "type" => CommandType::Type,
            "pwd" => CommandType::Pwd,
            _ => CommandType::External(value.to_string()),
        }
    }
}

pub struct ShellCommand {
    command_type: CommandType,
    arguments: Vec<String>,
}

impl ShellCommand {
    pub fn new(command: &str, arguments: Vec<String>) -> ShellCommand {
        ShellCommand {
            command_type: CommandType::from(command),
            arguments,
        }
    }

    pub fn execute(&self, path_variable: String) {
        match &self.command_type {
            CommandType::Exit => {
                if self.arguments.is_empty() {
                    println!("exit command expects integer");
                }

                match self.arguments[0].parse::<i32>() {
                    Ok(code) => process::exit(code),
                    Err(_) => println!("exit command expects integer"),
                }
            }
            CommandType::Echo => {
                if self.arguments.is_empty() {
                    println!();
                }

                let echo = self.arguments.join(" ");
                println!("{}", echo);
            }
            CommandType::Type => {
                if self.arguments.is_empty() {
                    println!("exit command expects argument");
                }

                let command = &self.arguments[0];
                if BUILTINS.contains(&command.as_str()) {
                    println!("{} is a shell builtin", command);
                    return;
                }

                let paths = &mut path_variable.split(":");
                if let Some(path) =
                    paths.find(|path| fs::metadata(format!("{path}/{command}")).is_ok())
                {
                    println!("{command} is {path}/{command}");
                    return;
                }

                println!("{}: not found", command);
            }
            CommandType::Pwd => todo!(),
            CommandType::External(command) => {
                let mut executable = None;
                for path in env::split_paths(&path_variable) {
                    let exe_path = path.join(&command);
                    if exe_path.exists() {
                        executable = Some(exe_path)
                    }
                }

                match executable {
                    Some(_) => {
                        Command::new(command)
                            .args(self.arguments.clone())
                            .status()
                            .expect("Unable to run command");
                    }
                    None => println!("{}: command not found", command),
                }
            }
        }
    }
}

fn main() {
    let path_variable = env::var("PATH").unwrap();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let tokens = input
            .split_whitespace()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        if tokens.is_empty() {
            continue;
        }

        let command = ShellCommand::new(tokens[0].as_str(), tokens[1..].to_vec());
        command.execute(path_variable.clone());
    }
}
