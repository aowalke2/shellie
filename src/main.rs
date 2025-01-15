#[allow(unused_imports)]
use std::io::{self, Write};
use std::process;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        match input.trim() {
            input if input.starts_with("exit") => {
                let tokens = input.split(" ").collect::<Vec<&str>>();
                if tokens.len() < 2 {
                    println!("exit command expects integer");
                    continue;
                }

                match tokens[1].parse::<i32>() {
                    Ok(code) => process::exit(code),
                    Err(_) => println!("exit command expects integer"),
                }
            }
            _ => println!("{}: command not found", input.trim()),
        }
    }
}
