use std::io::{self, Write};
use std::{env, process};

const BUILTIN_CMDS: [&str; 3] = ["exit", "echo", "type"];

fn main() {
    run();
}

fn run() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let source = input.trim();
        
        if source.is_empty() {
            continue;
        }

        let sequence: Vec<&str> = source.split_whitespace().collect();
        execute(sequence);
    }
}

fn find_cmd_path(cmd: &str) -> Option<String> {
    env::var("PATH").unwrap().split(':')
        .map(|path| format!("{}/{}", path, cmd))
        .find(|path| std::fs::metadata(path).is_ok())
}

fn execute(tokens: Vec<&str>) {
    let (cmd, args) = tokens.split_first().unwrap();

    if BUILTIN_CMDS.contains(cmd) {
        match *cmd {
            "exit" => execute_exit(args),
            "echo" => execute_echo(args),
            "type" => execute_type(args),
            _ => unreachable!()
        }
    } else if let Some(program) = find_cmd_path(cmd) {
        process::Command::new(program).status().expect("Error: failed to execute program");
    } else {
        println!("Command {} not found", cmd);
    }
}

fn execute_exit(args: &[&str]) {
    match args.len() {
        0 => process::exit(0),
        1 => {
            let code = args[0].parse::<i32>().expect("Error: numeric argument required");
            println!("Exited with code {:?}", code);
            process::exit(code);
        },
        _ => println!("Error: exit takes 1 argument but got {}", args.len())
    }
}

fn execute_echo(args: &[&str]) {
    println!("{}", args.join(" "));
}

fn execute_type(args: &[&str]) {
    for arg in args {
        if BUILTIN_CMDS.contains(arg) {
            println!("{} is a shell builtin", arg);
        } else if let Some(env_path) = find_cmd_path(arg) {
            println!("{} is {}", arg, env_path)
        } else {
            println!("type: {}: not found", arg);
        }
    }
}