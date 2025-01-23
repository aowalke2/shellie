use std::io::{self, Write};

use codecrafters_shell::shell::Shell;

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
