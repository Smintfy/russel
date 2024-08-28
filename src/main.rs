use std::io::{self, Write};
use std::{env, process};

fn execute(tokens: Vec<&str>) {
    // Split the tokens into [command] *[arg]
    let cmd = tokens[0];
    let argv = &tokens[1..];

    let builtin_cmds = ["exit", "echo", "type"];

    if builtin_cmds.contains(&cmd) {
        match cmd {
            "exit" => {
                match argv.len() {
                    0 => process::exit(0),
                    1 => {
                        let code = argv[0]
                                    .parse::<i32>()
                                    .expect("Error: Expect exit code to be type of i32");
                        
                        println!("Exited with code {}", code);
                        process::exit(code);
                    },
                    _ => println!("Error: {} takes 1 argument but got {}", cmd, argv.len())
                }
            },
            "echo" => println!("{}", argv.join(" ")),
            "type" => {
                for arg in argv {
                    if builtin_cmds.contains(arg) {
                        println!("{} is a shell builtin", arg);
                    } else {
                        if let Some(env_path) = env::var("PATH")
                                    .unwrap()
                                    .split(':')
                                    .map(|path| format!("{}/{}", path, arg))
                                    .find(|path| std::fs::metadata(path).is_ok())
                        {
                            println!("{} is {}", arg, env_path)
                        } else {
                            println!("type: {}: not found", arg);
                        }
                        
                    }
                }
            }
            _ => unreachable!()
        }
    } else if let Some(program) = env::var("PATH")
                                    .unwrap()
                                    .split(':')
                                    .map(|path| format!("{}/{}", path, cmd))
                                    .find(|path| std::fs::metadata(path).is_ok())
    {
        process::Command::new(program)
            .status()
            .expect("Error: failed to execute program");
    } else {
        println!("Command {} not found", cmd);
    }
}

fn run() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();

        io::stdin().read_line(&mut input).unwrap();
        let command = input.trim();

        if command.is_empty() {
            continue;
        }

        let tokens: Vec<&str> = command.split_whitespace().collect();
        execute(tokens);
    }
}

fn main() {
    run();
}
