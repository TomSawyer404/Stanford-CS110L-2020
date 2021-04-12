use grid::Grid; // For lcs()
use std::env;
use std::fs::File; // For read_file_lines()
use std::io::{self, BufRead}; // For read_file_lines()
use std::process;

pub mod grid;

/// Reads the file at the supplied path, and returns a vector of strings.
fn read_file_lines(filename: &String) -> Result<Vec<String>, io::Error> {
    // Be sure to delete the #[allow(unused)] line above
    let file = File::open(filename)?;
    
    let mut vec_str = Vec::new();
    for line in io::BufReader::new(file).lines() {
        let line_str = line?;
        vec_str.push(line_str);
    }
    return Ok(vec_str);
}

fn lcs(seq1: &Vec<String>, seq2: &Vec<String>) -> Grid {
    // Note: Feel free to use unwrap() in this code, as long as you're basically certain it'll
    // never happen. Conceptually, unwrap() is justified here, because there's not really any error
    // condition you're watching out for (i.e. as long as your code is written correctly, nothing
    // external can go wrong that we would want to handle in higher-level functions). The unwrap()
    // calls act like having asserts in C code, i.e. as guards against programming error.
    // Be sure to delete the #[allow(unused)] line above
    
    let mut dp: Grid = Grid::new( seq1.len() + 1, seq2.len() + 1 );
    
    // Fill boundary
    for j in 0 .. seq2.len() + 1 {
        dp.set(0, j, 0x0).unwrap();
    }
    for i in 0 .. seq1.len() + 1 {
        dp.set(i, 0, 0x0).unwrap();
    }

    for i in 0 .. seq1.len() {
        for j in 0 .. seq2.len() {
            if seq1[i] == seq2[j] {
                dp.set(i+1, j+1, dp.get(i, j).unwrap()+1).unwrap();
            } else {
                let mut max = dp.get(i+1, j);
                if max < dp.get(i, j+1) { max = dp.get(i, j+1) }
                dp.set(i+1, j+1, max.unwrap()).unwrap();
            }
        }
    }
    dp
}

fn print_diff(dp: &Grid, x: &Vec<String>, y: &Vec<String>, i: usize, j: usize) {
    // Be sure to delete the #[allow(unused)] line above
    if i > 0 && j > 0 && x[i-1] == y[j-1] {
        print_diff(dp, x, y, i-1, j-1);
        println!(" {}", x[i-1]);
    } else if j > 0 && (i == 0 || dp.get(i, j-1).unwrap() >= dp.get(i-1, j).unwrap()) {
        print_diff(dp, x, y, i, j-1);
        println!("> {}", y[j-1]);
    } else if i > 0 && (j == 0 || dp.get(i, j-1).unwrap() < dp.get(i-1, j).unwrap()) {
        print_diff(dp, x, y, i-1, j);
        println!("< {}", x[i-1])
    } else {
        println!("");
    }
    
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Too few arguments.");
        process::exit(1);
    }
    let filename1 = &args[1];
    let filename2 = &args[2];

    // Be sure to delete the #[allow(unused)] line above
    let x = read_file_lines(filename1).unwrap();
    let y = read_file_lines(filename2).unwrap();

    let dp = lcs(&x, &y);
    print_diff(&dp, &x, &y, x.len(), y.len());
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_file_lines() {
        let lines_result = read_file_lines(&String::from("handout-a.txt"));
        assert!(lines_result.is_ok());
        let lines = lines_result.unwrap();
        assert_eq!(lines.len(), 8);
        assert_eq!(
            lines[0],
            "This week's exercises will continue easing you into Rust and will feature some"
        );
    }

    #[test]
    fn test_lcs() {
        let mut expected = Grid::new(5, 4);
        expected.set(1, 1, 1).unwrap();
        expected.set(1, 2, 1).unwrap();
        expected.set(1, 3, 1).unwrap();
        expected.set(2, 1, 1).unwrap();
        expected.set(2, 2, 1).unwrap();
        expected.set(2, 3, 2).unwrap();
        expected.set(3, 1, 1).unwrap();
        expected.set(3, 2, 1).unwrap();
        expected.set(3, 3, 2).unwrap();
        expected.set(4, 1, 1).unwrap();
        expected.set(4, 2, 2).unwrap();
        expected.set(4, 3, 2).unwrap();

        println!("Expected:");
        expected.display();
        let result = lcs(
            &"abcd".chars().map(|c| c.to_string()).collect(),
            &"adb".chars().map(|c| c.to_string()).collect(),
        );
        println!("Got:");
        result.display();
        assert_eq!(result.size(), expected.size());
        for row in 0..expected.size().0 {
            for col in 0..expected.size().1 {
                assert_eq!(result.get(row, col), expected.get(row, col));
            }
        }
    }
}
