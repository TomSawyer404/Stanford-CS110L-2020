pub mod node;

use std::env;
use std::process;
use std::fs::File;              // For read_file_lines()
use std::io::{self, BufRead};   // For read_file_lines()
use node::Node;


/// Reads the file at the supplied path, and
/// returns a vector of strings.
fn read_file_lines(filename: &str) -> Result<Vec<Node>, io::Error> {
    let file = File::open(filename)?;

    let mut vec_str: Vec<Node> = Vec::new();
    for line in io::BufReader::new(file).lines() {
        let buf_line = line?;
        let cur_node: Node = Node::new(&buf_line[..]);
        vec_str.push(cur_node);
    }
    return Ok(vec_str);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Too few arguments.");
        process::exit(1);
    }
    let filename = &args[1];
    
    // Your code here :)
    let file_content: Vec<Node> = read_file_lines(&filename[..]).unwrap();
    
    let mut total_lines: usize = 0;
    let mut total_words = 0;
    let mut total_chars = 0;
    for i in file_content {
        //println!("{:?}", i);
        total_lines += 1;
        let word_char_tuple = i.get();
        total_words += word_char_tuple.unwrap().0;
        total_chars += word_char_tuple.unwrap().1;
    }
    println!("Total lines: {}", total_lines);
    println!("Total words: {}", total_words);
    println!("Total chars: {}", total_chars);
}
