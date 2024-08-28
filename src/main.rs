use std::io::{self, Write};
use std::process;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();

        io::stdin().read_line(&mut input).unwrap();
        let command = input.trim();

        if command.is_empty() {
            continue;
        }

        let token: Vec<&str> = command.split_whitespace().collect();

        match token[..] {
            ["exit"] => process::exit(0),
            ["exit", code] => {
                println!("Exit with code {}", code);
                process::exit(code.parse::<i32>()
                                        .expect("Error: Expect exit code to be type of i32"));
            },
            ["echo", ..] => println!("{}", token[1..].join(" ")),
            _ => println!("Command {} not found", command),
        };
    }
}
