use std::io::{self, Write};

use shell::Shell;

pub mod command;
pub mod shell;
pub mod trie;

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
