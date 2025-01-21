use std::fs::{File, OpenOptions};
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

    pub fn empty() -> ShellCommand {
        ShellCommand {
            command_type: CommandType::Exit,
            arguments: Vec::new(),
        }
    }

    pub fn check_for_redirect(&mut self, stdout: &mut Box<dyn Write>, stderr: &mut Box<dyn Write>) {
        if self.arguments.len() < 2 {
            return;
        }

        let redirect = self.arguments[self.arguments.len() - 2].clone();
        if !["2>", "2>>", "1>", ">", "1>>", ">>"].contains(&redirect.as_str()) {
            return;
        }

        let redirect_file = self.arguments.pop().unwrap();
        self.arguments.pop().unwrap();
        let file = if ["2>", "1>", ">"].contains(&redirect.as_str()) {
            OpenOptions::new()
                .write(true)
                .create(true)
                .open(redirect_file)
                .expect("Unable to open file")
        } else {
            OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open(redirect_file)
                .expect("Unable to open file")
        };

        if ["1>", ">", "1>>", ">>"].contains(&redirect.as_str()) {
            *stdout = Box::new(file)
        } else if ["2>", "2>>"].contains(&&redirect.as_str()) {
            *stderr = Box::new(file)
        }
    }
}

pub struct Shell {
    stdout: Box<dyn Write>,
    stderr: Box<dyn Write>,
    path: String,
    command: ShellCommand,
}

impl Shell {
    pub fn new() -> Shell {
        let path = env::var("PATH").unwrap();
        Shell {
            stdout: Box::new(io::stdout()),
            stderr: Box::new(io::stderr()),
            path,
            command: ShellCommand::empty(),
        }
    }

    pub fn parse_command(&mut self, input: String) {
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

        self.command = ShellCommand::new(command);
        self.command
            .check_for_redirect(&mut self.stdout, &mut self.stderr);
    }

    pub fn execute(&mut self) {
        match &self.command.command_type {
            CommandType::Exit => match self.command.arguments.join(" ").parse::<i32>() {
                Ok(code) => process::exit(code),
                Err(_) => println_output(&mut self.stderr, "command expects an integer"),
            },
            CommandType::Echo => {
                println_output(&mut self.stdout, self.command.arguments.join(" ").as_str())
            }
            CommandType::Type => {
                let command = &self.command.arguments.join(" ");

                if BUILTINS.contains(&command.as_str()) {
                    println_output(
                        &mut self.stdout,
                        format!("{} is a shell builtin", command).as_str(),
                    );
                    return;
                }

                let paths = &mut self.path.split(":");
                if let Some(path) =
                    paths.find(|path| fs::metadata(format!("{path}/{command}")).is_ok())
                {
                    println_output(
                        &mut self.stdout,
                        format!("{command} is {path}/{command}").as_str(),
                    );
                    return;
                }

                println_output(&mut self.stderr, format!("{}: not found", command).as_str());
            }
            CommandType::Pwd => match env::current_dir() {
                Ok(path) => {
                    println_output(&mut self.stdout, format!("{}", path.display()).as_str())
                }
                Err(_) => println_output(&mut self.stderr, "could not retreive working directory"),
            },
            CommandType::Cd => {
                let argument = self.command.arguments.join(" ");
                let path = match argument == "~" {
                    true => env::var("HOME").unwrap(),
                    false => argument.clone(),
                };

                if env::set_current_dir(path).is_err() {
                    println_output(
                        &mut self.stderr,
                        format!("cd: {}: No such file or directory", argument).as_str(),
                    );
                }
            }
            CommandType::External(command) => {
                let mut executable = None;
                for path in env::split_paths(&self.path) {
                    let exe_path = path.join(&command);
                    if exe_path.exists() {
                        executable = Some(exe_path)
                    }
                }

                match executable {
                    Some(_) => {
                        let output = Command::new(command)
                            .args(self.command.arguments.clone())
                            .output()
                            .expect("Unable to run command");
                        print_output(&mut self.stdout, &String::from_utf8_lossy(&output.stdout));
                        print_output(&mut self.stderr, &String::from_utf8_lossy(&output.stderr));
                    }
                    None => println_output(
                        &mut self.stderr,
                        format!("{}: command not found", command).as_str(),
                    ),
                }
            }
        }
    }
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let mut shell = Shell::new();
        shell.parse_command(input);
        shell.execute();
    }
}

fn print_output(output: &mut Box<dyn Write>, message: &str) {
    write!(output, "{}", message).unwrap();
    output.flush().unwrap();
}

fn println_output(output: &mut Box<dyn Write>, message: &str) {
    writeln!(output, "{}", message).unwrap();
    output.flush().unwrap();
}
