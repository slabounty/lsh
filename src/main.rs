use std::io::{self, Write};
use std::process::{Command, Stdio};

use anyhow::Result;

mod builtins;
use builtins::{builtins, ShellAction};

mod environment;
use environment::{ShellEnv};

fn main() -> Result<()> {
    println!("Welcome to lsh!");

    let builtins = builtins(); // build table once
    let mut env = ShellEnv::new();

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

        if let Some(builtin) = builtins.get(cmd) {
            match builtin(args, &mut env, &mut std::io::stdout(), &mut std::io::stderr()) {
                ShellAction::Exit => break,
                ShellAction::Continue => continue,
            }
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
