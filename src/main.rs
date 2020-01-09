use std::collections::{HashSet, HashMap};
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::time::Instant;

/// Source: https://stackoverflow.com/questions/27582739/how-do-i-create-a-hashmap-literal
macro_rules! map (
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
     };
);

const PATH_TO_PREFIXES: &str = r"data/prefixes/";
const PATH_TO_DICTIONARY: &str = r"data/TWL06/";
const PATH_TO_BOARD: &str = r"board.txt";

const MIN_WORD_LEN: usize = 2;
const MAX_WORD_LEN: usize = 12;

const BOARD_SIZE: usize = 4;
const BOARD_SIZE_I8: i8 = BOARD_SIZE as i8;

// These options can be tweaked to improve performance if necessary."""
const PREFIX_LOWER_BOUND: usize = 2;
const PREFIX_UPPER_BOUND: usize = 10;

const U64_TO_CHAR: [char; 27] = ['!', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'];

const D_U64: u64 = 4;
const T_U64: u64 = 20;
const TWO_U64: u64 = 27;
const THREE_U64: u64 = 28;

#[allow(dead_code)]
const DASH_U64: u64 = 29;

/// Returns a u64 representation of any string that is twelve characters or less, and only
/// contains the letters A-Z. Every five bits, up until 60 bits or five zero bits occur,
/// correspond to the letter in U64_TO_CHAR with the same index (eg. as usize).
fn string_to_u64(string_to_convert: String, str_to_u64: &HashMap<char, u64>) -> u64 {
    let mut output: u64 = 0;
    for c in string_to_convert.chars() {
        output = (output << 5) | str_to_u64[&c];
    }
    return output;
}

/// Generates the string representation of any string that is represented in the first 60
/// bits of str_as_num, where each group of five consecutive bits corresponds the the character
/// at the index in U64_TO_CHAR.
///
/// I could use the first four bits to track the length (allowing for 15 different str lengths),
/// but this would entail maintaining it, which I don't want to do.
fn parse_to_str(str_as_num: u64) -> String {
    let mut str_repr: [char; 12] = ['!'; 12];
    let mut str_numbers = str_as_num;
    let mut max_bit = 12;

    for _ in 0..12 {
        // Read the last five bits.
        let val = (str_numbers & 31) as usize;
        str_numbers = str_numbers >> 5;
        if val == 0 {
            break;
        } else {
            max_bit -= 1;
            str_repr[max_bit] = U64_TO_CHAR[val];
        }
    }

    return str_repr[max_bit..].iter().collect();
}

// TODO: This can probably be made a lot faster, since we discard the strings after we've
//       generated a u64 from them:
//       Should try to use a buffer string.
/// Returns a HashSet containing all prefixes of length PREFIX_LOWER_BOUND to PREFIX_UPPER_BOUND
/// in their u64 representation.
fn read_prefixes(str_to_u64: &HashMap<char, u64>) -> HashSet<u64> {
    let mut prefixes: HashSet<u64> = HashSet::with_capacity(170000);
    for i in PREFIX_LOWER_BOUND..PREFIX_UPPER_BOUND + 1 {
        let file_name = format!("./{}prefixes{}L.txt", PATH_TO_PREFIXES, i);
        let file = File::open(file_name).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            prefixes.insert(string_to_u64(line.unwrap(), str_to_u64));
        }
    }
    return prefixes;
}

// TODO: This can probably be made a lot faster, since we discard the strings after we've
//       generated a u64 from them:
//       Should try to use a buffer string.
/// Returns a HashSet containing all words in their u64 representation.
fn read_dict(str_to_u64: &HashMap<char, u64>) -> HashSet<u64> {
    let mut dict: HashSet<u64> = HashSet::with_capacity(163000);
    for i in 0..13 {
        let file_name = format!("./{}TWL06Trimmed{}L.txt", PATH_TO_DICTIONARY, i);
        let file = File::open(file_name).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            // Ignore errors.
            dict.insert(string_to_u64(line.unwrap(), str_to_u64));
        }
    }
    return dict;
}

/// Reads the file at file_path into a vector, line for line, and returns it.
fn read_board(file_path: String) -> Vec<String> {
    let mut board = Vec::new();
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap(); // Ignore errors.
        board.push(line);
    }
    return board;
}

/// Parses the raw board using str_to_u64 to provide the correct mapping of
/// characters to u64 values with the lower 5 bits set. The lines up to the first blank line
/// are parsed into the first return value. All proceeding lines are parsed into the second
/// return value.
fn parse_board_and_mults(
    raw_board: Vec<String>,
    str_to_u64: &HashMap<char, u64>) -> (Vec<Vec<u64>>, Vec<Vec<u64>>) {
    let mut board = Vec::new();
    let mut word_mults = Vec::new();
    let mut is_still_board = true;

    for line in raw_board {
        let line_empty = line.is_empty();
        is_still_board = !line_empty & is_still_board;

        let symbols: Vec<u64> = line
            .chars()
            .filter(|c| *c != ' ')
            .map(|c| str_to_u64[&c])
            .collect();

        if is_still_board {
            board.push(symbols);
        } else {
            if !line_empty {
                word_mults.push(symbols);
            }
        }
    }
    return (board, word_mults);
}

/// Takes the u64 mults (which correspond to characters in the alphabet), and maps them
/// to their usize multipliers.
fn parse_word_mults_to_int_mults(word_mults: &Vec<Vec<u64>>) -> Vec<Vec<usize>> {
    let mut word_int_mults = Vec::new();

    for line in word_mults {
        let mut line_mults: Vec<usize> = Vec::new();
        for letter in line {
            line_mults.push(match *letter {
                TWO_U64 => 2,
                THREE_U64 => 3,
                _ => 1,
            });
        }
        word_int_mults.push(line_mults)
    }
    return word_int_mults;
}

/// Returns the points for each letter on the board.
fn get_points(board: &Vec<Vec<u64>>, word_mults: &Vec<Vec<u64>>) -> Vec<Vec<usize>> {
    let mut points = Vec::new();

    // Each index is a point - eg.
    let point_vals: [usize; 27] = [0, 1, 4, 4, 2, 1, 4, 3, 4, 1, 10, 5, 1, 3, 1, 1, 4,
        10, 1, 1, 1, 2, 4, 4, 8, 4, 8];

    for (board_line, mult_line) in board.iter().zip(word_mults) {
        let mut line_points: Vec<usize> = Vec::new();
        for (letter, mult) in board_line.iter().zip(mult_line) {
            // let letter_point = *point_vals.get(&board_line[j]).unwrap();
            // this causes a panic.
            let letter_points = point_vals[*letter as usize];
            line_points.push(letter_points * match *mult {
                D_U64 => 2,
                T_U64 => 3,
                _ => 1,
            });
        }
        points.push(line_points);
    }
    return points;
}

/// Fills the board with information using the non recursive dfs algorithm.
fn all_combos(board: &mut Board) {
    for i in 0..4 {
        for j in 0..4 {
            dfs_helper_non_recursive(board, (i, j));
        }
    }
}

/// Generates the necessary arguments to launch dfs_non_recursive.
fn dfs_helper_non_recursive(board: &mut Board, start_point: (usize, usize)) {
    let (x, y) = start_point;
    board.visited = gen_visited();
    dfs_non_recursive(board, start_point, board.board[x][y],
                      board.points[x][y], board.word_int_mults[x][y]);
}

/// A non recursive depth first search which identifies all words, and adds them to
/// board.word_info_as_str with their string representation, score and path. Returns nothing.
fn dfs_non_recursive(board: &mut Board, start_point: (usize, usize), start_char: u64,
                     start_pts: usize, start_mult: usize) {
    let mut stack: Vec<(Vec<(usize, usize)>, u64, usize, usize, usize)> = Vec::with_capacity(25);
    stack.push((vec![start_point], start_char, start_pts, start_mult, 1));

    while !stack.is_empty() {
        let popped_item = stack.pop().unwrap();
        let (path, word, word_pts, word_mult, mut word_len) = popped_item;

        if (word_len >= MIN_WORD_LEN) & (word_len < MAX_WORD_LEN) & board.dictionary.contains(&word) {
            let mut score = word_pts * word_mult;
            if word_len > 4 {
                score += 5 * (word_len - 4);
            }
            board.word_info_as_str.push((parse_to_str(word), score, path.clone()));
        }

        let (xs, ys) = path[word_len - 1];
        word_len += 1;
        for vertex_addr in &board.graph[xs][ys] {
            let vertex = *vertex_addr;
            let (x, y) = vertex;

            if !path.contains(vertex_addr) {
                let temp_word = (word << 5) | board.board[x][y];

                if word_len >= MIN_WORD_LEN {
                    if (word_len <= PREFIX_UPPER_BOUND) & !board.prefixes.contains(&temp_word) {
                        continue;
                    }

                    if (word_len == MAX_WORD_LEN) & board.dictionary.contains(&temp_word) {
                        let score = word_pts * word_mult + 40;
                        let mut path_clone = path.clone();
                        path_clone.push(vertex);
                        board.word_info_as_str.push((parse_to_str(temp_word), score, path_clone));
                        continue;
                    }
                }
                let mut path_clone = path.clone();
                path_clone.push(vertex);
                stack.push((path_clone, temp_word, word_pts + board.points[x][y],
                            word_mult * board.word_int_mults[x][y], word_len));
            }
        }
    }
}

/// Fills the board with word info using the recursive DFS algorithm.
#[allow(dead_code)]
fn all_combos_r(board: &mut Board) {
    for i in 0..BOARD_SIZE {
        for j in 0..BOARD_SIZE {
            dfs_helper(board, (i, j));
        }
    }
}

/// Generates the necessary arguments to launch dfs.
#[allow(dead_code)]
fn dfs_helper(board: &mut Board, start_point: (usize, usize)) {
    let (x, y) = start_point;
    board.visited = gen_visited();
    dfs(board, start_point, board.board[x][y], board.points[x][y],
        board.word_int_mults[x][y], 1, vec![start_point]);
}

/// A recursive depth first search which identifies all possible words in the board, and adds
/// them to board.word_info_as_str with their string representation, score and path.
/// Returns nothing.
#[allow(dead_code)]
fn dfs(board: &mut Board, start_point: (usize, usize), word: u64, word_pts: usize,
       word_mult: usize, mut word_len: usize, path: Vec<(usize, usize)>) {
    if (word_len >= MIN_WORD_LEN) & board.dictionary.contains(&word) {
        let mut score = word_pts * word_mult;
        if word_len > 4 {
            score += 5 * (word_len - 4);
        }
        board.word_info_as_str.push((parse_to_str(word), score, path.clone()));
    }

    let (xs, ys) = start_point;

    board.visited[xs][ys] = true;
    word_len += 1;

    for vertex in board.graph[xs][ys].clone() {
        let (x, y) = vertex;
        if !board.visited[x][y] {
            let temp_word = (word << 5) | board.board[x][y];

            if word_len >= MIN_WORD_LEN {
                if (word_len <= PREFIX_UPPER_BOUND) & !board.prefixes.contains(&temp_word) {
                    continue;
                }
                if (word_len == MAX_WORD_LEN) & board.dictionary.contains(&temp_word) {
                    let score = word_pts * word_mult + 40;
                    let mut path_clone = path.clone();
                    path_clone.push(vertex);
                    board.word_info_as_str.push((parse_to_str(temp_word), score, path_clone));
                    continue;
                }
            }

            board.visited[x][y] = true;

            let mut path_clone = path.clone();
            path_clone.push(vertex);
            dfs(board, vertex, temp_word, word_pts + board.points[x][y],
                word_mult * board.word_int_mults[x][y], word_len, path_clone);

            board.visited[x][y] = false;
        }
    }
}

/// Generates a graph of all possible neighbouring vertices, represented with adjacency lists.
fn gen_graph() -> Vec<Vec<Vec<(usize, usize)>>> {
    let mut graph: Vec<Vec<Vec<(usize, usize)>>> = Vec::new();
    let directions: Vec<(i8, i8)> = vec![(1, 0), (-1, 0), (0, 1), (0, -1),
                                         (1, 1), (1, -1), (-1, 1), (-1, -1)];

    for i in 0..BOARD_SIZE_I8 {
        let mut graph_row: Vec<Vec<(usize, usize)>> = Vec::new();
        for j in 0..BOARD_SIZE_I8 {
            let mut neighbours = Vec::new();
            for (cx, cy) in &directions {
                let x = i + *cx;
                let y = j + *cy;
                if (0 <= x) & (x < 4) & (0 <= y) & (y < 4) {
                    neighbours.push((x as usize, y as usize));
                }
            }
            graph_row.push(neighbours);
        }
        graph.push(graph_row);
    }
    return graph;
}

/// Generates a matrix of vertices that have been visited.
fn gen_visited() -> Vec<Vec<bool>> {
    let mut visited: Vec<Vec<bool>> = Vec::new();
    for _i in 0..4 {
        visited.push([false; 4].to_vec());
    }
    return visited;
}

/// Maintains the board state and words found in the board.
struct Board {
    word_info_as_str: Vec<(String, usize, Vec<(usize, usize)>)>,
    board: Vec<Vec<u64>>,
    points: Vec<Vec<usize>>,
    word_int_mults: Vec<Vec<usize>>,
    prefixes: HashSet<u64>,
    dictionary: HashSet<u64>,
    graph: Vec<Vec<Vec<(usize, usize)>>>,
    visited: Vec<Vec<bool>>,
}

fn main() {
    let now = Instant::now();
    // The Python version can run in 0.09s, while this version consistently runs in 0.15s.
    // This code could be made faster by multithreading, since each DFS run is independent of
    // the rest.

    let str_to_u64_map: HashMap<char, u64> = map! {'A' => 1, 'B'=> 2, 'C'=> 3, 'D'=> 4, 'E'=> 5,
    'F'=> 6, 'G'=> 7, 'H'=> 8, 'I'=> 9, 'J'=> 10, 'K'=> 11, 'L' => 12, 'M' => 13, 'N'=> 14,
    'O'=> 15, 'P'=> 16, 'Q'=> 17, 'R'=> 18, 'S'=> 19, 'T'=> 20, 'U'=> 21, 'V'=> 22, 'W'=> 23,
    'X' => 24, 'Y' => 25, 'Z'=> 26, '2' => 27, '3' => 28, '-' => 29};

    let prefixes = read_prefixes(&str_to_u64_map);
    let dictionary = read_dict(&str_to_u64_map);
    let raw_board = read_board(PATH_TO_BOARD.to_string());

    let (board, word_mults) = parse_board_and_mults(raw_board, &str_to_u64_map);
    let word_int_mults = parse_word_mults_to_int_mults(&word_mults);
    let points = get_points(&board, &word_mults);

    let mut ruzzle_board = Board {
        word_info_as_str: Vec::new(),
        board,
        points,
        word_int_mults,
        prefixes,
        dictionary,
        graph: gen_graph(),
        visited: gen_visited(),
    };

    all_combos(&mut ruzzle_board);

    println!("{:#?}", now.elapsed().as_secs_f32());
    println!("{}", ruzzle_board.word_info_as_str.len());
    for (word, score, _) in ruzzle_board.word_info_as_str {
        println!("{}, {}", word, score);
    }
}
