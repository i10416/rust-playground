// refs
// build own text editor: kilo.
// https://viewsourcecode.org/snaptoken/kilo/
/*
## Objective

antirez's kilo を少し改変したテキストエディタをRustで書く.

## Supported Features

 */

use std::fmt::Debug;

// user input -> stdin variable -> `program world`
use std::io::{self, Read, Write};
use std::os::raw::{c_char, c_uint};

type Cflag = c_uint;
type Speed = c_uint;
const NCCS: usize = 32;
// メモリレイアウトの最適化をしないようにする
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
    }
    .unwrap();

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
fn clean_display() -> () {
    io::stdout().write(&"\x1b[2J\x1b[H".as_bytes()).unwrap();
}

struct Cursor {
    row: usize,
    col: usize,
}

impl Cursor {
    fn origin() -> (String, Cursor) {
        (String::from("\x1b[H"), Cursor { row: 0, col: 0 })
    }
    fn move_by(&self, x: usize, y: usize) -> (String, Cursor) {
        unimplemented!()
    }
}

fn build_row(content: &str) -> String {
    String::from(format!("~ {}", content))
}
fn build_screen(rows: Vec<String>) -> String {
    rows.into_iter().fold(String::new(), |acc, s| acc + &s + "\r\n")
}
// todo: render_screen(previous_state,reducer)
fn render_screen(rows: Vec<String>) {
    rows.into_iter().zip((0..)).for_each(|(row, row_number)| {})
}

fn read_rec() -> Result<String, String> {
    clean_display();
    match io::stdin().bytes().next() {
        Some(Ok(input)) if input == ('q' as u8) & 0x1f => Ok(String::from("exit!")),
        Some(Ok(_)) => {
            print!(
                "{}",
                build_screen((0..24).into_iter().map(|_| { build_row("") }).collect())
            );
            let (cmd, _) = Cursor::origin();
            io::stdout().write(cmd.as_bytes()).unwrap();
            // disable buffer
            io::stdout().flush().expect("success");
            read_rec()
        }
        Some(Err(_)) => unimplemented!(),
        None => {
            print!(
                "{}",
                build_screen((0..24).into_iter().map(|_| { build_row("") }).collect())
            );
            let (cmd, _) = Cursor::origin();
            io::stdout().write(cmd.as_bytes()).unwrap();
            io::stdout().flush().expect("success");
            read_rec()
        }
    }
}
