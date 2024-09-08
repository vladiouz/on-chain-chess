#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod on_chain_chess_sc_interactions;
pub mod on_chain_chess_sc_owner_interactions;
pub mod on_chain_chess_sc_requirements;
pub mod on_chain_chess_sc_storage;

#[multiversx_sc::contract]
pub trait OnChainChessSc:
    on_chain_chess_sc_storage::OnChainChessScStorage
    + on_chain_chess_sc_owner_interactions::OnChainChessScAdminInteractions
    + on_chain_chess_sc_interactions::OnChainChessScInteractions
    + on_chain_chess_sc_requirements::OnChainChessScRequirements
{
    #[init]
    fn init(&self) {
        self.is_paused().set(true);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
