use std::io::{self, Write};
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

        let simplified_cwd = get_simplified_cwd(&raw_cwd, &home);
        let git_branch = get_git_branch(&raw_cwd).unwrap_or_default();

        let prompt = if git_branch.is_empty() {
            format!("{}{}@{}{}: {}$ ", YELLOW, username.trim(), hostname.trim(), RESET, simplified_cwd)
        } else {
            format!("{}{}@{}{}: {} {}[{}]{}$ ", YELLOW, username.trim(), hostname.trim(), RESET, simplified_cwd, YELLOW, git_branch, RESET)
        };

        print!("{}", prompt);
        io::stdout().flush().unwrap();

        let mut source = String::new();
        io::stdin().read_line(&mut source).unwrap();

        let input = source.trim();
        
        if input.is_empty() {
            continue;
        }

        let tokens = tokenize(&input);
        execute(tokens);
    }
}

fn tokenize(source: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut inside_quotes = false;

    for c in source.chars() {
        match c {
            '"' => {
                inside_quotes = !inside_quotes;
                current_token.push(c);
            },
            ' ' if !inside_quotes => {
                if !current_token.is_empty() {
                    tokens.push(current_token.trim().to_string());
                    current_token.clear();
                }
            },
            _ => current_token.push(c),
        }
    }

    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    tokens
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

fn get_git_root(path: &Path) -> Option<PathBuf> {
    let mut current = path.to_path_buf();
    loop {
        if current.join(".git").is_dir() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn get_git_branch(cwd: &Path) -> Result<String, io::Error> {
    if let Some(git_root) = get_git_root(cwd) {
        let git_head_path = git_root.join(".git").join("HEAD");
        let head_content = fs::read_to_string(git_head_path)?;
        Ok(head_content
            .trim()
            .strip_prefix("ref: refs/heads/")
            .unwrap_or("")
            .to_string())
    } else {
        Ok(String::new())
    }
}

fn get_simplified_cwd(cwd: &Path, home: &str) -> String {
    if let Some(git_root) = get_git_root(cwd) {
        let relative = cwd.strip_prefix(&git_root).unwrap_or(cwd);
        if relative.as_os_str().is_empty() {
            git_root.file_name().unwrap().to_str().unwrap().to_string()
        } else {
            format!("{}/{}",
                    git_root.file_name().unwrap().to_str().unwrap(),
                    relative.to_str().unwrap())
        }
    } else {
        cwd.to_str().unwrap().replace(home, "~")
    }
}

fn execute(tokens: Vec<String>) {
    let (cmd, args) = tokens.split_first().unwrap();

    if BUILTIN_CMDS.contains(&cmd.as_str()) {
        match cmd.as_str() {
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

fn execute_exit(args: &[String]) {
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

fn execute_echo(args: &[String]) {
    println!("{}", args.join(" "));
}

fn execute_type(args: &[String]) {
    for arg in args {
        if BUILTIN_CMDS.contains(&arg.as_str()) {
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

fn execute_cd(args: &[String]) {
    let home = env::var("HOME").unwrap();
    let mut target_dir = args[0].to_string();

    if target_dir.starts_with('~') {
        target_dir = target_dir.replacen("~", &home, 1);
    }

    if std::env::set_current_dir(Path::new(&target_dir)).is_err() {
        println!("cd: {}: No such file or directory", args[0]);
    }
}