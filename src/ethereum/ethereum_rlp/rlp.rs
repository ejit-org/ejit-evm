//! Defines the serialization and deserialization format used throughout Ethereum.

use std::ops::{Deref, DerefMut};

use crate::ethereum::{cancun::fork_types::{Address, VersionedHash}, ethereum_types::{bytes::{Bytes, Bytes32}, numeric::{Uint, U256, U64}}};

use super::exceptions::RLPException;


pub trait Extended {
    fn encode<'a, 'b>(&self, buffer: &'a mut Bytes) -> Result<(), RLPException>;
    fn decode<'a, 'b>(&mut self, buffer: &'a mut &'b [u8]) -> Result<(), RLPException>;
}

#[macro_export]
macro_rules! impl_extended {
    ($t : ty : $($field : ident),*) => {
        impl Extended for $t {
            fn encode<'a, 'b>(&self, buffer: &'a mut Bytes) -> Result<(), RLPException> {
                encode_sequence(buffer, &[
                    $(&self.$field),*
                ])
            }
        
            fn decode<'a, 'b>(&mut self, buffer: &'a mut &'b [u8]) -> Result<(), RLPException> {
                decode_to_sequence(buffer, &mut [
                    $(&mut self.$field),*
                ])
            }
        }
                
    }
}

//
// RLP Encode
//

/// Encodes `raw_data` into a sequence of bytes using RLP.
///
pub fn encode<T : Extended>(t: &T) -> Result<Bytes, RLPException> {
    let mut res = Bytes::default();
    t.encode(&mut res)?;
    Ok(res)
}

impl Extended for String {
    fn encode<'a, 'b>(&self, buffer: &'a mut Bytes) -> Result<(), RLPException> {
        encode_bytes(buffer, self.as_bytes());
        Ok(())
    }

    fn decode<'a, 'b>(&mut self, buffer: &'a mut &'b [u8]) -> Result<(), RLPException> {
        let mut b = Bytes::default();
        b.decode(buffer)?;
        *self = String::from_utf8(b.0).map_err(|_| RLPException::DecodingError("not utf8"))?;
        Ok(())
    }
}

impl Extended for bool {
    fn encode<'a, 'b>(&self, buffer: &'a mut Bytes) -> Result<(), RLPException> {
        if *self {
            Ok(encode_bytes(buffer, b"\x01"))
        } else {
            Ok(encode_bytes(buffer, b""))
        }
    }
    
    fn decode<'a, 'b>(&mut self, buffer: &'a mut &'b [u8]) -> Result<(), RLPException> {
        let mut bytes = [0; 1];
        decode_to_bytes(buffer, &mut bytes[..])?;
        if bytes[0] > 1 {
            return Err(RLPException::DecodingError("invalid bool"));
        }
        *self = bytes[0] != 0;
        Ok(())
    }
}

impl Extended for Uint {
    fn encode<'a, 'b>(&self, buffer: &'a mut Bytes) -> Result<(), RLPException> {
        let bytes = self.to_be_bytes();
        let first_nz = bytes.iter().position(|b| *b != 0).unwrap_or(bytes.len());
        encode_bytes(buffer, &bytes[first_nz..]);
        Ok(())
    }
    
    fn decode<'a, 'b>(&mut self, buffer: &'a mut &'b [u8]) -> Result<(), RLPException> {
        let mut bytes = [0; size_of::<Self>()];
        decode_to_bytes(buffer, &mut bytes[..])?;
        *self = Self::from_be_bytes(bytes);
        Ok(())
    }
}

impl Extended for U256 {
    fn encode<'a, 'b>(&self, buffer: &'a mut Bytes) -> Result<(), RLPException> {
        let bytes = self.to_be_bytes();
        let first_nz = bytes.iter().position(|b| *b != 0).unwrap_or(bytes.len());
        encode_bytes(buffer, &bytes[first_nz..]);
        Ok(())
    }
    
    fn decode<'a, 'b>(&mut self, buffer: &'a mut &'b [u8]) -> Result<(), RLPException> {
        let mut bytes = [0; size_of::<Self>()];
        decode_to_bytes(buffer, &mut bytes[..])?;
        *self = Self::from_be_bytes(bytes);
        Ok(())
    }
}

impl Extended for Bytes32 {
    fn encode<'a, 'b>(&self, buffer: &'a mut Bytes) -> Result<(), RLPException> {
        let bytes = self.0;
        let first_nz = bytes.iter().position(|b| *b != 0).unwrap_or(bytes.len());
        encode_bytes(buffer, &bytes[first_nz..]);
        Ok(())
    }
    
    fn decode<'a, 'b>(&mut self, buffer: &'a mut &'b [u8]) -> Result<(), RLPException> {
        let mut bytes = [0; size_of::<Self>()];
        decode_to_bytes(buffer, &mut bytes[..])?;
        *self = Self(bytes);
        Ok(())
    }
}

impl Extended for Address {
    fn encode<'a, 'b>(&self, buffer: &'a mut Bytes) -> Result<(), RLPException> {
        let bytes = self.to_be_bytes();
        let first_nz = bytes.iter().position(|b| *b != 0).unwrap_or(bytes.len());
        encode_bytes(buffer, &bytes[first_nz..]);
        Ok(())
    }
    
    fn decode<'a, 'b>(&mut self, buffer: &'a mut &'b [u8]) -> Result<(), RLPException> {
        let mut bytes = [0; 20];
        decode_to_bytes(buffer, &mut bytes[..])?;
        *self = Self::from_be_bytes(bytes);
        Ok(())
    }
}

impl Extended for Bytes {
    fn encode<'a, 'b>(&self, buffer: &'a mut Bytes) -> Result<(), RLPException> {
        encode_bytes(buffer, self.deref());
        Ok(())
    }
    
    fn decode<'a, 'b>(&mut self, buffer: &'a mut &'b [u8]) -> Result<(), RLPException> {
        match decode_to_bytes(buffer, &mut []) {
            Ok(()) => {
                *self = Bytes::default();
                Ok(())
            }
            Err(RLPException::DestTooSmall(new_len)) => {
                self.0.resize(new_len, 0);
                decode_to_bytes(buffer, self.deref_mut())
            },
            Err(e) => Err(e),
        }
    }
}

impl Extended for U64 {
    fn encode<'a, 'b>(&self, buffer: &'a mut Bytes) -> Result<(), RLPException> {
        let bytes = self.to_be_bytes();
        let first_nz = bytes.iter().position(|b| *b != 0).unwrap_or(bytes.len());
        encode_bytes(buffer, &bytes[first_nz..]);
        Ok(())
    }
    
    fn decode<'a, 'b>(&mut self, buffer: &'a mut &'b [u8]) -> Result<(), RLPException> {
        let mut bytes = [0; size_of::<Self>()];
        decode_to_bytes(buffer, &mut bytes[..])?;
        *self = Self::from_be_bytes(bytes);
        Ok(())
    }
}


impl<A : Extended, B: Extended> Extended for (A, B) {
    fn encode<'a, 'b>(&self, buffer: &'a mut Bytes) -> Result<(), RLPException> {
        encode_sequence(buffer, &[&self.0, &self.1])
    }
    
    fn decode<'a, 'b>(&mut self, buffer: &'a mut &'b [u8]) -> Result<(), RLPException> {
        decode_to_sequence(buffer, &mut [&mut self.0 as &mut dyn Extended, &mut self.1 as &mut dyn Extended])
    }
}

impl<A : Extended, B: Extended, C: Extended> Extended for (A, B, C) {
    fn encode<'a, 'b>(&self, buffer: &'a mut Bytes) -> Result<(), RLPException> {
        encode_sequence(buffer, &[&self.0, &self.1, &self.2])
    }
    
    fn decode<'a, 'b>(&mut self, buffer: &'a mut &'b [u8]) -> Result<(), RLPException> {
        decode_to_sequence(buffer, &mut [
            &mut self.0 as &mut dyn Extended,
            &mut self.1 as &mut dyn Extended,
            &mut self.2 as &mut dyn Extended
        ])
    }
}

impl<T : Extended + Default> Extended for Option<T> {
    fn encode<'a, 'b>(&self, buffer: &'a mut Bytes) -> Result<(), RLPException> {
        if let Some(t) = self {
            t.encode(buffer)
        } else {
            // TODO: disallow None options before Some.
            // Maybe return a bool indicating this state.
            Ok(())
        }
    }

    fn decode<'a, 'b>(&mut self, buffer: &'a mut &'b [u8]) -> Result<(), RLPException> {
        // Optional items take zero bytes if they are None.
        // But they may only occur at the end of a structure.
        if buffer.is_empty() {
            *self = None;
            Ok(())
        } else {
            let mut t = T::default();
            t.decode(buffer)?;
            *self = Some(t);
            Ok(())
        }
    }
}

impl<T : Extended + Default + Clone> Extended for Vec<T> {
    fn encode<'a, 'b>(&self, buffer: &'a mut Bytes) -> Result<(), RLPException> {
        let refs : Vec<&dyn Extended> = self.iter().map(|e| e as &dyn Extended).collect();
        encode_sequence(buffer, &refs)
    }
    
    fn decode<'a, 'b>(&mut self, buffer: &'a mut &'b [u8]) -> Result<(), RLPException> {
    
        let mut joined_encodings = find_joined_encodings(buffer)?;
    

        let mut buffer = &mut joined_encodings;
        while !buffer.is_empty() {
            let mut t = T::default();
            t.decode(buffer)?;
            self.push(t);
        }
        Ok(())
    }
}

impl Extended for VersionedHash {
    fn encode<'a, 'b>(&self, buffer: &'a mut Bytes) -> Result<(), RLPException> {
        Ok(encode_bytes(buffer, &self.0))
    }

    fn decode<'a, 'b>(&mut self, buffer: &'a mut &'b [u8]) -> Result<(), RLPException> {
        decode_to_bytes(buffer, &mut self.0)
    }
}

/// Encodes `raw_bytes`, a sequence of bytes, using RLP.
pub fn encode_bytes(buffer: &mut Bytes, raw_bytes: &[u8]) {
    let len_raw_data = raw_bytes.len();

    if len_raw_data == 1 && raw_bytes[0] < 0x80 {
        buffer.push(raw_bytes[0]);
    } else if len_raw_data < 0x38 {
        buffer.push(0x80 + (len_raw_data as u8));
        buffer.extend(raw_bytes.iter().copied());
    } else {
        // length of raw data represented as big endian bytes
        let len_raw_data_as_be = len_raw_data.to_be_bytes();
        let lz = len_raw_data_as_be.iter()
            .position(|b| *b != 0)
            .unwrap(); // len_raw_data not zero.
        let len_raw_data_as_be = &len_raw_data_as_be[lz..];
        buffer.push(0xB7 + len_raw_data_as_be.len() as u8);
        buffer.extend(len_raw_data_as_be.iter().copied());
        buffer.extend(raw_bytes.iter().copied());
    }
}


/// Encodes a list of RLP encodable objects (`raw_sequence`) using RLP.
pub fn encode_sequence(dest: &mut Bytes, raw_sequence: &[&dyn Extended]) -> Result<(), RLPException> {
    let joined_encodings = join_encodings(raw_sequence)?;

    encode_joined_encodings(dest, joined_encodings);
    Ok(())
}

pub fn encode_joined_encodings(dest: &mut Bytes, joined_encodings: Bytes) {
    let len_joined_encodings = joined_encodings.len();
    if len_joined_encodings < 0x38 {
        dest.push(0xC0 + len_joined_encodings as u8);
    } else {
        let len_joined_encodings_as_be = len_joined_encodings.to_be_bytes();
        let lz = len_joined_encodings_as_be.iter()
            .position(|b| *b != 0)
            .unwrap(); // len_joined_encodings not zero.
        let len_joined_encodings_as_be = &len_joined_encodings_as_be[lz..];
        dest.push(0xF7 + len_joined_encodings_as_be.len() as u8);
        dest.extend(len_joined_encodings_as_be.iter().copied());
    }
    dest.extend(joined_encodings.iter().copied());
}

/// Obtain concatenation of rlp encoding for each item in the sequence
/// raw_sequence.
fn join_encodings(raw_sequence: &[&dyn Extended]) -> Result<Bytes, RLPException> {
    let mut res = Bytes::default();
    for e in raw_sequence {
        e.encode(&mut res)?;
    }
    Ok(res)
}


//
// RLP Decode
//


/// Decodes an integer, byte sequence, or list of RLP encodable objects
/// from the byte sequence `encoded_data`, using RLP.
pub fn decode_to<T : Extended  + Default>(mut encoded_data: &[u8]) -> Result<T, RLPException> {
    if encoded_data.len() <= 0 {
        return Err(RLPException::DecodingError("Cannot decode empty bytestring"));
    }
    let mut res = T::default();
    T::decode(&mut res, &mut encoded_data)?;
    if !encoded_data.is_empty() {
        return Err(RLPException::DecodingError("too short"));
    }
    Ok(res)
}

/// Decodes a rlp encoded byte stream assuming that the decoded data
/// should be of type `Sequence` of objects.
pub fn decode_to_sequence(encoded_sequence: &mut &[u8], dest: &mut [&mut dyn Extended]) -> Result<(), RLPException> {
    
    let joined_encodings = find_joined_encodings(encoded_sequence)?;

    decode_joined_encodings(joined_encodings, dest)?;
    Ok(())
}

fn find_joined_encodings<'a>(buffer: &mut &'a [u8]) -> Result<&'a [u8], RLPException> {
    if buffer.is_empty() || buffer[0] <= 0xBF {
        return Err(RLPException::DecodingError("expected sequence"));
    }
    let encoded_sequence_len = buffer.len();
    let joined_encodings = if buffer[0] <= 0xF7 {
        let len_joined_encodings = (buffer[0] - 0xC0) as usize;
        if len_joined_encodings >= encoded_sequence_len {
            return Err(RLPException::DecodingError("too long: decode_to_sequence 1"));
        }
        let res = &buffer[1..1 + len_joined_encodings];
        *buffer = &buffer[1 + len_joined_encodings..];
        res
    } else {
        let joined_encodings_start_idx = (1 + buffer[0] - 0xF7) as usize;
        if joined_encodings_start_idx - 1 >= encoded_sequence_len {
            return Err(RLPException::DecodingError("too long: decode_to_sequence 2"));
        }
        if buffer[1] == 0 {
            return Err(RLPException::DecodingError("incorrect length 1"));
        }
        let len_joined_encodings = decode_length(&buffer[1..joined_encodings_start_idx]);
        if len_joined_encodings < 0x38 {
            return Err(RLPException::DecodingError("incorrect length 2"));
        }
        let joined_encodings_end_idx = (
            joined_encodings_start_idx + len_joined_encodings
        );
        if joined_encodings_end_idx - 1 >= encoded_sequence_len {
            return Err(RLPException::DecodingError("too long: decode_to_sequence 1"));
        }
        let res = &buffer[
            joined_encodings_start_idx..joined_encodings_end_idx
        ];
        *buffer = &buffer[joined_encodings_end_idx..];
        res
    };
    Ok(joined_encodings)
}

/// Decodes `joined_encodings`, which is a concatenation of RLP encoded
/// objects.
/// 
/// Ths one is use for structs and fixed length 
fn decode_joined_encodings(mut joined_encodings: &[u8], dest: &mut [&mut dyn Extended]) -> Result<(), RLPException> {
    let mut buffer = &mut joined_encodings;
    for d in dest {
        d.decode(buffer)?;
    }
    Ok(())
}

/// Decodes a rlp encoded byte stream assuming that the decoded data
/// should be of type `bytes`.
/// 
/// This is not exactly like the original.
/// data is stored right-justified in the buffer.
/// if the source data exceeds the dest size, an error is returned.
/// 
/// It also screens out sequences.
pub fn decode_to_bytes<'d, 'a, 'b>(buffer: &'a mut &'b [u8], dest: &'d mut [u8]) -> Result<(), RLPException> {
    let dest_len = dest.len();
    dest.fill(0);
    if buffer.is_empty() || buffer[0] > 0xBF {
        return Err(RLPException::DecodingError("expected bytes, got a sequence"));
    } else if buffer[0] <= 0x80 {
        if dest_len < 1 {
            return Err(RLPException::DestTooSmall(1));
        }
        dest[dest_len-1] = buffer[0];
        *buffer = &buffer[1..];
    } else if buffer[0] <= 0xB7 {
        let len_raw_data = (buffer[0] - 0x80) as usize;
        if len_raw_data >= buffer.len() {
            return Err(RLPException::DecodingError("truncated"));
        }
        if len_raw_data > dest_len {
            return Err(RLPException::DestTooSmall(len_raw_data));
        }
        let raw_data = &buffer[1..1 + len_raw_data];
        if len_raw_data == 1 && raw_data[0] < 0x80 {
            return Err(RLPException::DecodingError("incorrect length"));
        }
        dest[dest_len-len_raw_data..].copy_from_slice(raw_data);
        *buffer = &buffer[1 + len_raw_data..];
    } else { // 0xb8..0xbf
        // This is the index in the encoded data at which decoded data
        // starts from.
        let decoded_data_start_idx = (1 + buffer[0] - 0xB7) as usize;
        if decoded_data_start_idx - 1 >= buffer.len() {
            return Err(RLPException::DecodingError("truncated"));
        }
        if buffer[1] == 0 {
            return Err(RLPException::DecodingError("incorrect length"));
        }
        let len_decoded_data = decode_length(&buffer[1..decoded_data_start_idx]);
        if len_decoded_data < 0x38 {
            return Err(RLPException::DecodingError("incorrect length"));
        }
        let decoded_data_end_idx = decoded_data_start_idx + len_decoded_data;
        if decoded_data_end_idx - 1 >= buffer.len() {
            return Err(RLPException::DecodingError("truncated"));
        }
        if len_decoded_data > dest_len {
            return Err(RLPException::DestTooSmall(len_decoded_data));
        }
        dest[dest_len-len_decoded_data..].copy_from_slice(&buffer[decoded_data_start_idx..decoded_data_end_idx]);
        *buffer = &buffer[decoded_data_end_idx..];
    }
    Ok(())
}

/// Decode a variable length slice to a usize.
fn decode_length(src: &[u8]) -> usize {
    assert!(src.len() <= size_of::<usize>());

    let mut res = [0; size_of::<usize>()];
    res[size_of::<usize>()-src.len()..].copy_from_slice(src);
    usize::from_be_bytes(res.try_into().unwrap())
}

#[cfg(test)]
mod tests;

