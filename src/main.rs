use std::env;
use std::io::{self, Write};
use std::process::{Command, Stdio};

use anyhow::Result;

// Simple enum for builtin result
#[derive(PartialEq)]
enum ShellAction {
    Continue,
    Exit,
}

fn main() -> Result<()> {
    println!("Welcome to lsh!");

    loop {
        // print the prompt
        print!("> ");
        io::stdout().flush().unwrap();

        // Get the input and continue if there's an error
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        // Trim input and skip empty lines
        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // Split the input into command and arguments
        let parts: Vec<&str> = input.split_whitespace().collect();
        let (cmd, args) = parts.split_first().unwrap();

        // Try builtin
        match run_builtin(cmd, args) {
            Some(ShellAction::Exit) => break,
            Some(ShellAction::Continue) => continue,
            None => {}
        }

        // Not a builtin → run external
        run_external(cmd, args);
    }

    Ok(())
}

/// Run an external command (non-builtin)
fn run_external(cmd: &str, args: &[&str]) {
    match Command::new(cmd)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(mut child) => {
            let _ = child.wait();
        }
        Err(err) => {
            eprintln!("error running '{}': {}", cmd, err);
        }
    }
}

/// Handle built-in shell commands
fn run_builtin(cmd: &str, args: &[&str]) -> Option<ShellAction> {
    match cmd {
        "exit" => Some(ShellAction::Exit),
        "cd" => {
            let target = args.get(0).copied().unwrap_or("/");
            if let Err(e) = env::set_current_dir(target) {
                eprintln!("cd: {}", e);
            }
            Some(ShellAction::Continue)
        }
        "pwd" => {
            if let Ok(dir) = env::current_dir() {
                println!("{}", dir.display());
            }
            Some(ShellAction::Continue)
        }
        _ => None,
    }
}
