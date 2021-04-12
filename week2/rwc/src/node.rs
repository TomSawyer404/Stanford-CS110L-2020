#[derive(Debug)]
pub struct Node {
    count_words:    usize,
    count_chars:    usize,
    buf:            String
}

impl Node {
    pub fn new(line: &str) -> Node {
        Node {
            count_chars : line.chars().count(),
            count_words : line.trim().split_whitespace().count(),
            buf : String::from(line),
        }
    }

    pub fn get(&self) -> Option<(usize, usize)> {
        Some( (self.count_words, self.count_chars) )
    }
}
