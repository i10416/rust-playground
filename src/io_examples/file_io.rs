pub mod file_io {
    use std::fs::File;
    use std::io::{prelude::*, BufReader};
    use std::path::Path;

    // expect: `linecounts    wordcounts    bytecounts    filename\n
    pub fn wc(p: &str) -> Result<String, String> {
        let path = Path::new(p);
        let (size, words, lines) = count_all(path);

        Ok(fmt(lines, words, size, path.display().to_string()))
    }

    fn count_all(path: &Path) -> (usize, usize, usize) {
        let file = File::open(path).unwrap();
        let buf = vec![];
        read_to_buffer_rec(BufReader::new(file), buf, (0, 0, 0))
    }

    fn read_to_buffer_rec(
        mut reader: BufReader<File>,
        mut buf: Vec<u8>,
        acc: (usize, usize, usize),
    ) -> (usize, usize, usize) {
        match reader.read(&mut buf) {
            // reach the end of the file or file contains 0 byte
            Ok(n) if n <= 0 => (
                acc.0 + count_bytes(&reader),
                acc.1 + count_words(&reader),
                acc.2 + count_line_ends(&reader),
            ),
            Ok(n) => {
                let ls = count_line_ends(&reader);
                let ws = count_words(&reader);
                read_to_buffer_rec(reader, vec![], (acc.0 + n, acc.1 + ws, acc.2 + ls))
            }
            Err(_) => acc,
        }
    }

    fn count_bytes(r: &BufReader<File>) -> usize {
        r.buffer().bytes().count()
    }
    fn count_line_ends(r: &BufReader<File>) -> usize {
        r.buffer().iter().filter(|b| (**b as char) == '\n').count()
    }
    fn count_words(r: &BufReader<File>) -> usize {
        String::from_utf8_lossy(r.buffer()).split(" ").count()
    }

    fn fmt(linecount: usize, wordcount: usize, bytecount: usize, filename: String) -> String {
        let s = format!(
            "{}    {}    {}    {}\n",
            linecount, wordcount, bytecount, filename
        );
        s
    }
}
