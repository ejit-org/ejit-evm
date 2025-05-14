use std::{ops::{Add, Div, Mul, Sub}, process::Output};

use crate::{ethereum::{exceptions::Exception, utils::hexadecimal::{self, hex_to_slice}}, json::{skip_whitespace, Decoder, JsonDecode, JsonError}};

pub type Int = i128;
pub type Uint = u128;
pub type U8 = u8;
pub type U32 = u32;
pub type U64 = u64;

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Default)]
pub struct U256([u64; 4]);

impl std::fmt::Debug for U256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = [0; 32*2+2];
        f.write_str(fmt_hex(&mut buf, &self.to_be_bytes()))
    }
}

impl U256 {
    pub const ZERO : U256 = U256([0; 4]);

    pub fn from_be_bytes(value: [u8; 32]) -> Self {
        Self::from_limbs([
            u64::from_be_bytes(value[0x00..0x08].try_into().unwrap()),
            u64::from_be_bytes(value[0x08..0x10].try_into().unwrap()),
            u64::from_be_bytes(value[0x10..0x18].try_into().unwrap()),
            u64::from_be_bytes(value[0x18..0x20].try_into().unwrap()),
        ])
    }

    pub const fn to_limbs(&self) -> [u64; 4] {
        self.0
    }

    pub fn to_be_bytes(&self) -> [u8; 32] {
        let [a, b, c, d] = self.to_limbs();
        let mut res = [0; 32];
        res[0x00..0x08].copy_from_slice(&a.to_be_bytes());
        res[0x08..0x10].copy_from_slice(&b.to_be_bytes());
        res[0x10..0x18].copy_from_slice(&c.to_be_bytes());
        res[0x18..0x20].copy_from_slice(&d.to_be_bytes());
        res
    }

    pub const fn from_limbs(value: [u64; 4]) -> Self {
        Self(value)
    }

    pub const fn from_int(value: i32) -> Self {
        let sign = if value < 0 { u64::MAX } else { 0 };
        let val = u64::from_be_bytes((value as i64).to_be_bytes());
        Self::from_limbs([sign, sign, sign, val])
    }

    pub const fn from_i128(value: i128) -> Self {
        let sign = if value < 0 { u64::MAX } else { 0 };
        let val2 = u64::from_be_bytes(((value>>64) as i64).to_be_bytes());
        let val3 = u64::from_be_bytes((value as i64).to_be_bytes());
        Self::from_limbs([sign, sign, val2, val3])
    }

    pub fn is_zero(&self) -> bool {
        (self.0[0] | self.0[1] | self.0[2] | self.0[3]) == 0
    }

    pub fn to_uint(&self) -> Result<Uint, Exception> {
        if self.0[0] != 0 || self.0[1] != 0 {
            return Err(Exception::NumericOverflow);
        }
        Ok(((self.0[2] as u128) << 64) | self.0[3] as u128)
    }

    pub fn from_uint(u: Uint) -> Self {
        Self::from_limbs([0, 0, (u >> 64) as u64, (u & 0xffffffffffffffff) as u64])
    }

    pub fn leading_zeros(&self) -> u32 {
        let a = self.to_limbs();
        if a[0] != 0 {
            return a[0].leading_zeros();
        }
        if a[1] != 0 {
            return a[1].leading_zeros() + 64;
        }
        if a[2] != 0 {
            return a[2].leading_zeros() + 64*2;
        }
        return a[3].leading_zeros() + 64*3;
    }

    pub fn shl(self, shift: u32) -> Self {
        let a = self.to_limbs();
        let sh = shift & 63;
        let nsh = shift.wrapping_neg() & 63;
        let s = if sh == 0 {
            a
        } else {
            [
                (a[0] << sh) | (a[1] >> nsh),
                (a[1] << sh) | (a[2] >> nsh),
                (a[2] << sh) | (a[3] >> nsh),
                (a[3] << sh),
            ]
        };
        Self::from_limbs(if shift >= 64*4 {
            [0, 0, 0, 0]
        } else if shift >= 64*3 {
            [s[3], 0, 0, 0]
        } else if shift >= 64*2 {
            [s[2], s[3], 0, 0]
        } else if shift >= 64*1 {
            [s[1], s[2], s[3], 0]
        } else {
            s
        })
    }

    pub fn shr(self, shift: u32) -> Self {
        let a = self.to_limbs();
        let sh = shift & 63;
        let nsh = shift.wrapping_neg() & 63;
        let s = if sh == 0 {
            a
        } else {
            [
                a[0] >> sh,
                a[0] << nsh | a[1] >> sh,
                a[1] << nsh | a[2] >> sh,
                a[2] << nsh | a[3] >> sh,
            ]
        };
        Self::from_limbs(if shift >= 64*4 {
            [0, 0, 0, 0]
        } else if shift >= 64*3 {
            [0, 0, 0, s[0]]
        } else if shift >= 64*2 {
            [0, 0, s[0], s[1]]
        } else if shift >= 64*1 {
            [0, s[0], s[1], s[2]]
        } else {
            s
        })
    }

    pub fn overflowing_div(self, rhs: Self) -> (Self, bool) {
        // TODO: use the algoritm from the Knuth book
        // and make an exception for power of two divides.
        if rhs.is_zero() {
            return (Self::ZERO, true)
        }

        let lz = self.leading_zeros();
        let mut q = Self::ZERO;
        let mut r = Self::ZERO;
        for i in (0..256-lz).rev() {
            r = r.shl(1);
            if self.bit(i) { r.set_bit(0) }
            if r >= rhs {
                r = r - rhs;
                q.set_bit(i);
            }
        }
        (q, false)
    }

    pub fn bit(&self, i: u32) -> bool {
        if i/64 >= 4 {
            false
        } else {
            let mask = 1 << i % 64;
            (self.0[3-(i/64) as usize] & mask) != 0
        }
    }

    pub fn set_bit(&mut self, i: u32) {
        if i/64 < 4 {
            let mask = 1 << i % 64;
            self.0[3-(i/64) as usize] |= mask;
        }
    }

}

impl From<i32> for U256 {
    fn from(value: i32) -> Self {
        let sign = if value < 0 { !0 } else { 0 };
        let val = u64::from_be_bytes(((value as i64) << 32 >> 32).to_be_bytes());
        Self::from_limbs([sign, sign, sign, val])
    }
}

impl From<u32> for U256 {
    fn from(value: u32) -> Self {
        Self::from_limbs([0, 0, 0, value as u64])
    }
}

impl From<u64> for U256 {
    fn from(value: u64) -> Self {
        Self::from_limbs([0, 0, 0, value])
    }
}

impl<'de> JsonDecode<'de> for U256 {
    fn decode_json(&mut self, buffer: &mut Decoder<'de>) -> Result<(), JsonError> {
        let mut s : &str = "";
        s.decode_json(buffer)?;
        *self = hexadecimal::hex_to_u256(s).map_err(|_| JsonError::ExpectedHexString)?;
        Ok(())
    }
}

impl Add<U256> for U256 {
    type Output = U256;

    fn add(self, rhs: U256) -> Self::Output {
        let ca = self.to_limbs();
        let cb = rhs.to_limbs();
        let (sum0, cy0) = ca[3].overflowing_add(cb[3]);

        let (sum1, cy1a) = ca[2].overflowing_add(cb[2]);
        let (sum1, cy1b) = sum1.overflowing_add(if cy0 { 1 } else {0} );
    
        let (sum2, cy2a) = ca[1].overflowing_add(cb[1]);
        let (sum2, cy2b) = sum2.overflowing_add(if cy1a || cy1b { 1 } else {0} );
    
        let (sum3, _cy3a) = ca[0].overflowing_add(cb[0]);
        let (sum3, _cy3b) = sum3.overflowing_add(if cy2a || cy2b { 1 } else {0} );
    
        Self::from_limbs([sum3, sum2, sum1, sum0])
    }
}

impl Sub<U256> for U256 {
    type Output = U256;

    fn sub(self, rhs: U256) -> Self::Output {
        let ca = self.to_limbs();
        let cb = rhs.to_limbs();
        let (sum0, cy0) = ca[3].overflowing_sub(cb[3]);

        let (sum1, cy1a) = ca[2].overflowing_sub(cb[2]);
        let (sum1, cy1b) = sum1.overflowing_sub(if cy0 { 1 } else {0} );
    
        let (sum2, cy2a) = ca[1].overflowing_sub(cb[1]);
        let (sum2, cy2b) = sum2.overflowing_sub(if cy1a || cy1b { 1 } else {0} );
    
        let (sum3, _cy3a) = ca[0].overflowing_sub(cb[0]);
        let (sum3, _cy3b) = sum3.overflowing_sub(if cy2a || cy2b { 1 } else {0} );
    
        Self::from_limbs([sum3, sum2, sum1, sum0])
    }
}

impl Mul<U256> for U256 {
    type Output = U256;

    fn mul(self, rhs: U256) -> Self::Output {
        let ca = self.to_limbs();
        let cb = rhs.to_limbs();
        let sum0 =
            ca[3] as u128 * cb[3] as u128
        ;

        let sum1 =
            ca[2] as u128 * cb[3] as u128 +
            ca[3] as u128 * cb[2] as u128 +
            sum0 >> 64
        ;

        let sum2 =
            ca[1] as u128 * cb[3] as u128 +
            ca[2] as u128 * cb[2] as u128 +
            ca[3] as u128 * cb[1] as u128 +
            sum1 >> 64
        ;

        let sum3 =
            ca[0] as u128 * cb[3] as u128 +
            ca[1] as u128 * cb[2] as u128 +
            ca[2] as u128 * cb[1] as u128 +
            ca[3] as u128 * cb[0] as u128 +
            sum2 >> 64
        ;

        fn t(x: u128) -> u64 {
            (x & (u64::MAX as u128)) as u64
        }
        Self::from_limbs([t(sum3), t(sum2), t(sum1), t(sum0)])
    }
}

pub fn fmt_hex<'a>(buf: &'a mut [u8], bytes: &[u8]) -> &'a str {
    assert!(buf.len() == bytes.len()*2+2);
    let lz = bytes.iter().position(|b| *b != 0).unwrap_or(bytes.len());
    let bytes = &bytes[lz..];
    if bytes.is_empty() {
        "0x0"
    } else {
        let hex = b"0123456789abcdef";
        buf[0] = b'0';
        buf[1] = b'x';
        for i in 0..bytes.len() {
            buf[i*2+2] = hex[(bytes[i] >> 4) as usize];
            buf[i*2+3] = hex[(bytes[i] & 0x0f) as usize];
        }
        std::str::from_utf8(&buf[0..bytes.len()*2+2]).unwrap()
    }
}

macro_rules! decode_int {
    ($t: ty) => {
        impl<'de> JsonDecode<'de> for $t {
            fn decode_json(&mut self, buffer: &mut Decoder<'de>) -> Result<(), JsonError> {
                skip_whitespace(buffer);
                if buffer.first() == Some(&b'"') {
                    let mut s = "";
                    s.decode_json(buffer)?;
                    let mut bytes = [0; size_of::<$t>()];
                    hex_to_slice(&mut bytes, s).map_err(|_| JsonError::ExpectedHexString)?;
                    *self = Self::from_be_bytes(bytes);
                } else {
                    if !matches!(buffer.first(), Some(x) if x.is_ascii_digit()) {
                        return Err(JsonError::ExpectedDigit);
                    }
                    let mut value : $t = (buffer[0] - b'0').into();
                    buffer.advance(1);
                    while matches!(buffer.first(), Some(x) if x.is_ascii_digit()) {
                        let (v, e1) = value.overflowing_mul(10);
                        let (v, e2) = v.overflowing_add((buffer[0] - b'0').into());
                        if e1 || e2 {
                            return Err(JsonError::NumericOverflow);
                        }
                        value = v;
                        buffer.advance(1);
                    }
                    *self = value;
                }
                Ok(())
            }
        }
                
    };
}

decode_int!(Uint);
decode_int!(U64);
decode_int!(U32);
decode_int!(U8);


#[test]
fn test_u256() {
    /// TODO: Test much, much more, especially with edge cases and random numbers.
    assert_eq!(U256::from_int(-0x80000000).to_limbs(), [!0, !0, !0, !0-0x80000000+1]);

    assert_eq!(U256::from_int(-1) + U256::from_int(1), U256::from_int(0));
    assert_eq!(U256::from_int(1) + U256::from_int(-1), U256::from_int(0));
    assert_eq!(U256::from_int(-0x7fffffff) + U256::from_int(0x7fffffff), U256::from_int(0));

    assert_eq!(U256::from_int(123456) * U256::from_int(7891011), U256::from_i128(123456*7891011));

    for i in 0..128 {
        // println!("{i} {:?}", U256::from_int(1).shl(i));
        assert_eq!(U256::from_int(1).shl(i).shr(i), U256::from_i128(1));
    }

    assert_eq!(U256::from_int(123456).overflowing_div(U256::from_int(100)), (U256::from_i128(1234), false));

    let json = r#""0x123""#;
    let mut value = U256::default();
    value.decode_json(&mut Decoder::new(json.as_bytes())).unwrap();
    assert_eq!(value, U256::from(0x123));

    let json = r#""0x1234""#;
    let mut value = U256::default();
    value.decode_json(&mut Decoder::new(json.as_bytes())).unwrap();
    assert_eq!(value, U256::from(0x1234));

    let json = r#""0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff""#;
    let mut value = U256::default();
    value.decode_json(&mut Decoder::new(json.as_bytes())).unwrap();
    assert_eq!(value, U256::from(-1));

    let json = r#""0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0""#;
    let mut value = U256::default();
    assert!(value.decode_json(&mut Decoder::new(json.as_bytes())).is_err());
}

