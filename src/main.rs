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
        let mut stdout = io::stdout();
        write!(stdout, "{}{}$ ", cursor::Left(1000), clear::CurrentLine).unwrap();
        stdout.flush().unwrap();

        let input = handle_input(&trie);

        let command = CommandParser::new(input).parse_command();
        let mut shell = Shell::new();
        shell.execute(command);
    }
}

// clean up line reset
pub fn handle_input(trie: &Trie) -> String {
    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let mut input: Vec<char> = Vec::new();
    let mut bell_rung = false;

    for key in io::stdin().keys() {
        match key.unwrap() {
            Key::Ctrl('c') => {
                stdout.suspend_raw_mode().unwrap();
                process::exit(0)
            }
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
                let mut suggestions = trie.search(&prefix);
                suggestions.sort();
                match suggestions.len() {
                    0 => {
                        write!(stdout, "\x07").unwrap();
                        stdout.flush().unwrap();
                        continue;
                    }
                    1 => {
                        write!(
                            stdout,
                            "{}{}$ {} ",
                            clear::CurrentLine,
                            cursor::Left(input.len() as u16 + 2),
                            suggestions[0]
                        )
                        .unwrap();
                        input = suggestions[0].chars().collect();
                        input.push(' ');
                        stdout.flush().unwrap();
                    }
                    _ => {
                        if bell_rung {
                            bell_rung = false;
                            write!(
                                stdout,
                                "\r\n{}\r\n$ {}",
                                suggestions.join("  "),
                                input.iter().collect::<String>()
                            )
                            .unwrap();
                            stdout.flush().unwrap();
                            continue;
                        }

                        bell_rung = true;
                        write!(stdout, "\x07").unwrap();
                        stdout.flush().unwrap();
                        continue;
                    }
                }
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

    input.iter().collect()
}
