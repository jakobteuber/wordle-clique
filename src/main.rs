use std::collections::HashMap;
use std::hint::black_box;
use std::io::Read;
use std::ops::Deref;
use clap::Parser;
use clio::Input;

/// Length of a word. Wordle considers only five-letter words
const WORD_LENGTH: usize = 5;

/// Size of the independent sets we are looking for.
const SET_SIZE: usize = 5;

/// Number of letters in the alphabet.
const LETTER_COUNT: usize = 26;

fn pattern_bit(ch: char) -> u32 {
    let ch = ch.to_ascii_lowercase();
    let index = ch as usize - 'a' as usize;
    1_u32 << index
}

fn pattern(word: &str) -> u32 {
    word.chars()
        .map(pattern_bit)
        .fold(0, |pattern, x| { pattern | x })
}

fn group_anagrams(words: &[String]) -> HashMap<u32, Vec<String>> {
    let mut groups = HashMap::new();
    for word in words {
        let pattern = pattern(&word);
        groups.entry(pattern)
            .or_insert_with(Vec::new)
            .push(word.clone());
    }
    groups
}

fn canonical_words(word: &str) -> Option<String> {
    if word.len() != WORD_LENGTH { return None; }
    if word.chars().any(|ch| !ch.is_ascii_alphabetic()) { return None; }
    if pattern(word).count_ones() != WORD_LENGTH as u32 { return None; }
    Some(word.to_ascii_lowercase())
}

fn read_words(word_file: &mut Input) -> Vec<String> {
    let mut contents = String::new();
    let result = word_file.read_to_string(&mut contents);
    if let Err(why) = result {
        panic!("couldn't read `{}`: {}", word_file.path().display(), why);
    }
    contents.lines()
        .flat_map(canonical_words)
        .collect()
}


#[derive(Debug)]
struct LetterGroup {
    letter: u32,
    words: Vec<u32>,
}

struct SearchSpace {
    letter_groups: [LetterGroup; LETTER_COUNT],
}

impl SearchSpace {
    fn new(words: &[u32]) -> Self {
        let mut letter_groups = ('a'..='z')
            .map(|ch| {
                let letter = pattern_bit(ch);
                let words = words.iter()
                    .filter(|word| (*word & letter) != 0)
                    .cloned()
                    .collect();
                LetterGroup { letter, words }
            }).collect::<Vec<_>>();
        letter_groups.sort_by_key(|it| it.words.len());
        let letter_groups = letter_groups.try_into().unwrap();
        SearchSpace { letter_groups }
    }
}

fn do_solve(words: &SearchSpace) -> Vec<[u32; SET_SIZE]> {
    let mut solutions = Vec::new();
    let mut current_solution = Vec::with_capacity(SET_SIZE);
    solve(words, 0, &mut solutions, &mut current_solution,
          0, true);
    solutions
}

fn solve(
    words: &SearchSpace,
    current_letter_idx: usize,
    solutions: &mut Vec<[u32; SET_SIZE]>,
    current_solution: &mut Vec<u32>,
    already_used: u32,
    can_skip_letter: bool,
) {
    if current_solution.len() == SET_SIZE {
        let current_solution: [u32; SET_SIZE] = current_solution.as_slice().try_into().unwrap();
        solutions.push(current_solution);
        return;
    }

    let mut current_letter_idx = current_letter_idx;
    loop {
        if current_letter_idx >= words.letter_groups.len() { return; }

        let current_letter = words.letter_groups[current_letter_idx].letter;
        if already_used & current_letter == 0 { break; }

        current_letter_idx += 1;
    }

    let current_letter = words.letter_groups[current_letter_idx].letter;
    let current_words = &words.letter_groups[current_letter_idx].words;

    for &word in current_words {
        if already_used & word != 0 { continue; }
        current_solution.push(word);
        solve(words, current_letter_idx + 1, solutions, current_solution,
              already_used | word, can_skip_letter);
        current_solution.pop();
    }

    if can_skip_letter {
        solve(words, current_letter_idx + 1, solutions, current_solution,
              already_used | current_letter, false);
    }
}

fn print_solutions(solutions: &[[u32; SET_SIZE]], anagram_map: &HashMap<u32, Vec<String>>) {
    for (i, solution) in solutions.iter().enumerate() {
        print!("Solution {:5}:   ", i + 1);
        for (j, &anagram) in solution.iter().enumerate() {
            let words = anagram_map.get(&anagram).unwrap().join(" or ");
            print!("({}) {}     ", j + 1, words);
        }
        println!();
    }
}

/// This program computes sets of words that share no letters in common. This puzzle was originally
/// posed in a video by Matt Parker (https://www.youtube.com/watch?v=_-AfhLQfb6w&t=207s), where he
/// wrote a Python program that took about a month to solve the problem on his laptop. This solution
/// can be improved using a few clever tricks (For more about the approach, see
/// https://github.com/jakobteuber/wordle-clique).
#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// A list containing the search space of words, one word on each line. Lines that don’t
    /// contain a five-letter word consisting only of Ascii letters are silently discarded.
    /// Specifying `-` as the file name will cause the program to read the words from standard
    /// input.
    #[clap(value_parser)]
    input: Input,

    /// Don’t print the solution, just materialize it in memory. Useful for benchmarking.
    #[clap(long, action)]
    no_print: bool,
}

fn main() {
    let mut args = Args::parse();
    let words = read_words(&mut args.input);
    if !args.no_print { print!("Read {} words. ", words.len()); }

    let anagram_map = group_anagrams(words.deref());
    let anagrams = anagram_map.keys().cloned().collect::<Vec<_>>();
    if !args.no_print { println!("Grouped into {} anagram sets. ", anagrams.len()); }

    let search_space = SearchSpace::new(anagrams.as_slice());
    let solutions = do_solve(&search_space);
    if !args.no_print {
        println!("Found {} solutions. ", solutions.len());
        print_solutions(solutions.as_slice(), &anagram_map);
    }

    black_box(solutions);
}
