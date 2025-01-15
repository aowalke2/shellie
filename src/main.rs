use std::io::{self, Write};
use std::process::Command;
use std::{env, fs, process};

const BUILTINS: &'static [&'static str] = &["exit", "echo", "type"];

fn main() {
    let stdin = io::stdin();
    let path_var = env::var("PATH").expect("PATH variable not found");

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let tokens = input.split_whitespace().collect::<Vec<&str>>();
        match tokens[0] {
            "exit" => {
                if tokens.len() < 2 {
                    println!("exit command expects integer");
                    continue;
                }

                let code = tokens[1]
                    .parse::<i32>()
                    .expect("exit command expects integer");
                process::exit(code);
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

                let command = &tokens[1];
                if BUILTINS.contains(&command) {
                    println!("{} is a shell builtin", command);
                    continue;
                }

                let paths = &mut path_var.split(":");
                if let Some(path) =
                    paths.find(|path| fs::metadata(format!("{path}/{command}")).is_ok())
                {
                    println!("{command} is {path}/{command}");
                    continue;
                }

                println!("{}: not found", command);
            }
            _ => {
                let output = Command::new(tokens[0]).args(tokens[1..].to_vec()).output();
                match output {
                    Ok(output) => {
                        let result = match output.status.success() {
                            true => String::from_utf8_lossy(&output.stdout),
                            false => String::from_utf8_lossy(&output.stderr),
                        };
                        println!("{}", result)
                    }
                    Err(_) => println!("{}: command not found", input.trim()),
                }
            }
        }
    }
}
