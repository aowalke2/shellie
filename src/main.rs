use std::io::{self, Write};
use std::process::Command;
use std::{env, fs, process};

use regex::Regex;

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
    has_quotes: bool,
}

impl ShellCommand {
    pub fn new(command: &str, args: &str) -> ShellCommand {
        let (arguments, has_quotes) = parse_arguments(args);
        ShellCommand {
            command_type: CommandType::from(command),
            arguments,
            has_quotes,
        }
    }

    pub fn execute(&self, path_variable: String) {
        match &self.command_type {
            CommandType::Exit => match self.arguments.join("").parse::<i32>() {
                Ok(code) => process::exit(code),
                Err(_) => println!("exit command expects integer"),
            },
            CommandType::Echo => match self.has_quotes {
                true => println!("{}", self.arguments.join("")),
                false => println!("{}", self.arguments.join(" ")),
            },
            CommandType::Type => {
                let command = &self.arguments.join("");
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
                let argument = self.arguments.join("");
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
                            .args(self.arguments.iter().filter(|s| *s != " ").clone())
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

fn parse_arguments(args: &str) -> (Vec<String>, bool) {
    let has_quotes = check_for_quoted_arguments(args);

    if !has_quotes && !args.contains('\\') {
        let arguments = parse_unquoted_arguments(args);
        return (arguments, has_quotes);
    }

    if has_quotes && (!args.starts_with('\\') || !args.starts_with('\\')) {
        let arguments = merge_quoted_args_with_spaces(args);
        return (arguments, has_quotes);
    }

    let arguments = parse_escaped_arguments(args);
    return (arguments, false);
}

fn check_for_quoted_arguments(args: &str) -> bool {
    let re = Regex::new(r#"'([^']*)'|"([^"]*)""#).unwrap();
    re.captures_iter(args).count() > 0
}

fn parse_escaped_arguments(args: &str) -> Vec<String> {
    let mut arguments = Vec::new();
    let mut argument = String::new();
    let mut escaped = false;
    for c in args.chars() {
        if c == '\\' {
            escaped = true
        } else if escaped {
            argument.push(c);
            escaped = false
        } else if !escaped && c == ' ' {
            arguments.push(argument);
            argument = String::new();
        } else {
            argument.push(c);
        }
    }

    if !argument.is_empty() {
        arguments.push(argument);
    }
    arguments
}

fn parse_unquoted_arguments(args: &str) -> Vec<String> {
    args.split_whitespace().map(|s| s.to_string()).collect()
}

fn parse_quoted_arguments(args: &str) -> Vec<String> {
    let re = Regex::new(r#"'([^']*)'|"([^"]*)"|(\S+)"#).unwrap();
    let mut arguments = Vec::new();
    for captures in re.captures_iter(args) {
        if let Some(single_quoted) = captures.get(1) {
            arguments.push(single_quoted.as_str().to_string());
        } else if let Some(double_quoted) = captures.get(2) {
            arguments.push(double_quoted.as_str().to_string());
        } else if let Some(unquoted) = captures.get(3) {
            arguments.push(unquoted.as_str().to_string());
        }
    }

    arguments
}

fn parse_spaces_between_quoted_arguments(args: &str) -> Vec<String> {
    let re = Regex::new(r#"'\s+'|"\s+""#).unwrap();
    let mut spaces = Vec::new();
    for captures in re.captures_iter(args) {
        if let Some(_) = captures.get(0) {
            spaces.push(" ".to_string());
        }
    }

    spaces
}

fn merge_quoted_args_with_spaces(args: &str) -> Vec<String> {
    let mut arguments = Vec::new();
    let quoted = parse_quoted_arguments(args);
    let spaces = parse_spaces_between_quoted_arguments(args);
    let mut i = 0;
    let mut j = 0;
    while i < quoted.len() && j < spaces.len() {
        arguments.push(quoted[i].clone());
        arguments.push(spaces[j].clone());
        i += 1;
        j += 1;
    }

    arguments.extend(quoted[j..].iter().cloned());
    arguments
}
