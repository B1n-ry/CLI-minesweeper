use crossterm::{event::{read, Event, KeyCode}, style::{Colorize, StyledContent}};
use std::collections::VecDeque;


static UNOPENED: char = '?';
static MINE: char = 'X';
static EMPTY: char = ' ';
static FLAG: char = 'P';
static NUMS: [char; 8] = ['1', '2', '3', '4', '5', '6', '7', '8'];

fn main() -> crossterm::Result<()> {
    // TODO: Add command line arguments for custom board size and bomb count
    let board_height: usize = 16;
    let board_width: usize = 16;

    let mut board: Vec<Vec<char>> = vec![vec![UNOPENED; board_width]; board_height];
    let mut bombs: Vec<[usize; 2]> = Vec::new();

    let mut cursor: [usize; 2] = [0, 0];
    update_board(&mut board, &mut cursor);

    let mut game_started: bool = false;

    let mut ticks: u32 = 0x12345678;
    let mut alive: bool = true;
    loop {
        ticks += 1;
        if let Event::Key(key_event) = read()? {
            if !alive {
                reset_game(&mut board, &mut bombs, &mut game_started, &mut alive);
                update_board(&mut board, &mut cursor);
                continue;
            }
            match key_event.code {
                KeyCode::Up => {
                    cursor[0] = (cursor[0] + board_height - 1) % board_height;
                },
                KeyCode::Down => {
                    cursor[0] = (cursor[0] + 1) % board_height;
                },
                KeyCode::Left => {
                    cursor[1] = (cursor[1] + board_width - 1) % board_width;
                },
                KeyCode::Right => {
                    cursor[1] = (cursor[1] + 1) % board_width;
                },
                KeyCode::Char(' ') => {
                    if board[cursor[0]][cursor[1]] != UNOPENED {
                        continue;
                    }
                    if !game_started {
                        generate_mines(&mut board, &mut bombs, &cursor, ticks);
                        game_started = true;
                    }

                    alive = explore(&mut board, &mut bombs, &mut cursor);
                },
                KeyCode::Char('f') => {
                    if board[cursor[0]][cursor[1]] == UNOPENED {
                        board[cursor[0]][cursor[1]] = FLAG;
                    } else if board[cursor[0]][cursor[1]] == FLAG {
                        board[cursor[0]][cursor[1]] = UNOPENED;
                    }
                },
                KeyCode::Esc => {
                    break Ok(());  // Exit the program
                },
                _ => {}
            }
            update_board(&mut board, &mut cursor);
        }
    }
}

fn update_board(board: &mut Vec<Vec<char>>, cursor: &mut [usize; 2]) {
    let mut output = String::new();
    output.push_str(&format!("{}[2J", 27 as char));  // Clear the terminal

    for i in 0..board.len() {
        for j in 0..board[i].len() {
            if i == cursor[0] && j == cursor[1] {
                output.push_str(&format!("[{}]", styled(board[i][j])));
            } else {
                output.push_str(&format!(" {} ", styled(board[i][j])));
            }
        }
        output.push('\n');
    }

    print!("{}", output);
}

/// Returns a styled character based on the input character
/// Completely unnecessary for the basic game, but fun to have
fn styled(c: char) -> StyledContent<char> {
    return match c {
        x if x == UNOPENED => UNOPENED.white(),
        x if x == MINE => MINE.dark_red(),
        x if x == EMPTY => EMPTY.white(),
        x if x == FLAG => FLAG.dark_red(),
        x if x == NUMS[0] => NUMS[0].blue(),
        x if x == NUMS[1] => NUMS[1].green(),
        x if x == NUMS[2] => NUMS[2].red(),
        x if x == NUMS[3] => NUMS[3].dark_blue(),
        x if x == NUMS[4] => NUMS[4].magenta(),
        x if x == NUMS[5] => NUMS[5].yellow(),
        x if x == NUMS[6] => NUMS[6].dark_cyan(),
        x if x == NUMS[7] => NUMS[7].red(),
        _ => ' '.white(),
    }
}

fn generate_mines(board: &mut Vec<Vec<char>>, bombs: &mut Vec<[usize; 2]>, cursor: &[usize; 2], mut semi_rand: u32) {
    let bomb_count: usize = 50;

    while bombs.len() < bomb_count {
        regen_random(&mut semi_rand);
        let x: usize = semi_rand as usize % board[0].len();
        regen_random(&mut semi_rand);
        let y: usize = semi_rand as usize % board.len();

        if (cursor_too_close(cursor, &[y, x])) || bombs.contains(&[y, x]) {
            continue;
        }
        bombs.push([y, x]);
    }
}
fn cursor_too_close(cursor: &[usize; 2], preliminar_pos: &[usize; 2]) -> bool {
    let range: i32 = 2;
    let x_diff: i32 = (cursor[1] as i32 - preliminar_pos[1] as i32).abs();
    let y_diff: i32 = (cursor[0] as i32 - preliminar_pos[0] as i32).abs();
    return x_diff < range && y_diff < range;
}
fn regen_random(rand: &mut u32) {
    let mut product: u64 = *rand as u64;
    product = (0xFFFF_FFFF & product) * (0xFFFF_FFFF & product);  // Avoid overflow
    // Must use base 10, as base 2 would approach 0 very fast, while this is somewhat stable
    product /= 100_000;
    product %= 10_000_000_000;
    *rand = product as u32;
}

fn explore(board: &mut Vec<Vec<char>>, bombs: &mut Vec<[usize; 2]>, cursor: &mut [usize; 2]) -> bool {
    if bombs.contains(cursor) {
        for bomb in bombs {
            let bomb_x: usize = bomb[1];
            let bomb_y: usize = bomb[0];
            board[bomb_y][bomb_x] = MINE;
        }
        return false;
    }

    let height = board.len();
    let width = board[0].len();

    let mut processed: Vec<[usize; 2]> = Vec::new();
    let mut queue: VecDeque<[usize; 2]> = VecDeque::new();
    queue.push_back(*cursor);
    

    let x_pos = cursor[1];
    let y_pos = cursor[0];

    let lower_y_bound = if y_pos == 0 { 0 } else { y_pos - 1 };
    let upper_y_bound = if y_pos == height - 1 { height - 1 } else { y_pos + 1 };
    let lower_x_bound = if x_pos == 0 { 0 } else { x_pos - 1 };
    let upper_x_bound = if x_pos == width - 1 { width - 1 } else { x_pos + 1 };
    for y in lower_y_bound..=upper_y_bound {
        for x in lower_x_bound..=upper_x_bound {
            if get_mines_around(&[y, x], bombs, height, width) == 0 {
                queue.push_back([y, x]);
            }
        }
    }

    while !queue.is_empty() {
        let pos: [usize; 2] = queue.pop_front().expect("Uh oh this is wrong");
        
        if processed.contains(&pos) {
            continue;
        }

        let x_pos = pos[1];
        let y_pos = pos[0];

        let mines_nearby: usize = get_mines_around(&pos, bombs, height, width);

        let lower_y_bound = if y_pos == 0 { 0 } else { y_pos - 1 };
        let upper_y_bound = if y_pos == height - 1 { height - 1 } else { y_pos + 1 };
        let lower_x_bound = if x_pos == 0 { 0 } else { x_pos - 1 };
        let upper_x_bound = if x_pos == width - 1 { width - 1 } else { x_pos + 1 };
        if mines_nearby > 0 {
            board[y_pos][x_pos] = NUMS[mines_nearby - 1];
        } else {
            board[y_pos][x_pos] = EMPTY;
            for y in lower_y_bound..=upper_y_bound {
                for x in lower_x_bound..=upper_x_bound {
                    queue.push_back([y, x]);
                }
            }
        }
        processed.push(pos);
    }

    return true;
}
fn get_mines_around(pos: &[usize; 2], bombs: &Vec<[usize; 2]>, height: usize, width: usize) -> usize {
    let x_pos = pos[1];
    let y_pos = pos[0];
    let mut mines_nearby: usize = 0;

    let lower_y_bound = if y_pos == 0 { 0 } else { y_pos - 1 };
    let upper_y_bound = if y_pos == height - 1 { height - 1 } else { y_pos + 1 };
    let lower_x_bound = if x_pos == 0 { 0 } else { x_pos - 1 };
    let upper_x_bound = if x_pos == width - 1 { width - 1 } else { x_pos + 1 };

    for y in lower_y_bound..=upper_y_bound {
        for x in lower_x_bound..=upper_x_bound {
            if bombs.contains(&[y, x]) {
                mines_nearby += 1;
            }
        }
    }

    return mines_nearby;
}

fn reset_game(board: &mut Vec<Vec<char>>, bombs: &mut Vec<[usize; 2]>, game_started: &mut bool, alive: &mut bool) {
    *board = vec![vec![UNOPENED; board[0].len()]; board.len()];
    bombs.clear();
    *game_started = false;
    *alive = true;
}
