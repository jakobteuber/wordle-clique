# Wordle Independent Set

This program computes sets of words that share no letters in common. This puzzle was originally
posed in a video by Matt Parker (https://www.youtube.com/watch?v=_-AfhLQfb6w&t=207s), where he
wrote a Python program that took about a month to solve the problem on his laptop. This solution
can be improved using a few clever tricks.


## Usage

`wordle-clique [OPTIONS] <INPUT>`

### Arguments

`<INPUT>` 
: A list containing the search space of words, one word on each line. 
Lines that don’t contain a five-letter word consisting only of 
Ascii letters are silently discarded. Specifying `-` as the file name will 
cause the program to read the words from standard input

### Options

`--no-print`
: Don’t print the solution, just materialize it in memory. Useful for benchmarking

`-h`, `--help`
: Print help

`-V`, `--version`  
: Print version
