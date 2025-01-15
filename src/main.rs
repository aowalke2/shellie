use std::collections::HashMap;
use std::io::{self, Write};
use std::{env, fs, process};

const BUILTINS: &'static [&'static str] = &["exit", "echo", "type"];

fn main() {
    let path_var = env::var("PATH").expect("PATH variable not found");
    let mut executables = HashMap::new();
    let paths = path_var.split(":").collect::<Vec<&str>>();
    println!("paths: {:?}", paths);
    for directory_path in paths {
        if let Ok(directory) = fs::read_dir(directory_path) {
            for entry in directory {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                        executables
                            .entry(file_name.to_string())
                            .or_insert(vec![])
                            .push(directory_path);
                    }
                }
            }
        }
    }

    loop {
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

                let argument = &tokens[1];
                if BUILTINS.contains(&argument.as_str()) {
                    println!("{} is a shell builtin", argument);
                    continue;
                }

                if let Some(path) = executables.get(argument) {
                    println!("{} is {:?}", argument, path);
                    continue;
                }
                println!("{}: not found", argument);
            }
            _ => println!("{}: command not found", input.trim()),
        }
    }
}
