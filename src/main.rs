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
    arguments: Vec<String>,
}

impl ShellCommand {
    pub fn new(command: Vec<String>) -> ShellCommand {
        let command_type = CommandType::from(command[0].as_str());
        let arguments = command[1..].to_vec();
        ShellCommand {
            command_type,
            arguments,
        }
    }

    pub fn execute(&self, path_variable: String) {
        match &self.command_type {
            CommandType::Exit => match self.arguments.join(" ").parse::<i32>() {
                Ok(code) => process::exit(code),
                Err(_) => println!("exit command expects integer"),
            },
            CommandType::Echo => println!("{}", self.arguments.join(" ")),
            CommandType::Type => {
                let command = &self.arguments.join(" ");
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
                let argument = self.arguments.join(" ");
                let path = match argument == "~" {
                    true => env::var("HOME").unwrap(),
                    false => argument.clone(),
                };
                if env::set_current_dir(path).is_err() {
                    println!("cd: {}: No such file or directory", argument)
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

        let command = parse_command(input);
        if command.is_empty() {
            continue;
        }
        let command = ShellCommand::new(command);

        command.execute(path_variable.clone());
    }
}

fn parse_command(input: String) -> Vec<String> {
    let mut input_iter = input.trim().chars().peekable();
    let mut fragment = String::new();
    let mut command = Vec::new();
    let mut inside_single_quote = false;
    let mut inside_double_quote = false;

    while let Some(c) = input_iter.next() {
        if c == '\'' && !inside_double_quote {
            inside_single_quote = !inside_single_quote
        } else if c == '"' && !inside_single_quote {
            inside_double_quote = !inside_double_quote
        } else if c == '\\' && !inside_single_quote && !inside_double_quote {
            let c = input_iter.next().unwrap();
            fragment.push(c);
        } else if c == '\\' && inside_double_quote {
            match input_iter.peek().unwrap() {
                '\\' | '$' | '"' => fragment.push(input_iter.next().unwrap()),
                _ => fragment.push(c),
            }
        } else if c == ' ' && !inside_single_quote && !inside_double_quote {
            if !fragment.is_empty() {
                command.push(fragment);
                fragment = String::new();
            }
        } else {
            fragment.push(c);
        }
    }

    if !fragment.is_empty() {
        command.push(fragment);
    }

    command
}
