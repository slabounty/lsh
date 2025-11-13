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
        let expanded_args = expand_args(&args, &env);

        if let Some(builtin) = builtins.get(cmd) {
            match builtin(&as_str_vec(&expanded_args), &mut env, &mut std::io::stdout(), &mut std::io::stderr()) {
                ShellAction::Exit => break,
                ShellAction::Continue => continue,
            }
        }

        // Not a builtin → run external
        run_external(cmd, &as_str_vec(&expanded_args), &env);
    }

    Ok(())
}

fn as_str_vec(strings: &[String]) -> Vec<&str> {
    strings.iter().map(|s| s.as_str()).collect::<Vec<_>>()
}

/// Use the environment to expand our argument list
fn expand_args(args: &[&str], env: &ShellEnv) -> Vec<String> {
    let mut expanded_args = Vec::new();

    for arg in args  {
        if arg.starts_with("$") {
            if let Some(expanded_arg) = env.get_var(&arg[1..]) {
                expanded_args.push(expanded_arg.to_string());
            }
            else {
                expanded_args.push(arg.to_string());
            }

        }
        else {
            expanded_args.push(arg.to_string());
        }
    }

    expanded_args
}

/// Run an external command (non-builtin)
fn run_external(cmd: &str, args: &[&str], env: &ShellEnv) {
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
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expanded_args_no_dollar() {
        let env = ShellEnv::empty();
        let args = ["hello"];
        let expanded_args = expand_args(&args, &env);

        assert_eq!(expanded_args.len(), args.len());
        assert_eq!(expanded_args[0], "hello");
    }

    #[test]
    fn test_expanded_args_dollar_no_match() {
        let env = ShellEnv::empty();
        let args = ["$HELLO"];
        let expanded_args = expand_args(&args, &env);

        assert_eq!(expanded_args.len(), args.len());
        assert_eq!(expanded_args[0], "$HELLO");
    }

    #[test]
    fn test_expanded_args_dollar_match() {
        let mut env = ShellEnv::empty();
        let args = ["$HELLO"];
        env.set_var("HELLO", "world");
        let expanded_args = expand_args(&args, &env);

        assert_eq!(expanded_args.len(), args.len());
        assert_eq!(expanded_args[0], "world");
    }

    #[test]
    fn test_expanded_args_mixed() {
        let mut env = ShellEnv::empty();
        env.set_var("HELLO", "world");
        env.set_var("THERE", "Rust");

        let args = ["say", "$HELLO", "to", "$THERE"];

        let expanded_args = expand_args(&args, &env);

        assert_eq!(expanded_args.len(), args.len());
        assert_eq!(expanded_args, ["say", "world", "to", "Rust"]);
    }

    #[test]
    fn test_as_str_vec_basic() {
        let strings = vec!["hello".to_string(), "world".to_string()];
        let str_refs = as_str_vec(&strings);

        assert_eq!(str_refs, vec!["hello", "world"]);
    }

    #[test]
    fn test_as_str_vec_preserves_references() {
        let strings = vec!["foo".to_string(), "bar".to_string()];
        let str_refs = as_str_vec(&strings);

        // Compare memory addresses to ensure they point to same data
        assert!(std::ptr::eq(str_refs[0].as_ptr(), strings[0].as_ptr()));
        assert!(std::ptr::eq(str_refs[1].as_ptr(), strings[1].as_ptr()));
    }

    #[test]
    fn test_as_str_vec_empty() {
        let strings: Vec<String> = Vec::new();
        let str_refs = as_str_vec(&strings);

        assert!(str_refs.is_empty());
    }
}
