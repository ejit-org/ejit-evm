use std::ops::Deref;

use crate::{ethereum::{crypto::hash::{keccak256, Hash32}, ethereum_rlp::{exceptions::RLPException, rlp::{self, decode_to_bytes, encode_bytes, Extended}}, ethereum_types::{bytes::{Bytes20, Bytes256, *}, numeric::*}, utils::hexadecimal::hex_to_slice}, impl_json, json::{Decoder, JsonDecode, JsonError, ObjectParser}};

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Default)]
pub struct Address([u8; 20]);

impl Address {
    pub const fn from_be_bytes(value: [u8; 20]) -> Self {
        Self(value)
    }

    pub fn to_be_bytes(&self) -> [u8; 20] {
        self.0
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|b| *b == 0)
    }
}

impl std::fmt::Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = [0; 20*2+2];
        f.write_str(fmt_hex(&mut buf, &self.0))
    }
}


impl Deref for Address {
    type Target = [u8; 20];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<[u8; 20]> for Address {
    fn from(value: [u8; 20]) -> Self {
        Self(value)
    }
}

impl<'de> JsonDecode<'de> for Address {
    fn decode_json(&mut self, buffer: &mut Decoder<'de>) -> Result<(), crate::json::JsonError> {
        let mut s = "";
        s.decode_json(buffer)?;
        let mut bytes = [0; 20];
        hex_to_slice(&mut bytes, s).map_err(|_| JsonError::ExpectedHexString)?;
        *self = Self(bytes);
        Ok(())
    }
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Default)]
pub struct Root(pub (crate)[u8; 32]);

impl From<[u8; 32]> for Root {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

impl std::fmt::Debug for Root {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut hex = [0; 32*2+2];
        fmt_hex(&mut hex, &self.0);
        write!(f, "Root({})", std::str::from_utf8(&hex).unwrap())
    }
}

impl Extended for Root {
    fn encode<'a, 'b>(&self, buffer: &'a mut Bytes) -> Result<(), RLPException> {
        Ok(encode_bytes(buffer, &self.0))
    }

    fn decode<'a, 'b>(&mut self, buffer: &'a mut &'b [u8]) -> Result<(), RLPException> {
        decode_to_bytes(buffer, &mut self.0)
    }
}

impl<'de> JsonDecode<'de> for Root {
    fn decode_json(&mut self, buffer: &mut Decoder<'de>) -> Result<(), crate::json::JsonError> {
        let mut s = "";
        s.decode_json(buffer)?;
        let mut bytes = [0; 32];
        hex_to_slice(&mut bytes, s).map_err(|_| JsonError::ExpectedHexString)?;
        *self = Self(bytes);
        Ok(())
    }
}

impl Deref for Root {
    type Target = [u8; 32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct VersionedHash(pub (crate) [u8; 32]);

impl Deref for VersionedHash {
    type Target = [u8; 32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Bloom(pub Bytes256);

impl Extended for Bloom {
    fn encode<'a, 'b>(&self, buffer: &'a mut Bytes) -> Result<(), RLPException> {
        self.0.encode(buffer)
    }

    fn decode<'a, 'b>(&mut self, buffer: &'a mut &'b [u8]) -> Result<(), RLPException> {
        self.0.decode(buffer)
    }
}

impl Deref for Bloom {
    type Target = [u8; 256];

    fn deref(&self) -> &Self::Target {
        &self.0.0
    }
}

#[derive(Default, Debug, PartialEq, Clone)]
/// State associated with an address.
pub struct Account {
    pub nonce: Uint,
    pub balance: U256,
    pub code: Bytes,
}

impl_json!(Account : nonce "nonce", balance "balance", code "code");

pub static EMPTY_ACCOUNT : Account = Account{
    nonce: 0,
    balance: U256::ZERO,
    code: Bytes(Vec::new()),
};


/// Encode `Account` dataclass.
/// 
/// Storage is not stored in the `Account` dataclass, so `Accounts` cannot be
/// encoded without providing a storage root.
pub fn encode_account(raw_account_data: &Account, storage_root: &Root) -> Result<Bytes, RLPException> {
    let mut dest = Bytes::default();
    rlp::encode_sequence(
        &mut dest,
        &[
            &raw_account_data.nonce,
            &raw_account_data.balance,
            storage_root,
            &keccak256(&raw_account_data.code),
        ]
    )?;
    Ok(dest)
}
