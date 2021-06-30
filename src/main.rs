use playground::io_examples::console_io;
use playground::io_examples::file_io::file_io;
use std::env;
fn main() -> Result<(), String> {
    // note: specify a path from the project root.

    // recieve command line args.
    let args: Vec<String> = env::args().collect();
    let filenames = args.iter().skip(1);
    filenames.for_each(|name| {
        match file_io::wc(name) {
            Ok(s) => println!("{}", s),
            Err(_) => println!("err"),
        };
    });
    Ok(())
}
