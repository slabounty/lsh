use std::io::{self, Write};

use anyhow::Result;

fn main() -> Result<()> {
    println!("Hello, world!");

    print!("> ");
    let _ = io::stdout().flush();

    let mut input_line = String::new();
    io::stdin()
        .read_line(&mut input_line)?;

    execute(&input_line)?;

    Ok(())
}

fn execute(input_line: &str) -> Result<()> {
    println!("executing = {}", input_line);

    Ok(())
}
