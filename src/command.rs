use std::{path::PathBuf, str::FromStr};

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ParserCommandError {
    #[error("Input was empty")]
    EmptyInput,
    #[error("Parsing redirct failed")]
    ParseRedirectFailed,
}

#[derive(Debug, Clone)]
pub enum FileDiscriptor {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RedirectType {
    Append,
    Truncate,
}

#[derive(Debug, Clone)]
pub struct Redirect {
    file_descriptor: FileDiscriptor,
    redirect_type: RedirectType,
    file: PathBuf,
}

impl Redirect {
    pub fn file_descriptor(&self) -> &FileDiscriptor {
        &self.file_descriptor
    }

    pub fn redirect_type(&self) -> &RedirectType {
        &self.redirect_type
    }

    pub fn file(&self) -> &PathBuf {
        &self.file
    }

    pub fn set_file(&mut self, file: PathBuf) {
        self.file = file
    }
}

impl FromStr for Redirect {
    type Err = ParserCommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "2>" => Ok(Redirect {
                file_descriptor: FileDiscriptor::Stderr,
                redirect_type: RedirectType::Truncate,
                file: "".into(),
            }),
            "2>>" => Ok(Redirect {
                file_descriptor: FileDiscriptor::Stderr,
                redirect_type: RedirectType::Append,
                file: "".into(),
            }),
            "1>" | ">" => Ok(Redirect {
                file_descriptor: FileDiscriptor::Stdout,
                redirect_type: RedirectType::Truncate,
                file: "".into(),
            }),
            "1>>" | ">>" => Ok(Redirect {
                file_descriptor: FileDiscriptor::Stdout,
                redirect_type: RedirectType::Append,
                file: "".into(),
            }),
            _ => return Err(ParserCommandError::ParseRedirectFailed),
        }
    }
}

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

impl FromStr for CommandType {
    type Err = ParserCommandError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "exit" => CommandType::Exit,
            "echo" => CommandType::Echo,
            "type" => CommandType::Type,
            "pwd" => CommandType::Pwd,
            "cd" => CommandType::Cd,
            _ => CommandType::External(s.to_string()),
        })
    }
}

pub struct Command {
    name: CommandType,
    arguments: Vec<String>,
    redirect: Option<Redirect>,
}

impl Command {
    pub fn name(&self) -> &CommandType {
        &self.name
    }

    pub fn arguments(&self) -> &Vec<String> {
        &self.arguments
    }

    pub fn redirect(&self) -> &Option<Redirect> {
        &self.redirect
    }
}

pub struct CommandParser {
    input: String,
}

impl CommandParser {
    pub fn new(input: String) -> CommandParser {
        CommandParser { input }
    }

    pub fn parse_command(&mut self) -> Command {
        let mut input_iter = self.input.trim().chars().peekable();
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
            return Command {
                name: CommandType::None,
                arguments: Vec::new(),
                redirect: None,
            };
        }

        let name = CommandType::from_str(command[0].as_str()).unwrap();
        let mut arguments = command[1..].to_vec();
        let redirect = self.try_parse_redirect(&mut arguments);

        Command {
            name,
            arguments,
            redirect,
        }
    }

    fn try_parse_redirect(&mut self, arguments: &mut Vec<String>) -> Option<Redirect> {
        if arguments.len() < 2 {
            return None;
        }

        match Redirect::from_str(&arguments[arguments.len() - 2]) {
            Ok(mut redirect) => {
                let redirect_file = arguments.pop().unwrap();
                redirect.set_file(redirect_file.into());
                arguments.pop().unwrap();
                Some(redirect)
            }
            Err(_) => None,
        }
    }
}
