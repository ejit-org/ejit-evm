//! Ethereum Virtual Machine (EVM) Stack
//! ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//! 
//! .. contents:: Table of Contents
//!     :backlinks: none
//!     :local:
//! 
//! Introduction
//! ------------
//! 
//! Implementation of the stack operators for the EVM.

use crate::ethereum::ethereum_types::numeric::U256;

use super::exceptions::VmError;

/// Pops the top item off of `stack`.
/// 
/// Parameters
/// ----------
/// stack :
///     EVM stack.
/// 
/// Returns
/// -------
/// value : `U256`
///     The top element on the stack.
/// 
pub fn pop(stack: &mut Vec<U256>) -> Result<U256, VmError> {
    stack.pop().ok_or(VmError::StackUnderflowError)
}


/// Pushes `value` onto `stack`.
/// 
/// Parameters
/// ----------
/// stack :
///     EVM stack.
/// 
/// value :
///     Item to be pushed onto `stack`.
/// 
pub fn push(stack: &mut Vec<U256>, value: U256) -> Result<(), VmError> {
    if stack.len() == 1024 {
        Err(VmError::StackOverflowError)
    } else {
        stack.push(value);
        Ok(())
    }
}
