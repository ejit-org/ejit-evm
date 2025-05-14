//! Utility Functions For Numeric Operations
//! ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//! 
//! .. contents:: Table of Contents
//!     :backlinks: none
//!     :local:
//! 
//! Introduction
//! ------------
//! 
//! Numeric operations specific utility functions used in this specification.

use crate::ethereum::ethereum_types::{bytes::Bytes, numeric::{Int, Uint, U32}};


/// Determines the sign of a number.
/// 
/// Parameters
/// ----------
/// value :
///     The value whose sign is to be determined.
/// 
/// Returns
/// -------
/// sign : `int`
///     The sign of the number (-1 or 0 or 1).
///     The return value is based on math signum function.
pub fn get_sign(value: Int) -> Int {
    if value < 0 {
        -1
    } else if value == 0 {
        0
    } else {
        1
    }
}


/// Converts a unsigned integer to the next closest multiple of 32.
/// 
/// Parameters
/// ----------
/// value :
///     The value whose ceil32 is to be calculated.
/// 
/// Returns
/// -------
/// ceil32 : `ethereum.base_types.U256`
///     The same value if it's a perfect multiple of 32
///     else it returns the smallest multiple of 32
///     that is greater than `value`.
pub fn ceil32(value: Uint) -> Uint {
    let ceiling = Uint::from(32_u32);
    let remainder = value % ceiling;
    // Note: this could fail if value == 2**n-1
    if remainder == Uint::from(0_u32) {
        return value
    } else {
        return value + ceiling - remainder;
    }
}


/// """
/// Checks if `number` is a prime number.
/// 
/// Parameters
/// ----------
/// number :
///     The number to check for primality.
/// 
/// Returns
/// -------
/// is_number_prime : `bool`
///     Boolean indicating if `number` is prime or not.
/// """
pub fn is_prime<SupportsInt : Into<Int>>(number: SupportsInt) -> bool {
    let number = number.into();
    if number <= 1 {
        return false;
    }

    todo!();
    // # number ** 0.5 is faster than math.sqrt(number)
    // for x in range(2, int(number**0.5) + 1):
    //     # Return False if number is divisible by x
    //     if number % x == 0:
    //         return False

    return true;
}


/// """
/// Convert little endian byte stream `data` to a little endian U32
/// sequence i.e., the first U32 number of the sequence is the least
/// significant U32 number.
/// 
/// Parameters
/// ----------
/// data :
///     The byte stream (little endian) which is to be converted to a U32
///     stream.
/// 
/// Returns
/// -------
/// uint32_sequence : `Tuple[U32, ...]`
///     Sequence of U32 numbers obtained from the little endian byte
///     stream.
/// """
pub fn le_bytes_to_uint32_sequence(data: &[u8]) -> Vec<U32> {
    let mut sequence = Vec::new();
    for i in (0..data.len()).step_by(4) {
        sequence.push(u32::from_le_bytes(data[i..i + 4].try_into().unwrap()));
    }
    sequence
}


/// r"""
/// Obtain little endian byte stream from a little endian U32 sequence
/// i.e., the first U32 number of the sequence is the least significant
/// U32 number.
/// 
/// Note - In this conversion, the most significant byte (byte at the end of
/// the little endian stream) may have leading zeroes. This function doesn't
/// take care of removing these leading zeroes as shown in below example.
/// 
/// >>> le_uint32_sequence_to_bytes([U32(8)])
/// b'\x08\x00\x00\x00'
/// 
/// 
/// Parameters
/// ----------
/// sequence :
///     The U32 stream (little endian) which is to be converted to a
///     little endian byte stream.
/// 
/// Returns
/// -------
/// result : `bytes`
///     The byte stream obtained from the little endian U32 stream.
/// """
pub fn le_uint32_sequence_to_bytes(sequence: &[U32]) -> Bytes {
    let mut result_bytes = Vec::new();
    for item in sequence {
        result_bytes.extend(item.to_le_bytes());
    }

    Bytes(result_bytes)
}


/// """
/// Obtain Uint from a U32 sequence assuming that this sequence is little
/// endian i.e., the first U32 number of the sequence is the least
/// significant U32 number.
/// 
/// Parameters
/// ----------
/// sequence :
///     The U32 stream (little endian) which is to be converted to a Uint.
/// 
/// Returns
/// -------
/// value : `Uint`
///     The Uint number obtained from the conversion of the little endian
///     U32 stream.
/// """
pub fn le_uint32_sequence_to_uint(sequence: &[U32]) -> Uint {
    let sequence_as_bytes = le_uint32_sequence_to_bytes(sequence);
    let mut bytes = [0; 16];
    // May panic
    bytes[16-sequence_as_bytes.len()..].copy_from_slice(&sequence_as_bytes);
    return Uint::from_le_bytes(bytes);
}


/// """
/// Approximates factor * e ** (numerator / denominator) using
/// Taylor expansion.
/// 
/// Parameters
/// ----------
/// factor :
///     The factor.
/// numerator :
///     The numerator of the exponential.
/// denominator :
///     The denominator of the exponential.
/// 
/// Returns
/// -------
/// output : `ethereum.base_types.Uint`
///     The approximation of factor * e ** (numerator / denominator).
/// 
/// """
pub fn taylor_exponential(
    factor: Uint, numerator: Uint, denominator: Uint
) -> Uint {
    let mut i = Uint::from(1_u32);
    let mut output = Uint::from(0_u32);
    let mut numerator_accumulated = factor * denominator;
    while numerator_accumulated > Uint::from(0_u32) {
        output += numerator_accumulated;
        numerator_accumulated = (numerator_accumulated * numerator) / (
            denominator * i
        );
        i += Uint::from(1_u32);
    }
    output / denominator
}
