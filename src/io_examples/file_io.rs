pub mod file_io {
    use std::fs::File;
    use std::io::{prelude::*, BufReader};

    use std::path::Path;
    // expect: `linecounts    wordcounts    bytecounts    filename\n
    pub fn wc(p: &str) -> Result<String, String> {
        let path = Path::new(p);
        let file = match File::open(path) {
            Err(reason) => panic!("couldn't open {}: {}", path.display(), reason.to_string()),
            Ok(f) => f,
        };
        let size = count_byte(&file);
        Ok(fmt(0, size, 0, path.display().to_string()))
    }
    // count file byte size
    fn count_byte(file: &File) -> usize {
        let b = BufReader::new(file);
        let size = b.bytes().count();
        size
    }
    fn count_lines(file: &File) -> usize {
        1
    }
    fn count_words(file: &File) -> usize {
        1
    }
    fn fmt(linecount: usize, wordcount: usize, bytecount: usize, filename: String) -> String {
        let s = format!(
            "{}    {}    {}    {}\n",
            linecount, wordcount, bytecount, filename
        );
        s
    }
    fn write() {}
}
