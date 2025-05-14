//! """
//! Ethereum Virtual Machine (EVM) Interpreter
//! ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//! 
//! .. contents:: Table of Contents
//!     :backlinks: none
//!     :local:
//! 
//! Introduction
//! ------------
//! 
//! A straightforward interpreter that executes EVM code.
//! """

use std::collections::BTreeSet;

use crate::ethereum::{cancun::{blocks::Log, fork_types::Address}, ethereum_types::numeric::{Uint, U256}, exceptions::Exception};

use super::{Environment, Evm, Message};


pub const STACK_DEPTH_LIMIT : usize = 1024;
pub const MAX_CODE_SIZE : usize = 0x6000;

/// """
/// Output of a particular message call
/// 
/// Contains the following:
/// 
///       1. `gas_left`: remaining gas after execution.
///       2. `refund_counter`: gas to refund after execution.
///       3. `logs`: list of `Log` generated during execution.
///       4. `accounts_to_delete`: Contracts which have self-destructed.
///       5. `touched_accounts`: Accounts that have been touched.
///       6. `error`: The error from the execution if any.
/// """
pub struct MessageCallOutput {
    pub gas_left: Uint,
    pub refund_counter: U256,
    pub logs: Vec<Log>,
    pub accounts_to_delete: BTreeSet<Address>,
    pub touched_accounts: BTreeSet<Address>,
    pub error: Option<Exception>,
}

/// """
/// If `message.current` is empty then it creates a smart contract
/// else it executes a call from the `message.caller` to the `message.target`.
/// 
/// Parameters
/// ----------
/// message :
///     Transaction specific items.
/// 
/// env :
///     External items required for EVM execution.
/// 
/// Returns
/// -------
/// output : `MessageCallOutput`
///     Output of the message call
/// """
pub fn process_message_call(
    message: &Message, env: &Environment
) -> Result<MessageCallOutput, Exception> {
    // let evm = if message.target == Bytes0(b"") {
    //     let is_collision = account_has_code_or_nonce(
    //         env.state, message.current_target
    //     ) || account_has_storage(env.state, message.current_target);
    //     if is_collision {
    //         return Ok(MessageCallOutput{
    //             Uint(0), U256(0), tuple(), set(), set(), AddressCollision()

    //         });
    //     } else {
    //         process_create_message(message, env)?
    //     }
    // } else {
    //     let evm = process_message(message, env)?;
    //     if account_exists_and_is_empty(env.state, Address(message.target)):
    //         evm.touched_accounts.add(Address(message.target))
    //     evm
    // };

    // let (logs,
    // accounts_to_delete,
    // touched_accounts, 
    // refund_counter) = if evm.error {
    //     (Vec::new(),
    //     BTreeSet::new(),
    //     BTreeSet::new(),
    //     U256::from(0_u32))
    // } else {
    //     (evm.logs,
    //     evm.accounts_to_delete,
    //     evm.touched_accounts,
    //     U256(evm.refund_counter))
    // };

    // let tx_end = TransactionEnd {
    //     int(message.gas) - int(evm.gas_left), evm.output, evm.error
    // };
    // evm_trace(evm, tx_end);

    // Ok(MessageCallOutput {
    //     gas_left: evm.gas_left,
    //     refund_counter,
    //     logs,
    //     accounts_to_delete,
    //     touched_accounts,
    //     error: evm.error,
    // })

    todo!()
}


/// """
/// Executes a call to create a smart contract.
/// 
/// Parameters
/// ----------
/// message :
///     Transaction specific items.
/// env :
///     External items required for EVM execution.
/// 
/// Returns
/// -------
/// evm: :py:class:`~ethereum.cancun.vm.Evm`
///     Items containing execution specific objects.
/// """
pub fn process_create_message<'a>(message: &Message, env: &'a Environment) -> Evm<'a> {
    // # take snapshot of state before processing the message
    // begin_transaction(env.state, env.transient_storage)

    // # If the address where the account is being created has storage, it is
    // # destroyed. This can only happen in the following highly unlikely
    // # circumstances:
    // # * The address created by a `CREATE` call collides with a subsequent
    // #   `CREATE` or `CREATE2` call.
    // # * The first `CREATE` happened before Spurious Dragon and left empty
    // #   code.
    // destroy_storage(env.state, message.current_target)

    // # In the previously mentioned edge case the preexisting storage is ignored
    // # for gas refund purposes. In order to do this we must track created
    // # accounts.
    // mark_account_created(env.state, message.current_target)

    // increment_nonce(env.state, message.current_target)
    // evm = process_message(message, env)
    // if not evm.error:
    //     contract_code = evm.output
    //     contract_code_gas = Uint(len(contract_code)) * GAS_CODE_DEPOSIT
    //     try:
    //         if len(contract_code) > 0:
    //             if contract_code[0] == 0xEF:
    //                 raise InvalidContractPrefix
    //         charge_gas(evm, contract_code_gas)
    //         if len(contract_code) > MAX_CODE_SIZE:
    //             raise OutOfGasError
    //     except ExceptionalHalt as error:
    //         rollback_transaction(env.state, env.transient_storage)
    //         evm.gas_left = Uint(0)
    //         evm.output = b""
    //         evm.error = error
    //     else:
    //         set_code(env.state, message.current_target, contract_code)
    //         commit_transaction(env.state, env.transient_storage)
    // else:
    //     rollback_transaction(env.state, env.transient_storage)
    // return evm

    todo!()
}


/// """
/// Executes a call to create a smart contract.
/// 
/// Parameters
/// ----------
/// message :
///     Transaction specific items.
/// env :
///     External items required for EVM execution.
/// 
/// Returns
/// -------
/// evm: :py:class:`~ethereum.cancun.vm.Evm`
///     Items containing execution specific objects
/// """
pub fn process_message<'a>(message: &Message, env: &'a Environment) -> Evm<'a> {
    // if message.depth > STACK_DEPTH_LIMIT:
    //     raise StackDepthLimitError("Stack depth limit reached")

    // # take snapshot of state before processing the message
    // begin_transaction(env.state, env.transient_storage)

    // touch_account(env.state, message.current_target)

    // if message.should_transfer_value and message.value != 0:
    //     move_ether(
    //         env.state, message.caller, message.current_target, message.value
    //     )

    // evm = execute_code(message, env)
    // if evm.error:
    //     # revert state to the last saved checkpoint
    //     # since the message call resulted in an error
    //     rollback_transaction(env.state, env.transient_storage)
    // else:
    //     commit_transaction(env.state, env.transient_storage)
    // return evm

    todo!()
}


/// """
/// Executes bytecode present in the `message`.
/// 
/// Parameters
/// ----------
/// message :
///     Transaction specific items.
/// env :
///     External items required for EVM execution.
/// 
/// Returns
/// -------
/// evm: `ethereum.vm.EVM`
///     Items containing execution specific objects
/// """
pub fn execute_code<'a>(message: &Message, env: &'a Environment) -> Evm<'a> {
    // code = message.code
    // valid_jump_destinations = get_valid_jump_destinations(code)

    // evm = Evm(
    //     pc=Uint(0),
    //     stack=[],
    //     memory=bytearray(),
    //     code=code,
    //     gas_left=message.gas,
    //     env=env,
    //     valid_jump_destinations=valid_jump_destinations,
    //     logs=(),
    //     refund_counter=0,
    //     running=True,
    //     message=message,
    //     output=b"",
    //     accounts_to_delete=set(),
    //     touched_accounts=set(),
    //     return_data=b"",
    //     error=None,
    //     accessed_addresses=message.accessed_addresses,
    //     accessed_storage_keys=message.accessed_storage_keys,
    // )
    // try:
    //     if evm.message.code_address in PRE_COMPILED_CONTRACTS:
    //         evm_trace(evm, PrecompileStart(evm.message.code_address))
    //         PRE_COMPILED_CONTRACTS[evm.message.code_address](evm)
    //         evm_trace(evm, PrecompileEnd())
    //         return evm

    //     while evm.running and evm.pc < ulen(evm.code):
    //         try:
    //             op = Ops(evm.code[evm.pc])
    //         except ValueError:
    //             raise InvalidOpcode(evm.code[evm.pc])

    //         evm_trace(evm, OpStart(op))
    //         op_implementation[op](evm)
    //         evm_trace(evm, OpEnd())

    //     evm_trace(evm, EvmStop(Ops.STOP))

    // except ExceptionalHalt as error:
    //     evm_trace(evm, OpException(error))
    //     evm.gas_left = Uint(0)
    //     evm.output = b""
    //     evm.error = error
    // except Revert as error:
    //     evm_trace(evm, OpException(error))
    //     evm.error = error
    // return evm

    todo!()
}
