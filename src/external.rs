use std::process::{Command, Stdio};

use crate::builtins::{ShellAction};
use crate::environment::ShellEnv;

/// Run an external command (non-builtin)
pub fn run_external(cmd: &str, args: &[&str], env: &ShellEnv) -> ShellAction {
    match Command::new(cmd)
        .args(args)
        .env_clear()      // <-- clear inherited env first
        .envs(&env.vars)  // ← Send our environment
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
    };

    ShellAction::Continue
}
