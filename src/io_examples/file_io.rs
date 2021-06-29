

pub mod file_io {
  use std::io::prelude::*;
  use std::fs::File;

    use std::path::Path;

  pub fn open(p: &str){
    let path = Path::new(p);
    let mut file = match File::open(&path){
      Err(reason) => panic!("couldn't open {}: {}",path.display(),reason.to_string()),
      Ok(f) => f,
    };
    let mut s = String::new();
    match file.read_to_string(&mut s) {
      Err(reason) => panic!("counldn't read {}: {}",path.display(),reason.to_string()),
      Ok(_) => print!("{} contains:\n{}",path.display(),s), 
    };
  }
  pub fn write(){}
}
