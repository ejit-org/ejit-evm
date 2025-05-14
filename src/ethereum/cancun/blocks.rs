//! A `Block` is a single link in the chain that is Ethereum. Each `Block` contains
//! a `Header` and zero or more transactions. Each `Header` contains associated
//! metadata like the block number, parent block hash, and how much gas was
//! consumed by its transactions.
//!
//! Together, these blocks form a cryptographically secure journal recording the
//! history of all state transitions that have happened since the genesis of the
//! chain.

use crate::{
    ethereum::{
        cancun::fork_types::{Address, Bloom, Root},
        crypto::hash::Hash32,
        ethereum_rlp::{exceptions::RLPException, rlp::{self, decode_to_sequence, encode_sequence, Extended}},
        ethereum_types::{
            bytes::{Bytes, Bytes32, Bytes8},
            numeric::{Uint, U256, U64},
        },
    }, impl_extended
};

use super::transactions::{LegacyTransaction, Transaction};

#[derive(Debug, Clone, Default)]
/// Withdrawals that have been validated on the consensus layer.
pub struct Withdrawal {
    pub index: U64,
    pub validator_index: U64,
    pub address: Address,
    pub amount: U256,
}

impl_extended!(Withdrawal: index, validator_index, address, amount);

// impl Extended for Withdrawal {
//     fn encode<'a, 'b>(&self, buffer: &'a mut Bytes) -> Result<(), RLPException> {
//         encode_sequence(buffer, &[
//             &self.index,
//             &self.validator_index,
//             &self.address,
//             &self.amount,
//         ])
//     }

//     fn decode<'a, 'b>(&mut self, buffer: &'a mut &'b [u8]) -> Result<(), RLPException> {
//         decode_to_sequence(buffer, &mut [
//             &mut self.index,
//             &mut self.validator_index,
//             &mut self.address,
//             &mut self.amount,
//         ])
//     }
// }

#[derive(Debug, Clone, Default)]
/// Header portion of a block on the chain.
pub struct Header {
    pub parent_hash: Hash32,
    pub ommers_hash: Hash32,
    pub coinbase: Address,
    pub state_root: Root,
    pub transactions_root: Root,
    pub receipt_root: Root,
    pub bloom: Bloom,
    pub difficulty: Uint,
    pub number: Uint,
    pub gas_limit: Uint,
    pub gas_used: Uint,
    pub timestamp: U256,
    pub extra_data: Bytes,
    pub prev_randao: Bytes32,
    pub nonce: Bytes8,
    pub base_fee_per_gas: Option<Uint>,
    pub withdrawals_root: Option<Root>,
    pub blob_gas_used: Option<U64>,
    pub excess_blob_gas: Option<U64>,
    pub parent_beacon_block_root: Option<Root>,
}

impl_extended!(Header: parent_hash, ommers_hash, coinbase, state_root, transactions_root, receipt_root, bloom, difficulty, number, gas_limit, gas_used, timestamp, extra_data, prev_randao, nonce, base_fee_per_gas, withdrawals_root, blob_gas_used, excess_blob_gas, parent_beacon_block_root);

#[derive(Debug, Clone, Default)]
/// A complete block.
pub struct Block {
    pub header: Header,
    pub transactions: Vec<Transaction>,
    pub ommers: Vec<Header>,
    pub withdrawals: Option<Vec<Withdrawal>>,
}

impl_extended!(Block: header,transactions,ommers,withdrawals);

#[derive(Debug, Clone, Default)]
/// Data record produced during the execution of a transaction.
pub struct Log {
    pub address: Address,
    pub topics: Vec<Hash32>,
    pub data: Bytes,
}

impl_extended!(Log: address, topics, data);

#[derive(Debug, Clone, Default)]
/// Result of a transaction.
pub struct Receipt {
    pub succeeded: bool,
    pub cumulative_gas_used: Uint,
    pub bloom: Bloom,
    pub logs: Vec<Log>,
}

impl_extended!(Receipt: succeeded, cumulative_gas_used, bloom, logs);

