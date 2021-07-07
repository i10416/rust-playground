/*
## What is a shell?
A shell is a program which allows you to control your computer.
It does this largely by making it easy to launch other applications.
*/
pub mod core {
    use std::{
        env,
        io::{stdin, stdout, Write},
        path::Path,
        process::Command,
    };

    pub fn main() {
        loop {
            print!("> ");
            match stdout().flush() {
                Ok(()) => (),
                Err(e) => return println!("Err: Failed to flash stdout.\n{}", e),
            };
            let mut input = String::new();
            let args_opt = if let Ok(n) = stdin().read_line(&mut input) {
                println!("Got {}: {} bytes", input, n);
                Some(input.trim().split_whitespace())
            } else {
                None
            };
            if let (Some(cmd), Some(args)) = match args_opt {
                Some(mut iter) => (iter.next(), Some(iter)),
                _ => (None, None),
            } {
                match cmd {
                    "cd" => {
                        let new_dir = args.peekable().peek().map_or("/", |x| *x);
                        let root = Path::new(new_dir);
                        if let Err(e) = env::set_current_dir(&root) {
                            eprintln!("{}", e);
                        }
                    }
                    _ => {
                        if let Ok(mut child) = Command::new(cmd).args(args).spawn() {
                            println!("Command: {:?} started successfully.", child);
                            child.wait().expect("Failed to run command");
                        } else {
                            eprintln!("Failed to start command.");
                        }
                    }
                }
            } else {
                eprintln!("Invalid Input")
            };
        }
    }
}
