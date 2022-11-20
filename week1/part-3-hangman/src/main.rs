// Simple Hangman Program
// User gets five incorrect guesses
// Word chosen randomly from words.txt
// Inspiration from: https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html
// This assignment will introduce you to some fundamental syntax in Rust:
// - variable declaration
// - string manipulation
// - conditional statements
// - loops
// - vectors
// - files
// - user input
// We've tried to limit/hide Rust's quirks since we'll discuss those details
// more in depth in the coming lectures.
extern crate rand;
use rand::Rng;
use std::fs;
use std::io;
use std::io::Write;

const NUM_INCORRECT_GUESSES: u32 = 5;
const WORDS_PATH: &str = "words.txt";

fn pick_a_random_word() -> String {
    let file_string = fs::read_to_string(WORDS_PATH).expect("Unable to read file.");
    let words: Vec<&str> = file_string.split('\n').collect();
    String::from(words[rand::thread_rng().gen_range(0, words.len())].trim())
}

fn main() {
    let secret_word = pick_a_random_word();
    // Note: given what you know about Rust so far, it's easier to pull characters out of a
    // vector than it is to pull them out of a string. You can get the ith character of
    // secret_word by doing secret_word_chars[i].
    let secret_word_chars: Vec<char> = secret_word.chars().collect();
    // Uncomment for debugging:
    println!("random word: {}", secret_word);
    // println!("random word: {:?}", secret_word_chars);

    // Your code here! :)

    println!("Welcome to CS110L Hangman!");

    let mut guessed_letters = String::new();
    let mut current_word = ['-'].repeat(secret_word_chars.len());
    let mut left_num = NUM_INCORRECT_GUESSES;
    let mut failed_flag = false;

    while current_word.iter().collect::<String>() != secret_word {
        println!(
            "The word so far is {}",
            current_word.iter().collect::<String>()
        );
        println!(
            "You have guessed the following letters: {}",
            guessed_letters
        );
        println!("You have {} guesses left", left_num);

        print!("Please guess a letter: ");
        // Make sure the prompt from the previous line gets displayed:
        io::stdout().flush().expect("Error flushing stdout.");
        let mut guess = String::new();
        io::stdin()
            .read_line(&mut guess)
            .expect("Error reading line.");
        let guess = guess.chars().nth(0).unwrap();

        guessed_letters.push(guess);

        let mut flag: bool = false;
        for i in 0..secret_word_chars.len() {
            if secret_word_chars[i] == guess && secret_word_chars[i] != current_word[i] {
                current_word[i] = secret_word_chars[i];
                flag = true;
                break;
            }
        }
        if !flag {
            left_num -= 1;
            println!("Sorry, that letter is not in the word");
        }

        println!();

        if left_num <= 0 {
            failed_flag = true;
            println!("Sorry, you ran out of guesses!");
            break;
        }
    }

    if !failed_flag {
        println!(
            "Congratulations you guessed the secret word: {}!",
            secret_word
        );
    }
}
