use core::num;

use crate::ethereum::{ethereum_types::{bytes::{Bytes, Bytes32, Bytes8}, numeric::{Uint, U256}}, exceptions::Exception};

pub fn hex_to_slice(d: &mut [u8], s: &str) -> Result<(), Exception> {
    let mut s = s.as_bytes();
    if s == b"0x0" {
        return Ok(())
    }
    if s.len() >= 2 && &s[0..2] != b"0x" {
        s = &s[2..];
    }
    // if s.len() < 2 || &s[0..2] != b"0x" {
    //     return Err(Exception::EthereumException("expected 0x"));
    // }

    fn nib(c: u8) -> Result<u8, Exception> {
        if c.is_ascii_digit() {
            Ok(c & 0x0f)
        } else if c.is_ascii_hexdigit() {
            Ok((c+9) & 0x0f)
        } else {
            Err(Exception::EthereumException("bad hex digit"))
        }
    }

    if s.len() % 2 != 0 {
        let num_bytes = (s.len()-3)/2 + 1;
        let dlen = d.len();
        if num_bytes > dlen {
            return Err(Exception::EthereumException("hex number too long"))
        }
        let d = &mut d[dlen-num_bytes..];
        d[0] = nib(s[2])?;
        for (i, c) in s[3..].chunks_exact(2).enumerate() {
            d[i+1] = nib(c[0])? * 16 + nib(c[1])?
        }
    } else {
        let num_bytes = (s.len()-2)/2;
        let dlen = d.len();
        if num_bytes > dlen {
            return Err(Exception::EthereumException("hex number too long"))
        }
        let d = &mut d[dlen-num_bytes..];
        for (i, c) in s[2..].chunks_exact(2).enumerate() {
            d[i] = nib(c[0])? * 16 + nib(c[1])?
        }
    }
    Ok(())
}

pub fn hex_to_bytes8(s: &str) -> Result<Bytes8, Exception> {
    let mut bytes = [0; 8];
    hex_to_slice(&mut bytes, s)?;
    Ok(Bytes8(bytes))
}

pub fn hex_to_bytes(s: &str) -> Result<Bytes, Exception> {
    let num_bytes = if s.len() % 2 != 0 {
        (s.len()-3)/2 + 1
    } else {
        (s.len()-2)/2
    };

    let mut bytes = vec![0; num_bytes];
    hex_to_slice(&mut bytes, s)?;
    Ok(Bytes(bytes))
}

pub fn hex_to_bytes32(s: &str) -> Result<Bytes32, Exception> {
    let mut bytes = [0; 32];
    hex_to_slice(&mut bytes, s)?;
    Ok(Bytes32(bytes))
}

pub fn hex_to_uint(s: &str) -> Result<Uint, Exception> {
    let mut bytes = [0; 16];
    hex_to_slice(&mut bytes, s)?;
    Ok(Uint::from_be_bytes(bytes))
}

pub fn hex_to_u256(s: &str) -> Result<U256, Exception> {
    let mut bytes = [0; 32];
    hex_to_slice(&mut bytes, s)?;
    Ok(U256::from_be_bytes(bytes))
}


