// refs
// build own text editor: kilo.
// https://viewsourcecode.org/snaptoken/kilo/
/*
## Objective

antirez's kilo を少し改変したテキストエディタをRustで書く.

## Supported Features

 */

use std::borrow::BorrowMut;
use std::fmt::{Debug, Display};

// user input -> stdin variable -> `program world`
use std::io::{self, Read, Write};
use std::mem::size_of;
use std::os::raw::{c_char, c_uint};
use std::ptr::{null, NonNull};


type Cflag = c_uint;
type Speed = c_uint;
const NCCS: usize = 32;
#[repr(C)]
pub struct Termios {
    c_iflag: Cflag,       /* input mode flags */
    c_oflag: Cflag,       /* output mode flags */
    c_cflag: Cflag,       /* control mode flags */
    c_lflag: Cflag,       /* local mode flags */
    c_line: c_char,       /* line discipline */
    c_cc: [c_char; NCCS], /* control characters */
    c_ispeed: Speed,      /* input speed */
    c_ospeed: Speed,
}

impl Debug for Termios {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Termios")
            .field("c_iflag", &self.c_iflag)
            .field("c_oflag", &self.c_oflag)
            .field("c_cflag", &self.c_cflag)
            .field("c_lflag", &self.c_lflag)
            .field("c_line", &self.c_line)
            .field("c_cc", &self.c_cc)
            .field("c_ispeed", &self.c_ispeed)
            .field("c_ospeed", &self.c_ospeed)
            .finish()
    }
}
impl std::fmt::Display for Termios {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Termios({},{},{},{},{})",
            self.c_cflag, self.c_iflag, self.c_ispeed, self.c_ospeed, self.c_lflag
        )
    }
}

impl Default for Termios {
    fn default() -> Self {
        Self {
            c_iflag: Default::default(),
            c_oflag: Default::default(),
            c_cflag: Default::default(),
            c_lflag: Default::default(),
            c_line: Default::default(),
            c_cc: Default::default(),
            c_ispeed: Default::default(),
            c_ospeed: Default::default(),
        }
    }
}

#[link(name = "enable_raw_mode.a")]
extern "C" {
    fn enable_raw_mode(priginal: *mut Termios) -> i32;
    fn restore(original: *const Termios) -> i32;
}

fn main() -> Result<(), ()> {
    let original = unsafe {
        let mut t = Termios::default();
        if enable_raw_mode(&mut t) == 0 {
            Some(t)
        } else {
            None
        }
    }.unwrap();

    match read_rec() {
        Ok(result) => {
            println!("{}", result);
            unsafe {
                restore(&original);
            }
            Ok(())
        }
        Err(_) => unimplemented!(),
    }
}
fn read_rec() -> Result<String, String> {
    match io::stdin().bytes().next() {
        Some(Ok(b)) if (b as char) == 'q' => Ok(String::from("exit!")),
        Some(Ok(b)) if (b as char).is_control() => {
            print!("{}", b as char);
            // disable buffer
            io::stdout().flush().expect("success");
            read_rec()
        }
        Some(Ok(b)) => {
            print!("{}", b as char);
            // disable buffer
            io::stdout().flush().expect("success");
            read_rec()
        }
        Some(Err(_)) => unimplemented!(),
        None => {
            io::stdout().flush().expect("success");
            read_rec()
        }
    }
}
