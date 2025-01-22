use std::{
    env, fs,
    io::{self, Write},
    process::{self, Command},
};

use crate::{
    auto_complete::Trie,
    command::{CommandType, ShellCommand},
};

const BUILTINS: &'static [&'static str] = &["exit", "echo", "type", "pwd", "cd"];

pub struct Shell {
    stdout: Box<dyn Write>,
    stderr: Box<dyn Write>,
    path: String,
    command: ShellCommand,
    auto_complete: Trie,
}

impl Shell {
    pub fn new() -> Shell {
        let path = env::var("PATH").unwrap();
        let mut auto_complete = Trie::default();
        for command in BUILTINS {
            auto_complete.insert(&command);
        }

        Shell {
            stdout: Box::new(io::stdout()),
            stderr: Box::new(io::stderr()),
            path,
            command: ShellCommand::empty(),
            auto_complete,
        }
    }

    pub fn auto_complete(&mut self, input: String) -> Vec<String> {
        self.auto_complete.auto_complete(&input)
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

        if command.is_empty() {
            return;
        }

        self.command = ShellCommand::new(command);
        self.command
            .check_for_redirect(&mut self.stdout, &mut self.stderr);
    }

    pub fn execute(&mut self) {
        match &self.command.command_type() {
            CommandType::Exit => match self.command.arguments().join(" ").parse::<i32>() {
                Ok(code) => process::exit(code),
                Err(_) => println_output(&mut self.stderr, "command expects an integer"),
            },
            CommandType::Echo => println_output(
                &mut self.stdout,
                self.command.arguments().join(" ").as_str(),
            ),
            CommandType::Type => {
                let command = &self.command.arguments().join(" ");

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
                let argument = self.command.arguments().join(" ");
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
                            .args(self.command.arguments().clone())
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
            CommandType::None => {}
        }
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
