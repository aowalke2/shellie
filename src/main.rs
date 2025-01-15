use std::io::{self, Write};
use std::process;

fn main() {
    loop {
        let builtins = ["exit", "echo", "type"];

        print!("$ ");
        io::stdout().flush().unwrap();

        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let tokens = input
            .trim()
            .split(" ")
            .map(|t| t.to_string())
            .collect::<Vec<String>>();
        match tokens[0].as_str() {
            "exit" => {
                if tokens.len() < 2 {
                    println!("exit command expects integer");
                    continue;
                }

                match tokens[1].parse::<i32>() {
                    Ok(code) => process::exit(code),
                    Err(_) => println!("exit command expects integer"),
                }
            }
            "echo" => {
                if tokens.len() < 2 {
                    println!();
                    continue;
                }

                let echo = tokens[1..].join(" ");
                println!("{}", echo);
            }
            "type" => {
                if tokens.len() < 2 {
                    println!("exit command expects argument");
                    continue;
                }

                let builtin = &tokens[1];
                if !builtins.contains(&builtin.as_str()) {
                    println!("{}: not found", builtin);
                    continue;
                }

                println!("{} is a shell builtin", builtin);
            }
            _ => println!("{}: command not found", input.trim()),
        }
    }
}
