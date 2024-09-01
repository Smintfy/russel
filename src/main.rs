use std::io::{self, Error, Write};
use std::{env, process};
use std::path::{Path, PathBuf};
use std::fs;

// TODO: Use enum instead.
const BUILTIN_CMDS: [&str; 5] = ["exit", "echo", "type", "pwd", "cd"];

fn main() {
    run();
}

fn run() {
    const YELLOW: &str = "\x1b[33m";
    const RESET: &str = "\x1b[0m";

    let username = get_command_output("whoami");
    let hostname = get_command_output("hostname");

    loop {
        let home = env::var("HOME").unwrap();
        let raw_cwd = get_cwd();
        let cwd = raw_cwd.to_str().unwrap();

        let alias_home_cwd = cwd.replace(&home, "~");
        let simplified_cwd = cwd.split('/').last().unwrap();

        let git_branch = get_git_branch(&raw_cwd).unwrap();

        let prompt = if git_branch.is_empty() {
            format!("{}{}@{}{}: {}$ ", YELLOW, username.trim(), hostname.trim(), RESET, alias_home_cwd)
        } else {
            format!("{}{}@{}{}: {} {}[{}]{}$ ", YELLOW, username.trim(), hostname.trim(), RESET, simplified_cwd, YELLOW, git_branch, RESET)
        };

        print!("{}", prompt);
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

fn get_command_output(command: &str) -> String {
    let output = process::Command::new(command).output().expect("Failed to execute command");
    String::from_utf8(output.stdout).unwrap()
}

fn get_git_branch(cwd: &PathBuf) -> Result<String, Error> {
    let git_head_path = cwd.join(".git").join("HEAD");

    if git_head_path.exists() {
        let head_content = fs::read_to_string(&git_head_path).unwrap();
        let branch_name = head_content.trim().split('/').last().map(String::from).unwrap();
        Ok(branch_name)
    } else {
        Ok(String::new())
    }
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