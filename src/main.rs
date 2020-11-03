use std::collections::{HashSet, HashMap};
use std::fs::File;
use std::io::{BufReader, BufRead, Read, Write, BufWriter};
use std::time::Instant;
use fnv::{FnvHashSet, FnvHasher};
use std::hash::BuildHasherDefault;

// const PREFIXES: &'static [u8] = include_bytes!("../data/prefixes/binary.bin");
// const DICT: &'static [u8] = include_bytes!("../data/TWL06/binary.bin");

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

// const PATH_TO_PREFIXES: &str = r"data/prefixes/";
// const PATH_TO_DICTIONARY: &str = r"data/";
const PATH_TO_BOARD: &str = r"board.txt";

const MIN_WORD_LEN: u8 = 2;
const MAX_WORD_LEN: u8 = 12;

const BOARD_SIZE: usize = 4;
const BOARD_SIZE_I8: i8 = BOARD_SIZE as i8;
const POINT_VALS: [u8; 27] = [0, 1, 4, 4, 2, 1, 4, 3, 4, 1, 10, 5, 1, 3, 1, 1, 4,
    10, 1, 1, 1, 2, 4, 4, 8, 4, 8];

// These options can be tweaked to improve performance if necessary.
const PREFIX_LOWER_BOUND: u8 = 2;
const PREFIX_UPPER_BOUND: u8 = 8;

const U64_TO_CHAR: [char; 30] = ['!', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '2', '3', '-'];

const U64_TO_U8: [u8; 30] = *b"!ABCDEFGHIJKLMNOPQRSTUVWXYZ23-";

// Convenience.
const D_U8: u8 = 4;
const T_U8: u8 = 20;


const TWO_U8: u8 = 27 as u8;
const THREE_U8: u8 = 28 as u8;

// const TWO_U64: u64 = 27;
// const THREE_U64: u64 = 28;
// const DASH_U64: u64 = 29;

//const DICT_BYTES: &'static [u8] = include_bytes!("../data/TWL06/binary.bin");
//const PREFIX_BYTES: &'static [u8] = include_bytes!("../data/prefixes/binary.bin");

/// Maintains the board state and words found in the board.
struct Board {
    //              word, score, path
    word_info: Vec<(u64, u16, u64)>,
    board: [u8; BOARD_SIZE * BOARD_SIZE],
    points: [u8; BOARD_SIZE * BOARD_SIZE],
    word_int_mults: [u8; BOARD_SIZE * BOARD_SIZE],
    prefixes: HashSet<u64, BuildHasherDefault<FnvHasher>>,
    dictionary: HashSet<u64, BuildHasherDefault<FnvHasher>>,
}

impl Board {
    fn sort_entries(self: &mut Self) {
        self.word_info.sort_by(|a, b| b.1.cmp(&a.1))
    }

    fn write_to_file(self: &Self) {
        let file = File::create("./words.txt").unwrap();
        let mut buf_writer = BufWriter::with_capacity(24 * 1024, file);

        let mut path_buf = *b"(0, 0), (0, 0), (0, 0), (0, 0), \
        (0, 0), (0, 0), (0, 0), (0, 0), \
        (0, 0), (0, 0), (0, 0), (0, 0), ";
        let mut path_buf_ = [(0, 0); 12];
        let mut str_repr = [0; 12];
        let mut first_word_buf = [0u8; 160];
        for (word, score, mut path_as_u64) in self.word_info.iter() {
            // assert!(path.len() * 8 <= path_buf.len());
            let max_bit = path_to_vec_buffered(path_as_u64, &mut path_buf_);
            for (i, &(x, y)) in path_buf_[max_bit..11].iter().enumerate() {
                path_buf[i * 8 + 1] = x + 48;
                path_buf[i * 8 + 4] = y + 48;
            }
            let (x, y) = path_buf_[11];
            let i = 11 - max_bit;
            path_buf[i * 8 + 1] = x + 48;
            path_buf[i * 8 + 4] = y + 48;
            let max_bit = parse_to_str_buf(*word, &mut str_repr);
            first_word_buf[..12-max_bit].clone_from_slice(&str_repr[max_bit..]);
            first_word_buf[12-max_bit] = b',';
            first_word_buf[13-max_bit] = b' ';
            let score = score.to_string().into_bytes();
            let score_len = score.len();
            first_word_buf[14-max_bit..14-max_bit+score_len].clone_from_slice(score.as_slice());
            first_word_buf[14-max_bit+score_len] = b',';
            first_word_buf[15-max_bit+score_len] = b' ';
            first_word_buf[16-max_bit+score_len] = b'[';
            first_word_buf[17-max_bit+score_len..17-max_bit+score_len+i*8+6].clone_from_slice(&path_buf[..i * 8 + 6]);
            first_word_buf[17-max_bit+score_len+i*8+6] = b']';
            first_word_buf[17-max_bit+score_len+i*8+7] = b'\n';
            buf_writer.write_all(&first_word_buf[..17-max_bit+score_len+i*8+8]).unwrap();
        }
    }
}

/// A non recursive depth first search which identifies all words, and adds them to
/// board.word_info_as_str with their string representation, score and path. Returns nothing.
fn dfs(board: &mut Board, graph: Vec<Vec<u8>>) {
    let mut stack: Vec<(u64, u64, u16, u8, u8, u16)> = Vec::with_capacity(120);
    for i in 0..BOARD_SIZE * BOARD_SIZE {
        stack.push(((0b10000 | i) as u64,
        board.board[i] as u64,
        board.points[i] as u16,
        board.word_int_mults[i],
        1, 0));
    }
    /// Paths consist of 12 five bit vertices:
    /// [continuation_flag:1][x:2][y:2]

    // This whole thing takes about 100ns per iteration, on average (0.0006s for 6000 fn calls).
    // Out of 72846 values, we narrow it down to 6000 -> produce 410 results.
    // Most pruning occurs around 4-8 values. 2-3 doesn't really do much, but the cost of hashing
    // is roughly equal to the cost of going through a full operation. Past 9 values, most of the tree
    // is already completed.
    while let Some((path, word, word_pts, word_mult, mut word_len, mut visited)) = stack.pop() {
        if (word_len >= MIN_WORD_LEN) & (word_len <= MAX_WORD_LEN) & board.dictionary.contains(&word) {
            let mut score = word_pts * (word_mult as u16);
            if word_len > 4 {
                score += 5 * (word_len as u16 - 4);
            }

            // Parsing words takes very little time - only ~3% of calls get this far.
            board.word_info.push((word, score, path));
        }

        let vert = path & 0xF;
        visited |= 1 << vert;
        word_len += 1;

        for &vertex in &graph[vert as usize] {
            if ((visited >> vertex) & 1) == 0 {
                let temp_word = (word << 5) | (board.board[vertex as usize] as u64);

                // Testing bloom filters doesn't really suggest a significant difference.
                if word_len >= PREFIX_LOWER_BOUND && word_len <= PREFIX_UPPER_BOUND && !board.prefixes.contains(&temp_word) {
                    continue;
                }

                let path_clone = (path << 5) | 0b10000 | (vertex as u64);

                if word_len == MAX_WORD_LEN {
                    if board.dictionary.contains(&temp_word) {
                        let score = word_pts * (word_mult as u16) + 40;
                        board.word_info.push((temp_word,
                                              score, path_clone));
                    }
                    continue;
                }

                stack.push((path_clone, temp_word, word_pts + board.points[vertex as usize] as u16,
                            word_mult * board.word_int_mults[vertex as usize], word_len, visited | (1 << vertex)));
            }
        }
    }
}


// /// Returns a u64 representation of any string that is twelve characters or less, and only
// /// contains the letters A-Z. Every five bits, up until 60 bits or five zero bits occur,
// /// correspond to the letter in U64_TO_CHAR with the same index (eg. as usize).
// fn string_to_u64(string_to_convert: &String, str_to_u64: &HashMap<char, u8>) -> u64 {
//     let mut output: u64 = 0;
//     for c_u64 in string_to_convert.chars().map(|c| str_to_u64[&c]) {
//         output = (output << 5) | (c_u64 as u64);
//     }
//     return output;
// }

fn parse_to_str_buf(str_as_num: u64, str_repr: &mut [u8; 12]) -> usize {
    let mut str_numbers = str_as_num;
    let mut max_bit = 12;

    for _ in 0..12 {
        // Read the last five bits.
        let val = (str_numbers & 0b11111) as usize;
        if val != 0 {
            max_bit -= 1;
            str_numbers >>= 5;
            str_repr[max_bit] = U64_TO_U8[val];
        } else {
            break;
        }
    }

    max_bit
}

/// Generates the string representation of any string that is represented in the first 60
/// bits of str_as_num, where each group of five consecutive bits corresponds the the character
/// at the index in U64_TO_CHAR.
fn parse_to_str(str_as_num: u64) -> String {
    let mut str_repr = [U64_TO_U8[0]; 12];
    let mut str_numbers = str_as_num;
    let mut max_bit = 12;

    for _ in 0..12 {
        // Read the last five bits.
        let val = (str_numbers & 0b11111) as usize;
        if val != 0 {
            max_bit -= 1;
            str_numbers >>= 5;
            str_repr[max_bit] = U64_TO_U8[val];
        } else {
            break;
        }
    }

    unsafe {
        return std::str::from_utf8_unchecked(&str_repr[max_bit..]).to_string();
    }
    // str_repr[max_bit..].iter().collect()
}

fn path_to_vec_buffered(path_as_u64: u64, buf: &mut [(u8, u8); 12]) -> usize {
    let mut mut_path = path_as_u64;
    let mut max_bit = 12;

    while mut_path & 0b10000 == 0b10000 {
        let y = (mut_path & 0b11) as u8;
        mut_path >>= 2;
        let x = (mut_path & 0b11) as u8;
        mut_path >>= 3;
        max_bit -= 1;
        // unsafe {
        //     let ind = buf.get_unchecked_mut(max_bit);
        //     *ind = (x, y);
        // }

        buf[max_bit] = (x, y);
    }

    max_bit
}

fn path_to_vec(path_as_u64: u64) -> Vec<(u8, u8)> {
    let mut path_repr = [(0, 0); 12];

    let mut mut_path = path_as_u64;
    let mut max_bit = 12;

    while mut_path & 0b10000 == 0b10000 {
        let y = (mut_path & 0b11) as u8;
        mut_path >>= 2;
        let x = (mut_path & 0b11) as u8;
        mut_path >>= 3;
        max_bit -= 1;
        path_repr[max_bit] = (x, y);
    }

    return path_repr[max_bit..].to_vec();
}


// /// Returns a HashSet containing all prefixes of length PREFIX_LOWER_BOUND to PREFIX_UPPER_BOUND
// /// in their u64 representation.
// fn read_prefixes(str_to_u64: &HashMap<char, u8>) -> HashSet<u64, BuildHasherDefault<FnvHasher>> {
//     let mut prefixes = FnvHashSet::with_capacity_and_hasher(250000, Default::default());
//     let file = File::open("./data/prefixes/prefixes.txt").unwrap();
//
//     let mut reader = BufReader::new(file);
//     let mut s = String::new();
//
//     while let res = reader.read_line(&mut s).expect("Reading prefixes failed.") {
//         if res == 0 {
//             break;
//         }
//         prefixes.insert(string_to_u64(&s[..res - 2].to_owned(), str_to_u64));
//     }
//     return prefixes;
// }

/// Reads files which have been preprocessed to be compressed string representations.
fn read_binary_prefixes() -> HashSet<u64, BuildHasherDefault<FnvHasher>> {
    let mut prefixes = FnvHashSet::with_capacity_and_hasher(275944 + 1, Default::default());
    let file = File::open("./data/prefixes/binary.bin").unwrap();

    let mut reader = BufReader::new(file);
    let mut s = [0; 8];

    while reader.read(&mut s).expect("Reading binary prefix file failed.") == 8 {
        prefixes.insert(u64::from_be_bytes(s));
    }

    return prefixes;
}


// fn parse_prefixes() -> HashSet<u64, BuildHasherDefault<FnvHasher>> {
//     assert_eq!(PREFIXES.len() & 0b111, 0);
//     let mut prefixes = FnvHashSet::with_capacity_and_hasher(275944 + 1, Default::default());
//
//     let mut s = [0u8; 8];
//
//     for i in 0..PREFIXES.len() / 8 {
//         s.clone_from_slice(&PREFIXES[i*8..i*8+8]);
//         prefixes.insert(u64::from_be_bytes(s));
//     }
//
//     return prefixes;
// }
//

// /// Returns a HashSet containing all words in their u64 representation.
// fn read_dict(str_to_u64: &HashMap<char, u8>) -> HashSet<u64, BuildHasherDefault<FnvHasher>> {
//     let mut dict = FnvHashSet::with_capacity_and_hasher(200000, Default::default());
//     let file = File::open("./data/TWL06Trimmed.txt").unwrap();
//
//     let mut reader = BufReader::new(file);
//     let mut s = String::new();
//
//     while let res = reader.read_line(&mut s).expect("Reading dictionary failed.") {
//         if res == 0 {
//             break;
//         }
//         dict.insert(string_to_u64(&s[..res - 2].to_owned(), str_to_u64));
//     }
//
//     return dict;
// }

fn read_binary_dict() -> HashSet<u64, BuildHasherDefault<FnvHasher>> {
    let mut dict = FnvHashSet::with_capacity_and_hasher(162725 + 1, Default::default());
    let file = File::open("./data/TWL06/binary.bin").unwrap();
    let mut reader = BufReader::new(file);

    let mut s = [0; 8];
    while reader.read(&mut s).expect("Reading binary dictionary failed.") == 8 {
        dict.insert(u64::from_be_bytes(s));
    }

    return dict;
}

// fn parse_dict() -> HashSet<u64, BuildHasherDefault<FnvHasher>> {
//     assert_eq!(DICT.len() & 0b111, 0);
//     let mut dict = FnvHashSet::with_capacity_and_hasher(162725 + 1, Default::default());
//     let mut s = [0u8; 8];
//
//     for i in 0..DICT.len() / 8 {
//         s.clone_from_slice(&DICT[i*8..i*8+8]);
//         dict.insert(u64::from_be_bytes(s));
//     }
//
//     return dict;
// }
//
/// Reads the file at file_path into a vector, line for line, and returns it.
fn read_board(file_path: String) -> Vec<String> {
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);
    reader.lines().map(|line| line.unwrap()).collect()
}

/// Parses the raw board using str_to_u64 to provide the correct mapping of
/// characters to u64 values with the lower 5 bits set. The lines up to the first blank line
/// are parsed into the first return value. All proceeding lines are parsed into the second
/// return value.
fn parse_board_and_mults(
    raw_board: Vec<String>,
    str_to_u8: &HashMap<char, u8>) -> ([u8; BOARD_SIZE * BOARD_SIZE], [u8; BOARD_SIZE * BOARD_SIZE]) {
    let mut board = [0; BOARD_SIZE * BOARD_SIZE];
    let mut word_mults = [0; BOARD_SIZE * BOARD_SIZE];
    let mut all_chars = Vec::with_capacity(2 * BOARD_SIZE * BOARD_SIZE);

    for line in raw_board {
        let mut symbols = line
            .chars()
            .filter(|c| *c != ' ')
            .map(|c| str_to_u8[&c]).collect();

        all_chars.append(& mut symbols);

        if all_chars.len() > 2 * BOARD_SIZE * BOARD_SIZE {
            break;
        }
    }

    assert!(all_chars.len() >= 2 * BOARD_SIZE * BOARD_SIZE);

    for i in 0..BOARD_SIZE * BOARD_SIZE {
        board[i] = all_chars[i];
        word_mults[i] = match all_chars[BOARD_SIZE * BOARD_SIZE + i] {
            TWO_U8 => 2,
            THREE_U8 => 3,
            _ => 1,
        };
    }

    return (board, word_mults);
}

// /// Takes the u64 mults (which correspond to characters in the alphabet), and maps them
// /// to their usize multipliers.
// fn parse_word_mults_to_int_mults(word_mults: &Vec<Vec<u64>>) -> Vec<Vec<u16>> {
//     let mut word_int_mults = Vec::new();
//
//     for line in word_mults {
//         word_int_mults.push(line.iter().map(|l| match *l {
//             TWO_U64 => 2,
//             THREE_U64 => 3,
//             _ => 1,
//         }).collect());
//     }
//     return word_int_mults;
// }

/// Returns the points for each letter on the board.
fn get_points(board: &[u8; BOARD_SIZE * BOARD_SIZE], word_mults: &[u8; BOARD_SIZE * BOARD_SIZE]) -> [u8; BOARD_SIZE * BOARD_SIZE] {
    let mut points = [0; BOARD_SIZE * BOARD_SIZE];

    for (index, (letter, mult)) in board.iter().zip(word_mults).enumerate() {
        points[index] = POINT_VALS[*letter as usize] * match *mult {
                D_U8 => 2,
                T_U8 => 3,
                _ => 1,
            };
    }

    return points;
}

/// Generates a graph of all possible neighbouring vertices, represented with adjacency lists.
fn gen_graph() -> Vec<Vec<u8>> {
    let mut graph: Vec<Vec<u8>> = Vec::new();
    let directions: [(i8, i8); 8] = [(1, 0), (-1, 0), (0, 1), (0, -1),
        (1, 1), (1, -1), (-1, 1), (-1, -1)];

    for i in 0..BOARD_SIZE_I8 {
        for j in 0..BOARD_SIZE_I8 {
            let mut neighbours = Vec::new();
            for (cx, cy) in &directions {
                let x = i + *cx;
                let y = j + *cy;
                if (0 <= x) && (x < 4) && (0 <= y) && (y < 4) {
                    neighbours.push(((x << 2) | y) as u8);
                }
            }
            graph.push(neighbours);
        }
    }
    return graph;
}


fn main() {
    let str_to_u64_map: HashMap<char, u8> = map!{'A' => 1, 'B'=> 2, 'C'=> 3, 'D'=> 4, 'E'=> 5,
    'F'=> 6, 'G'=> 7, 'H'=> 8, 'I'=> 9, 'J'=> 10, 'K'=> 11, 'L' => 12, 'M' => 13, 'N'=> 14,
    'O'=> 15, 'P'=> 16, 'Q'=> 17, 'R'=> 18, 'S'=> 19, 'T'=> 20, 'U'=> 21, 'V'=> 22, 'W'=> 23,
    'X' => 24, 'Y' => 25, 'Z'=> 26, '2' => 27, '3' => 28, '-' => 29};

    let now = Instant::now();

    let raw_board = read_board(PATH_TO_BOARD.to_string());
    let prefixes = read_binary_prefixes();
    let dictionary = read_binary_dict();

    println!("Files took {}s to read.", now.elapsed().as_secs_f32());

    let (board, word_int_mults) = parse_board_and_mults(raw_board, &str_to_u64_map);
    let points = get_points(&board, &word_int_mults);

    let mut ruzzle_board = Board {
        word_info: Vec::with_capacity(500),
        board,
        points,
        word_int_mults,
        prefixes,
        dictionary,
    };

    let now = Instant::now();

    dfs(&mut ruzzle_board, gen_graph());

    println!("Board solving took {}s.", now.elapsed().as_secs_f32());

    ruzzle_board.sort_entries();
    println!("{} solutions were found.", ruzzle_board.word_info.len());

    let now = Instant::now();
    ruzzle_board.write_to_file();
    println!("File writing took {}s.", now.elapsed().as_secs_f32());
}