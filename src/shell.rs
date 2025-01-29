use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, Write},
    process::{self},
};

use crate::command::{Command, CommandType, FileDiscriptor, RedirectType};

pub const BUILTINS: &'static [&'static str] = &["exit", "echo", "type", "pwd", "cd"];

pub struct Shell {
    stdout: Box<dyn Write>,
    stderr: Box<dyn Write>,
    path: String,
}

impl Shell {
    pub fn new() -> Shell {
        let path = env::var("PATH").unwrap();

        Shell {
            stdout: Box::new(io::stdout()),
            stderr: Box::new(io::stderr()),
            path,
        }
    }

    pub fn execute(&mut self, command: Command) {
        match command.redirect() {
            Some(redirect) => {
                let should_append = *redirect.redirect_type() == RedirectType::Append;
                let file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .append(should_append)
                    .open(redirect.file())
                    .expect("Unable to open file");

                match redirect.file_descriptor() {
                    FileDiscriptor::Stdout => self.stdout = Box::new(file),
                    FileDiscriptor::Stderr => self.stderr = Box::new(file),
                }
            }
            None => {}
        };

        match command.name() {
            CommandType::Exit => match command.arguments().join(" ").parse::<i32>() {
                Ok(code) => process::exit(code),
                Err(_) => println_output(&mut self.stderr, "command expects an integer"),
            },
            CommandType::Echo => {
                println_output(&mut self.stdout, command.arguments().join(" ").as_str())
            }
            CommandType::Type => {
                let command = &command.arguments().join(" ");

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
                let argument = command.arguments().join(" ");
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
            CommandType::External(command_name) => {
                let mut executable = None;
                for path in env::split_paths(&self.path) {
                    let exe_path = path.join(&command_name);
                    if exe_path.exists() {
                        executable = Some(exe_path)
                    }
                }

                match executable {
                    Some(_) => {
                        let output = process::Command::new(command_name)
                            .args(command.arguments().clone())
                            .output()
                            .expect("Unable to run command");
                        print_output(&mut self.stdout, &String::from_utf8_lossy(&output.stdout));
                        print_output(&mut self.stderr, &String::from_utf8_lossy(&output.stderr));
                    }
                    None => println_output(
                        &mut self.stderr,
                        format!("{}: command not found", command_name).as_str(),
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
