use crate::{
    on_chain_chess_sc_requirements,
    on_chain_chess_sc_storage::{self, Game, GameId},
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

const START_BOARD: [u8; 64] = [
    9, 11, 10, 8, 7, 10, 11, 9, 12, 12, 12, 12, 12, 12, 12, 12, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 5, 5, 5, 5, 5, 5, 5, 5, 2, 4, 3, 1,
    0, 3, 4, 2,
];

#[multiversx_sc::module]
pub trait OnChainChessScInteractions:
    on_chain_chess_sc_requirements::OnChainChessScRequirements
    + on_chain_chess_sc_storage::OnChainChessScStorage
{
    #[payable("*")]
    #[endpoint(joinGame)]
    fn join_game(&self) {
        self.require_is_active();

        let payment = self.call_value().single_esdt();

        require!(
            payment.amount == self.wager_amount().get(),
            "Wrong wager amount"
        );
        require!(
            payment.token_identifier == self.wager_token_id().get(),
            "Wrong wager token identifier"
        );

        let caller = self.blockchain().get_caller();

        match self.is_player_waiting_opt().get() {
            Some(player) => {
                require!(player != caller, "You cannot play against yourself");

                let game_id = self.games().len() + 1;
                let game = Game::new(game_id, player, caller, self.blockchain().get_block_epoch());
                self.games().push(&game);
                self.is_player_waiting_opt().set(None);
                self.draw_offer(game_id).set(None);
                self.board(game_id).set(START_BOARD);
            }
            None => {
                self.is_player_waiting_opt().set(Some(caller));
            }
        }
    }

    #[endpoint(makeMove)]
    fn make_move(&self, game_id: GameId, from: u8, to: u8) {
        self.require_is_active();
        self.require_game_exists(game_id);
        self.require_game_is_ongoing(game_id);

        let caller = self.blockchain().get_caller();
        let mut game = self.games().get(game_id);

        let color_to_move = game.player_turn;

        let player_to_move = if color_to_move == 0 {
            game.white_player.clone()
        } else {
            game.black_player.clone()
        };

        require!(
            player_to_move == caller,
            "It is not your turn or you are not part of the game"
        );

        let current_epoch = self.blockchain().get_block_epoch();

        require!(
            current_epoch <= game.last_move_epoch + 1,
            "You took too long to make a move"
        );

        game.last_move_epoch = current_epoch;

        self.require_is_valid_move(game_id, color_to_move, from, to);

        let mut board = self.board(game_id).get();
        board[to as usize] = board[from as usize];
        board[from as usize] = 6;
        self.board(game_id).set(board);

        game.player_turn = 1 - color_to_move;
        self.games().set(game_id, &game);
    }

    #[endpoint(draw)]
    fn draw(&self, game_id: GameId) {
        self.require_is_active();
        self.require_game_exists(game_id);
        self.require_game_is_ongoing(game_id);

        let caller = self.blockchain().get_caller();
        let mut game = self.games().get(game_id);

        require!(
            caller == game.white_player || caller == game.black_player,
            "You are not part of the game"
        );

        let draw_offer = self.draw_offer(game_id).get();
        if draw_offer.is_none() {
            self.draw_offer(game_id).set(Some(caller));
        } else if draw_offer.unwrap() != caller {
            self.draw_offer(game_id).set(None);

            game.state = 3;
            self.games().set(game_id, &game);

            self.send().direct_esdt(
                &game.white_player,
                &self.wager_token_id().get(),
                0u64,
                &self.wager_amount().get(),
            );

            self.send().direct_esdt(
                &game.black_player,
                &self.wager_token_id().get(),
                0u64,
                &self.wager_amount().get(),
            );

            self.score(game.white_player.clone())
                .set(self.score(game.white_player).get() + 1);
            self.score(game.black_player.clone())
                .set(self.score(game.black_player).get() + 1);
        }
    }

    // checks it opponent finished their move while in check
    #[endpoint(callIllegalMove)]
    fn call_illegal_move(&self, game_id: GameId) {
        self.require_is_active();
        self.require_game_exists(game_id);
        self.require_game_is_ongoing(game_id);

        let caller = self.blockchain().get_caller();
        let mut game = self.games().get(game_id);

        require!(
            caller == game.white_player || caller == game.black_player,
            "You are not part of the game"
        );

        let player_to_move = if game.player_turn == 0 {
            game.white_player.clone()
        } else {
            game.black_player.clone()
        };

        require!(player_to_move == caller, "You just moved");
        self.require_king_in_check(1 - game.player_turn, game_id);

        game.state = 1 + game.player_turn;
        self.games().set(game_id, &game);

        self.send().direct_esdt(
            &player_to_move,
            &self.wager_token_id().get(),
            0u64,
            &(self.wager_amount().get() * BigUint::from(2u64)),
        );

        self.score(player_to_move.clone())
            .set(self.score(player_to_move).get() + 2);
    }

    #[endpoint(signalInactivity)]
    fn signal_inactivity(&self, game_id: GameId) {
        self.require_is_active();
        self.require_game_exists(game_id);
        self.require_game_is_ongoing(game_id);

        let caller = self.blockchain().get_caller();
        let mut game = self.games().get(game_id);

        require!(
            caller == game.white_player || caller == game.black_player,
            "You are not part of the game"
        );

        let player_to_move = if game.player_turn == 0 {
            game.white_player.clone()
        } else {
            game.black_player.clone()
        };

        require!(player_to_move != caller, "You are up to move");

        let current_epoch = self.blockchain().get_block_epoch();

        require!(
            current_epoch > game.last_move_epoch + 1,
            "Opponent can still make a move"
        );

        game.state = 2 - game.player_turn;
        self.games().set(game_id, &game);

        self.send().direct_esdt(
            &caller,
            &self.wager_token_id().get(),
            0u64,
            &(self.wager_amount().get() * BigUint::from(2u64)),
        );

        self.score(caller.clone()).set(self.score(caller).get() + 2);
    }

    #[endpoint(resign)]
    fn resign(&self, game_id: GameId) {
        self.require_is_active();
        self.require_game_exists(game_id);
        self.require_game_is_ongoing(game_id);

        let caller = self.blockchain().get_caller();
        let mut game = self.games().get(game_id);

        require!(
            caller == game.white_player || caller == game.black_player,
            "You are not part of the game"
        );

        if caller == game.white_player {
            game.state = 2;

            self.send().direct_esdt(
                &game.black_player,
                &self.wager_token_id().get(),
                0u64,
                &(self.wager_amount().get() * BigUint::from(2u64)),
            );

            self.score(game.black_player.clone())
                .set(self.score(game.black_player.clone()).get() + 2);
        } else {
            game.state = 1;

            self.send().direct_esdt(
                &game.white_player,
                &self.wager_token_id().get(),
                0u64,
                &(self.wager_amount().get() * BigUint::from(2u64)),
            );

            self.score(game.white_player.clone())
                .set(self.score(game.white_player.clone()).get() + 2);
        }
        self.games().set(game_id, &game);
    }
}
