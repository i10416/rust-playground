extern crate cc;
fn main() {
    cc::Build::new()
        .warnings(true)
        .flag("-Wall")
        .flag("-Wextra")
        .file("src/c/enable_raw_mode.c")
        .file("src/c/terminal_size.c")
        .include("src/c")
        .compile("texteditor.a");
}
