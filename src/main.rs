use std::collections::{HashSet, HashMap};
use std::fs::File;
use std::io::{BufReader, BufRead, Read, Write};
use std::time::Instant;
use fnv::{FnvHashSet, FnvHasher};
use std::hash::BuildHasherDefault;

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
const PATH_TO_DICTIONARY: &str = r"data/";
const PATH_TO_BOARD: &str = r"board.txt";

const MIN_WORD_LEN: usize = 2;
const MAX_WORD_LEN: usize = 12;

const BOARD_SIZE: usize = 4;
const BOARD_SIZE_I8: i8 = BOARD_SIZE as i8;

// These options can be tweaked to improve performance if necessary.
const PREFIX_LOWER_BOUND: usize = 2;
const PREFIX_UPPER_BOUND: usize = 8;

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
fn string_to_u64(string_to_convert: &String, str_to_u64: &HashMap<char, u64>) -> u64 {
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

fn parse_to_vec(path_as_u64: u64) -> Vec<(usize, usize)> {
    let mut path_repr: [(usize, usize); 12] = [(0, 0); 12];

    let mut mut_path = path_as_u64;
    let mut max_bit = 12;

    while mut_path & 16 == 16 {
        let y = (mut_path & 3) as usize;
        mut_path >>= 2;
        let x = (mut_path & 3) as usize;
        mut_path >>= 3;
        max_bit -= 1;
        path_repr[max_bit] = (x, y);
    }

    return path_repr[max_bit..].iter().map(|c| *c).collect();
}

// TODO: This can probably be made a lot faster, since we discard the strings after we've
//       generated a u64 from them:
//       Should try to use a buffer string.
/// Returns a HashSet containing all prefixes of length PREFIX_LOWER_BOUND to PREFIX_UPPER_BOUND
/// in their u64 representation.
fn read_prefixes(str_to_u64: &HashMap<char, u64>) -> HashSet<u64, BuildHasherDefault<FnvHasher>> {
//    let mut prefixes: HashSet<u64> = HashSet::with_capacity(250000);
    let mut prefixes = FnvHashSet::with_capacity_and_hasher(250000, Default::default());
    let file = File::open("./data/prefixes/prefixes.txt").unwrap();

    let mut reader = BufReader::new(file);
    let mut s = String::new();
    loop {
        s.clear();
        let res = reader.read_line(&mut s).unwrap();
        if res != 0 {
            prefixes.insert(string_to_u64(&s[..res-2].to_owned(), str_to_u64));
        } else {
            break;
        }
    }
    return prefixes;
}

fn read_prefixes_to_u64() -> HashSet<u64, BuildHasherDefault<FnvHasher>> {
//    let mut prefixes: HashSet<u64> = HashSet::with_capacity(250000);
    let mut prefixes = FnvHashSet::with_capacity_and_hasher(350000, Default::default());
    let file = File::open("./data/prefixes/binary.bin").unwrap();

    //         let res = reader.read_u64::<BigEndian>().unwrap();
    let mut reader = BufReader::new(file);
    let mut s = [0; 8];
    loop {
        let res = reader.read(&mut s).unwrap();
        if res == 8 {
            let val = u64::from_be_bytes(s);
            prefixes.insert(val);
        } else {
            break;
        }
    }
    return prefixes;
}

// TODO: This can probably be made a lot faster, since we discard the strings after we've
//       generated a u64 from them:
//       Should try to use a buffer string.
/// Returns a HashSet containing all words in their u64 representation.
fn read_dict(str_to_u64: &HashMap<char, u64>) -> HashSet<u64> {
    let mut dict: HashSet<u64> = HashSet::with_capacity(200000);
    let file = File::open("./data/TWL06Trimmed.txt").unwrap();

    let mut reader = BufReader::new(file);
    let mut s = String::new();
    loop {
        s.clear();
        let res = reader.read_line(&mut s).unwrap();
        if res != 0 {
            dict.insert(string_to_u64(&s[..res-2].to_owned(), str_to_u64));
        } else {
            break;
        }
    }
    return dict;
}

fn read_binary_dict() -> HashSet<u64> {
    let mut dict = HashSet::with_capacity(200000);
    let file = File::open("./data/TWL06/binary.bin").unwrap();

    //         let res = reader.read_u64::<BigEndian>().unwrap();
    let mut reader = BufReader::new(file);
    let mut s = [0; 8];
    loop {
        let res = reader.read(&mut s).unwrap();
        if res == 8 {
            dict.insert(u64::from_be_bytes(s));
        } else {
            break;
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
    for i in 0..BOARD_SIZE {
        for j in 0..BOARD_SIZE {
            dfs_helper_non_recursive(board, (i << 2) | j);
        }
    }
}

/// Generates the necessary arguments to launch dfs_non_recursive.
fn dfs_helper_non_recursive(board: &mut Board, start_point: usize) {
    dfs_non_recursive(board, gen_graph(), start_point, board.board[start_point],
                      board.points[start_point], board.word_int_mults[start_point]);
}

/// A non recursive depth first search which identifies all words, and adds them to
/// board.word_info_as_str with their string representation, score and path. Returns nothing.
fn dfs_non_recursive(board: &mut Board, graph: Vec<Vec<usize>>, start_point: usize, start_char: u64,
                     start_pts: usize, start_mult: usize) {
    let mut stack: Vec<(u64, u64, usize, usize, usize)> = Vec::with_capacity(40);
    let path = (16 | start_point) as u64;
    stack.push((path, start_char, start_pts, start_mult, 1));

    while !stack.is_empty() {
        let popped_item = stack.pop().unwrap();
        let (path, word, word_pts, word_mult, mut word_len) = popped_item;

        if (word_len >= MIN_WORD_LEN) & (word_len <= MAX_WORD_LEN) & board.dictionary.contains(&word) {
            let mut score = word_pts * word_mult;
            if word_len > 4 {
                score += 5 * (word_len - 4);
            }
            board.word_info_as_str.push((parse_to_str(word), score, parse_to_vec(path)));
        }

        let vert = (path & 15) as usize;
        word_len += 1;
        for vertex_addr in graph[vert].iter() {
            let vertex = *vertex_addr;

            let mut path_contains = false;
            let mut mut_path = path as usize;

            for _i in 0..word_len - 1 {
                if (mut_path & 15) == vertex {
                    path_contains = true;
                    break;
                }
                mut_path = mut_path >> 5;
            }

            if !path_contains {
                let temp_word = (word << 5) | board.board[vertex];
                let path_clone = (path << 5) | 16 | (vertex as u64);

                if (word_len <= PREFIX_UPPER_BOUND) & !board.prefixes.contains(&temp_word) {
                    continue;
                }

                if word_len == MAX_WORD_LEN {
                    if board.dictionary.contains(&temp_word) {
                        let score = word_pts * word_mult + 40;
                        board.word_info_as_str.push((parse_to_str(temp_word),
                                                     score, parse_to_vec(path_clone)));
                    }

                    continue;
                }


                stack.push((path_clone, temp_word, word_pts + board.points[vertex],
                            word_mult * board.word_int_mults[vertex], word_len));
            }
        }
    }
}

///// Fills the board with word info using the recursive DFS algorithm.
//#[allow(dead_code)]
//fn all_combos_r(board: &mut Board) {
//    for i in 0..BOARD_SIZE {
//        for j in 0..BOARD_SIZE {
//            dfs_helper(board, (i, j));
//        }
//    }
//}
//
///// Generates the necessary arguments to launch dfs.
//#[allow(dead_code)]
//fn dfs_helper(board: &mut Board, start_point: (usize, usize)) {
//    let (x, y) = start_point;
//    board.visited = gen_visited();
//    dfs(board, start_point, board.board[x][y], board.points[x][y],
//        board.word_int_mults[x][y], 1, vec![start_point]);
//}
//
///// A recursive depth first search which identifies all possible words in the board, and adds
///// them to board.word_info_as_str with their string representation, score and path.
///// Returns nothing.
//#[allow(dead_code)]
//fn dfs(board: &mut Board, start_point: (usize, usize), word: u64, word_pts: usize,
//       word_mult: usize, mut word_len: usize, path: Vec<(usize, usize)>) {
//    if (word_len >= MIN_WORD_LEN) & board.dictionary.contains(&word) {
//        let mut score = word_pts * word_mult;
//        if word_len > 4 {
//            score += 5 * (word_len - 4);
//        }
//        board.word_info_as_str.push((parse_to_str(word), score, path.clone()));
//    }
//
//    let (xs, ys) = start_point;
//
//    board.visited[xs][ys] = true;
//    word_len += 1;
//
//    for vertex in board.graph[xs][ys].clone() {
//        let (x, y) = vertex;
//        if !board.visited[x][y] {
//            let temp_word = (word << 5) | board.board[x][y];
//
//            if word_len >= MIN_WORD_LEN {
//                if (word_len <= PREFIX_UPPER_BOUND) & !board.prefixes.contains(&temp_word) {
//                    continue;
//                }
//                if (word_len == MAX_WORD_LEN) & board.dictionary.contains(&temp_word) {
//                    let score = word_pts * word_mult + 40;
//                    let mut path_clone = path.clone();
//                    path_clone.push(vertex);
//                    board.word_info_as_str.push((parse_to_str(temp_word), score, path_clone));
//                    continue;
//                }
//            }
//
//            board.visited[x][y] = true;
//
//            let mut path_clone = path.clone();
//            path_clone.push(vertex);
//            dfs(board, vertex, temp_word, word_pts + board.points[x][y],
//                word_mult * board.word_int_mults[x][y], word_len, path_clone);
//
//            board.visited[x][y] = false;
//        }
//    }
//}

/// Generates a graph of all possible neighbouring vertices, represented with adjacency lists.
fn gen_graph() -> Vec<Vec<usize>> {
    let mut graph: Vec<Vec<usize>> = Vec::new();
    let directions: Vec<(i8, i8)> = vec![(1, 0), (-1, 0), (0, 1), (0, -1),
                                         (1, 1), (1, -1), (-1, 1), (-1, -1)];

    for i in 0..BOARD_SIZE_I8 {
        for j in 0..BOARD_SIZE_I8 {
            let mut neighbours = Vec::new();
            for (cx, cy) in &directions {
                let x = i + *cx;
                let y = j + *cy;
                if (0 <= x) & (x < 4) & (0 <= y) & (y < 4) {
                    neighbours.push(((x << 2) | y) as usize);
                }
            }
            graph.push(neighbours);
        }
    }
    return graph;
}

fn write_to_file(board: Board) {
    let mut file = File::create("./words.txt").unwrap();

    for (word, score, path) in board.word_info_as_str {
        file.write(format!("{}, {}, [", word, score).as_bytes()).unwrap();

        for i in 0..path.len() - 1 {
            let (x, y) = path[i];
            file.write(format!("({}, {}), ", x, y).as_bytes()).unwrap();
        }
        let (x, y) = *path.last().unwrap();
        file.write(format!("({}, {})]\n", x, y).as_bytes()).unwrap();
    }
}
/// Generates a matrix of vertices that have been visited.
fn gen_visited() -> Vec<bool> {
    [false; BOARD_SIZE * BOARD_SIZE].to_vec()
}

/// Maintains the board state and words found in the board.
struct Board {
    word_info_as_str: Vec<(String, usize, Vec<(usize, usize)>)>,
    board: Vec<u64>,
    points: Vec<usize>,
    word_int_mults: Vec<usize>,
    prefixes: HashSet<u64, BuildHasherDefault<FnvHasher>>,
    dictionary: HashSet<u64>,
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

    let prefixes = read_prefixes_to_u64();
    // read_prefixes(&str_to_u64_map);
    let dictionary = read_binary_dict();
    let raw_board = read_board(PATH_TO_BOARD.to_string());

    println!("{}", now.elapsed().as_secs_f32());

    let (board, word_mults) = parse_board_and_mults(raw_board, &str_to_u64_map);
    let word_int_mults = parse_word_mults_to_int_mults(&word_mults);
    let points = get_points(&board, &word_mults);

    let mut flat_board = Vec::with_capacity(16);
    for mut row in board {
        flat_board.append(&mut row);
    }
    let mut flat_points = Vec::with_capacity(16);
    for mut row in points {
        flat_points.append(&mut row);
    }
    let mut flat_word_int_mults = Vec::with_capacity(16);
    for mut row in word_int_mults {
        flat_word_int_mults.append(&mut row);
    }

    let mut ruzzle_board = Board {
        word_info_as_str: Vec::new(),
        board: flat_board,
        points: flat_points,
        word_int_mults: flat_word_int_mults,
        prefixes,
        dictionary,
    };

    let solver_now = Instant::now();

    all_combos(&mut ruzzle_board);

    write_to_file(ruzzle_board);
}
