#include <iostream>
#include <vector>
#include <deque>
#include <crossterm/event.hpp>

static const char UNOPENED = '0';
static const char MINE = 'X';
static const char EMPTY = ' ';
static const char FLAG = 'P';
static const std::array<char, 8> NUMS = {'1', '2', '3', '4', '5', '6', '7', '8'};

void update_board(std::vector<std::vector<char>>& board, std::array<size_t, 2>& cursor) {
    std::string output;
    output += "\033[2J";  // Clear the terminal
    for (size_t i = 0; i < board.size(); ++i) {
        for (size_t j = 0; j < board[i].size(); ++j) {
            if (i == cursor[0] && j == cursor[1]) {
                output += "[" + std::string(1, board[i][j]) + "]";
            } else {
                output += " " + std::string(1, board[i][j]) + " ";
            }
        }
        output += "\n";
    }
    std::cout << output;
}

void generate_mines(std::vector<std::vector<char>>& board, std::vector<std::array<size_t, 2>>& bombs, const std::array<size_t, 2>& cursor, unsigned int& semi_rand) {
    size_t bomb_count = 30;
    while (bombs.size() < bomb_count) {
        regen_random(semi_rand);
        size_t x = semi_rand % board[0].size();
        regen_random(semi_rand);
        size_t y = semi_rand % board.size();
        if (cursor_too_close(cursor, {y, x}) || std::find(bombs.begin(), bombs.end(), std::array<size_t, 2>{y, x}) != bombs.end()) {
            continue;
        }
        bombs.push_back({y, x});
    }
}

int main() {
    // TODO: Add command line arguments for custom board size and bomb count
    size_t board_height = 16;
    size_t board_width = 16;
    std::vector<std::vector<char>> board(board_height, std::vector<char>(board_width, UNOPENED));
    std::vector<std::array<size_t, 2>> bombs;
    std::array<size_t, 2> cursor = {0, 0};
    update_board(board, cursor);
    bool game_started = false;
    unsigned int ticks = 0x12345678;
    bool alive = true;
    while (true) {
        ++ticks;
        auto key_event = read();
        if (key_event.is_ok()) {
            if (!alive) {
                reset_game(board, bombs, game_started, alive);
                update_board(board, cursor);
                continue;
            }
            switch (key_event.unwrap().code) {
                case KeyCode::Up:
                    cursor[0] = (cursor[0] + 15) % board_height;
                    break;
                case KeyCode::Down:
                    cursor[0] = (cursor[0] + 1) % board_height;
                    break;
                case KeyCode::Left:
                    cursor[1] = (cursor[1] + 15) % board_width;
                    break;
                case KeyCode::Right:
                    cursor[1] = (cursor[1] + 1) % board_width;
                    break;
                case KeyCode::Char(' '):
                    if (board[cursor[0]][cursor[1]] != UNOPENED) {
                        continue;
                    }
                    if (!game_started) {
                        generate_mines(board, bombs, cursor, ticks);
                        game_started = true;
                    }
                    alive = explore(board, bombs, cursor);
                    break;
                case KeyCode::Char('f'):
                    if (board[cursor[0]][cursor[1]] == UNOPENED) {
                        board[cursor[0]][cursor[1]] = FLAG;
                    } else if (board[cursor[0]][cursor[1]] == FLAG) {
                        board[cursor[0]][cursor[1]] = UNOPENED;
                    }
                    break;
                case KeyCode::Esc:
                    return 0;  // Exit the program
                default:
                    break;
            }
            update_board(board, cursor);
        }
    }
    return 0;
}

bool cursor_too_close(int cursor[2], int preliminar_pos[2]) {
    int range = 2;
    int x_diff = abs(cursor[1] - preliminar_pos[1]);
    int y_diff = abs(cursor[0] - preliminar_pos[0]);
    return x_diff < range && y_diff < range;
}

void regen_random(uint32_t *rand) {
    uint64_t product = *rand;
    product = (0xFFFFFFFF & product) * (0xFFFFFFFF & product);  // Avoid overflow
    product /= 100000;
    product %= 10000000000;
    *rand = (uint32_t)product;
}

int get_mines_around(int pos[2], int bombs[][2], int height, int width) {
    int count = 0;
    int y_pos = pos[0];
    int x_pos = pos[1];
    for (int i = -1; i <= 1; i++) {
        for (int j = -1; j <= 1; j++) {
            if (i == 0 && j == 0) {
                continue;
            }
            int new_y = y_pos + i;
            int new_x = x_pos + j;
            if (new_y >= 0 && new_y < height && new_x >= 0 && new_x < width) {
                for (int k = 0; k < sizeof(bombs) / sizeof(bombs[0]); k++) {
                    if (bombs[k][0] == new_y && bombs[k][1] == new_x) {
                        count++;
                    }
                }
            }
        }
    }
    return count;
}

bool explore(char board[][8], int bombs[][2], int num_bombs, int cursor[2]) {
    for (int i = 0; i < num_bombs; i++) {
        if (bombs[i][0] == cursor[0] && bombs[i][1] == cursor[1]) {
            for (int j = 0; j < num_bombs; j++) {
                int bomb_x = bombs[j][1];
                int bomb_y = bombs[j][0];
                board[bomb_y][bomb_x] = MINE;
            }
            return false;
        }
    }
    int height = 8;
    int width = 8;
    int processed[64][2];
    int processed_count = 0;
    int queue[64][2];
    int queue_front = 0;
    int queue_rear = 0;
    queue[queue_rear][0] = cursor[0];
    queue[queue_rear][1] = cursor[1];
    queue_rear++;
    while (queue_front < queue_rear) {
        int pos[2];
        pos[0] = queue[queue_front][0];
        pos[1] = queue[queue_front][1];
        queue_front++;
        if (processed_count >= 64) {
            break;
        }
        if (processed_count > 0) {
            int already_processed = 0;
            for (int i = 0; i < processed_count; i++) {
                if (processed[i][0] == pos[0] && processed[i][1] == pos[1]) {
                    already_processed = 1;
                    break;
                }
            }
            if (already_processed) {
                continue;
            }
        }
        int x_pos = pos[1];
        int y_pos = pos[0];
        int mines_nearby = get_mines_around(pos, bombs, height, width);
        if (mines_nearby > 0) {
            board[y_pos][x_pos] = NUMS[mines_nearby - 1];
        } else {
            board[y_pos][x_pos] = EMPTY;
            for (int i = -1; i <= 1; i++) {
                for (int j = -1; j <= 1; j++) {
                    int new_y = y_pos + i;
                    int new_x = x_pos + j;
                    if (new_y >= 0 && new_y < height && new_x >= 0 && new_x < width) {
                        queue[queue_rear][0] = new_y;
                        queue[queue_rear][1] = new_x;
                        queue_rear++;
                    }
                }
            }
        }
        processed[processed_count][0] = pos[0];
        processed[processed_count][1] = pos[1];
        processed_count++;
    }
    return true;
}
size_t get_mines_around(const std::array<size_t, 2>& pos, const std::vector<std::array<size_t, 2>>& bombs, size_t height, size_t width) {
    size_t x_pos = pos[1];
    size_t y_pos = pos[0];
    size_t mines_nearby = 0;
    size_t lower_y_bound = (y_pos == 0) ? 0 : y_pos - 1;
    size_t upper_y_bound = (y_pos == height - 1) ? height - 1 : y_pos + 1;
    size_t lower_x_bound = (x_pos == 0) ? 0 : x_pos - 1;
    size_t upper_x_bound = (x_pos == width - 1) ? width - 1 : x_pos + 1;
    for (size_t y = lower_y_bound; y <= upper_y_bound; ++y) {
        for (size_t x = lower_x_bound; x <= upper_x_bound; ++x) {
            if (std::find(bombs.begin(), bombs.end(), std::array<size_t, 2>{y, x}) != bombs.end()) {
                mines_nearby += 1;
            }
        }
    }
    return mines_nearby;
}

void reset_game(std::vector<std::vector<char>>& board, std::vector<std::array<size_t, 2>>& bombs, bool& game_started, bool& alive) {
    board = std::vector<std::vector<char>>(board.size(), std::vector<char>(board[0].size(), UNOPENED));
    bombs.clear();
    game_started = false;
    alive = true;
}
