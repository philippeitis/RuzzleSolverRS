use std::collections::{HashSet, HashMap};
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::time::Instant;

static STR_TO_U8_MAP: HashMap<char, u8> = map! {'A' => 1, 'B'=> 2, 'C'=> 3, 'D'=> 4, 'E'=> 5,
 'F'=> 6, 'G'=> 7, 'H'=> 8, 'I'=> 9, 'J'=> 10, 'K'=> 11, 'L' => 12, 'M' => 13, 'N'=> 14,
  'O'=> 15, 'P'=> 16, 'Q'=> 17, 'R'=> 18, 'S'=> 10, 'T'=> 20, 'U'=> 21, 'V'=> 22, 'W'=> 23,
   'X' => 24, 'Y' => 25, 'Z'=> 26};

static PATH_TO_PREFIXES: &str = r"data/prefixes/";
static PATH_TO_DICTIONARY: &str = r"data/TWL06/";
static PATH_TO_BOARD: &str = r"board.txt";
//static PRINT_INFO: bool = true;

// Ruzzle Rules
static MIN_WORD_LEN: usize = 2;
static MAX_WORD_LEN: usize = 12;
static BOARD_SIZE: usize = 4;
static BOARD_SIZE_I8: i8 = BOARD_SIZE as i8;

// These options can be tweaked to improve performance if necessary."""
static PREFIX_LOWER_BOUND: usize = 2;
static PREFIX_UPPER_BOUND: usize = 12;

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
fn read_prefixes() -> HashSet<String> {
    let mut prefixes: HashSet<String> = HashSet::with_capacity(170000);
    for i in PREFIX_LOWER_BOUND..PREFIX_UPPER_BOUND + 1 {
        let file_name = format!("./{}prefixes{}L.txt", PATH_TO_PREFIXES, i);
        let file = File::open(file_name).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.unwrap(); // Ignore errors.
            prefixes.insert(line);
        }
    }
    return prefixes;
}

fn read_dict() -> HashSet<String> {
    let mut dict: HashSet<String> = HashSet::with_capacity(163000);
    for i in 0..MAX_WORD_LEN + 1 {
        let file_name = format!("./{}TWL06Trimmed{}L.txt", PATH_TO_DICTIONARY, i);
        let file = File::open(file_name).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            // Ignore errors.
            let line = line.unwrap();
            dict.insert(line);
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

fn parse_board_and_mults(raw_board: Vec<String>) -> (Vec<Vec<char>>, Vec<Vec<char>>) {
    let mut board = Vec::new();
    let mut word_mults = Vec::new();
    let mut is_still_board = true;

    for line in raw_board {
        let line_empty = line.is_empty();
        is_still_board = !line_empty & is_still_board;

        let mut symbols: Vec<char> = Vec::new();
        for character in line.split(" ") {
            if !character.is_empty() {
                symbols.push(character.parse().unwrap());
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

fn parse_word_mults_to_int_mults(word_mults: &Vec<Vec<char>>) -> Vec<Vec<usize>> {
    let mut word_int_mults = Vec::new();

    let c2: char = "2".parse().unwrap();
    let c3: char = "3".parse().unwrap();

    for line in word_mults {
        let mut line_mults: Vec<usize> = Vec::new();
        for letter in line {
            if letter.eq(&c2) {
                line_mults.push(2);
            } else if letter.eq(&c3) {
                line_mults.push(3);
            } else {
                line_mults.push(1);
            }
        }
        word_int_mults.push(line_mults)
    }
    return word_int_mults;
}

fn get_points(board: &Vec<Vec<char>>, word_mults: &Vec<Vec<char>>) -> Vec<Vec<usize>> {
    let mut points = Vec::new();

    let point_vals: HashMap<char, usize> = map! {'A' => 1, 'B'=> 4, 'C'=> 4, 'D'=> 2, 'E'=> 1, 'F'=> 4,
     'G'=> 3, 'H'=> 4, 'I'=> 1, 'J'=> 10, 'K'=> 5, 'L' => 1, 'M' => 3, 'N'=> 1, 'O'=> 1, 'P'=> 4,
      'Q'=> 10, 'R'=> 1, 'S'=> 1, 'T'=> 1, 'U'=> 2, 'V'=> 4, 'W'=> 4, 'X' => 8, 'Y' => 4, 'Z'=> 8};
    let d: char = "D".parse().unwrap();
    let t: char = "T".parse().unwrap();

    for i in 0..board.len() {
        let mut line_points: Vec<usize> = Vec::new();
        let board_line = &board[i];
        let mult_line = &word_mults[i];
        for j in 0..board_line.len() {
            // let letter_point = *point_vals.get(&board_line[j]).unwrap();
            // this causes a panic.
            let letter_point = point_vals[&board_line[j]];
            let letter = &mult_line[j];
            if letter.eq(&d) {
                line_points.push(letter_point * 2);
            } else if letter.eq(&t) {
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
    dfs_non_recursive(board, start_point, board.board[x][y].to_string(),
                      board.points[x][y], board.word_int_mults[x][y]);
}

fn dfs_non_recursive(board: &mut Board, start_point: (usize, usize), start_char: String, start_pts: usize,
                     start_mult: usize) {
    let mut stack: Vec<(Vec<(usize, usize)>, String, usize, usize, usize)> = Vec::with_capacity(25);

    stack.push((vec![start_point], start_char, start_pts, start_mult, 1));
    while !stack.is_empty() {
        let popped_item = stack.pop().unwrap();
        let (path, word, word_pts, word_mult, mut word_len) = popped_item;

        if (word_len >= MIN_WORD_LEN) & board.dictionary.contains(&word) {
            let mut score = word_pts * word_mult;
            if word_len > 4 {
                score += 5 * (word_len - 4);
            }
            board.words_info.push((word.clone(), score, path.clone()));
        }

        let (xs, ys) = path[word_len-1];
        word_len += 1;
        for vertex_addr in &board.graph[xs][ys] {
            let vertex = *vertex_addr;
            let (x, y) = vertex;

            if !path.contains(vertex_addr) {
                // This creates a nice speed boost since strings are allocated with 5 bytes of
                // data by default (I think), and this may require up to 2 resizes.
                let mut temp_word = String::with_capacity(word_len);
                temp_word.push_str(&word);
                temp_word.push(board.board[x][y]);

                if !board.prefixes.contains(&temp_word) {
                    continue;
                }

                if (word_len == MAX_WORD_LEN) & board.dictionary.contains(&temp_word) {
                    let score = word_pts * word_mult + 40;
                    let mut path_clone = path.clone();
                    path_clone.push(vertex);
                    board.words_info.push((temp_word, score, path_clone));
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

fn dfs(board: &mut Board, start_point: (usize, usize), word: String, word_pts: usize,
       word_mult: usize, mut word_len: usize, path: Vec<(usize, usize)>) {
    if (word_len >= MIN_WORD_LEN) & board.dictionary.contains(&word) {
        let mut score = word_pts * word_mult;
        if word_len > 4 {
            score += 5 * (word_len - 4);
        }
        board.words_info.push((word.clone(), score, path.clone()));
    }

    let (xs, ys) = start_point;

    board.visited[xs][ys] = true;
    word_len += 1;

    for vertex in board.graph[xs][ys].clone() {
        let (x, y) = vertex;
        if !board.visited[x][y] {
            let mut temp_word = String::with_capacity(word_len);
            temp_word.push_str(&word);
            temp_word.push(board.board[x][y]);

            if word_len >= MIN_WORD_LEN {
                if !board.prefixes.contains(&temp_word) {
                    continue;
                }
                if (word_len == MAX_WORD_LEN) & board.dictionary.contains(&temp_word) {
                    let score = word_pts * word_mult + 40;
                    let mut path_clone = path.clone();
                    path_clone.push(vertex);
                    board.words_info.push((temp_word, score, path_clone));
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

    for i in 0..BOARD_SIZE_I8 {
        let mut graph_row: Vec<Vec<(usize, usize)>> = Vec::new();
        for j in 0..BOARD_SIZE_I8 {
            let mut neighbours = Vec::new();
            for (cx, cy) in &directions {
                let x = i + *cx;
                let y = j + *cy;
                if (0 <= x) & (x < BOARD_SIZE_I8) & (0 <= y) & (y < BOARD_SIZE_I8) {
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
    for _i in 0..BOARD_SIZE {
        let mut visited_row: Vec<bool> = Vec::new();
        for _j in 0..BOARD_SIZE {
            visited_row.push(false);
        }
        visited.push(visited_row)
    }
    return visited;
}

fn dfs_helper(board: &mut Board, start_point: (usize, usize)) {
    let (x, y) = start_point;
    board.visited = gen_visited();
    dfs(board, start_point, board.board[x][y].to_string(), board.points[x][y],
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
    let mut str_repr = String::with_capacity(12);

}

struct Board {
    words_info: Vec<(String, usize, Vec<(usize, usize)>)>,
    board: Vec<Vec<char>>,
    points: Vec<Vec<usize>>,
    word_int_mults: Vec<Vec<usize>>,
    prefixes: HashSet<String>,
    dictionary: HashSet<String>,
    graph: Vec<Vec<Vec<(usize, usize)>>>,
    visited: Vec<Vec<bool>>,
}

fn main() {
    let prefixes = read_prefixes();
    let dictionary = read_dict();
    let raw_board = read_board(PATH_TO_BOARD.to_string());

    let (board, word_mults) = parse_board_and_mults(raw_board);
    let word_int_mults = parse_word_mults_to_int_mults(&word_mults);
    let points = get_points(&board, &word_mults);
    let mut ruzzle_board = Board {
        words_info: Vec::new(),
        board,
        points,
        word_int_mults,
        prefixes,
        dictionary,
        graph: gen_graph(),
        visited: gen_visited(),
    };

    let mut n_recurse_vect = Vec::new();
    let mut recurse_vect = Vec::new();

    /// Should beat 0.006s
    for _ in 0..10 {
        let mut now = Instant::now();
        all_combos(&mut ruzzle_board);
        n_recurse_vect.push(now.elapsed().as_secs_f32());
        now = Instant::now();
        all_combos_r(&mut ruzzle_board);
        recurse_vect.push(now.elapsed().as_secs_f32());
    }
    println!("{:#?}", n_recurse_vect);
    println!("{:#?}", recurse_vect);
    println!("{}", ruzzle_board.words_info.len());
}
