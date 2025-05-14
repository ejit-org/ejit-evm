//! 
//! Very simple JSON deserialiser
//!
//! See https://www.json.org/json-en.html 

use std::{collections::BTreeMap, io::Write, ops::Deref};

use crate::ethereum::{ethereum_types::numeric::{Uint, U64}, utils::hexadecimal::hex_to_slice};

#[derive(Debug)]
pub enum JsonError {
    UnexpectedEof,
    UnexpectedChar,
    UnterminatedString,
    Expected(char),
    MissingKey,
    ExpectedDigit,
    ExpectedIdentifier,
    ExpectedBool,
    NumericOverflow,
    BadString,
    ExpectedHexString,
    BadNumber,
}

#[derive(Debug)]
pub struct Decoder<'de> {
    buffer: &'de [u8],
    start: * const u8,
    len: usize,
}

pub struct Context {
    text: String,
}

pub enum Value {
    String(Box<str>),
    Numeric(Box<str>),
    Bool(bool),
    Null,
    Array(Box<[Value]>),
    Map(Box<[(Box<str>, Value)]>),
}

impl<'de> From<&Decoder<'de>> for Context {
    fn from(d: &Decoder<'de>) -> Self {
        unsafe {
            let pos = d.buffer.as_ptr().byte_offset_from(d.start);
            let pos = (pos.max(0) as usize).min(d.len);
            let all = std::slice::from_raw_parts(d.start, d.len);
            let range = &all[pos.saturating_sub(10)..(pos+10).min(d.len)];
            let text = std::str::from_utf8_unchecked(range).to_string();
            Context { text }
        }
    }
}


impl<'de> Decoder<'de> {
    pub fn new(buffer: &'de [u8]) -> Self {
        Self { buffer, start: buffer.as_ptr(), len: buffer.len() }
    }
    
    pub fn advance(&mut self, n: usize) -> &'de [u8] {
        let bytes = &self.buffer[0..n];
        self.buffer = &self.buffer[n..];
        bytes
    }
    
    pub fn cur(&self) -> &'de [u8] {
        self.buffer
    }
}

impl<'de> std::ops::Deref for Decoder<'de> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

pub trait JsonDecode<'de> : where Self : 'de {
    fn decode_json(&mut self, decoder: &mut Decoder<'de>) -> Result<(), JsonError>;
}

#[macro_export]
macro_rules! impl_json {
    ($t : ty : $f1 : ident $n1 : expr) => {
        impl<'de> JsonDecode<'de> for $t {
            fn decode_json(&mut self, decoder: &mut Decoder<'de>) -> Result<(), JsonError> {
                let mut p = ObjectParser::new(decoder);
                p.decode_one(&mut self.$f1, $n1)
            }
        }
    };
    ($t : ty : $f1 : ident $n1 : expr, $f2 : ident $n2 : expr) => {
        impl<'de> JsonDecode<'de> for $t {
            fn decode_json(&mut self, decoder: &mut Decoder<'de>) -> Result<(), JsonError> {
                let mut p = ObjectParser::new(decoder);
                p.decode_two(&mut self.$f1, $n1, &mut self.$f2, $n2)
            }
        }
    };
    ($t : ty : $f1 : ident $n1 : expr, $f2 : ident $n2 : expr, $f3 : ident $n3 : expr) => {
        impl<'de> JsonDecode<'de> for $t {
            fn decode_json(&mut self, decoder: &mut Decoder<'de>) -> Result<(), JsonError> {
                let mut p = ObjectParser::new(decoder);
                p.decode_three(&mut self.$f1, $n1, &mut self.$f2, $n2, &mut self.$f3, $n3)
            }
        }
    };
    ($t : ty : $($field : ident $name : expr),*) => {
        impl<'de> JsonDecode<'de> for $t {
            fn decode_json(&mut self, decoder: &mut Decoder<'de>) -> Result<(), JsonError> {
                let mut p = ObjectParser::new(decoder);
                loop {
                    match p.next_key()? {
                        $(Some(k) if k == $name => self.$field.decode_json(p.decoder)?,)*
                        None => return Ok(()),
                        _ => return Err(crate::json::JsonError::MissingKey),
                    };
                }
            }
        }
    };
}

impl<'de> JsonDecode<'de> for &'de str {
    fn decode_json(&mut self, decoder: &mut Decoder<'de>) -> Result<(), JsonError> {
        let mut s = parse_string(decoder)?;
        *self = std::str::from_utf8(s)
            .map_err(|_| JsonError::BadString)?;
        Ok(())
    }
}

impl<'de, T : JsonDecode<'de> + Default> JsonDecode<'de> for Vec<T> {
    fn decode_json(&mut self, decoder: &mut Decoder<'de>) -> Result<(), JsonError> {
        skip_whitespace(decoder);
        expect(decoder, b'[')?;
        skip_whitespace(decoder);
        if decoder.first() != Some(&b']') {
            loop {
                let mut t : T = Default::default();
                t.decode_json(decoder)?;
                self.push(t);
                skip_whitespace(decoder);
                if decoder.first() == Some(&b']') {
                    break
                };
                expect(decoder, b',')?;
            }
        }
        decoder.advance(1);
        Ok(())
    }
}

impl<'de, K : JsonDecode<'de> + Default + Ord, V : JsonDecode<'de> + Default> JsonDecode<'de> for BTreeMap<K, V> {
    fn decode_json(&mut self, decoder: &mut Decoder<'de>) -> Result<(), JsonError> {
        let mut p = ObjectParser::new(decoder);
        loop {
            let Some(key) = p.next_map_key::<K>()? else { break; };
            let mut value = V::default();
            value.decode_json(p.decoder)?;
            self.insert(key, value);
        }
        Ok(())
    }
}

impl<'de> JsonDecode<'de> for bool {
    fn decode_json(&mut self, decoder: &mut Decoder<'de>) -> Result<(), JsonError> {
        skip_whitespace(decoder);
        *self = match parse_indent(decoder)? {
            b"true" => true,
            b"false" => false,
            _ => return Err(JsonError::ExpectedBool),
        };
        Ok(())
    }
}

impl<'de> JsonDecode<'de> for String {
    fn decode_json(&mut self, decoder: &mut Decoder<'de>) -> Result<(), JsonError> {
        let mut s = parse_string(decoder)?;
        if s.iter().any(|b| b.is_ascii_control()) {
            return Err(JsonError::BadString);
        }
        if !s.contains(&b'\\') {
            *self = String::from_utf8(s.to_vec())
                .map_err(|_| JsonError::BadString)?;
        } else {
            let mut i = 0;
            *self = String::with_capacity(s.len());
            while i != s.len() {
                if s[i] != b'\\' {
                    self.push(s[i].into());
                    i += 1;
                } else {
                    match s[i+1] {
                        b'"' => self.push('"'),
                        b'\\' => self.push('\\'),
                        b'b' => self.push('\x08'),
                        b'f' => self.push('\x0c'),
                        b'n' => self.push('\n'),
                        b'r' => self.push('\r'),
                        b'u' => {
                            if i + 6 > s.len() {
                                return Err(JsonError::BadString);
                            }
                            let x = std::str::from_utf8(&s[i+2..i+6])
                                .map_err(|_| JsonError::BadString)?;
                            let c = u32::from_str_radix(x, 16)
                                .map_err(|_| JsonError::BadString)?
                                .try_into()
                                .map_err(|_| JsonError::BadString)?;
                            self.push(c);
                            i += 4;
                        }
                        _ => return Err(JsonError::BadString),
                    }
                    i += 2;
                }
            }
        }
        Ok(())
    }
}

impl<'de> JsonDecode<'de> for Value {
    fn decode_json(&mut self, decoder: &mut Decoder<'de>) -> Result<(), JsonError> {
        skip_whitespace(decoder);
        match decoder.first() {
            Some(b'"') => {
                let mut s = String::new();
                s.decode_json(decoder)?;
                *self = Value::String(s.into());
                Ok(())
            }
            Some(c) if c.is_ascii_digit() || *c == b'-' => {
                let mut s = String::new();
                s.decode_json(decoder)?;
                *self = Value::String(s.into());
                Ok(())
            }
            Some(x) if x.is_ascii_alphabetic() => {
                let ident = parse_indent(decoder)?;
                match ident {
                    b"true" => { *self = Value::Bool(true); Ok(()) }
                    b"false" => { *self = Value::Bool(false); Ok(()) }
                    b"null" => { *self = Value::Null; Ok(()) }
                    _ => Err(JsonError::UnexpectedChar),
                }
            }
            Some(b'[') => {
                decoder.advance(1);
                skip_whitespace(decoder);
                let mut array = Vec::new();
                if decoder.first() != Some(&b']') {
                    loop {
                        let mut t = Value::Null;
                        t.decode_json(decoder)?;
                        array.push(t);
                        skip_whitespace(decoder);
                        if decoder.first() == Some(&b']') {
                            break
                        };
                        expect(decoder, b',')?;
                    }
                }
                decoder.advance(1);
                *self = Value::Array(array.into());
                Ok(())
            }
            Some(b'{') => {
                let mut p = ObjectParser::new(decoder);
                let mut map = Vec::new();
                loop {
                    match p.next_key()? {
                        Some(k) => {
                            let mut v = Value::Null;
                            v.decode_json(p.decoder)?;
                            map.push((k.into(), v));
                        }
                        None => break,
                    }
                }
                *self = Value::Map(map.into());
                Ok(())
            }
            Some(_) => Err(JsonError::UnexpectedChar),
            None => Err(JsonError::UnexpectedEof),
        }
    }
}


pub fn skip_whitespace<'de>(decoder: &mut Decoder<'de>) {
    while decoder.first().map(u8::is_ascii_whitespace) == Some(true) {
        decoder.advance(1);
    }
}

pub fn expect<'de>(decoder: &mut Decoder<'de>, chr: u8) -> Result<(), JsonError> {
    skip_whitespace(decoder);
    if !decoder.first().is_some_and(|c| *c == chr) { return Err(JsonError::Expected(chr.into())); };
    decoder.advance(1);
    Ok(())
}

pub fn parse_string<'de>(decoder: &mut Decoder<'de>) -> Result<&'de [u8], JsonError> {
    skip_whitespace(decoder);
    match decoder.first() {
        Some(b'"') => {
            if let Some(nbytes) = decoder.windows(2).position(|w| w[1] == b'"' && w[0] != b'\\') {
                let res = &decoder.cur()[1..nbytes+1];
                decoder.advance(nbytes+2);
                Ok(res)
            } else {
                Err(JsonError::UnterminatedString)
            }
        }
        Some(_) => Err(JsonError::UnexpectedChar),
        None => Err(JsonError::UnexpectedEof),
    }
}

pub fn parse_number<'de>(decoder: &mut Decoder<'de>) -> Result<&'de [u8], JsonError> {
    skip_whitespace(decoder);
    let res = decoder.cur();
    if decoder.first() == Some(&b'-') {
        decoder.advance(1);
    }
    let mut ok = false;
    while matches!(decoder.first(), Some(b) if b.is_ascii_digit()) {
        decoder.advance(1);
        ok = true;
    }
    if !ok { return Err(JsonError::BadNumber); }

    if decoder.first() == Some(&b'.') {
        let mut ok = false;
        while matches!(decoder.first(), Some(b) if b.is_ascii_digit()) {
            decoder.advance(1);
        }
        ok = true;
        if !ok { return Err(JsonError::BadNumber); }
    }

    if decoder.first() == Some(&b'e') || decoder.first() == Some(&b'E') {
        decoder.advance(1);
        if decoder.first() == Some(&b'+') || decoder.first() == Some(&b'-') {
            decoder.advance(1);
        }
        let mut ok = false;
        while matches!(decoder.first(), Some(b) if b.is_ascii_digit()) {
            decoder.advance(1);
        }
        ok = true;
        if !ok { return Err(JsonError::BadNumber); }
    }
    Ok(res)
}

pub fn parse_indent<'de>(decoder: &mut Decoder<'de>) -> Result<&'de [u8], JsonError> {
    skip_whitespace(decoder);
    let start = decoder.cur();
    if !matches!(decoder.first(), Some(x) if x.is_ascii_alphabetic()) {
        return Err(JsonError::ExpectedIdentifier);
    }
    decoder.advance(1);
    let mut n = 1;
    while matches!(decoder.first(), Some(x) if x.is_ascii_alphabetic()) {
        decoder.advance(1);
        n += 1;
    }
    Ok(&start[0..n])
}

pub fn decode_object<'de>(dest: &mut [(&mut dyn JsonDecode<'de>, &str)], decoder: &mut Decoder<'de>) -> Result<(), JsonError> {
    expect(decoder, b'{')?;
    if let Some(b'}') = decoder.first() {
        decoder.advance(1);
        return Ok(());
    }
    loop {
        let key = parse_string(decoder)?;

        println!("{key:02x?} {decoder:02x?}");
        expect(decoder, b':')?;

        let Some((obj, _)) = dest.iter_mut().find(|(_, k)| k.as_bytes() == key) else {
            return Err(JsonError::MissingKey);
        };

        obj.decode_json(decoder)?;

        skip_whitespace(decoder);

        match decoder.first() {
            Some(b'}') => { decoder.advance(1); break; }
            Some(b',') => { decoder.advance(1); }
            Some(_) => return Err(JsonError::UnexpectedChar),
            None => return Err(JsonError::UnexpectedEof),
        }
    }

    skip_whitespace(decoder);
    Ok(())
}

pub struct ObjectParser<'b, 'de> {
    pub decoder: &'b mut Decoder<'de>,
    started: bool,
}

impl<'b, 'de> ObjectParser<'b, 'de> {
    pub fn new(decoder: &'b mut Decoder<'de>) -> Self {
        Self { decoder, started: false }
    }

    pub fn next_key(&mut self) -> Result<Option<&'de str>, JsonError> {
        if !self.started {
            expect(self.decoder, b'{')?;
            if expect(self.decoder, b'}').is_ok() {
                return Ok(None)
            }
            self.started = true;
        } else {
            if expect(self.decoder, b'}').is_ok() {
                return Ok(None);
            } else {
                expect(self.decoder, b',')?;
            }
        }
        let mut key = "";
        key.decode_json(self.decoder)?;
        expect(self.decoder, b':')?;
        return Ok(Some(key));
    }

    /// When decoding maps, we do accept non-strings as keys.
    /// 
    /// Also many types have string encodings.
    pub fn next_map_key<T : JsonDecode<'de> + Default>(&mut self) -> Result<Option<T>, JsonError> {
        if !self.started {
            expect(self.decoder, b'{')?;
            if expect(self.decoder, b'}').is_ok() {
                return Ok(None)
            }
            self.started = true;
        } else {
            if expect(self.decoder, b'}').is_ok() {
                return Ok(None);
            } else {
                expect(self.decoder, b',')?;
            }
        }
        let mut key = T::default();
        key.decode_json(self.decoder)?;
        expect(self.decoder, b':')?;
        return Ok(Some(key));
    }

    pub fn decode_one(&mut self, a: &mut dyn JsonDecode<'de>, ka: &str) -> Result<(), JsonError> {
        // Note that even with one target, the JSON may repeat the key.
        loop {
            match self.next_key()? {
                Some(k) if k == ka => a.decode_json(self.decoder)?,
                None => return Ok(()),
                _ => return Err(crate::json::JsonError::MissingKey),
            }
        }
    }

    pub fn decode_two(&mut self, a: &mut dyn JsonDecode<'de>, ka: &str, b: &mut dyn JsonDecode<'de>, kb: &str) -> Result<(), JsonError> {
        loop {
            match self.next_key()? {
                Some(k) if k == ka => a.decode_json(self.decoder)?,
                Some(k) if k == kb => b.decode_json(self.decoder)?,
                None => return Ok(()),
                _ => return Err(crate::json::JsonError::MissingKey),
            }
        }
    }

    pub fn decode_three(&mut self, a: &mut dyn JsonDecode<'de>, ka: &str, b: &mut dyn JsonDecode<'de>, kb: &str, c: &mut dyn JsonDecode<'de>, kc: &str) -> Result<(), JsonError> {
        loop {
            match self.next_key()? {
                Some(k) if k == ka => a.decode_json(self.decoder)?,
                Some(k) if k == kb => b.decode_json(self.decoder)?,
                Some(k) if k == kc => c.decode_json(self.decoder)?,
                None => return Ok(()),
                _ => return Err(crate::json::JsonError::MissingKey),
            }
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::json::{decode_object, expect, skip_whitespace, Decoder, ObjectParser};

    use super::{JsonDecode, JsonError};

    #[test]
    fn test_bool() {
        let mut cursor = Decoder::new(b"true".as_slice());
        let mut b = false;
        b.decode_json(&mut cursor).unwrap();
        assert_eq!(b, true);
        assert!(cursor.is_empty());

        let mut cursor = Decoder::new(b"false".as_slice());
        let mut b = false;
        b.decode_json(&mut cursor).unwrap();
        assert_eq!(b, false);
        assert!(cursor.is_empty());

        let mut cursor = Decoder::new(b"null".as_slice());
        let mut b = false;
        assert!(b.decode_json(&mut cursor).is_err());
    }

    #[test]
    fn test_int() {
        let mut cursor = Decoder::new(b"1234".as_slice());
        let mut b : u128 = 0;
        b.decode_json(&mut cursor).unwrap();
        assert_eq!(b, 1234);
        assert!(cursor.cur().is_empty());

        let mut cursor = Decoder::new(b"340282366920938463463374607431768211455".as_slice());
        let mut b : u128 = 0;
        b.decode_json(&mut cursor).unwrap();
        assert_eq!(b, 340282366920938463463374607431768211455);
        assert!(cursor.cur().is_empty());

        let mut cursor = Decoder::new(b"340282366920938463463374607431768211456".as_slice());
        let mut b : u128 = 0;
        assert!(b.decode_json(&mut cursor).is_err());
    }

    // #[test]
    // fn test_struct() {
    //     #[derive(Debug, PartialEq, Default)]
    //     struct ABC {
    //         a: u128,
    //         b: bool,
    //         c: u128,
    //     }

    //     // impl_json!(ABC : a "a", b "b", c "c");
    //     impl_json!(ABC : a "a");

    //     // impl<'de> JsonDecode<'de> for ABC {
    //     //     fn decode_json(&mut self, decoder: &mut Decoder<'de>) -> Result<(), super::JsonError> {
    //     //         let mut p = ObjectParser::new(decoder);
    //     //         p.decode_three(&mut self.a, "a", &mut self.b, "b", &mut self.c, "c")
    //     //     }
    //     // }

    //     let mut cursor = b"{}".as_slice();
    //     let mut b : ABC = Default::default();
    //     b.decode_json(&mut Decoder::new(cursor)).unwrap();
    //     assert_eq!(b, ABC{..Default::default()});

    //     let mut cursor = br#"{"a":1,"b":true,"c":2}"#.as_slice();
    //     let mut b : ABC = Default::default();
    //     b.decode_json(&mut Decoder::new(cursor)).unwrap();
    //     assert_eq!(b, ABC{a:1, b:true, c:2});

    // }

    #[test]
    fn test_string() {
        let mut cursor = br#""abc""#.as_slice();
        let mut b : String = Default::default();
        b.decode_json(&mut Decoder::new(cursor)).unwrap();
        assert_eq!(b, "abc");
        let mut cursor = br#""abc\\\"\b\f\n\r\u4f60def""#.as_slice();
        let mut b : String = Default::default();
        b.decode_json(&mut Decoder::new(cursor)).unwrap();
        assert_eq!(b, "abc\\\"\u{8}\u{c}\n\r你def");
    }
}
