//! Exceptions specific to this fork.

/// Unknown [EIP-2718] transaction type byte.
pub struct TransactionTypeError {
    pub transaction_type: u8,
}

// class TransactionTypeError(InvalidTransaction):
//     """
//     Unknown [EIP-2718] transaction type byte.

//     [EIP-2718]: https://eips.ethereum.org/EIPS/eip-2718
//     """

//     transaction_type: Final[int]
//     """
//     The type byte of the transaction that caused the error.
//     """

//     def __init__(self, transaction_type: int):
//         super().__init__(f"unknown transaction type `{transaction_type}`")
//         self.transaction_type = transaction_type
