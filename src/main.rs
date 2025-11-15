use std::fs;

use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

mod builtins;
use builtins::{builtins, ShellAction, BuiltinMap};

mod environment;
use environment::{ShellEnv};

mod welcome;
use welcome::print_welcome;

mod command_processor;
use command_processor::handle_command;

mod external;

fn main() -> Result<()> {
    // Print our welcome message.
    print_welcome(&mut std::io::stdout());

    // Create our line editor
    let mut rl = DefaultEditor::new()?;

    // Set up our history with either and existing file
    // or create a new one.
    let history_path = "history.txt";
    match rl.load_history(history_path) {
        Ok(_) => {}
        Err(ReadlineError::Io(_)) => {
            // History file doesn't exist, create it
            fs::File::create(history_path)?;
        }
        Err(err) => {
            eprintln!("minishell: Error loading history: {}", err);
        }
    }

    // Create our builtin table and our shell environment.
    let builtins = builtins(); // build table once
    let mut env = ShellEnv::new();

    // Call our repl loop. This'll run until we get either
    // and exit or cntl-C/cntl-D
    repl(&mut env, &builtins, &mut rl)?;

    // Save our history for next time.
    rl.save_history(history_path)?;

    // Exit the shell
    println!("Exiting lsh");

    Ok(())
}

fn repl(env: &mut ShellEnv, builtins: &BuiltinMap, rl_editor: &mut DefaultEditor) -> rustyline::Result<()>  {
    loop {
        let readline = rl_editor.readline(">> ");
        match readline {
            Ok(input) => {
                {
                    rl_editor.add_history_entry(input.as_str())?;
                }

                if handle_command(&input, env, builtins) == ShellAction::Exit {
                    break;
                }

            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }

    Ok(())
}
