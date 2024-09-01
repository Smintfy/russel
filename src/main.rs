use std::io::{self, Write};
use std::{env, process};
use std::path::{Path, PathBuf};

// TODO: Use enum instead.
const BUILTIN_CMDS: [&str; 5] = ["exit", "echo", "type", "pwd", "cd"];

fn main() {
    run();
}

fn run() {
    const YELLOW: &str = "\x1b[33m";
    const RESET: &str = "\x1b[0m";

    let username = String::from_utf8(
        process::Command::new("whoami").output().unwrap().stdout
    ).unwrap();
    let hostname = String::from_utf8(
        process::Command::new("hostname").output().unwrap().stdout
    ).unwrap();

    loop {
        let cwd = get_cwd();
        let simplified_cwd = cwd.to_str().unwrap().split('/').last().unwrap();

        print!("{}{}@{}{}: {}$ ", YELLOW, username.trim(), hostname.trim(), RESET, simplified_cwd.replace("\"", ""));
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

fn get_cwd() -> PathBuf {
    env::current_dir().unwrap()
}

fn execute(tokens: Vec<&str>) {
    let (cmd, args) = tokens.split_first().unwrap();

    if BUILTIN_CMDS.contains(cmd) {
        match *cmd {
            "exit" => execute_exit(args),
            "echo" => execute_echo(args),
            "type" => execute_type(args),
            "pwd" => execute_pwdir(),
            "cd" => execute_cd(args),
            _ => unreachable!()
        }
    } else if let Some(program) = find_cmd_path(cmd) {
        process::Command::new(program)
            .args(args)
            .status()
            .expect("Error: failed to execute program");
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

fn execute_pwdir() {
    println!("{}", get_cwd().display());
}

fn execute_cd(args: &[&str]) {
    let home = env::var("HOME").unwrap();
    let mut target_dir = args[0].to_string();

    if target_dir.starts_with('~') {
        target_dir = target_dir.replacen("~", &home, 1);
    }

    if std::env::set_current_dir(Path::new(&target_dir)).is_err() {
        println!("cd: {}: No such file or directory", args[0]);
    }
}