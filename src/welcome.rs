use std::io::Write;

pub fn print_welcome(out: &mut dyn Write) {
    writeln!(out,
        r"
         _            _            _       _
        _\ \         / /\         / /\    / /\
       /\__ \       / /  \       / / /   / / /
      / /_ \_\     / / /\ \__   / /_/   / / /
     / / /\/_/    / / /\ \___\ / /\ \__/ / /
    / / /         \ \ \ \/___// /\ \___\/ /
   / / /           \ \ \     / / /\/___/ /
  / / / ____   _    \ \ \   / / /   / / /
 / /_/_/ ___/\/_/\__/ / /  / / /   / / /
/_______/\__\/\ \/___/ /  / / /   / / /
\_______\/     \_____\/   \/_/    \/_/
"
    ).unwrap();
    writeln!(out, "Welcome to lsh (pronounced leash)! Type 'exit' to quit.\n").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_welcome() {
        let mut buf = Vec::new();
        let _result = print_welcome(&mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Welcome"));
    }
}
