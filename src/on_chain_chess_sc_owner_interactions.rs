use crate::on_chain_chess_sc_storage;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait OnChainChessScAdminInteractions:
    on_chain_chess_sc_storage::OnChainChessScStorage
{
    #[only_owner]
    #[endpoint(pause)]
    fn pause(&self) {
        self.is_paused().set(true);
    }

    #[only_owner]
    #[endpoint(unpause)]
    fn unpause(&self) {
        self.is_paused().set(false);
    }

    #[only_owner]
    #[endpoint(setWager)]
    fn set_wager(&self, wager_token_id: TokenIdentifier, wager_amount: BigUint) {
        self.wager_token_id().set_if_empty(wager_token_id);
        self.wager_amount().set_if_empty(wager_amount);
    }
}
