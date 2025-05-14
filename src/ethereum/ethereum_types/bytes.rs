use std::ops::DerefMut;

use crate::{ethereum::{ethereum_rlp::{exceptions::RLPException, rlp::{decode_to_bytes, encode_bytes, Extended}}, utils::hexadecimal::{hex_to_bytes, hex_to_slice}}, json::{Decoder, JsonDecode, JsonError}};

use super::numeric::fmt_hex;

#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Bytes0(pub [u8; 0]);

#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Bytes1(pub [u8; 1]);

#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Bytes4(pub [u8; 4]);


#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Bytes8(pub [u8; 8]);

impl std::fmt::Debug for Bytes8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = [0; 8*2+2];
        f.write_str(fmt_hex(&mut buf, &self.0))
    }
}

impl<'de> JsonDecode<'de> for Bytes8 {
    fn decode_json(&mut self, buffer: &mut Decoder<'de>) -> Result<(), crate::json::JsonError> {
        let mut s = "";
        s.decode_json(buffer)?;
        let mut bytes = [0; 8];
        hex_to_slice(&mut bytes, s).map_err(|_| JsonError::ExpectedHexString)?;
        *self = Self(bytes);
        Ok(())
    }
}


impl Extended for Bytes8 {
    fn encode<'a, 'b>(&self, buffer: &'a mut Bytes) -> Result<(), RLPException> {
        Ok(encode_bytes(buffer, &self.0))
    }

    fn decode<'a, 'b>(&mut self, buffer: &'a mut &'b [u8]) -> Result<(), RLPException> {
        decode_to_bytes(buffer, &mut self.0)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Bytes20(pub [u8; 20]);

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Bytes32(pub [u8; 32]);

impl std::fmt::Debug for Bytes32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = [0; 32*2+2];
        f.write_str(fmt_hex(&mut buf, &self.0))
    }
}

impl<'de> JsonDecode<'de> for Bytes32 {
    fn decode_json(&mut self, buffer: &mut Decoder<'de>) -> Result<(), crate::json::JsonError> {
        let mut s = "";
        s.decode_json(buffer)?;
        let mut bytes = [0; 32];
        hex_to_slice(&mut bytes, s).map_err(|_| JsonError::ExpectedHexString)?;
        *self = Self(bytes);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Bytes48(pub [u8; 48]);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Bytes64(pub [u8; 64]);

impl Default for Bytes64 {
    fn default() -> Self {
        Self([0; 64])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Bytes96(pub [u8; 96]);

impl Default for Bytes96 {
    fn default() -> Self {
        Self([0; 96])
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Bytes256(pub [u8; 256]);

impl std::fmt::Debug for Bytes256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = [0; 256*2+2];
        f.write_str(fmt_hex(&mut buf, &self.0))
    }
}

impl Extended for Bytes256 {
    fn encode<'a, 'b>(&self, buffer: &'a mut Bytes) -> Result<(), RLPException> {
        Ok(encode_bytes(buffer, &self.0))
    }

    fn decode<'a, 'b>(&mut self, buffer: &'a mut &'b [u8]) -> Result<(), RLPException> {
        decode_to_bytes(buffer, &mut self.0)
    }
}

impl Default for Bytes256 {
    fn default() -> Self {
        Self([0; 256])
    }
}

/// Sequence of bytes (octets) of arbitrary length.
#[derive(Clone, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Bytes(pub Vec<u8>);

impl<T : AsRef<[u8]>> From<T> for Bytes {
    fn from(value: T) -> Self {
        Bytes(value.as_ref().to_vec())
    }
}

impl std::fmt::Debug for Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = vec![0; self.len()*2+2];
        f.write_str(fmt_hex(&mut buf, &self.0))
    }
}

impl std::ops::Deref for Bytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

// impl AsRef<[u8]> for Bytes {
//     fn as_ref(&self) -> &[u8] {
//         &self.0
//     }
// }

impl std::ops::DerefMut for Bytes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Bytes {
    pub fn push(&mut self, value: u8) {
        self.0.push(value);
    }

    pub fn extend<T : IntoIterator<Item=u8>>(&mut self, value: T) {
        self.0.extend(value);
    }

    pub fn into_verbatim(self) -> Verbatim {
        Verbatim(self.0)
    }
}

impl<'de> JsonDecode<'de> for Bytes {
    fn decode_json(&mut self, buffer: &mut Decoder<'de>) -> Result<(), crate::json::JsonError> {
        let mut s = "";
        s.decode_json(buffer)?;
        *self = hex_to_bytes(s).map_err(|_| JsonError::ExpectedHexString)?;
        Ok(())
    }
}

#[derive(Clone)]
/// Verbatim RLP encoding.
pub struct Verbatim(pub Vec<u8>);

impl Extended for Verbatim {
    fn encode<'a, 'b>(&self, buffer: &'a mut Bytes) -> Result<(), RLPException> {
        buffer.extend(self.0.iter().copied());
        Ok(())
    }

    fn decode<'a, 'b>(&mut self, buffer: &'a mut &'b [u8]) -> Result<(), RLPException> {
        todo!();
    }
}

impl std::fmt::Debug for Verbatim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = vec![0; self.0.len()*2+2];
        f.write_str(fmt_hex(&mut buf, &self.0))
    }
}

