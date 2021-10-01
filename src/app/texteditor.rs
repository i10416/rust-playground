// refs
// build own text editor: kilo.
// https://viewsourcecode.org/snaptoken/kilo/
/*
## Objective

antirez's kilo を少し改変したテキストエディタをRustで書く.

 */
// todo: fix horizontal scroll
// todo: insert chars
// todo: delete chars
// todo: save
// todo: handle horizontal scroll
// todo: replace recursion with loop
// todo: use stream and event
use std::fmt::Debug;
use std::io::{self, BufRead, Read, Write};
use std::os::raw::{c_char, c_uint};
use std::os::unix::raw::mode_t;

type Cflag = c_uint;
type Speed = c_uint;
const NCCS: usize = 32;
const KILO_VERSION: &str = "0.0.0";
const TAB_SIZE: usize = 4;
// prevent memory layout optimization
#[repr(C)]
#[derive(Debug, Default)]
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
#[derive(Debug)]
struct Viewport {
    // offset: (col,row)
    offset: (usize, usize),
    //  size: (col,row)
    size: (usize, usize),
    max_height: usize,
}

impl Viewport {
    pub fn update_with(&self, cursor: &terminal::Cursor) -> Viewport {
        let next_offset = match (cursor.col, cursor.row) {
            (col, row) if row > self.offset.1 + self.size.1 - 1 && col > self.offset.0 + self.size.0 - 1 => {
                (col - self.size.0 + 1, row - self.size.1 + 1)
            }
            (_, row) if row > self.offset.1 + self.size.1 - 1 => (self.offset.0, row - self.size.1 + 1),
            (_, row) if row < self.offset.1 => (self.offset.0, row),
            (col, _) if col > self.offset.0 + self.size.0 - 1 => (col - self.size.0 + 1, self.offset.1),
            (col, _) if col < self.offset.0 => (col, self.offset.1),
            _ => self.offset,
        };
        Viewport {
            offset: next_offset,
            size: self.size,
            max_height: self.max_height,
        }
    }

    pub fn contains(&self, cursor: &terminal::Cursor) -> bool {
        match (cursor.col, cursor.row) {
            (col, _) if col > self.offset.0 + self.size.0 - 1 => false,
            (col, _) if col < self.offset.0 => false,
            (_, row) if row > self.offset.1 + self.size.1 - 1 => false,
            (_, row) if row < self.offset.1 => false,
            _ => true,
        }
    }
}

#[link(name = "texteditor.a")]
extern "C" {
    fn enable_raw_mode(original: *mut Termios) -> i32;
    fn restore(original: *const Termios) -> i32;
}

#[derive(Debug, PartialEq)]
enum Mode {
    Insert,
    Visual,
}
/**
* represent app state
*/
#[derive(Debug)]
struct State<'a> {
    termios: Termios,
    cursor: terminal::Cursor,
    size: terminal::TermSize,
    rows: Vec<String>,
    viewport: Viewport,
    statusbar: StatusBar<'a>,
    mode: Mode,
}

impl<'a> State<'a> {
    fn new(
        t: Termios,
        size: terminal::TermSize,
        cursor: terminal::Cursor,
        rows: Vec<String>,
        viewport: Viewport,
        statusbar: StatusBar<'a>,
        mode: Mode,
    ) -> State<'a> {
        State {
            termios: t,
            size,
            cursor: cursor,
            rows: rows,
            viewport: viewport,
            statusbar: statusbar,
            mode: Mode::Visual,
        }
    }
}
#[derive(Debug)]
struct StatusBar<'a> {
    lines: usize,
    filename: Option<&'a str>,
    filetype: Option<&'a str>,
    has_change: bool,
    width: usize,
}
impl<'a> StatusBar<'a> {
    pub fn render(&self, mode: &Mode) -> String {
        let truncated_filename = self
            .filename
            .map(|s| {
                if s.len() >= self.width - 6 {
                    s.get(0..self.width - 7).unwrap()
                } else {
                    s
                }
            })
            .unwrap_or("[No Name]");
        let mode_type = match mode {
            Mode::Insert => "insert",
            Mode::Visual => "visual",
        };

        format!(
            "\x1b[7m{}{}{} lines[{}]\x1b[m",
            truncated_filename,
            " ".repeat(self.width - 8 - truncated_filename.len() - self.lines.to_string().len() - mode_type.len()),
            self.lines,
            mode_type
        )
    }
}

fn replace_tabs(line: String) -> String {
    line.chars()
        .zip(0..)
        .map(|(c, idx)| {
            if c == '\t' {
                " ".repeat(TAB_SIZE - (idx % TAB_SIZE))
            } else {
                c.to_string()
            }
        })
        .collect::<String>()
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
    let rows: Vec<String> = read_file("README.md")
        .unwrap_or(vec![build_welcome_message(term_size.col, 1)])
        .into_iter()
        .map(|row| replace_tabs(row))
        .collect();
    let pad_size = rows.len().to_string().len();
    io::stdout()
        .write(clean_display().as_bytes())
        .and_then(|_| io::stdout().flush())
        .expect("success");
    let content_width = std::cmp::max((term_size.col as i32) - (pad_size as i32) - 2, 0) as usize;
    let viewport = Viewport {
        offset: (0, 0),
        size: (content_width, (term_size.row - 1).into()),
        max_height: rows.len() - 1,
    };
    let (_, cursor) = terminal::Cursor::origin(pad_size + 2, rows.first().map_or(0, |s| s.len()), rows.len() - 1);

    let statusbar = StatusBar {
        lines: rows.len(),
        filename: None,
        filetype: None,
        has_change: false,
        width: term_size.col.into(),
    };
    let state = State::new(original, term_size, cursor, rows, viewport, statusbar, Mode::Visual);

    match tick(state) {
        Ok((message, state)) => {
            unsafe {
                restore(&state.termios);
            }
            println!("{}", clean_display() + &message);
            Ok(())
        }
        Err(_) => unimplemented!(),
    }
}
fn clean_display() -> String {
    String::from("\x1b[2J\x1b[0;0H")
}
/*
* read file into Vec<String>. each element of the vec represents a row.
 */
fn read_file(file_path: &str) -> Result<Vec<String>, std::io::Error> {
    let f = std::fs::File::open(file_path);
    match f {
        Ok(file) => Ok(std::io::BufReader::new(file)
            .lines()
            .into_iter()
            .map(|result| result.unwrap_or(String::from("<err>")))
            .fold(vec![], |mut acc, line| {
                acc.push(line);
                acc
            })),
        Err(e) => Err(e),
    }
}
/*
* prepend leading symbol ~<row_number>: and append clean_line escape sequence to row
*/
fn build_row(content: &str, idx: usize, pad_size: usize, viewport: &Viewport) -> String {
    let visibleContent = match (viewport.offset.0, viewport.size.0) {
        (col_offset, _) if col_offset >= content.len() => "",
        (col_offset, viewport_width) => content
            .get(col_offset..std::cmp::min(content.len(), col_offset + viewport_width))
            .unwrap(),
    };
    format!(
        "~{}:{}{}",
        format!("{:0>width$}", idx, width = pad_size),
        visibleContent,
        clean_line()
    )
}

fn build_screen(
    rows: Vec<String>,
    pad_size: usize,
    offset: usize,
    height: usize,
    viewport: &Viewport,
    mode: &Mode,
    statusbar: &StatusBar,
) -> String {
    rows.into_iter()
        .skip(offset)
        .take(height - 1)
        .zip((offset + 1)..)
        .fold(String::from("\x1b[0;0H"), |acc, (row, idx)| {
            acc + &build_row(&row, idx, pad_size, viewport) + "\r\n"
        })
        + &(statusbar.render(mode))
}

fn clean_line() -> String {
    String::from("\x1b[K")
}

fn build_welcome_message(col_count: u16, row_number: usize) -> String {
    match format!("Rusty Editor -- version {}", KILO_VERSION) {
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

fn render(cmd: String) {
    io::stdout().write(cmd.as_bytes()).unwrap();
    // disable buffer
    io::stdout().flush().expect("success");
}

fn tick(state: State) -> Result<(String, State), String> {
    let (hide_cursor_cmd, cursor) = state.cursor.hide();

    let current_line_length = state.rows[cursor.row].len();
    let prev_line_length = if cursor.row == cursor.bounds.top {
        state.rows[cursor.bounds.top].len()
    } else {
        state.rows[cursor.row - 1].len()
    };
    let pad_size = state.rows.len().to_string().len();
    let next_line_length = state.rows[std::cmp::min(cursor.row + 1, cursor.bounds.bottom)].len();
    let bounds = cursor.bounds.update_right_bounds(
        current_line_length + pad_size + 2,
        prev_line_length + pad_size + 2,
        next_line_length + pad_size + 2,
    );
    let cursor = cursor.update_bounds(bounds);
    match io::stdin().bytes().next() {
        Some(Ok(input)) if input == b'q' & 0x1f => Ok((String::from("Bye!"), state)),
        Some(Ok(input)) if input == b'u' & 0x1f || input == b'd' & 0x1f => {
            let dy = if input == b'u' & 0x1f {
                -(state.size.row as i32)
            } else {
                state.size.row as i32
            };
            // move cursor by termSize.height
            let (_, cursor) = cursor.move_by(0, dy);
            let viewport = state.viewport.update_with(&cursor);
            let move_cmd = cursor.move_cmd(viewport.offset.1);
            let render_textarea_cmd = build_screen(
                state.rows.clone(),
                pad_size,
                viewport.offset.1,
                state.size.row.into(),
                &viewport,
                &state.mode,
                &state.statusbar,
            );
            let (show_cursor_cmd, cursor) = cursor.show();
            render(hide_cursor_cmd + &render_textarea_cmd + &move_cmd + &show_cursor_cmd);
            tick(State {
                size: state.size,
                cursor: cursor,
                termios: state.termios,
                rows: state.rows,
                viewport: viewport,
                statusbar: state.statusbar,
                mode: state.mode,
            })
        }
        // todo: ネストが深くてつらいのであり得ないケースは unwrap,expect などで雑にハンドリングする
        // handle arrow keys(\x1b[A,\x1b[B,\x1b[C,\x1b[D)
        Some(Ok(b'\x1b')) => {
            match io::stdin().bytes().peekable().peek() {
                Some(Ok(b'[')) => match io::stdin().bytes().next() {
                    Some(Ok(input @ (b'A' | b'B' | b'C' | b'D'))) => {
                        let (dx, dy) = arrow_key_to_move(input);
                        let (_, cursor) = cursor.move_by(dx, dy);
                        let viewport = if state.viewport.contains(&cursor) {
                            state.viewport
                        } else {
                            state.viewport.update_with(&cursor)
                        };
                        let move_cmd = cursor.move_cmd(viewport.offset.1);

                        let render_textarea_cmd = build_screen(
                            state.rows.clone(),
                            pad_size,
                            viewport.offset.1,
                            state.size.row.into(),
                            &viewport,
                            &state.mode,
                            &state.statusbar,
                        );
                        let (show_cursor_cmd, cursor) = cursor.show();
                        render(hide_cursor_cmd + &render_textarea_cmd + &move_cmd + &show_cursor_cmd);
                        tick(State {
                            size: state.size,
                            cursor: cursor,
                            termios: state.termios,
                            rows: state.rows,
                            viewport: viewport,
                            statusbar: state.statusbar,
                            mode: state.mode,
                        })
                    }
                    // handle delete key
                    Some(Ok(b'3')) => match io::stdin().bytes().peekable().peek() {
                        Some(Ok(b'~')) => {
                            let move_cmd = cursor.move_cmd(state.viewport.offset.1);

                            let render_textarea_cmd = build_screen(
                                state.rows.clone(),
                                pad_size,
                                state.viewport.offset.1,
                                state.size.row.into(),
                                &state.viewport,
                                &state.mode,
                                &state.statusbar,
                            );
                            let (show_cursor_cmd, cursor) = cursor.show();

                            render(hide_cursor_cmd + &render_textarea_cmd + &move_cmd + &show_cursor_cmd);
                            tick(State {
                                size: state.size,
                                cursor: cursor,
                                termios: state.termios,
                                rows: state.rows,
                                viewport: state.viewport,
                                statusbar: state.statusbar,
                                mode: state.mode,
                            })
                        }
                        _ => unimplemented!(),
                    },
                    _ => {
                        unimplemented!()
                    }
                },
                Some(Ok(_)) => unimplemented!(),
                Some(Err(_)) => unimplemented!(),
                None => {
                    let move_cmd = cursor.move_cmd(state.viewport.offset.1);

                    let render_textarea_cmd = build_screen(
                        state.rows.clone(),
                        pad_size,
                        state.viewport.offset.1,
                        state.size.row.into(),
                        &state.viewport,
                        &state.mode,
                        &state.statusbar,
                    );
                    let (show_cursor_cmd, cursor) = cursor.show();

                    render(hide_cursor_cmd + &render_textarea_cmd + &move_cmd + &show_cursor_cmd);
                    tick(State {
                        size: state.size,
                        cursor: cursor,
                        termios: state.termios,
                        rows: state.rows,
                        viewport: state.viewport,
                        statusbar: state.statusbar,
                        mode: Mode::Visual,
                    })
                }
            }
        }
        Some(Ok(input)) if input == b'i' && state.mode == Mode::Visual => {
            let move_cmd = cursor.move_cmd(state.viewport.offset.1);

            let render_textarea_cmd = build_screen(
                state.rows.clone(),
                pad_size,
                state.viewport.offset.1,
                state.size.row.into(),
                &state.viewport,
                &state.mode,
                &state.statusbar,
            );
            let (show_cursor_cmd, cursor) = cursor.show();

            render(hide_cursor_cmd + &render_textarea_cmd + &move_cmd + &show_cursor_cmd);
            tick(State {
                size: state.size,
                cursor: cursor,
                termios: state.termios,
                rows: state.rows,
                viewport: state.viewport,
                statusbar: state.statusbar,
                mode: Mode::Insert,
            })
        }
        Some(Ok(input)) if state.mode == Mode::Insert => {
            let rows: Vec<String> = state
                .rows
                .into_iter()
                .zip(0..)
                .map(|(row, idx)| {
                    if idx == cursor.row {
                        if cursor.col + 1 > row.len() {
                            format!("{}{}", row, input as char)
                        } else {
                            let (front, back) = row.split_at(cursor.col);
                            [front, &(input as char).to_string(), back].join("")
                        }
                    } else {
                        row
                    }
                })
                .collect();
            let cursor = cursor.update_bounds(cursor.bounds.update_right_bounds(
                cursor.bounds.right + 1,
                cursor.bounds.right_prev_line,
                cursor.bounds.right_next_line,
            ));
            let (move_cmd, cursor) = cursor.move_by(1, 0);

            let render_textarea_cmd = build_screen(
                rows.clone(),
                pad_size,
                state.viewport.offset.1,
                state.size.row.into(),
                &state.viewport,
                &state.mode,
                &state.statusbar,
            );
            let (show_cursor_cmd, cursor) = cursor.show();

            render(hide_cursor_cmd + &render_textarea_cmd + &move_cmd + &show_cursor_cmd);
            tick(State {
                size: state.size,
                cursor: cursor,
                termios: state.termios,
                rows: rows,
                viewport: state.viewport,
                statusbar: state.statusbar,
                mode: Mode::Insert,
            })
        }
        Some(Ok(input)) => {
            let (dx, dy) = key_to_move(input);
            let (_, cursor) = cursor.move_by(dx, dy);
            let viewport = if state.viewport.contains(&cursor) {
                state.viewport
            } else {
                state.viewport.update_with(&cursor)
            };
            let move_cmd = cursor.move_cmd(viewport.offset.1);

            let render_textarea_cmd = build_screen(
                state.rows.clone(),
                pad_size,
                viewport.offset.1,
                state.size.row.into(),
                &viewport,
                &state.mode,
                &state.statusbar,
            );
            let (show_cursor_cmd, cursor) = cursor.show();
            render(hide_cursor_cmd + &render_textarea_cmd + &move_cmd + &show_cursor_cmd);
            tick(State {
                size: state.size,
                cursor: cursor,
                termios: state.termios,
                rows: state.rows,
                viewport: viewport,
                statusbar: state.statusbar,
                mode: state.mode,
            })
        }
        Some(Err(_)) => unimplemented!(),
        None => {
            let move_cmd = cursor.move_cmd(state.viewport.offset.1);
            let render_textarea_cmd = build_screen(
                state.rows.clone(),
                pad_size,
                state.viewport.offset.1,
                state.size.row.into(),
                &state.viewport,
                &state.mode,
                &state.statusbar,
            );
            let (show_cursor_cmd, cursor) = cursor.show();
            render(hide_cursor_cmd + &render_textarea_cmd + &move_cmd + &show_cursor_cmd);
            tick(State {
                size: state.size,
                cursor: cursor,
                termios: state.termios,
                rows: state.rows,
                viewport: state.viewport,
                statusbar: state.statusbar,
                mode: state.mode,
            })
        }
    }
}

mod terminal {
    use std::io::{self, Read, Write};
    use std::os::raw::c_ushort;

    #[derive(Debug)]
    pub struct Cursor {
        pub row: usize,
        pub col: usize,
        pub visibility: bool,
        pub bounds: Bounds,
    }
    #[derive(Debug, Clone)]
    pub struct Bounds {
        pub left: usize,
        pub top: usize,
        pub right: usize,
        pub bottom: usize,
        pub right_next_line: usize,
        pub right_prev_line: usize,
    }
    impl Bounds {
        pub fn new(
            left: usize,
            top: usize,
            right: usize,
            bottom: usize,
            right_prev_line: usize,
            right_next_line: usize,
        ) -> Bounds {
            Bounds {
                left: left,
                top: top,
                right: right,
                bottom: bottom,
                right_prev_line: right_prev_line,
                right_next_line: right_next_line,
            }
        }
        pub fn update_right_bounds(&self, right: usize, right_prev_line: usize, right_next_line: usize) -> Bounds {
            Bounds {
                left: self.left,
                top: self.top,
                right: right,
                bottom: self.bottom,
                right_next_line: right_next_line,
                right_prev_line: right_prev_line,
            }
        }

        pub fn contains(&self, col: i32, row: i32) -> bool {
            self.contains_horizontal(row) && self.contains_vertical(col)
        }

        fn contains_vertical(&self, col: i32) -> bool {
            (self.left as i32) <= col && col <= (self.right as i32)
        }

        fn contains_horizontal(&self, row: i32) -> bool {
            (self.top as i32) <= row && row <= (self.bottom as i32)
        }
    }

    // todo: handle terminal size bounds
    impl Cursor {
        pub fn origin(left: usize, right: usize, bottom: usize) -> (String, Cursor) {
            (
                String::from("\x1b[H"),
                Cursor {
                    row: 0,
                    col: 0,
                    visibility: true,
                    bounds: Bounds::new(left, 0, right, bottom, 0, 0),
                },
            )
        }

        pub fn update_bounds(&self, bounds: Bounds) -> Cursor {
            Cursor {
                row: self.row.clamp(bounds.top, bounds.bottom),
                col: self.col.clamp(bounds.left, bounds.right),
                visibility: self.visibility,
                bounds: bounds,
            }
        }

        pub fn move_by(&self, col: i32, row: i32) -> (String, Cursor) {
            match (
                self.col as i32 + col,
                (self.row as i32 + row).clamp(self.bounds.top as i32, self.bounds.bottom as i32),
            ) {
                (col, row) if self.bounds.contains(col, row) => self.move_to(col as usize, row as usize),
                (col, row) if col > self.bounds.right as i32 => {
                    // move to next line start
                    self.move_to(
                        self.bounds.left,
                        std::cmp::min(row + 1, self.bounds.bottom as i32) as usize,
                    )
                }
                (col, row) if col < self.bounds.left as i32 => {
                    // move to prev line end
                    self.move_to(self.bounds.right_prev_line, std::cmp::max(row - 1, 0) as usize)
                }
                _ => self.move_to(
                    (self.col as i32 + col).clamp(self.bounds.left as i32, self.bounds.right as i32) as usize,
                    (self.row as i32 + row).clamp(self.bounds.top as i32, self.bounds.bottom as i32) as usize,
                ),
            }
        }

        pub fn move_cmd(&self, offset: usize) -> String {
            format!("\x1b[{};{}H", self.row - offset + 1, self.col + 1)
        }

        pub fn move_to(&self, col: usize, row: usize) -> (String, Cursor) {
            let next = match row {
                next_row if next_row > self.row => Cursor {
                    row: next_row.clamp(self.bounds.top, self.bounds.bottom),
                    col: col.clamp(self.bounds.left, self.bounds.right_next_line),
                    visibility: self.visibility,
                    bounds: self.bounds.clone(),
                },
                prev_row if prev_row < self.row => Cursor {
                    row: prev_row.clamp(self.bounds.top, self.bounds.bottom),
                    col: col.clamp(self.bounds.left, self.bounds.right_prev_line),
                    visibility: self.visibility,
                    bounds: self.bounds.clone(),
                },
                row => Cursor {
                    row: row.clamp(self.bounds.top, self.bounds.bottom),
                    col: col.clamp(self.bounds.left, self.bounds.right),
                    visibility: self.visibility,
                    bounds: self.bounds.clone(),
                },
            };
            let cmd = format!("\x1b[{};{}H", next.row + 1, next.col + 1);
            (cmd, next)
        }

        pub fn hide(&self) -> (String, Cursor) {
            (
                format!("\x1b[?25l"),
                Cursor {
                    row: self.row,
                    col: self.col,
                    visibility: false,
                    bounds: self.bounds.clone(),
                },
            )
        }
        pub fn show(&self) -> (String, Cursor) {
            (
                format!("\x1b[?25h"),
                Cursor {
                    col: self.col,
                    row: self.row,
                    visibility: true,
                    bounds: self.bounds.clone(),
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
        let (cmd, _) = Cursor::origin(0, 999, 999).1.move_by(999, 999);
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
