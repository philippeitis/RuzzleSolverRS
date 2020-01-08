use std::collections::{HashSet, HashMap};
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::time::Instant;

static PATH_TO_PREFIXES: &str = r"data/prefixes/";
static PATH_TO_DICTIONARY: &str = r"data/TWL06/";
static PATH_TO_BOARD: &str = r"board.txt";

// These options can be tweaked to improve performance if necessary."""
static PREFIX_LOWER_BOUND: usize = 2;
static PREFIX_UPPER_BOUND: usize = 12;
static U64_TO_CHAR: [char; 27] = ['!', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'];


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


static D_U64: u64 = 4;
static T_U64: u64 = 20;
static TWO_U64: u64 = 27;
static THREE_U64: u64 = 28;
static DASH_U64: u64 = 29;


/// First, parse word to u64:
/// each char from A to Z corresponds to a single 5 bit # (eg. 00001, 00010... - skip 0)
/// Can do for i in 1..27, char_hashmap.insert(i , chars[i-1]);
/// to concat to word, (word << 5) | char;
/// to translate back, each 5 bit # maps to a char, which we push onto a str
/// (should be preallocated with 12 chars)
/// This requires updating
/// read_prefixes
/// read_dict
/// read_board
/// dfs
///
/// Note that path can be represented as an u64 as well -> 2 bits is one coordinate,
/// and 4 bits is a coordinate pair - allowing 16 coordinate pairs in 1 i64
///

fn string_to_u64(string_to_convert: String, str_to_u64: &HashMap<char, u64>) -> u64 {
    let mut output: u64 = 0;
    for c in string_to_convert.chars() {
        output = (output << 5) | str_to_u64[&c];
    }
    return output;
}

fn read_prefixes(str_to_u64: &HashMap<char, u64>) -> HashSet<u64> {
    let mut prefixes: HashSet<u64> = HashSet::with_capacity(170000);
    for i in PREFIX_LOWER_BOUND..PREFIX_UPPER_BOUND + 1 {
        let file_name = format!("./{}prefixes{}L.txt", PATH_TO_PREFIXES, i);
        let file = File::open(file_name).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            prefixes.insert(string_to_u64( line.unwrap(), str_to_u64));
        }
    }
    return prefixes;
}

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

fn parse_board_and_mults(raw_board: Vec<String>, str_to_u64: &HashMap<char, u64>) -> (Vec<Vec<u64>>, Vec<Vec<u64>>) {
    /// Chars can be: upper case alphabet
    /// -, T, D, 2, 3
    let mut board = Vec::new();
    let mut word_mults = Vec::new();
    let mut is_still_board = true;

    for line in raw_board {
        let line_empty = line.is_empty();
        is_still_board = !line_empty & is_still_board;

        let mut symbols: Vec<u64> = Vec::new();
        for character in line.split(" ") {
            if !character.is_empty() {
                let single_character: char = character[0..].chars().next().unwrap();
                symbols.push(str_to_u64[&single_character]);
            }
        }

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

fn parse_word_mults_to_int_mults(word_mults: &Vec<Vec<u64>>) -> Vec<Vec<usize>> {
    let mut word_int_mults = Vec::new();

    for line in word_mults {
        let mut line_mults: Vec<usize> = Vec::new();
        for letter in line {
            if *letter == TWO_U64 {
                line_mults.push(2);
            } else if *letter == THREE_U64 {
                line_mults.push(3);
            } else {
                line_mults.push(1);
            }
        }
        word_int_mults.push(line_mults)
    }
    return word_int_mults;
}

fn get_points(board: &Vec<Vec<u64>>, word_mults: &Vec<Vec<u64>>) -> Vec<Vec<usize>> {
    let mut points = Vec::new();

    // Each index is a point - eg.
    let point_vals: [usize; 27] = [0, 1, 4, 4, 2, 1, 4, 3, 4, 1, 10, 5, 1, 3, 1, 1, 4,
        10, 1, 1, 1, 2, 4, 4, 8, 4, 8];

    for i in 0..board.len() {
        let mut line_points: Vec<usize> = Vec::new();
        let board_line = &board[i];
        let mult_line = &word_mults[i];
        for j in 0..board_line.len() {
            // let letter_point = *point_vals.get(&board_line[j]).unwrap();
            // this causes a panic.
            let letter_point = point_vals[board_line[j] as usize];
            let letter = mult_line[j];
            if letter == D_U64 {
                line_points.push(letter_point * 2);
            } else if letter == T_U64 {
                line_points.push(letter_point * 3);
            } else {
                line_points.push(letter_point);
            }
        }
        points.push(line_points);
    }
    return points;
}

fn dfs_helper_non_recursive(board: &mut Board, start_point: (usize, usize)) {
    let (x, y) = start_point;
    board.visited = gen_visited();
    dfs_non_recursive(board, start_point, board.board[x][y],
                      board.points[x][y], board.word_int_mults[x][y]);
}

fn dfs_non_recursive(board: &mut Board, start_point: (usize, usize), start_char: u64,
                     start_pts: usize, start_mult: usize) {
    let mut stack: Vec<(Vec<(usize, usize)>, u64, usize, usize, usize)> = Vec::with_capacity(25);
    stack.push((vec![start_point], start_char, start_pts, start_mult, 1));

    while !stack.is_empty() {
        let popped_item = stack.pop().unwrap();
        let (path, word, word_pts, word_mult, mut word_len) = popped_item;

        if (word_len >= 2) & board.dictionary.contains(&word) {
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

                if !board.prefixes.contains(&temp_word) {
                    continue;
                }

                if (word_len == 12) & board.dictionary.contains(&temp_word) {
                    let score = word_pts * word_mult + 40;
                    let mut path_clone = path.clone();
                    path_clone.push(vertex);
                    board.word_info_as_str.push((parse_to_str(temp_word), score, path_clone));
                    continue;
                }

                let mut path_clone = path.clone();
                path_clone.push(vertex);
                stack.push((path_clone, temp_word, word_pts + board.points[x][y],
                            word_mult * board.word_int_mults[x][y], word_len));
            }
        }
    }
}

fn dfs(board: &mut Board, start_point: (usize, usize), word: u64, word_pts: usize,
       word_mult: usize, mut word_len: usize, path: Vec<(usize, usize)>) {
    if (word_len >= 2) & board.dictionary.contains(&word) {
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

            if word_len >= 2 {
                if !board.prefixes.contains(&temp_word) {
                    continue;
                }
                if (word_len == 12) & board.dictionary.contains(&temp_word) {
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

fn gen_graph() -> Vec<Vec<Vec<(usize, usize)>>> {
    let mut graph: Vec<Vec<Vec<(usize, usize)>>> = Vec::new();
    let directions: Vec<(i8, i8)> = vec![(1, 0), (-1, 0), (0, 1), (0, -1),
                                         (1, 1), (1, -1), (-1, 1), (-1, -1)];

    for i in 0..4 {
        let mut graph_row: Vec<Vec<(usize, usize)>> = Vec::new();
        for j in 0..4 {
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

fn gen_visited() -> Vec<Vec<bool>> {
    let mut visited: Vec<Vec<bool>> = Vec::new();
    for _i in 0..4 {
        let mut visited_row: Vec<bool> = Vec::new();
        for _j in 0..4 {
            visited_row.push(false);
        }
        visited.push(visited_row)
    }
    return visited;
}

fn dfs_helper(board: &mut Board, start_point: (usize, usize)) {
    let (x, y) = start_point;
    board.visited = gen_visited();
    dfs(board, start_point, board.board[x][y], board.points[x][y],
        board.word_int_mults[x][y], 1, vec![start_point]);
}

fn all_combos(board: &mut Board) {
    for i in 0..4 {
        for j in 0..4 {
            dfs_helper_non_recursive(board, (i, j));
        }
    }
}

fn all_combos_r(board: &mut Board) {
    for i in 0..4 {
        for j in 0..4 {
            dfs_helper(board, (i, j));
        }
    }
}

fn parse_to_str(str_as_num: u64) -> String {
    let mut str_repr: [char; 12] = ['!'; 12];
    let mut str_numbers = str_as_num;
    let mut max_bit = 12;
    for i in 0..12 {
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
    let mut now = Instant::now();

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

//    let mut n_recurse_vect = Vec::new();
//    let mut recurse_vect = Vec::new();
    all_combos(&mut ruzzle_board);

    /// Should beat 0.006s
//    for _ in 0..10 {
//        n_recurse_vect.push(now.elapsed().as_secs_f32());
//        now = Instant::now();
//        all_combos_r(&mut ruzzle_board);
//        recurse_vect.push(now.elapsed().as_secs_f32());
//    }

    println!("{:#?}", now.elapsed().as_secs_f32());
//    println!("{:#?}", recurse_vect);
    let mut i = 0;
    for (word, score, _) in ruzzle_board.word_info_as_str {
        if i == 820 {
            break;
        }
        if i >= 410 {
            println!("{}, {}", word, score);
        }
        i += 1;
    }
}
