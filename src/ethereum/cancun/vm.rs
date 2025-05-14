//! https://github.com/ethereum/execution-specs/blob/master/src/ethereum/cancun/vm/__init__.py
//! 

use std::collections::{BTreeMap, BTreeSet};

use exceptions::VmError;

use crate::{ethereum::{cancun::fork_types::*, crypto::hash::Hash32, ethereum_types::{bytes::*, numeric::*}}};

use super::{blocks::Log, state::{State, TransientStorage}};

pub mod exceptions;
pub mod gas;
pub mod instructions;
pub mod interpreter;
pub mod memory;
pub mod precompiled_contracts;
pub mod runtime;
pub mod stack;


/// Items external to the virtual machine itself, provided by the environment.
pub struct Environment<'a> {
    pub caller: Address,
    pub block_hashes: Vec<Hash32>,
    pub origin: Address,
    pub coinbase: Address,
    pub number: Uint,
    pub base_fee_per_gas: Uint,
    pub gas_limit: Uint,
    pub gas_price: Uint,
    pub time: U256,
    pub prev_randao: Bytes32,
    pub state: &'a mut State,
    pub chain_id: U64,
    pub traces: Vec<BTreeMap<String, String>>,
    pub excess_blob_gas: U64,
    pub blob_versioned_hashes: Vec<VersionedHash>,
    pub transient_storage: TransientStorage,
}

/// Items that are used by contract creation or message call.
pub struct Message<'a> {
    pub caller: Address,
    pub target: Address,
    pub current_target: Address,
    pub gas: Uint,
    pub value: U256,
    pub data: Bytes,
    pub code_address: Option<Address>,
    pub code: Bytes,
    pub depth: Uint,
    pub should_transfer_value: bool,
    pub is_static: bool,
    pub accessed_addresses: BTreeSet<Address>,
    pub accessed_storage_keys: BTreeSet<(Address, Bytes32)>,
    pub parent_evm: Option<&'a Evm<'a>>,
}


/// The internal state of the virtual machine.
pub struct Evm<'a> {
    pub pc: Uint,
    pub stack: Vec<U256>,
    pub memory: Vec<u8>,
    pub code: Bytes,
    pub gas_left: Uint,
    pub env: &'a Environment<'a>,
    pub valid_jump_destinations: Vec<Uint>,
    pub logs: Vec<Log>,
    pub refund_counter: i64,
    pub running: bool,
    pub message: Message<'a>,
    pub output: Bytes,
    pub accounts_to_delete: Vec<Address>,
    pub touched_accounts: Vec<Address>,
    pub return_data: Bytes,
    pub error: Option<VmError>,
    pub accessed_addresses: Vec<Address>,
    pub accessed_storage_keys: Vec<(Address, Bytes32)>,
}
