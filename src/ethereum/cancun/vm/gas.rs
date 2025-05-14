//! Ethereum Virtual Machine (EVM) Gas
//! ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//! 
//! .. contents:: Table of Contents
//!     :backlinks: none
//!     :local:
//! 
//! Introduction
//! ------------
//! 
//! EVM gas constants and calculators.

use crate::ethereum::{cancun::{blocks::Header, transactions::Transaction}, ethereum_types::numeric::{Uint, U256, U64}, exceptions::Exception, utils::numeric::{ceil32, taylor_exponential}};

use super::{exceptions::VmError, Evm};

// https://github.com/ethereum/execution-specs/blob/master/src/ethereum/cancun/vm/gas.py
const GAS_JUMPDEST : Uint = 1_u128;
const GAS_BASE : Uint = 2_u128;
const GAS_VERY_LOW : Uint = 3_u128;
const GAS_STORAGE_SET : Uint = 20000_u128;
const GAS_STORAGE_UPDATE : Uint = 5000_u128;
const GAS_STORAGE_CLEAR_REFUND : Uint = 4800_u128;
const GAS_LOW : Uint = 5_u128;
const GAS_MID : Uint = 8_u128;
const GAS_HIGH : Uint = 10_u128;
const GAS_EXPONENTIATION : Uint = 10_u128;
const GAS_EXPONENTIATION_PER_BYTE : Uint = 50_u128;
const GAS_MEMORY : Uint = 3_u128;
const GAS_KECCAK256 : Uint = 30_u128;
const GAS_KECCAK256_WORD : Uint = 6_u128;
const GAS_COPY : Uint = 3_u128;
const GAS_BLOCK_HASH : Uint = 20_u128;
const GAS_LOG : Uint = 375_u128;
const GAS_LOG_DATA : Uint = 8_u128;
const GAS_LOG_TOPIC : Uint = 375_u128;
const GAS_CREATE : Uint = 32000_u128;
const GAS_CODE_DEPOSIT : Uint = 200_u128;
const GAS_ZERO : Uint = 0_u128;
const GAS_NEW_ACCOUNT : Uint = 25000_u128;
const GAS_CALL_VALUE : Uint = 9000_u128;
const GAS_CALL_STIPEND : Uint = 2300_u128;
const GAS_SELF_DESTRUCT : Uint = 5000_u128;
const GAS_SELF_DESTRUCT_NEW_ACCOUNT : Uint = 25000_u128;
const GAS_ECRECOVER : Uint = 3000_u128;
const GAS_SHA256 : Uint = 60_u128;
const GAS_SHA256_WORD : Uint = 12_u128;
const GAS_RIPEMD160 : Uint = 600_u128;
const GAS_RIPEMD160_WORD : Uint = 120_u128;
const GAS_IDENTITY : Uint = 15_u128;
const GAS_IDENTITY_WORD : Uint = 3_u128;
const GAS_RETURN_DATA_COPY : Uint = 3_u128;
const GAS_FAST_STEP : Uint = 5_u128;
const GAS_BLAKE2_PER_ROUND : Uint = 1_u128;
const GAS_COLD_SLOAD : Uint = 2100_u128;
const GAS_COLD_ACCOUNT_ACCESS : Uint = 2600_u128;
const GAS_WARM_ACCESS : Uint = 100_u128;
const GAS_INIT_CODE_WORD_COST : Uint = 2_u128;
const GAS_BLOBHASH_OPCODE : Uint = 3_u128;
const GAS_POINT_EVALUATION : Uint = 50000_u128;
const TARGET_BLOB_GAS_PER_BLOCK : U64 = 393216;
const GAS_PER_BLOB : Uint = 1_u128<<17;
const MIN_BLOB_GASPRICE : Uint = 1_u128;
const BLOB_GASPRICE_UPDATE_FRACTION : Uint = 3338477_u128;



/// Define the parameters for memory extension in opcodes
/// 
/// `cost`: `ethereum.base_types.Uint`
///     The gas required to perform the extension
/// `expand_by`: `ethereum.base_types.Uint`
///     The size by which the memory will be extended
struct ExtendMemory {
    cost: Uint,
    expand_by: Uint,
}



///    Define the gas cost and stipend for executing the call opcodes.
///
///    `cost`: `ethereum.base_types.Uint`
///        The non-refundable portion of gas reserved for executing the
///        call opcode.
///    `stipend`: `ethereum.base_types.Uint`
///        The portion of gas available to sub-calls that is refundable
///        if not consumed
struct MessageCallGas {
    cost: Uint,
    stipend: Uint,
}



/// """
/// Subtracts `amount` from `evm.gas_left`.
/// 
/// Parameters
/// ----------
/// evm :
///     The current EVM.
/// amount :
///     The amount of gas the current operation requires.
/// 
/// """
fn charge_gas(evm: &mut Evm, amount: Uint) -> Result<(), VmError> {
    // evm_trace(evm, GasAndRefund(int(amount)));

    if evm.gas_left < amount {
        return Err(VmError::OutOfGasError);
    } else {
        evm.gas_left -= amount;
    }
    Ok(())
}


/// """
/// Calculates the gas cost for allocating memory
/// to the smallest multiple of 32 bytes,
/// such that the allocated size is at least as big as the given size.
/// 
/// Parameters
/// ----------
/// size_in_bytes :
///     The size of the data in bytes.
/// 
/// Returns
/// -------
/// total_gas_cost : `ethereum.base_types.Uint`
///     The gas cost for storing data in memory.
/// """
pub fn calculate_memory_gas_cost(size_in_bytes: Uint) -> Result<Uint, Exception> {
    let size_in_words = ceil32(size_in_bytes) / Uint::from(32_u32);
    let linear_cost = size_in_words * GAS_MEMORY;
    // TODO: use checked multiply.
    let quadratic_cost = size_in_words * size_in_words / Uint::from(512_u32);
    let total_gas_cost = linear_cost + quadratic_cost;
    Ok(total_gas_cost)
    // try:
    //     return total_gas_cost
    // except ValueError:
    //     return Err(Exception::OutOfGasError);
}


/// """
/// Calculates the gas amount to extend memory
/// 
/// Parameters
/// ----------
/// memory :
///     Memory contents of the EVM.
/// extensions:
///     List of extensions to be made to the memory.
///     Consists of a tuple of start position and size.
/// 
/// Returns
/// -------
/// extend_memory: `ExtendMemory`
/// """
fn calculate_gas_extend_memory(
    memory: &[u8], extensions: &[(U256, U256)]
) -> Result<ExtendMemory, Exception> {
    let mut size_to_extend = Uint::from(0_u32);
    let mut to_be_paid = Uint::from(0_u32);
    let mut current_size = Uint::from(memory.len() as u128);
    for (start_position, size) in extensions {
        if size.is_zero() {
            continue;
        }
        let before_size = ceil32(current_size);
        let after_size = ceil32(start_position.to_uint()? + size.to_uint()?);
        if after_size <= before_size {
            continue;
        }

        size_to_extend += after_size - before_size;
        let already_paid = calculate_memory_gas_cost(before_size)?;
        let total_cost = calculate_memory_gas_cost(after_size)?;
        to_be_paid += total_cost - already_paid;

        current_size = after_size;
    }

    Ok(ExtendMemory { cost: to_be_paid, expand_by: size_to_extend })
}


/// """
/// Calculates the MessageCallGas (cost and stipend) for
/// executing call Opcodes.
/// 
/// Parameters
/// ----------
/// value:
///     The amount of `ETH` that needs to be transferred.
/// gas :
///     The amount of gas provided to the message-call.
/// gas_left :
///     The amount of gas left in the current frame.
/// memory_cost :
///     The amount needed to extend the memory in the current frame.
/// extra_gas :
///     The amount of gas needed for transferring value + creating a new
///     account inside a message call.
/// call_stipend :
///     The amount of stipend provided to a message call to execute code while
///     transferring value(ETH).
/// 
/// Returns
/// -------
/// message_call_gas: `MessageCallGas`
/// """
pub fn calculate_message_call_gas(
    value: U256,
    gas: Uint,
    gas_left: Uint,
    memory_cost: Uint,
    extra_gas: Uint,
    call_stipend: Uint, //  = GAS_CALL_STIPEND
) -> MessageCallGas {
    let call_stipend = if value.is_zero() { Uint::from(0_u32) } else { call_stipend };
    if gas_left < extra_gas + memory_cost {
        return MessageCallGas { cost: gas + extra_gas, stipend: gas + call_stipend };
    }

    let gas = Uint::min(gas, max_message_call_gas(gas_left - memory_cost - extra_gas));

    MessageCallGas { cost: gas + extra_gas, stipend: gas + call_stipend }
}

/// """
/// Calculates the maximum gas that is allowed for making a message call
/// 
/// Parameters
/// ----------
/// gas :
///     The amount of gas provided to the message-call.
/// 
/// Returns
/// -------
/// max_allowed_message_call_gas: `ethereum.base_types.Uint`
///     The maximum gas allowed for making the message-call.
/// """
pub fn max_message_call_gas(gas: Uint) -> Uint {
    gas - (gas / Uint::from(64_u32))
}


/// """
/// Calculates the gas to be charged for the init code in CREAT*
/// opcodes as well as create transactions.
/// 
/// Parameters
/// ----------
/// init_code_length :
///     The length of the init code provided to the opcode
///     or a create transaction
/// 
/// Returns
/// -------
/// init_code_gas: `ethereum.base_types.Uint`
///     The gas to be charged for the init code.
/// """
pub fn init_code_cost(init_code_length: Uint) -> Uint {
    GAS_INIT_CODE_WORD_COST * ceil32(init_code_length) / Uint::from(32_u32)
}


/// Calculated the excess blob gas for the current block based
/// on the gas used in the parent block.
/// 
/// Parameters
/// ----------
/// parent_header :
///     The parent block of the current block.
/// 
/// Returns
/// -------
/// excess_blob_gas: `ethereum.base_types.U64`
///     The excess blob gas for the current block.
pub fn calculate_excess_blob_gas(parent_header: &Header) -> Option<U64> {
    // At the fork block, these are defined as zero.
    let mut excess_blob_gas = U64::from(0_u64);
    let mut blob_gas_used = U64::from(0_u64);

    // todo: How do we determine if the header is from a previous fork?
    // After the fork block, read them from the parent header.
    if let Header {
        excess_blob_gas: Some(excess_blob_gas),
        blob_gas_used: Some(blob_gas_used),
        ..
    } = parent_header {
        let parent_blob_gas = excess_blob_gas + blob_gas_used;
        if parent_blob_gas < TARGET_BLOB_GAS_PER_BLOCK {
            Some(U64::from(0_u64))
        } else {
            Some(parent_blob_gas - TARGET_BLOB_GAS_PER_BLOCK)
        }
    } else {
        None
    }

}

/// """
/// Calculate the total blob gas for a transaction.
/// 
/// Parameters
/// ----------
/// tx :
///     The transaction for which the blob gas is to be calculated.
/// 
/// Returns
/// -------
/// total_blob_gas: `ethereum.base_types.Uint`
///     The total blob gas for the transaction.
/// """
pub fn calculate_total_blob_gas(tx: &Transaction) -> Uint {
    if let Transaction::BlobTransaction(tx) = tx {
        GAS_PER_BLOB * Uint::from(tx.blob_versioned_hashes.len() as u64)
    } else {
        Uint::from(0_u32)
    }
}


/// """
/// Calculate the blob gasprice for a block.
/// 
/// Parameters
/// ----------
/// excess_blob_gas :
///     The excess blob gas for the block.
/// 
/// Returns
/// -------
/// blob_gasprice: `Uint`
///     The blob gasprice.
/// """
pub fn calculate_blob_gas_price(excess_blob_gas: U64) -> Uint {
    taylor_exponential(
        MIN_BLOB_GASPRICE,
        Uint::from(excess_blob_gas),
        BLOB_GASPRICE_UPDATE_FRACTION,
    )
}


/// """
/// Calculate the blob data fee for a transaction.
/// 
/// Parameters
/// ----------
/// excess_blob_gas :
///     The excess_blob_gas for the execution.
/// tx :
///     The transaction for which the blob data fee is to be calculated.
/// 
/// Returns
/// -------
/// data_fee: `Uint`
///     The blob data fee.
/// """
pub fn calculate_data_fee(excess_blob_gas: U64, tx: &Transaction) -> Uint {
    calculate_total_blob_gas(tx) * calculate_blob_gas_price(
        excess_blob_gas
    )
}
