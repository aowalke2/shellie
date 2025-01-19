use regex::Regex;
use std::io::{self, Write};
use std::process::Command;
use std::{env, fs, process};

const BUILTINS: &'static [&'static str] = &["exit", "echo", "type", "pwd", "cd"];

#[derive(Debug, Clone)]
pub enum CommandType {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
    External(String),
}

impl From<&str> for CommandType {
    fn from(value: &str) -> Self {
        match value {
            "exit" => CommandType::Exit,
            "echo" => CommandType::Echo,
            "type" => CommandType::Type,
            "pwd" => CommandType::Pwd,
            "cd" => CommandType::Cd,
            _ => CommandType::External(value.to_string()),
        }
    }
}

pub struct ShellCommand {
    command_type: CommandType,
    arguments: String,
}

impl ShellCommand {
    pub fn new(command: &str, arguments: &str) -> ShellCommand {
        ShellCommand {
            command_type: CommandType::from(command),
            arguments: arguments.to_string(),
        }
    }

    pub fn execute(&self, path_variable: String) {
        match &self.command_type {
            CommandType::Exit => {
                if self.arguments.is_empty() {
                    println!("exit command expects integer");
                }

                match self.arguments.parse::<i32>() {
                    Ok(code) => process::exit(code),
                    Err(_) => println!("exit command expects integer"),
                }
            }
            CommandType::Echo => {
                if self.arguments.is_empty() {
                    println!();
                }

                let arguments = if (self.arguments.starts_with('\'')
                    && self.arguments.ends_with('\''))
                    || (self.arguments.starts_with('"') && self.arguments.ends_with('"'))
                {
                    self.arguments
                        .chars()
                        .filter(|c| *c != '\'' && *c != '"')
                        .collect()
                } else {
                    self.arguments
                        .split_whitespace()
                        .collect::<Vec<&str>>()
                        .join(" ")
                };
                println!("{}", arguments);
            }
            CommandType::Type => {
                if self.arguments.is_empty() {
                    println!("exit command expects argument");
                }

                let command = &self.arguments;
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
            CommandType::Pwd => match env::current_dir() {
                Ok(path) => println!("{}", path.display()),
                Err(_) => println!("could not retreive working directory"),
            },
            CommandType::Cd => {
                let path = match self.arguments == "~" {
                    true => env::var("HOME").unwrap(),
                    false => self.arguments.clone(),
                };
                if env::set_current_dir(path).is_err() {
                    println!("cd: {}: No such file or directory", self.arguments)
                }
            }
            CommandType::External(command) => {
                let mut executable = None;
                for path in env::split_paths(&path_variable) {
                    let exe_path = path.join(&command);
                    if exe_path.exists() {
                        executable = Some(exe_path)
                    }
                }

                let re = Regex::new(r#"'([^']*)'|"([^"]*)"|(\S+)"#).unwrap();
                let mut arguments = Vec::new();
                for captures in re.captures_iter(&self.arguments) {
                    if let Some(single_quoted) = captures.get(1) {
                        arguments.push(single_quoted.as_str().to_string());
                    } else if let Some(double_quoted) = captures.get(2) {
                        arguments.push(double_quoted.as_str().to_string());
                    } else if let Some(unquoted) = captures.get(3) {
                        arguments.push(unquoted.as_str().to_string());
                    }
                }

                match executable {
                    Some(_) => {
                        Command::new(command)
                            .args(arguments)
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

        let command = if let Some((command, arguments)) = input.trim().split_once(' ') {
            ShellCommand::new(command, arguments)
        } else {
            ShellCommand::new(&input.trim(), "")
        };
        command.execute(path_variable.clone());
    }
}
