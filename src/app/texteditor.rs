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
use std::os::raw::{c_char, c_short, c_uint};

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

#[link(name = "texteditor.a")]
extern "C" {
    fn enable_raw_mode(original: *mut Termios) -> i32;
    fn restore(original: *const Termios) -> i32;
}

#[derive(Debug)]
struct State {
    termios: Termios,
    size: terminal::TermSize,
}

impl State {
    fn new(t: Termios, size: terminal::TermSize) -> State {
        State { termios: t, size: size }
    }
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
    let term_size = terminal::get_terminal_size().unwrap();
    let state = State::new(original, term_size);

    match tick(state) {
        Ok((message, state)) => {
            clean_display();
            unsafe {
                restore(&state.termios);
            }
            Ok(())
        }
        Err(_) => unimplemented!(),
    }
}
fn clean_display() -> () {
    io::stdout().write(&"\x1b[2J\x1b[H".as_bytes()).unwrap();
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

fn tick(state: State) -> Result<(String, State), String> {
    clean_display();
    match io::stdin().bytes().next() {
        Some(Ok(input)) if input == ('q' as u8) & 0x1f => Ok((String::from("Bye!"), state)),
        Some(Ok(_)) => {
            print!(
                "{}",
                build_screen(
                    (0..state.size.row)
                        .into_iter()
                        .map(|_| { build_row(&format!("{:?}", state.size)) })
                        .collect()
                )
            );
            let (cmd, _) = terminal::Cursor::origin();
            io::stdout().write(cmd.as_bytes()).unwrap();
            // disable buffer
            io::stdout().flush().expect("success");
            tick(state)
        }
        Some(Err(_)) => unimplemented!(),
        None => {
            print!(
                "{}",
                build_screen((0..24).into_iter().map(|_| { build_row("") }).collect())
            );
            let (cmd, _) = terminal::Cursor::origin();
            io::stdout().write(cmd.as_bytes()).unwrap();
            io::stdout().flush().expect("success");
            tick(state)
        }
    }
}

mod terminal {
    use std::io::{self, Read, Write};
    use std::os::raw::c_short;

    pub struct Cursor {
        row: usize,
        col: usize,
    }

    impl Cursor {
        pub fn origin() -> (String, Cursor) {
            (String::from("\x1b[H"), Cursor { row: 0, col: 0 })
        }
        pub fn move_by(&self, col: usize, row: usize) -> (String, Cursor) {
            let next = Cursor {
                row: self.row + row,
                col: self.col + col,
            };
            (format!("\x1b[{}C\x1b[{}B", next.row, next.col), next)
        }
    }

    #[derive(Default, Debug)]
    #[repr(C)]
    pub struct TermSize {
        pub row: c_short,
        pub col: c_short,
    }

    #[link(name = "texteditor.a")]
    extern "C" {
        fn terminal_size(size: *mut TermSize) -> i32;
    }
    type ColCount = usize;
    type RowCount = usize;

    // TODO(i10416): https://viewsourcecode.org/snaptoken/kilo/03.rawInputAndOutput.html#window-size-the-hard-way

    pub fn get_terminal_size() -> Option<TermSize> {
        let mut size = TermSize::default();
        match unsafe { terminal_size(&mut size) } {
            0 => Some(size),
            _ => match get_term_size_fallback() {
                Ok((row, col)) => Some(TermSize { row: row, col: col }),
                Err(_) => None,
            },
        }
    }

    pub fn get_term_size_fallback() -> Result<(i16, i16), std::io::Error> {
        let (cmd, _) = Cursor::origin().1.move_by(999, 999);
        // attempt to move cursor at right bottom
        io::stdout().write(cmd.as_bytes())?;

        // query cursor position
        match io::stdout().write("\x1b[6n".as_bytes()) {
            Ok(n) => {
                println!("query: {}", n);
                Ok(read_res().unwrap())
            }
            Err(e) => Err(e),
        }
    }

    fn read_res() -> Option<(i16, i16)> {
        let result = io::stdin()
            .bytes()
            .take_while(|res| match res {
                Ok(u) if u == &b'R' => false,
                Ok(_) => true,
                Err(_) => {
                    println!("something went wrong");
                    false
                }
            })
            .flat_map(|c| c.map(|c| c as char))
            .fold(String::new(), |mut acc, input| acc + &input.to_string());
        println!("{:?}", result);
        result.strip_prefix("\x1b[").and_then(|s| {
            match s.split(";").map(|s| s.parse().unwrap()).collect::<Vec<i16>>()[..2] {
                [row, col] => Some((row, col)),
                _ => None,
            }
        })
    }
}
