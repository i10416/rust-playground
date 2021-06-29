use std::env;
use playground::io_examples::file_io::file_io;
use playground::io_examples::console_io;
fn main() -> Result<(),String> {
    // note: specify a path from the project root.
    
    // recieve command line args.
    // args[0] is the name of program.
    let args:Vec<String> = env::args().collect();
    let s:&str = &args[1];

    file_io::wc(s).map(|f| print!("{}",f))
}
