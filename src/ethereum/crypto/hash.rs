//! https://github.com/ethereum/execution-specs/blob/master/src/ethereum/crypto/hash.py

use tiny_keccak::Hasher;

use crate::{ethereum::{ethereum_rlp::{exceptions::RLPException, rlp::{decode_to_bytes, encode_bytes, Extended}}, ethereum_types::bytes::*, utils::hexadecimal::hex_to_bytes32}, json::{Decoder, JsonDecode, JsonError}};

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Hash32(pub (crate)[u8; 32]);

impl Extended for Hash32 {
    fn encode<'a, 'b>(&self, buffer: &'a mut Bytes) -> Result<(), RLPException> {
        Ok(encode_bytes(buffer, &self.0))
    }

    fn decode<'a, 'b>(&mut self, buffer: &'a mut &'b [u8]) -> Result<(), RLPException> {
        decode_to_bytes(buffer, &mut self.0)
    }
}

impl<'de> JsonDecode<'de> for Hash32 {
    fn decode_json(&mut self, buffer: &mut Decoder<'de>) -> Result<(), crate::json::JsonError> {
        let mut s = "";
        s.decode_json(buffer)?;
        let b32 = hex_to_bytes32(s).map_err(|_| JsonError::ExpectedHexString)?;
        *self = Self(b32.0);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Hash64([u8; 64]);

impl std::ops::Deref for Hash32 {
    type Target = [u8; 32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::Deref for Hash64 {
    type Target = [u8; 64];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Computes the keccak256 hash of the input `buffer`.
///
/// Parameters
/// ----------
/// buffer :
///     Input for the hashing function.
///
/// Returns
/// -------
/// hash : `ethereum.base_types.Hash32`
///     Output of the hash function.
pub fn keccak256(buffer: &[u8]) -> Hash32 {
    let mut hasher = tiny_keccak::Keccak::v256();
    hasher.update(buffer);
    let mut output = [0; 32];
    hasher.finalize(&mut output);
    Hash32(output)
}

/// Computes the keccak512 hash of the input `buffer`.
///
/// Parameters
/// ----------
/// buffer :
///     Input for the hashing function.
///
/// Returns
/// -------
/// hash : `ethereum.base_types.Hash32`
///     Output of the hash function.
fn keccak512(buffer: Bytes) -> Hash64 {
    // k = keccak.new(digest_bits=512)
    // return Hash64(k.update(buffer).digest())
    todo!();
}
