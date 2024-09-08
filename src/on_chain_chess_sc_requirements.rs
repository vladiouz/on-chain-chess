use crate::on_chain_chess_sc_storage::{self, GameId};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait OnChainChessScRequirements: on_chain_chess_sc_storage::OnChainChessScStorage {
    fn require_is_active(&self) {
        require!(!self.is_paused().get(), "Contract is paused")
    }

    fn require_game_exists(&self, game_id: GameId) {
        require!(
            game_id >= 1 && game_id <= self.games().len() && !self.games().item_is_empty(game_id),
            "Game does not exist"
        )
    }

    fn require_game_is_ongoing(&self, game_id: GameId) {
        let game = self.games().get(game_id);
        require!(game.state == 0, "Game is not active")
    }

    fn require_is_valid_move(&self, game_id: GameId, color_to_move: u8, from: u8, to: u8) {
        require!(to < 64, "Move is not on the board");
        require!(to != from, "You did not move anything");
        let board = self.board(game_id).get();

        // Check that player is moving their piece and not capturing one of their own
        let piece = board[from as usize];
        require!(
            (color_to_move == 0 && piece < 6 && board[to as usize] >= 6)
                || (color_to_move == 1 && piece > 6 && board[to as usize] <= 6),
            "Invalid move"
        );

        let from_x = from % 8;
        let from_y = from / 8;
        let to_x = to % 8;
        let to_y = to / 8;

        match piece % 7 {
            0 => self.require_valid_king_move(from_x, from_y, to_x, to_y),
            1 => self.require_valid_queen_move(board, from_x, from_y, to_x, to_y),
            2 => self.require_valid_rook_move(board, from_x, from_y, to_x, to_y),
            3 => self.require_valid_bishop_move(board, from_x, from_y, to_x, to_y),
            4 => self.require_valid_knight_move(from_x, from_y, to_x, to_y),
            5 => {
                self.require_valid_pawn_move(board, color_to_move, from_x, from_y, to_x, to_y);
                self.board(game_id).set(board);
            }
            _ => require!(false, "Invalid move"),
        }
    }

    fn require_king_in_check(&self, player_that_moved: u8, game_id: GameId) {
        let board = self.board(game_id).get();
        let mut king_position = 64;
        let king_piece = if player_that_moved == 0 { 0 } else { 7 };
        let stabilizer = 7 * (1 - player_that_moved);

        for i in 0..64 {
            if board[i] == king_piece {
                king_position = i;
                break;
            }
        }

        if king_position != 64 {
            let king_x = king_position % 8;
            let king_y = king_position / 8;

            let mut is_king_in_check = false;

            // check for rooks and queens
            for i in 0..8 {
                if king_x + i >= 8 {
                    break;
                }
                if (board[king_x + i + king_y * 8] == 2 + stabilizer)
                    || (board[king_x + i + king_y * 8] == 1 + stabilizer)
                {
                    is_king_in_check = true;
                    break;
                }

                if board[king_x + i + king_y * 8] != 6 {
                    break;
                }
            }

            if !is_king_in_check {
                for i in 0..8 {
                    if king_x < i {
                        break;
                    }

                    if (board[king_x - i + king_y * 8] == 2 + stabilizer)
                        || (board[king_x - i + king_y * 8] == 1 + stabilizer)
                    {
                        is_king_in_check = true;
                        break;
                    }

                    if board[king_x - i + king_y * 8] != 6 {
                        break;
                    }
                }
            }

            if !is_king_in_check {
                for i in 0..8 {
                    if king_y + i >= 8 {
                        break;
                    }

                    if (board[king_x + (king_y + i) * 8] == 2 + stabilizer)
                        || (board[king_x + (king_y + i) * 8] == 1 + stabilizer)
                    {
                        is_king_in_check = true;
                        break;
                    }

                    if board[king_x + (king_y + i) * 8] != 6 {
                        break;
                    }
                }
            }

            if !is_king_in_check {
                for i in 0..8 {
                    if king_y < i {
                        break;
                    }

                    if (board[king_x + (king_y - i) * 8] == 2 + stabilizer)
                        || (board[king_x + (king_y - i) * 8] == 1 + stabilizer)
                    {
                        is_king_in_check = true;
                        break;
                    }

                    if board[king_x + (king_y - i) * 8] != 6 {
                        break;
                    }
                }
            }

            // check for bishops and queens
            if !is_king_in_check {
                for i in 0..8 {
                    if king_x + i >= 8 || king_y + i >= 8 {
                        break;
                    }

                    if (board[king_x + i + (king_y + i) * 8] == 3 + stabilizer)
                        || (board[king_x + i + (king_y + i) * 8] == 1 + stabilizer)
                    {
                        is_king_in_check = true;
                        break;
                    }

                    if board[king_x + i + (king_y + i) * 8] != 6 {
                        break;
                    }
                }
            }

            if !is_king_in_check {
                for i in 0..8 {
                    if king_x < i || king_y < i {
                        break;
                    }

                    if (board[king_x - i + (king_y - i) * 8] == 3 + stabilizer)
                        || (board[king_x - i + (king_y - i) * 8] == 1 + stabilizer)
                    {
                        is_king_in_check = true;
                        break;
                    }

                    if board[king_x - i + (king_y - i) * 8] != 6 {
                        break;
                    }
                }
            }

            if !is_king_in_check {
                for i in 0..8 {
                    if king_x + i >= 8 || king_y < i {
                        break;
                    }

                    if (board[king_x + i + (king_y - i) * 8] == 3 + stabilizer)
                        || (board[king_x + i + (king_y - i) * 8] == 1 + stabilizer)
                    {
                        is_king_in_check = true;
                        break;
                    }

                    if board[king_x + i + (king_y - i) * 8] != 6 {
                        break;
                    }
                }
            }

            if !is_king_in_check {
                for i in 0..8 {
                    if king_x < i || king_y + i >= 8 {
                        break;
                    }

                    if (board[king_x - i + (king_y + i) * 8] == 3 + stabilizer)
                        || (board[king_x - i + (king_y + i) * 8] == 1 + stabilizer)
                    {
                        is_king_in_check = true;
                        break;
                    }

                    if board[king_x - i + (king_y + i) * 8] != 6 {
                        break;
                    }
                }
            }

            // check for knights
            if !is_king_in_check {
                if king_x + 1 < 8 && king_y + 2 < 8 {
                    if board[king_x + 1 + (king_y + 2) * 8] == 4 + stabilizer {
                        is_king_in_check = true;
                    }
                }

                if king_x + 2 < 8 && king_y + 1 < 8 {
                    if board[king_x + 2 + (king_y + 1) * 8] == 4 + stabilizer {
                        is_king_in_check = true;
                    }
                }

                if king_x + 2 < 8 && king_y > 0 {
                    if board[king_x + 2 + (king_y - 1) * 8] == 4 + stabilizer {
                        is_king_in_check = true;
                    }
                }

                if king_x + 1 < 8 && king_y > 1 {
                    if board[king_x + 1 + (king_y - 2) * 8] == 4 + stabilizer {
                        is_king_in_check = true;
                    }
                }

                if king_x > 0 && king_y > 1 {
                    if board[king_x - 1 + (king_y - 2) * 8] == 4 + stabilizer {
                        is_king_in_check = true;
                    }
                }

                if king_x > 1 && king_y > 0 {
                    if board[king_x - 2 + (king_y - 1) * 8] == 4 + stabilizer {
                        is_king_in_check = true;
                    }
                }

                if king_x > 1 && king_y + 1 < 8 {
                    if board[king_x - 2 + (king_y + 1) * 8] == 4 + stabilizer {
                        is_king_in_check = true;
                    }
                }

                if king_x > 0 && king_y + 2 < 8 {
                    if board[king_x - 1 + (king_y + 2) * 8] == 4 + stabilizer {
                        is_king_in_check = true;
                    }
                }
            }

            // check for pawns
            if !is_king_in_check {
                if player_that_moved == 0 {
                    if king_x > 0 && king_y > 0 {
                        if board[king_x - 1 + (king_y - 1) * 8] == 5 + stabilizer {
                            is_king_in_check = true;
                        }
                    }

                    if king_x < 7 && king_y > 0 {
                        if board[king_x + 1 + (king_y - 1) * 8] == 5 + stabilizer {
                            is_king_in_check = true;
                        }
                    }
                } else {
                    if king_x > 0 && king_y < 7 {
                        if board[king_x - 1 + (king_y + 1) * 8] == 5 + stabilizer {
                            is_king_in_check = true;
                        }
                    }

                    if king_x < 7 && king_y < 7 {
                        if board[king_x + 1 + (king_y + 1) * 8] == 5 + stabilizer {
                            is_king_in_check = true;
                        }
                    }
                }
            }

            // check for enemqy king
            if !is_king_in_check {
                if king_x > 0 && king_y > 0 {
                    if board[king_x - 1 + (king_y - 1) * 8] == stabilizer {
                        is_king_in_check = true;
                    }
                }

                if king_x < 7 && king_y > 0 {
                    if board[king_x + 1 + (king_y - 1) * 8] == stabilizer {
                        is_king_in_check = true;
                    }
                }

                if king_x > 0 && king_y < 7 {
                    if board[king_x - 1 + (king_y + 1) * 8] == stabilizer {
                        is_king_in_check = true;
                    }
                }

                if king_x < 7 && king_y < 7 {
                    if board[king_x + 1 + (king_y + 1) * 8] == stabilizer {
                        is_king_in_check = true;
                    }
                }

                if king_x + 1 < 8 && king_y + 2 < 8 {
                    if board[king_x + 1 + (king_y + 2) * 8] == stabilizer {
                        is_king_in_check = true;
                    }
                }

                if king_x + 2 < 8 && king_y + 1 < 8 {
                    if board[king_x + 2 + (king_y + 1) * 8] == stabilizer {
                        is_king_in_check = true;
                    }
                }

                if king_x + 2 < 8 && king_y > 0 {
                    if board[king_x + 2 + (king_y - 1) * 8] == stabilizer {
                        is_king_in_check = true;
                    }
                }

                if king_x + 1 < 8 && king_y > 1 {
                    if board[king_x + 1 + (king_y - 2) * 8] == stabilizer {
                        is_king_in_check = true;
                    }
                }

                if king_x > 0 && king_y > 1 {
                    if board[king_x - 1 + (king_y - 2) * 8] == stabilizer {
                        is_king_in_check = true;
                    }
                }
            }

            require!(is_king_in_check, "King is not in check");
        }
    }

    fn require_valid_king_move(&self, from_x: u8, from_y: u8, to_x: u8, to_y: u8) {
        require!(
            (from_x as i8 - to_x as i8).abs() <= 1 && (from_y as i8 - to_y as i8).abs() <= 1,
            "Invalid king move"
        );
    }

    fn require_valid_queen_move(
        &self,
        board: [u8; 64],
        from_x: u8,
        from_y: u8,
        to_x: u8,
        to_y: u8,
    ) {
        if from_x == to_x || from_y == to_y {
            self.require_valid_rook_move(board, from_x, from_y, to_x, to_y);
        } else if (from_x as i8 - to_x as i8).abs() == (from_y as i8 - to_y as i8).abs() {
            self.require_valid_bishop_move(board, from_x, from_y, to_x, to_y);
        }
    }

    fn require_valid_rook_move(&self, board: [u8; 64], from_x: u8, from_y: u8, to_x: u8, to_y: u8) {
        require!((from_x == to_x || from_y == to_y), "Invalid rook move");
        if from_x == to_x {
            let min_y = from_y.min(to_y);
            let max_y = from_y.max(to_y);
            for y in min_y + 1..max_y {
                require!(
                    board[to_x as usize + y as usize * 8] == 6,
                    "Invalid rook move"
                );
            }
        } else {
            let min_x = from_x.min(to_x);
            let max_x = from_x.max(to_x);
            for x in min_x + 1..max_x {
                require!(
                    board[x as usize + to_y as usize * 8] == 6,
                    "Invalid rook move"
                );
            }
        }
    }

    fn require_valid_bishop_move(
        &self,
        board: [u8; 64],
        from_x: u8,
        from_y: u8,
        to_x: u8,
        to_y: u8,
    ) {
        require!(
            (from_x as i8 - to_x as i8).abs() == (from_y as i8 - to_y as i8).abs(),
            "Invalid bishop move"
        );

        let min_x = from_x.min(to_x);
        let max_x = from_x.max(to_x);
        let min_y = from_y.min(to_y);

        let mut x = min_x;
        let mut y = min_y;

        while x < max_x - 1 {
            x += 1;
            y += 1;
            require!(
                board[x as usize + y as usize * 8] == 6,
                "Invalid bishop move"
            );
        }
    }

    fn require_valid_knight_move(&self, from_x: u8, from_y: u8, to_x: u8, to_y: u8) {
        require!(
            ((from_x as i8 - to_x as i8).abs() + (from_y as i8 - to_y as i8).abs() == 3)
                && from_y != to_y
                && from_x != to_x,
            "Invalid knight move"
        );
    }

    fn require_valid_pawn_move(
        &self,
        mut board: [u8; 64],
        color_to_move: u8,
        from_x: u8,
        from_y: u8,
        to_x: u8,
        to_y: u8,
    ) {
        if color_to_move == 0 {
            require!(
                ((from_y == to_y + 1
                    && from_x == to_x
                    && board[to_x as usize + to_y as usize * 8] == 6)
                    || (from_y == to_y + 1
                        && (from_x == to_x + 1 || from_x == to_x - 1)
                        && board[to_x as usize + to_y as usize * 8] > 6)
                    || (from_y == 6
                        && to_y == 4
                        && from_x == to_x
                        && board[to_x as usize + to_y as usize * 8] == 6
                        && board[to_x as usize + 5 * 8] == 6)),
                "Invalid pawn move"
            );

            if to_y == 0 {
                board[from_x as usize + from_y as usize * 8] = 1;
            }
        } else {
            require!(
                ((from_y == to_y - 1
                    && from_x == to_x
                    && board[to_x as usize + to_y as usize * 8] == 6)
                    || (from_y == to_y - 1
                        && (from_x == to_x + 1 || from_x == to_x - 1)
                        && board[to_x as usize + to_y as usize * 8] < 6)
                    || (from_y == 1
                        && to_y == 3
                        && from_x == to_x
                        && board[to_x as usize + to_y as usize * 8] == 6
                        && board[to_x as usize + 2 * 8] == 6)),
                "Invalid pawn move"
            );

            if to_y == 7 {
                board[from_x as usize + from_y as usize * 8] = 1;
            }
        }
    }
}
