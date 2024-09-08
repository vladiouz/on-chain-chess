multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type GameId = usize;

#[derive(TypeAbi, NestedEncode, NestedDecode, PartialEq, Debug, TopEncode, TopDecode)]
pub struct Game<M: ManagedTypeApi> {
    pub game_id: GameId,
    pub white_player: ManagedAddress<M>,
    pub black_player: ManagedAddress<M>,
    pub player_turn: u8,
    pub last_move_epoch: u64,
    pub state: u8,
}

impl<M: ManagedTypeApi> Game<M> {
    pub fn new(
        game_id: GameId,
        white_player: ManagedAddress<M>,
        black_player: ManagedAddress<M>,
        last_move_epoch: u64,
    ) -> Self {
        Game {
            game_id,
            white_player,
            black_player,
            player_turn: 0u8,
            last_move_epoch,
            state: 0u8,
        }
    }
}

#[multiversx_sc::module]
pub trait OnChainChessScStorage {
    #[view(isPaused)]
    #[storage_mapper("isPaused")]
    fn is_paused(&self) -> SingleValueMapper<bool>;

    #[view(getWagerTokenId)]
    #[storage_mapper("wagerTokenId")]
    fn wager_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getWagerAmount)]
    #[storage_mapper("wagerAmount")]
    fn wager_amount(&self) -> SingleValueMapper<BigUint>;

    #[view(getScore)]
    #[storage_mapper("score")]
    fn score(&self, player: ManagedAddress) -> SingleValueMapper<u64>;

    #[view(getIsPlayerWaitingOpt)]
    #[storage_mapper("isPlayerWaitingOpt")]
    fn is_player_waiting_opt(&self) -> SingleValueMapper<Option<ManagedAddress>>;

    #[view(getGames)]
    #[storage_mapper("games")]
    fn games(&self) -> VecMapper<Game<Self::Api>>;

    #[view(getBoard)]
    #[storage_mapper("board")]
    fn board(&self, game_id: GameId) -> SingleValueMapper<[u8; 64]>;

    #[view(getDrawOffer)]
    #[storage_mapper("drawOffer")]
    fn draw_offer(&self, game_id: GameId) -> SingleValueMapper<Option<ManagedAddress>>;
}
