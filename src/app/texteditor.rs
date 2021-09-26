// refs
// build own text editor: kilo.
// https://viewsourcecode.org/snaptoken/kilo/
/*
## Objective

antirez's kilo を少し改変したテキストエディタをRustで書く.

 */

use std::fmt::Debug;
use std::io::{self, Read, Write};
use std::os::raw::{c_char, c_uint};

type Cflag = c_uint;
type Speed = c_uint;
const NCCS: usize = 32;
const KILO_VERSION: &str = "0.0.0";
// prevent memory layout optimization
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
    cursor: terminal::Cursor,
    size: terminal::TermSize,
}

impl State {
    fn new(t: Termios, size: terminal::TermSize, cursor: terminal::Cursor) -> State {
        State {
            termios: t,
            size: size,
            cursor: cursor,
        }
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
    let (_, cursor) = terminal::Cursor::origin();
    io::stdout()
        .write(clean_display().as_bytes())
        .and_then(|_| io::stdout().flush())
        .expect("success");
    let state = State::new(original, term_size, cursor);

    match tick(state) {
        Ok((message, state)) => {
            let clean_cmd = clean_display();
            unsafe {
                restore(&state.termios);
            }
            println!("{}", clean_cmd + &message);
            Ok(())
        }
        Err(_) => unimplemented!(),
    }
}
fn clean_display() -> String {
    String::from("\x1b[2J")
}

fn build_row(content: &str, idx: usize) -> String {
    format!("~{}:{} {}", idx, content, clean_line())
}
fn build_screen(rows: Vec<String>) -> String {
    let last = rows.len();
    let cleaner = clean_line();
    rows.into_iter().zip(1..).fold(String::new(), |acc, (s, i)| {
        acc + &s + &(if i == last { "" } else { "\r\n" })
    })
}
// todo: render_screen(previous_state,reducer)
fn render_screen(rows: Vec<String>) {
    rows.into_iter().zip(1..).for_each(|(row, row_number)| {})
}
fn clean_line() -> String {
    String::from("\x1b[K")
}

fn build_welcome_message(col_count: u16, row_number: usize) -> String {
    match format!("Kilo Editor -- version {}", KILO_VERSION) {
        s if s.len() > col_count.into() => s[..col_count.into()].to_string() + &clean_line(),
        s => {
            format!("~{}:", row_number)
                + &" ".repeat((col_count as usize) / 2 - s.len() / 2 - 3)
                + &s
                + &" ".repeat((col_count as usize) / 2 - s.len() / 2)
                + &clean_line()
        }
    }
}

fn key_to_move(c: u8) -> (i32, i32) {
    match c as char {
        'k' => (0, -1),
        'j' => (0, 1),
        'h' => (-1, 0),
        'l' => (1, 0),
        _ => (0, 0),
    }
}

fn arrow_key_to_move(c: u8) -> (i32, i32) {
    match c as char {
        'A' => (0, -1),
        'B' => (0, 1),
        'C' => (1, 0),
        'D' => (-1, 0),
        _ => (0, 0),
    }
}

fn tick(state: State) -> Result<(String, State), String> {
    let (hide_cursor_cmd, cursor) = state.cursor.hide();
    let (set_cursor_cmd, _) = cursor.move_to(0, 0);
    let render_textarea_cmd = build_screen(
        (0..state.size.row)
            .into_iter()
            .zip(1..)
            .map(|(row, idx)| build_row("", idx))
            .collect(),
    );
    match io::stdin().bytes().next() {
        Some(Ok(input)) if input == b'q' & 0x1f => Ok((cursor.move_to(0, 0).0 + &clean_display() + "Bye!", state)),
        // handle \x1b[A,\x1b[B,\x1b[C,\x1b[D
        Some(Ok(b'\x1b')) => {
            match io::stdin().bytes().peekable().peek() {
                Some(Ok(u)) if u == &b'[' => match io::stdin().bytes().next() {
                    Some(Ok(input @ (b'k' | b'j' | b'l' | b'h'))) => {
                        let (dx, dy) = arrow_key_to_move(input);
                        let (_, cursor) = cursor.move_by(dx, dy);
                        let (show_cursor_cmd, cursor) = cursor.show();
                        io::stdout()
                            .write(
                                (hide_cursor_cmd + &set_cursor_cmd + &render_textarea_cmd + &show_cursor_cmd)
                                    .as_bytes(),
                            )
                            .unwrap();
                        // disable buffer
                        io::stdout().flush().expect("success");
                        tick(State {
                            size: state.size,
                            cursor: cursor,
                            termios: state.termios,
                        })
                    }
                    Some(Ok(b'3')) => {
                        match io::stdin().bytes().peekable().peek() {
                            Some(Ok(b'~')) => {
                                let (show_cursor_cmd, cursor) = cursor.show();
                                io::stdout()
                                    .write(
                                        (hide_cursor_cmd + &set_cursor_cmd + &render_textarea_cmd + &show_cursor_cmd)
                                            .as_bytes(),
                                    )
                                    .unwrap();
                                // disable buffer
                                io::stdout().flush().expect("success");
                                tick(State {
                                    size: state.size,
                                    cursor: cursor,
                                    termios: state.termios,
                                })
                            }
                            _ => unimplemented!(),
                        }
                    }
                    _ => {
                        unimplemented!()
                    }
                },
                Some(Ok(_)) => unimplemented!(),
                Some(Err(_)) => unimplemented!(),
                None => unimplemented!(),
            }
        }
        Some(Ok(input)) => {
            let (dx, dy) = key_to_move(input);
            let (_, cursor) = cursor.move_by(dx, dy);
            let (show_cursor_cmd, cursor) = cursor.show();
            io::stdout()
                .write((hide_cursor_cmd + &set_cursor_cmd + &render_textarea_cmd + &show_cursor_cmd).as_bytes())
                .unwrap();
            // disable buffer
            io::stdout().flush().expect("success");
            tick(State {
                size: state.size,
                cursor: cursor,
                termios: state.termios,
            })
        }
        Some(Err(_)) => unimplemented!(),
        None => {
            let textarea = build_screen(
                (0..state.size.row)
                    .into_iter()
                    .into_iter()
                    .zip(1..)
                    .map(|(_, idx)| {
                        if idx == 6 {
                            build_welcome_message(state.size.col, idx)
                        } else {
                            build_row("", idx)
                        }
                    })
                    .collect(),
            );

            let (show_cursor_cmd, cursor) = cursor.show();
            io::stdout()
                .write((hide_cursor_cmd + &set_cursor_cmd + &textarea + &show_cursor_cmd).as_bytes())
                .unwrap();
            io::stdout().flush().expect("success");
            tick(State {
                size: state.size,
                cursor: cursor,
                termios: state.termios,
            })
        }
    }
}

mod terminal {
    use std::io::{self, Read, Write};
    use std::os::raw::c_ushort;

    #[derive(Debug)]
    pub struct Cursor {
        row: usize,
        col: usize,
        visibility: bool,
    }

    impl Cursor {
        pub fn origin() -> (String, Cursor) {
            (
                String::from("\x1b[H"),
                Cursor {
                    row: 0,
                    col: 0,
                    visibility: true,
                },
            )
        }
        pub fn move_by(&self, col: i32, row: i32) -> (String, Cursor) {
            self.move_to(
                std::cmp::max(self.col as i32 + col, 0) as usize,
                std::cmp::max(self.row as i32 + row, 0) as usize,
            )
            /*let next = Cursor {
                row: std::cmp::max(self.row as i32 + row, 0) as usize,
                col: std::cmp::max(self.col as i32 + col, 0) as usize,
                visibility: self.visibility,
            };*/
            //(format!("\x1b[{}C\x1b[{}B", next.col, next.row), next)
        }

        pub fn move_to(&self, col: usize, row: usize) -> (String, Cursor) {
            let next = Cursor {
                row: row,
                col: col,
                visibility: self.visibility,
            };
            let cmd = format!("\x1b[{};{}H", next.row, next.col);
            (cmd, next)
        }

        pub fn hide(&self) -> (String, Cursor) {
            (
                format!("\x1b[?25l"),
                Cursor {
                    row: self.row,
                    col: self.col,
                    visibility: false,
                },
            )
        }
        pub fn show(&self) -> (String, Cursor) {
            let (cmd, cursor) = self.move_to(self.col, self.row);
            (
                cmd + &format!("\x1b[?25h"),
                Cursor {
                    col: cursor.col,
                    row: cursor.row,
                    visibility: true,
                },
            )
        }
    }

    #[derive(Default, Debug)]
    #[repr(C)]
    pub struct TermSize {
        pub row: c_ushort,
        pub col: c_ushort,
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

    pub fn get_term_size_fallback() -> Result<(u16, u16), std::io::Error> {
        let (cmd, _) = Cursor::origin().1.move_by(999, 999);
        // attempt to move cursor at right bottom
        io::stdout().write(cmd.as_bytes())?;

        // query cursor position
        match io::stdout().write("\x1b[6n".as_bytes()) {
            Ok(_) => Ok(read_res().unwrap()),
            Err(e) => Err(e),
        }
    }

    fn read_res() -> Option<(u16, u16)> {
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
            .fold(String::new(), |acc, input| acc + &input.to_string());
        result.strip_prefix("\x1b[").and_then(|s| {
            match s.split(";").map(|s| s.parse().unwrap()).collect::<Vec<u16>>()[..2] {
                [row, col] => Some((row, col)),
                _ => None,
            }
        })
    }
}
