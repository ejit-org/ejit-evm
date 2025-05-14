//!     """
//! Ethereum Virtual Machine (EVM) Exceptions
//! ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//!
//! .. contents:: Table of Contents
//!     :backlinks: none
//!     :local:
//!
//! Introduction
//! ------------
//!
//! Exceptions which cause the EVM to halt exceptionally.
//! """

pub enum VmError {
    /// """
    /// Indicates that the EVM has experienced an exceptional halt. This causes
    /// execution to immediately end with all gas being consumed.
    /// """
    ExceptionalHalt,

    /// """
    /// Raised by the `REVERT` opcode.
    ///
    /// Unlike other EVM exceptions this does not result in the consumption of all
    /// gas.
    /// """
    Revert,

    /// """
    /// Occurs when a pop is executed on an empty stack.
    /// """
    StackUnderflowError,

    /// """
    /// Occurs when a push is executed on a stack at max capacity.
    /// """
    StackOverflowError,

    /// """
    /// Occurs when an operation costs more than the amount of gas left in the
    /// frame.
    /// """
    OutOfGasError,

    /// """
    /// Raised when an invalid opcode is encountered.
    /// """
    InvalidOpcode,

    /// """
    /// Occurs when the destination of a jump operation doesn't meet any of the
    /// following criteria:
    ///
    ///     * The jump destination is less than the length of the code.
    ///     * The jump destination should have the `JUMPDEST` opcode (0x5B).
    ///     * The jump destination shouldn't be part of the data corresponding to
    ///     `PUSH-N` opcodes.
    /// """
    InvalidJumpDestError,

    /// """
    /// Raised when the message depth is greater than `1024`
    /// """
    StackDepthLimitError,

    /// """
    /// Raised when an attempt is made to modify the state while operating inside
    /// of a STATICCALL context.
    /// """
    WriteInStaticContext,

    /// """
    /// Raised when an attempt was made to read data beyond the
    /// boundaries of the buffer.
    /// """
    OutOfBoundsRead,

    /// """
    /// Raised when invalid parameters are passed.
    /// """
    InvalidParameter,

    /// """
    /// Raised when the new contract code starts with 0xEF.
    /// """
    InvalidContractPrefix,

    /// """
    /// Raised when the new contract address has a collision.
    /// """
    AddressCollision,

    /// """
    /// Raised when the point evaluation precompile can't verify a proof.
    /// """
    KZGProofError,
}
