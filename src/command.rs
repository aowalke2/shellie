use std::{fs::OpenOptions, io::Write};

#[derive(Debug, Clone)]
pub enum CommandType {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
    External(String),
    None,
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

    pub fn command_type(&self) -> &CommandType {
        &self.command_type
    }

    pub fn arguments(&self) -> &Vec<String> {
        &self.arguments
    }

    pub fn empty() -> ShellCommand {
        ShellCommand {
            command_type: CommandType::None,
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
