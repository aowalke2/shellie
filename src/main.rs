use std::{
    io::{self, Write},
    process,
};

use command::CommandParser;
use shell::Shell;
use termion::{
    clear,
    cursor::{self, DetectCursorPos},
    event::Key,
    input::TermRead,
    raw::IntoRawMode,
};
use trie::{build_trie, Trie};

pub mod command;
pub mod shell;
pub mod trie;

fn main() {
    let trie = build_trie();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let input = handle_input(&trie);

        let command = CommandParser::new(input).parse_command().unwrap();
        let mut shell = Shell::new();
        shell.execute(command);
    }
}

pub fn handle_input(trie: &Trie) -> String {
    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let mut input: Vec<char> = Vec::new();

    for key in io::stdin().keys() {
        match key.unwrap() {
            Key::Ctrl('c') => process::exit(0),
            Key::Backspace => {
                let cursor_position = stdout.cursor_pos().unwrap();
                if cursor_position.0 == 3 {
                    continue;
                }

                write!(stdout, "{}{}", cursor::Left(1), clear::AfterCursor).unwrap();
                stdout.flush().unwrap();
                input.pop().unwrap();
            }

            Key::Char('\t') => {
                if input.is_empty() {
                    continue;
                }

                let prefix = input.iter().collect::<String>();
                let suggestions = trie.search(&prefix);
                if suggestions.is_empty() {
                    continue;
                }

                write!(
                    stdout,
                    "{}{}$ {} ",
                    clear::CurrentLine,
                    cursor::Left(1000),
                    suggestions[0]
                )
                .unwrap();
                input = suggestions[0].chars().collect();
                stdout.flush().unwrap();
            }
            Key::Char('\n') => {
                write!(stdout, "\r\n").unwrap();
                break;
            }
            Key::Char(c) => {
                write!(stdout, "{}", c).unwrap();
                stdout.flush().unwrap();
                input.push(c);
            }
            _ => {}
        }
        stdout.flush().unwrap();
    }

    stdout.suspend_raw_mode().unwrap();
    input.iter().collect()
}
