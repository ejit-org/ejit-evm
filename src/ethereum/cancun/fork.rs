//! Ethereum Specification
//! ^^^^^^^^^^^^^^^^^^^^^^
//!
//! .. contents:: Table of Contents
//!     :backlinks: none
//!     :local:
//!
//! Introduction
//! ------------
//!
//! Entry point for the Ethereum specification.

use std::collections::{BTreeSet, HashSet};

use crate::ethereum::{
        crypto::hash::{keccak256, Hash32},
        ethereum_rlp::rlp::{self, Extended},
        ethereum_types::{
            bytes::{Bytes, Bytes20, Bytes32, Bytes8},
            numeric::{Uint, U256, U64},
        },
        exceptions::Exception, genesis::Genesis,
    };

use super::{
    blocks::{Block, Header, Log, Receipt, Withdrawal},
    fork_types::{Address, Bloom, Root},
    state::{get_account, State, TransientStorage},
    transactions::{AccessListTransaction, BlobTransaction, FeeMarketTransaction, LegacyTransaction, Transaction},
    trie::Trie,
    vm::{self, exceptions::VmError, gas::calculate_excess_blob_gas, interpreter::process_message_call},
};

const BASE_FEE_MAX_CHANGE_DENOMINATOR: Uint = 8;
const ELASTICITY_MULTIPLIER: Uint = 2;
const GAS_LIMIT_ADJUSTMENT_FACTOR: Uint = 1024;
const GAS_LIMIT_MINIMUM: Uint = 5000;
const EMPTY_OMMER_HASH: Hash32 = Hash32([0; 32]); //keccak256(rlp.encode([]));
const SYSTEM_ADDRESS: Address = Address::from_be_bytes([
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xfe,
]);
const BEACON_ROOTS_ADDRESS: Address = Address::from_be_bytes([
    0x00, 0x0F, 0x3d, 0xf6, 0xD7, 0x32, 0x80, 0x7E, 0xf1, 0x31, 0x9f, 0xB7, 0xB8, 0xbB, 0x85, 0x22,
    0xd0, 0xBe, 0xac, 0x02,
]);
const SYSTEM_TRANSACTION_GAS: Uint = 30000000;
const MAX_BLOB_GAS_PER_BLOCK: Uint = 786432;
const VERSIONED_HASH_VERSION_KZG: &'static [u8] = b"\x01";

#[derive(Debug)]
/// History and current state of the block chain.
pub struct BlockChain {
    pub blocks: Vec<Block>,
    pub state: State,
    pub chain_id: U64,
}

impl BlockChain {
    pub fn from_genesis(genesis: Genesis) -> Self {
        let block = Block {
            header: genesis.header,
            transactions: Default::default(),
            ommers: Default::default(),
            withdrawals: Default::default(),
        };
        let state = State::from_alloc(genesis.alloc);
        Self {
            blocks: vec![block],
            state,
            chain_id: genesis.chain_id,
        }
    }
}

/// Transforms the state from the previous hard fork (`old`) into the block
/// chain object for this hard fork and returns it.

/// When forks need to implement an irregular state transition, this function
/// is used to handle the irregularity. See the :ref:`DAO Fork <dao-fork>` for
/// an example.

/// Parameters
/// ----------
/// old :
///     Previous block chain object.

/// Returns
/// -------
/// new : `BlockChain`
///     Upgraded block chain object for this hard fork.
fn apply_fork(chain: BlockChain) -> BlockChain {
    chain
}

///     Obtain the list of hashes of the previous 256 blocks in order of
///     increasing block number.
///
///     This function will return less hashes for the first 256 blocks.
///
///     The ``BLOCKHASH`` opcode needs to access the latest hashes on the chain,
///     therefore this function retrieves them.
///
///     Parameters
///     ----------
///     chain :
///         History and current state.
///
///     Returns
///     -------
///     recent_block_hashes : `List[Hash32]`
///         Hashes of the recent 256 blocks in order of increasing block number.
fn get_last_256_block_hashes(chain: &BlockChain) -> Vec<Hash32> {
    let start = chain.blocks.len().saturating_sub(256);
    let recent_blocks = &chain.blocks[start..chain.blocks.len()];
    let mut recent_block_hashes = recent_blocks
        .iter()
        .map(|b| b.header.parent_hash.clone())
        .collect();
    // recent_block_hashes.append(keccak256(rlp::encode(recent_blocks[-1].header));
    recent_block_hashes
}

///    Attempts to apply a block to an existing block chain.
///
///    All parts of the block's contents need to be verified before being added
///    to the chain. Blocks are verified by ensuring that the contents of the
///    block make logical sense with the contents of the parent block. The
///    information in the block's header must also match the corresponding
///    information in the block.
///
///    To implement Ethereum, in theory clients are only required to store the
///    most recent 255 blocks of the chain since as far as execution is
///    concerned, only those blocks are accessed. Practically, however, clients
///    should store more blocks to handle reorgs.
///
///    Parameters
///    ----------
///    chain :
///        History and current state.
///    block :
///        Block to apply to `chain`.
fn state_transition(chain: &mut BlockChain, block: Block) -> Result<(), Exception> {
    let parent_header = chain
        .blocks
        .get(chain.blocks.len() - 1)
        .map(|b| &b.header)
        .unwrap();
    let excess_blob_gas = calculate_excess_blob_gas(parent_header);
    if block.header.excess_blob_gas != excess_blob_gas {
        return Err(Exception::InvalidBlock(
            "block.header.excess_blob_gas != excess_blob_gas"
        ));
    }

    validate_header(&block.header, parent_header);
    if !block.ommers.is_empty() {
        return Err(Exception::InvalidBlock("!block.ommers.is_empty()"));
    }

    let last_256_block_hashes = get_last_256_block_hashes(chain);
    let apply_body_output = apply_body(
        &mut chain.state,
        &last_256_block_hashes,
        &block.header.coinbase,
        &block.header.number,
        &block.header.base_fee_per_gas,
        &block.header.gas_limit,
        &block.header.timestamp,
        &block.header.prev_randao,
        &block.transactions,
        chain.chain_id,
        block.withdrawals.as_deref(),
        &block.header.parent_beacon_block_root,
        &excess_blob_gas,
    )?;
    if apply_body_output.block_gas_used != block.header.gas_used {
        return Err(Exception::InvalidBlock(
            "apply_body_output.block_gas_used != block.header.gas_used"
        ));
    }
    if apply_body_output.transactions_root != block.header.transactions_root {
        return Err(Exception::InvalidBlock(
            "apply_body_output.transactions_root != block.header.transactions_root"
        ));
    }
    if apply_body_output.state_root != block.header.state_root {
        return Err(Exception::InvalidBlock(
            "apply_body_output.state_root != block.header.state_root"
        ));
    }
    if apply_body_output.receipt_root != block.header.receipt_root {
        return Err(Exception::InvalidBlock(
            "apply_body_output.receipt_root != block.header.receipt_root"
        ));
    }
    if apply_body_output.block_logs_bloom != block.header.bloom {
        return Err(Exception::InvalidBlock(
            "apply_body_output.block_logs_bloom != block.header.bloom"
        ));
    }
    if apply_body_output.withdrawals_root != block.header.withdrawals_root {
        return Err(Exception::InvalidBlock(
            "apply_body_output.withdrawals_root != block.header.withdrawals_root"
        ));
    }
    if apply_body_output.blob_gas_used != block.header.blob_gas_used {
        return Err(Exception::InvalidBlock(
            "apply_body_output.blob_gas_used != block.header.blob_gas_used"
        ));
    }

    chain.blocks.push(block);
    // if self.blocks.len() > 255 {
    //     // Real clients have to store more blocks to deal with reorgs, but the
    //     // protocol only requires the last 255
    //     self.blocks.drain(0..self.blocks.len().saturating_sub(255));
    // }
    Ok(())
}

/// Calculates the base fee per gas for the block.
///
/// Parameters
/// ----------
/// block_gas_limit :
///     Gas limit of the block for which the base fee is being calculated.
/// parent_gas_limit :
///     Gas limit of the parent block.
/// parent_gas_used :
///     Gas used in the parent block.
/// parent_base_fee_per_gas :
///     Base fee per gas of the parent block.
///
/// Returns
/// -------
/// base_fee_per_gas : `Uint`
///     Base fee per gas for the block.
fn calculate_base_fee_per_gas(
    block_gas_limit: Uint,
    parent_gas_limit: Uint,
    parent_gas_used: Uint,
    parent_base_fee_per_gas: Uint,
) -> Result<Uint, Exception> {
    let parent_gas_target = parent_gas_limit / ELASTICITY_MULTIPLIER;
    if !check_gas_limit(block_gas_limit, parent_gas_limit) {
        return Err(Exception::InvalidBlock(
            "!check_gas_limit(block_gas_limit, parent_gas_limit)"
        ));
    }

    if parent_gas_used == parent_gas_target {
        Ok(parent_base_fee_per_gas)
    } else if parent_gas_used > parent_gas_target {
        let gas_used_delta = parent_gas_used - parent_gas_target;

        let parent_fee_gas_delta = parent_base_fee_per_gas * gas_used_delta;
        let target_fee_gas_delta = parent_fee_gas_delta / parent_gas_target;

        let base_fee_per_gas_delta =
            (target_fee_gas_delta / BASE_FEE_MAX_CHANGE_DENOMINATOR).max(1);

        Ok(parent_base_fee_per_gas + base_fee_per_gas_delta)
    } else {
        let gas_used_delta = parent_gas_target - parent_gas_used;

        let parent_fee_gas_delta = parent_base_fee_per_gas * gas_used_delta;
        let target_fee_gas_delta = parent_fee_gas_delta / parent_gas_target;

        let base_fee_per_gas_delta = (target_fee_gas_delta / BASE_FEE_MAX_CHANGE_DENOMINATOR);

        Ok(parent_base_fee_per_gas - base_fee_per_gas_delta)
    }
}

/// Verifies a block header.
///
/// In order to consider a block's header valid, the logic for the
/// quantities in the header should match the logic for the block itself.
/// For example the header timestamp should be greater than the block's parent
/// timestamp because the block was created *after* the parent block.
/// Additionally, the block's number should be directly following the parent
/// block's number since it is the next block in the sequence.
///
/// Parameters
/// ----------
/// header :
///     Header to check for correctness.
/// parent_header :
///     Parent Header of the header to check for correctness
fn validate_header(header: &Header, parent_header: &Header) -> Result<(), Exception> {
    if header.gas_used > header.gas_limit {
        return Err(Exception::InvalidBlock(
            "header.gas_used > header.gas_limit"
        ));
    }

    if let Some(parent_base_fee_per_gas) = parent_header.base_fee_per_gas {
        if let Some(base_fee_per_gas) = header.base_fee_per_gas {
            let expected_base_fee_per_gas = calculate_base_fee_per_gas(
                header.gas_limit,
                parent_header.gas_limit,
                parent_header.gas_used,
                parent_base_fee_per_gas,
            )?;
            if expected_base_fee_per_gas != base_fee_per_gas {
                return Err(Exception::InvalidBlock(
                    "expected_base_fee_per_gas != header.base_fee_per_gas"
                ));
            }
        }
    }

    if header.timestamp <= parent_header.timestamp {
        return Err(Exception::InvalidBlock(
            "header.timestamp <= parent_header.timestamp"
        ));
    }
    if header.number != parent_header.number + 1 {
        return Err(Exception::InvalidBlock(
            "header.number != parent_header.number + Uint(1)"
        ));
    }
    if header.extra_data.len() > 32 {
        return Err(Exception::InvalidBlock(
            "header.extra_data.len() > 32"
        ));
    }
    if header.difficulty != 0 {
        return Err(Exception::InvalidBlock("header.difficulty != 0"));
    }
    if header.nonce != Bytes8(*b"\x00\x00\x00\x00\x00\x00\x00\x00") {
        return Err(Exception::InvalidBlock(
            "header.nonce != Bytes8(*b\"\x00\x00\x00\x00\x00\x00\x00\x00\")"
        ));
    }
    if header.ommers_hash != EMPTY_OMMER_HASH {
        return Err(Exception::InvalidBlock(
            "header.ommers_hash != EMPTY_OMMER_HASH"
        ));
    }

    let block_parent_hash = keccak256(&rlp::encode(parent_header)?);
    if header.parent_hash != block_parent_hash {
        return Err(Exception::InvalidBlock(
            "header.parent_hash != block_parent_hash"
        ));
    }

    Ok(())
}

// def check_transaction(
//     state: State,
//     tx: Transaction,
//     gas_available: Uint,
//     chain_id: U64,
//     base_fee_per_gas: Uint,
//     excess_blob_gas: U64,
// ) -> Tuple[Address, Uint, Tuple[VersionedHash, ...]]:
//     """
//     Check if the transaction is includable in the block.

//     Parameters
//     ----------
//     state :
//         Current state.
//     tx :
//         The transaction.
//     gas_available :
//         The gas remaining in the block.
//     chain_id :
//         The ID of the current chain.
//     base_fee_per_gas :
//         The block base fee.
//     excess_blob_gas :
//         The excess blob gas.

//     Returns
//     -------
//     sender_address :
//         The sender of the transaction.
//     effective_gas_price :
//         The price to charge for gas when the transaction is executed.
//     blob_versioned_hashes :
//         The blob versioned hashes of the transaction.

//     Raises
//     ------
//     InvalidBlock :
//         If the transaction is not includable.
//     """
//     if tx.gas > gas_available:
//         raise InvalidBlock
//     sender_address = recover_sender(chain_id, tx)
//     sender_account = get_account(state, sender_address)

//     if isinstance(tx, (FeeMarketTransaction, BlobTransaction)):
//         if tx.max_fee_per_gas < tx.max_priority_fee_per_gas:
//             raise InvalidBlock
//         if tx.max_fee_per_gas < base_fee_per_gas:
//             raise InvalidBlock

//         priority_fee_per_gas = min(
//             tx.max_priority_fee_per_gas,
//             tx.max_fee_per_gas - base_fee_per_gas,
//         )
//         effective_gas_price = priority_fee_per_gas + base_fee_per_gas
//         max_gas_fee = tx.gas * tx.max_fee_per_gas
//     else:
//         if tx.gas_price < base_fee_per_gas:
//             raise InvalidBlock
//         effective_gas_price = tx.gas_price
//         max_gas_fee = tx.gas * tx.gas_price

//     if isinstance(tx, BlobTransaction):
//         if not isinstance(tx.to, Address):
//             raise InvalidBlock
//         if len(tx.blob_versioned_hashes) == 0:
//             raise InvalidBlock
//         for blob_versioned_hash in tx.blob_versioned_hashes:
//             if blob_versioned_hash[0:1] != VERSIONED_HASH_VERSION_KZG:
//                 raise InvalidBlock

//         blob_gas_price = calculate_blob_gas_price(excess_blob_gas)
//         if Uint(tx.max_fee_per_blob_gas) < blob_gas_price:
//             raise InvalidBlock

//         max_gas_fee += calculate_total_blob_gas(tx) * Uint(
//             tx.max_fee_per_blob_gas
//         )
//         blob_versioned_hashes = tx.blob_versioned_hashes
//     else:
//         blob_versioned_hashes = ()
//     if sender_account.nonce != tx.nonce:
//         raise InvalidBlock
//     if Uint(sender_account.balance) < max_gas_fee + Uint(tx.value):
//         raise InvalidBlock
//     if sender_account.code != bytearray():
//         raise InvalidSenderError("not EOA")

//     return sender_address, effective_gas_price, blob_versioned_hashes

// def make_receipt(
//     tx: Transaction,
//     error: Optional[EthereumException],
//     cumulative_gas_used: Uint,
//     logs: Tuple[Log, ...],
// ) -> Union[Bytes, Receipt]:
//     """
//     Make the receipt for a transaction that was executed.

//     Parameters
//     ----------
//     tx :
//         The executed transaction.
//     error :
//         Error in the top level frame of the transaction, if any.
//     cumulative_gas_used :
//         The total gas used so far in the block after the transaction was
//         executed.
//     logs :
//         The logs produced by the transaction.

//     Returns
//     -------
//     receipt :
//         The receipt for the transaction.
//     """
//     receipt = Receipt(
//         succeeded=error is None,
//         cumulative_gas_used=cumulative_gas_used,
//         bloom=logs_bloom(logs),
//         logs=logs,
//     )

//     if isinstance(tx, AccessListTransaction):
//         return b"\x01" + rlp.encode(receipt)
//     elif isinstance(tx, FeeMarketTransaction):
//         return b"\x02" + rlp.encode(receipt)
//     elif isinstance(tx, BlobTransaction):
//         return b"\x03" + rlp.encode(receipt)
//     else:
//         return receipt

///     Output from applying the block body to the present state.
///
///     Contains the following:
///
///     block_gas_used : `ethereum.base_types.Uint`
///         Gas used for executing all transactions.
///     transactions_root : `ethereum.fork_types.Root`
///         Trie root of all the transactions in the block.
///     receipt_root : `ethereum.fork_types.Root`
///         Trie root of all the receipts in the block.
///     block_logs_bloom : `Bloom`
///         Logs bloom of all the logs included in all the transactions of the
///         block.
///     state_root : `ethereum.fork_types.Root`
///         State root after all transactions have been executed.
///     withdrawals_root : `ethereum.fork_types.Root`
///         Trie root of all the withdrawals in the block.
///     blob_gas_used : `ethereum.base_types.Uint`
///         Total blob gas used in the block.
pub struct ApplyBodyOutput {
    block_gas_used: Uint,
    transactions_root: Root,
    receipt_root: Root,
    block_logs_bloom: Bloom,
    state_root: Root,
    withdrawals_root: Option<Root>,
    blob_gas_used: Option<U64>,
}

/// Executes a block.
///
/// Many of the contents of a block are stored in data structures called
/// tries. There is a transactions trie which is similar to a ledger of the
/// transactions stored in the current block. There is also a receipts trie
/// which stores the results of executing a transaction, like the post state
/// and gas used. This function creates and executes the block that is to be
/// added to the chain.
///
/// Parameters
/// ----------
/// state :
///     Current account state.
/// block_hashes :
///     List of hashes of the previous 256 blocks in the order of
///     increasing block number.
/// coinbase :
///     Address of account which receives block reward and transaction fees.
/// block_number :
///     Position of the block within the chain.
/// base_fee_per_gas :
///     Base fee per gas of within the block.
/// block_gas_limit :
///     Initial amount of gas available for execution in this block.
/// block_time :
///     Time the block was produced, measured in seconds since the epoch.
/// prev_randao :
///     The previous randao from the beacon chain.
/// transactions :
///     Transactions included in the block.
/// ommers :
///     Headers of ancestor blocks which are not direct parents (formerly
///     uncles.)
/// chain_id :
///     ID of the executing chain.
/// withdrawals :
///     Withdrawals to be processed in the current block.
/// parent_beacon_block_root :
///     The root of the beacon block from the parent block.
/// excess_blob_gas :
///     Excess blob gas calculated from the previous block.
///
/// Returns
/// -------
/// apply_body_output : `ApplyBodyOutput`
///     Output of applying the block body to the state.
pub fn apply_body(
    state: &mut State,
    block_hashes: &[Hash32],
    coinbase: &Address,
    block_number: &Uint,
    base_fee_per_gas: &Option<Uint>,
    block_gas_limit: &Uint,
    block_time: &U256,
    prev_randao: &Bytes32,
    transactions: &[Transaction],
    chain_id: U64,
    withdrawals: Option<&[Withdrawal]>,
    parent_beacon_block_root: &Option<Root>,
    excess_blob_gas: &Option<U64>,
) -> Result<ApplyBodyOutput, Exception> {
    // let blob_gas_used = 0;
    // let mut gas_available = block_gas_limit;
    // let transactions_trie: Trie<Bytes, Option<Either<LegacyTransaction, Bytes>>> =
    //     Trie::new(false, None);
    // let receipts_trie: Trie<Bytes, Option<Either<Receipt, Bytes>>> = Trie::new(false, None);
    // let withdrawals_trie: Trie<Bytes, Option<Either<Withdrawal, Bytes>>> = Trie::new(false, None);

    // let mut block_logs = Vec::new();

    // let beacon_block_roots_contract_code = get_account(state, &BEACON_ROOTS_ADDRESS).code;

    // let system_tx_message = vm::Message {
    //     caller: SYSTEM_ADDRESS,
    //     target: Either::B(BEACON_ROOTS_ADDRESS),
    //     gas: SYSTEM_TRANSACTION_GAS,
    //     value: U256::from(0_u32),
    //     data: Bytes::from(parent_beacon_block_root.as_ref()),
    //     code: beacon_block_roots_contract_code,
    //     depth: Uint::from(0_u32),
    //     current_target: BEACON_ROOTS_ADDRESS,
    //     code_address: Some(BEACON_ROOTS_ADDRESS),
    //     should_transfer_value: false,
    //     is_static: false,
    //     accessed_addresses: BTreeSet::new(),
    //     accessed_storage_keys: BTreeSet::new(),
    //     parent_evm: None,
    // };

    // let mut system_tx_env = vm::Environment {
    //     caller: SYSTEM_ADDRESS,
    //     origin: SYSTEM_ADDRESS,
    //     block_hashes: block_hashes.to_vec(),
    //     coinbase: coinbase,
    //     number: block_number,
    //     gas_limit: block_gas_limit,
    //     base_fee_per_gas: base_fee_per_gas,
    //     gas_price: base_fee_per_gas,
    //     time: block_time,
    //     prev_randao: prev_randao,
    //     state: state,
    //     chain_id: chain_id,
    //     traces: Vec::new(),
    //     excess_blob_gas: excess_blob_gas,
    //     blob_versioned_hashes: Vec::new(),
    //     transient_storage: TransientStorage::default(),
    // };

    // let system_tx_output = process_message_call(&system_tx_message, &mut system_tx_env)?;

    // destroy_touched_empty_accounts(system_tx_env.state, system_tx_output.touched_accounts);

    // for (i, tx) in transactions.iter().map(decode_transaction).enumerate() {
    //     trie_set(
    //         transactions_trie,
    //         rlp.encode(Uint::from(i)),
    //         encode_transaction(tx),
    //     );

    //     let (sender_address, effective_gas_price, blob_versioned_hashes) = check_transaction(
    //         state,
    //         tx,
    //         gas_available,
    //         chain_id,
    //         base_fee_per_gas,
    //         excess_blob_gas,
    //     );

    //     let env = vm::Environment {
    //         caller: sender_address,
    //         origin: sender_address,
    //         block_hashes: block_hashes,
    //         coinbase: coinbase,
    //         number: block_number,
    //         gas_limit: block_gas_limit,
    //         base_fee_per_gas: base_fee_per_gas,
    //         gas_price: effective_gas_price,
    //         time: block_time,
    //         prev_randao: prev_randao,
    //         state: state,
    //         chain_id: chain_id,
    //         traces: Vec::new(),
    //         excess_blob_gas: excess_blob_gas,
    //         blob_versioned_hashes: blob_versioned_hashes,
    //         transient_storage: TransientStorage::default(),
    //     };

    //     let (gas_used, logs, error) = process_transaction(&env, tx)?;
    //     gas_available -= gas_used;

    //     let receipt = make_receipt(tx, error, (block_gas_limit - gas_available), logs);

    //     trie_set(receipts_trie, rlp::encode(&Uint::from(i)), receipt);

    //     block_logs += logs;
    //     blob_gas_used += calculate_total_blob_gas(tx);
    // }

    // if blob_gas_used > MAX_BLOB_GAS_PER_BLOCK {
    //     return Err(Exception::InvalidBlock(
    //         "blob_gas_used > MAX_BLOB_GAS_PER_BLOCK"
    //     ));
    // }
    // let block_gas_used = block_gas_limit - gas_available;

    // let block_logs_bloom = logs_bloom(block_logs);

    // for (i, wd) in withdrawals.iter().enumerate() {
    //     trie_set(withdrawals_trie, rlp.encode(Uint(i)), rlp.encode(wd));

    //     process_withdrawal(state, wd);

    //     if account_exists_and_is_empty(state, wd.address) {
    //         destroy_account(state, wd.address);
    //     }
    // }

    // return Ok(ApplyBodyOutput {
    //     block_gas_used,
    //     transactions_root: transactions_trie.root(),
    //     receipt_root: receipts_trie.root(),
    //     block_logs_bloom,
    //     state_root: state.state_root(),
    //     withdrawals_root: withdrawals_trie.root(),
    //     blob_gas_used,
    // });

    todo!()
}


/// """
/// Execute a transaction against the provided environment.
/// 
/// This function processes the actions needed to execute a transaction.
/// It decrements the sender's account after calculating the gas fee and
/// refunds them the proper amount after execution. Calling contracts,
/// deploying code, and incrementing nonces are all examples of actions that
/// happen within this function or from a call made within this function.
/// 
/// Accounts that are marked for deletion are processed and destroyed after
/// execution.
/// 
/// Parameters
/// ----------
/// env :
///     Environment for the Ethereum Virtual Machine.
/// tx :
///     Transaction to execute.
/// 
/// Returns
/// -------
/// gas_left : `ethereum.base_types.U256`
///     Remaining gas after execution.
/// logs : `Tuple[ethereum.blocks.Log, ...]`
///     Logs generated during execution.
/// """
pub fn process_transaction(
    env: &vm::Environment, tx: &Transaction
) -> (Uint, Vec<Log>, Option<VmError>) {
    // if !validate_transaction(tx) {
    //     return Err(Exception::InvalidBlock(
    //         "!validate_transaction(tx)"
    //     ));
    // }

    // let sender = env.origin;
    // let sender_account = get_account(env.state, &sender);

    // let blob_gas_fee = if let Transaction::BlobTransaction(tx) = tx {
    //     calculate_data_fee(env.excess_blob_gas, tx)
    // } else {
    //     Uint::from(0_u32)
    // };

    // let effective_gas_fee = tx.gas * env.gas_price;

    // let gas = tx.gas - calculate_intrinsic_cost(tx);
    // increment_nonce(env.state, sender);

    // let sender_balance_after_gas_fee = (
    //     Uint(sender_account.balance) - effective_gas_fee - blob_gas_fee
    // );
    // set_account_balance(env.state, sender, U256(sender_balance_after_gas_fee));

    // let mut preaccessed_addresses = HashSet::new();
    // let mut preaccessed_storage_keys = HashSet::new();
    // preaccessed_addresses.insert(env.coinbase);

    // match tx {
    //     Transaction::AccessListTransaction(AccessListTransaction{access_list, ..}) |
    //     Transaction::FeeMarketTransaction(FeeMarketTransaction{access_list, ..}) |
    //     Transaction::BlobTransaction(BlobTransaction{access_list, ..}) => {
    //         for (address, keys) in access_list {
    //             preaccessed_addresses.insert(address.clone());
    //             for key in keys {
    //                 preaccessed_storage_keys.insert((address, key));
    //             }
    //         }
    //     }
    //     _ => (),
    // }

    // let message = prepare_message(
    //     sender,
    //     tx.to,
    //     tx.value,
    //     tx.data,
    //     gas,
    //     env,
    //     preaccessed_addresses=frozenset(preaccessed_addresses),
    //     preaccessed_storage_keys=frozenset(preaccessed_storage_keys),
    // );

    // let output = process_message_call(message, env);

    // let gas_used = tx.gas - output.gas_left;
    // let gas_refund = min(gas_used / Uint::from(5_u32), Uint::from(output.refund_counter));
    // let gas_refund_amount = (output.gas_left + gas_refund) * env.gas_price;

    // //  For non-1559 transactions env.gas_price == tx.gas_price
    // let priority_fee_per_gas = env.gas_price - env.base_fee_per_gas;
    // transaction_fee = (
    //     tx.gas - output.gas_left - gas_refund
    // ) * priority_fee_per_gas;

    // let total_gas_used = gas_used - gas_refund;

    // // refund gas
    // let sender_balance_after_refund = get_account(
    //     env.state, &sender
    // ).balance + U256::from(gas_refund_amount);
    // set_account_balance(env.state, sender, sender_balance_after_refund);

    // // transfer miner fees
    // coinbase_balance_after_mining_fee = get_account(
    //     env.state, &env.coinbase
    // ).balance + U256::from(transaction_fee);
    // if coinbase_balance_after_mining_fee != 0 {
    //     set_account_balance(
    //         env.state, env.coinbase, coinbase_balance_after_mining_fee
    //     )
    // } else if account_exists_and_is_empty(env.state, env.coinbase) {
    //     destroy_account(env.state, env.coinbase);
    // }

    // for address in output.accounts_to_delete {
    //     destroy_account(env.state, address);
    // }

    // destroy_touched_empty_accounts(env.state, output.touched_accounts);

    // (total_gas_used, output.logs, output.error)
    todo!()
}

/// """
/// Computes the hash of a block header.
/// 
/// The header hash of a block is the canonical hash that is used to refer
/// to a specific block and completely distinguishes a block from another.
/// 
/// ``keccak256`` is a function that produces a 256 bit hash of any input.
/// It also takes in any number of bytes as an input and produces a single
/// hash for them. A hash is a completely unique output for a single input.
/// So an input corresponds to one unique hash that can be used to identify
/// the input exactly.
/// 
/// Prior to using the ``keccak256`` hash function, the header must be
/// encoded using the Recursive-Length Prefix. See :ref:`rlp`.
/// RLP encoding the header converts it into a space-efficient format that
/// allows for easy transfer of data between nodes. The purpose of RLP is to
/// encode arbitrarily nested arrays of binary data, and RLP is the primary
/// encoding method used to serialize objects in Ethereum's execution layer.
/// The only purpose of RLP is to encode structure; encoding specific data
/// types (e.g. strings, floats) is left up to higher-order protocols.
/// 
/// Parameters
/// ----------
/// header :
///     Header of interest.
/// 
/// Returns
/// -------
/// hash : `ethereum.crypto.hash.Hash32`
///     Hash of the header.
/// """
fn compute_header_hash(header: &Header) -> Result<Hash32, Exception> {
    Ok(keccak256(&rlp::encode(header)?))
}

/// Validates the gas limit for a block.
/// 
/// The bounds of the gas limit, ``max_adjustment_delta``, is set as the
/// quotient of the parent block's gas limit and the
/// ``GAS_LIMIT_ADJUSTMENT_FACTOR``. Therefore, if the gas limit that is
/// passed through as a parameter is greater than or equal to the *sum* of
/// the parent's gas and the adjustment delta then the limit for gas is too
/// high and fails this function's check. Similarly, if the limit is less
/// than or equal to the *difference* of the parent's gas and the adjustment
/// delta *or* the predefined ``GAS_LIMIT_MINIMUM`` then this function's
/// check fails because the gas limit doesn't allow for a sufficient or
/// reasonable amount of gas to be used on a block.
/// 
/// Parameters
/// ----------
/// gas_limit :
///     Gas limit to validate.
/// 
/// parent_gas_limit :
///     Gas limit of the parent block.
/// 
/// Returns
/// -------
/// check : `bool`
///     True if gas limit constraints are satisfied, False otherwise.
pub fn check_gas_limit(gas_limit: Uint, parent_gas_limit: Uint) -> bool {
    let max_adjustment_delta = parent_gas_limit / GAS_LIMIT_ADJUSTMENT_FACTOR;

    if gas_limit >= parent_gas_limit + max_adjustment_delta {
        return false;
    }

    if gas_limit <= parent_gas_limit - max_adjustment_delta {
        return false;
    }

    if gas_limit < GAS_LIMIT_MINIMUM {
        return false;
    }

    true
}


#[cfg(test)]
mod tests;
