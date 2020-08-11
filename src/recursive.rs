

pub mod recursive_dfs {
    /// Fills the board with word info using the recursive DFS algorithm.
    fn all_combos(board: &mut Board) {
        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                helper(board, (i << 2) | j);
            }
        }
    }

    /// Generates the necessary arguments to launch dfs.
    fn helper(board: &mut Board, start_point: (usize, usize)) {
        let (x, y) = start_point;
        board.visited = gen_visited();
        dfs_recursive(board, gen_graph(), 0, start_point, board.board[start_point],
                      board.points[start_point], board.word_int_mults[start_point]);
    }

    /// A recursive depth first search which identifies all possible words in the board, and adds
    /// them to board.word_info_as_str with their string representation, score and path.
    /// Returns nothing.
    fn dfs(board: &mut Board, graph: Vec<Vec<usize>>, mut visited: usize, start_point: usize, start_word: u64,
           start_pts: usize, start_mult: usize, path: u64, mut word_len: usize) {
        if (word_len >= MIN_WORD_LEN) & (word_len <= MAX_WORD_LEN) & board.dictionary.contains(&word) {
            let mut score = word_pts * word_mult;
            if word_len > 4 {
                score += 5 * (word_len - 4);
            }
            board.word_info_as_str.push((parse_to_str(word), score, parse_to_vec(path)));
        }

        let vert = (path & 15) as usize;

        visited |= 1 << vert;
        word_len += 1;

        for vertex in *graph[vert].iter() {
            if !((visited >> vertex) & 1) {
                let temp_word = (word << 5) | board.board[vertex];
                let path_clone = (path << 5) | 16 | (vertex as u64);

                if word_len >= MIN_WORD_LEN {
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
                }

                let temp_visited = visited | (1 << vertex);
                dfs(board, graph, temp_visited, vertex, temp_word, word_pts + board.points[x][y],
                    word_mult * board.word_int_mults[x][y], path_clone, word_len);
            }
        }
    }
}

pub mod stack_dfs {
    /// Fills the board with information using the non recursive dfs algorithm.
    fn all_combos(board: &mut Board) {
        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                dfs_helper_non_recursive(board, (i << 2) | j);
            }
        }
    }

    /// Generates the necessary arguments to launch dfs_non_recursive.
    fn dfs_helper(board: &mut Board, start_point: usize) {
        dfs(board, gen_graph(), start_point, board.board[start_point],
                          board.points[start_point], board.word_int_mults[start_point]);
    }

    /// A non recursive depth first search which identifies all words, and adds them to
    /// board.word_info_as_str with their string representation, score and path. Returns nothing.
    fn dfs(board: &mut Board, graph: Vec<Vec<usize>>, start_point: usize, start_char: u64,
                         start_pts: usize, start_mult: usize) {
        let mut stack: Vec<(u64, u64, usize, usize, usize, usize)> = Vec::with_capacity(40);
        let path = (16 | start_point) as u64;
        stack.push((path, start_char, start_pts, start_mult, 1, 0));

        while !stack.is_empty() {
            let popped_item = stack.pop().unwrap();
            let (path, word, word_pts,
                word_mult, mut word_len, mut visited) = popped_item;

            if (word_len >= MIN_WORD_LEN) & (word_len <= MAX_WORD_LEN) & board.dictionary.contains(&word) {
                let mut score = word_pts * word_mult;
                if word_len > 4 {
                    score += 5 * (word_len - 4);
                }
                board.word_info_as_str.push((parse_to_str(word), score, parse_to_vec(path)));
            }

            let vert = (path & 15) as usize;
            visited |= 1 << vert;
            word_len += 1;

            for vertex_addr in graph[vert].iter() {
                let vertex = *vertex_addr;

                let mut path_contains = false;
                let mut mut_path = path as usize;

                if (visited & vertex) != vertex {
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

                    let temp_visited = visited | (1 << vertex);
                    stack.push((path_clone, temp_word, word_pts + board.points[vertex],
                                word_mult * board.word_int_mults[vertex], word_len, temp_visited));
                }
            }
        }
    }
}
