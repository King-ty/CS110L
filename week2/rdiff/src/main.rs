use grid::Grid; // For lcs()
use std::cmp;
use std::env;
use std::fs::File; // For read_file_lines()
use std::io::{self, BufRead}; // For read_file_lines()
use std::process;

pub mod grid;

/// Reads the file at the supplied path, and returns a vector of strings.
fn read_file_lines(filename: &String) -> Result<Vec<String>, io::Error> {
    let mut lines = Vec::new();
    let file = File::open(filename)?;
    for line in io::BufReader::new(file).lines() {
        let line_str = line?;
        lines.push(line_str);
    }

    Ok(lines)
}

fn lcs(seq1: &Vec<String>, seq2: &Vec<String>) -> Grid {
    // Note: Feel free to use unwrap() in this code, as long as you're basically certain it'll
    // never happen. Conceptually, unwrap() is justified here, because there's not really any error
    // condition you're watching out for (i.e. as long as your code is written correctly, nothing
    // external can go wrong that we would want to handle in higher-level functions). The unwrap()
    // calls act like having asserts in C code, i.e. as guards against programming error.

    let mut lcs_table = Grid::new(seq1.len() + 1, seq2.len() + 1); // 初始化即为0
    for i in 0..seq1.len() {
        for j in 0..seq2.len() {
            if seq1.get(i).unwrap() == seq2.get(j).unwrap() {
                lcs_table
                    .set(i + 1, j + 1, lcs_table.get(i, j).unwrap() + 1)
                    .unwrap();
            } else {
                lcs_table
                    .set(
                        i + 1,
                        j + 1,
                        cmp::max(
                            lcs_table.get(i + 1, j).unwrap(),
                            lcs_table.get(i, j + 1).unwrap(),
                        ),
                    )
                    .unwrap();
            }
        }
    }
    lcs_table
}

fn print_diff(lcs_table: &Grid, lines1: &Vec<String>, lines2: &Vec<String>, i: usize, j: usize) {
    if i > 0 && j > 0 && lines1.get(i - 1).unwrap() == lines2.get(j - 1).unwrap() {
        print_diff(lcs_table, lines1, lines2, i - 1, j - 1);
        println!("  {}", lines1.get(i - 1).unwrap());
    } else if j > 0
        && (i == 0 || lcs_table.get(i, j - 1).unwrap() > lcs_table.get(i - 1, j).unwrap())
    {
        print_diff(lcs_table, lines1, lines2, i, j - 1);
        println!("> {}", lines2.get(j - 1).unwrap());
    } else if i > 0
        && (j == 0 || lcs_table.get(i, j - 1).unwrap() <= lcs_table.get(i - 1, j).unwrap())
    {
        print_diff(lcs_table, lines1, lines2, i - 1, j);
        println!("< {}", lines1.get(i - 1).unwrap());
    } else {
        println!();
    }
}

/// Wrong: 必须从后向前，存一下输出
// fn print_diff2(lcs_table: &Grid, lines1: &Vec<String>, lines2: &Vec<String>) {
//     let mut i = 0;
//     let mut j = 0;
//     let len1 = lines1.len();
//     let len2 = lines2.len();
//     while i < len1 || j < len2 {
//         if i < len1 && j < len2 && lines1.get(i).unwrap() == lines2.get(j).unwrap() {
//             println!("  {}", lines1.get(i).unwrap());
//             i += 1;
//             j += 1;
//         } else if j < len2
//             && (i == len1 || lcs_table.get(i + 1, j).unwrap() >= lcs_table.get(i, j + 1).unwrap())
//         {
//             println!("> {}", lines2.get(j).unwrap());
//             j += 1;
//         } else if i < len1
//             && (j == len2 || lcs_table.get(i + 1, j).unwrap() < lcs_table.get(i, j + 1).unwrap())
//         {
//             println!("< {}", lines1.get(i).unwrap());
//             i += 1;
//         } else {
//             println!();
//         }
//     }
// }

fn print_diff2(lcs_table: &Grid, lines1: &Vec<String>, lines2: &Vec<String>) {
    let mut i = lines1.len();
    let mut j = lines2.len();
    let mut out_lines = Vec::new();
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && lines1.get(i - 1).unwrap() == lines2.get(j - 1).unwrap() {
            out_lines.push("  ".to_string() + lines1.get(i - 1).unwrap());
            i -= 1;
            j -= 1;
        } else if j > 0
            && (i == 0 || lcs_table.get(i, j - 1).unwrap() > lcs_table.get(i - 1, j).unwrap())
        {
            out_lines.push("> ".to_string() + lines2.get(j - 1).unwrap());
            j -= 1;
        } else if i > 0
            && (j == 0 || lcs_table.get(i, j - 1).unwrap() <= lcs_table.get(i - 1, j).unwrap())
        {
            out_lines.push("< ".to_string() + lines1.get(i - 1).unwrap());
            i -= 1;
        } else {
            println!();
        }
    }

    for out_line in out_lines.iter().rev() {
        println!("{}", out_line);
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

    let file_lines1 = read_file_lines(&filename1).expect("read file1 lines failed");
    let file_lines2 = read_file_lines(&filename2).expect("read file2 lines failed");

    let lcs_table = lcs(&file_lines1, &file_lines2);

    print_diff(
        &lcs_table,
        &file_lines1,
        &file_lines2,
        file_lines1.len(),
        file_lines2.len(),
    );

    println!();

    print_diff2(&lcs_table, &file_lines1, &file_lines2);
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
