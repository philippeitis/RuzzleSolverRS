# RuzzleSolverRS
A Ruzzle solver implemented using Rust, based on https://github.com/dchen327/ruzzle_solver.

# Getting Started
To get started, simply clone this repository and run cargo build --release. Afterwards, you will have a complete executable, which can be run with the data files in the directory - board.txt needs to be at the top level release folder for things to work correctly.

# How does it work?
It utilizes depth first search to explore all possible paths through the board. It trims paths that do not lead to valid words by checking if these words exist in the prefixes. It also tracks the score and path traversed.

As for the actual implementation, the code uses a compressed representation of strings, where all strings appear in upper case form, and only the letters A-Z, !, -, 2, 3 are encoded. This avoids a major slowdown that doesn't contribute anything significant to the code: parsing UTF-16 strings. Since our input dictionary is restricted to words from length 2 to 12, and only consists of the letters A-Z, it is in fact possible to store our strings in just 60 bits: each character takes 5 bits, and there are at most 12 words - though, the closest we can get with Rust's built in types is 64 bits, or 8 bytes, compared to 24 bytes for an empty string. These strings are easy to translate back, and using them yields a 5-10x increase in speed (and also memory) for the depth first search algorithm over a string based version. This convienient compression is also used to both reduce the size of the source dictionary, and allow reading bytes directly with no translation required - which yields a 3-5x speed up for reading files. 
