pub mod file_io {
    use std::env;
    use std::fs::File;
    use std::io::{prelude::*, BufReader};
    use std::path::Path;

    // expect: `linecounts    wordcounts    bytecounts    filepath\n
    pub fn wc(p: &str) -> Result<String, String> {
        let path = Path::new(p);
        let (size, words, lines) = count_all(path);

        Ok(fmt(lines, words, size, path.display().to_string()))
    }

    fn count_all(path: &Path) -> (usize, usize, usize) {
        let file = File::open(path).unwrap();
        let mut buf = String::new();
        read_to_buffer_rec(BufReader::new(file), &mut buf, (0, 0, 0))
    }

    fn read_to_buffer_rec(
        mut reader: BufReader<File>,
        mut buf: &mut String,
        acc: (usize, usize, usize),
    ) -> (usize, usize, usize) {
        match reader.read_line(&mut buf) {
            // There remains 0 byte unread in the file.
            Ok(n) if n <= 0 => (
                acc.0 + count_bytes(&reader),
                acc.1 + count_words_by_line(&buf),
                acc.2 + 1,
            ),
            Ok(n) => {
                let ws = count_words_by_line(&buf);
                let mut b = String::new();
                read_to_buffer_rec(reader, &mut b, (acc.0 + n, acc.1 + ws, acc.2 + 1))
            }
            Err(_) => acc,
        }
    }

    fn count_bytes(r: &BufReader<File>) -> usize {
        r.buffer().bytes().count()
    }
    fn count_words_by_line(s: &String) -> usize {
        s.split(" ")
            .fold(0, |acc, word| if word.is_empty() { acc } else { acc + 1 })
    }

    fn fmt(linecount: usize, wordcount: usize, bytecount: usize, filename: String) -> String {
        let s = format!(
            "{}    {}    {}    {}\n",
            linecount, wordcount, bytecount, filename
        );
        s
    }
    pub fn set_dir(s:&str) {
        let path = Path::new(s);
        if let Err(e) = env::set_current_dir(&path) {
            eprintln!("{}", e);
        }
    }
}
