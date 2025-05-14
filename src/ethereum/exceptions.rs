//! Error types common across all Ethereum forks.
//! 
//! 

use crate::json::JsonError;

use super::ethereum_rlp::exceptions::RLPException;

#[derive(Debug)]
pub enum Exception {
    /// Base class for all exceptions _expected_ to be thrown during normal
    /// operation.
    EthereumException(&'static str),
    /// Thrown when a block being processed is found to be invalid.
    InvalidBlock(&'static str),
    /// Thrown when a transaction being processed is found to be invalid.
    InvalidTransaction(&'static str),
    /// Thrown when a transaction originates from an account that cannot send
    /// transactions.
    InvalidSenderError(&'static str),
    /// Thrown when a transaction has an invalid signature.
    InvalidSignatureError(&'static str),

    /// Rlp
    RLPException(RLPException),

    /// Rlp
    JsonError(JsonError),

    TransactionTypeError{ transaction_type: u8 },
    NumericOverflow,
}

impl From<RLPException> for Exception {
    fn from(value: RLPException) -> Self {
        Exception::RLPException(value)
    }
}
