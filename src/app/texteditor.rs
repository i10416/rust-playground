// refs
// build own text editor: kilo.
// https://viewsourcecode.org/snaptoken/kilo/
/*
## Objective

antirez's kilo を少し改変したテキストエディタをRustで書く.

## Supported Features

 */

use std::fmt::{Debug, Display};
// user input -> stdin variable -> `program world`
//
use std::io::{self, Read};
use std::os::raw::{c_char, c_uint};
type Cflag = c_uint;
type Speed = c_uint;
const NCCS: usize = 32;

#[repr(C)]
struct Termios {
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

#[link(name = "enable_raw_mode.a")]
extern "C" {
    fn enable_raw_mode() -> Termios;
    fn restore(original: *const Termios) -> i32;
}

fn main() -> Result<(), ()> {
    let original = unsafe { enable_raw_mode() };
    println!("{}", original);
    let buf = String::new();
    match read_rec(buf) {
        Ok(result) => {
            println!("result is {}", result);
            unsafe {
                restore(&original);
            }
            Ok(())
        }
        Err(_) => unimplemented!(),
    }
}
fn read_rec(mut buf: String) -> Result<String, String> {
    match io::stdin().read_to_string(&mut buf) {
        Ok(_) if buf.as_str().contains("q") => Ok(buf),
        Ok(_) => read_rec(buf),
        Err(_) => unimplemented!(),
    }
}

// todo: echo
// note: Rustのプログラムでは必ずCの標準ライブラリがリンクされている
// https://qiita.com/deta-mamoru/items/045c5569ebf2cf39c29e
/*
use std::os::raw::c_int;
extern {
    fn abs(n: c_int) -> c_int;
}
fn main() {
    unsafe {
        println!("{}", abs(-1)); // 1
    }
}
*/
// externブロックの上に#[link(...)]属性を付けることで、特定のライブラリの関数を宣言することができる
/*
#[link(name="libname")]
extern {
    fn fn_name();
}
*/
// user input -> stdin -> do something -> stdout
