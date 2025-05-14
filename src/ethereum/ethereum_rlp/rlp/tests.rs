use std::ops::Deref;

use crate::ethereum::{ethereum_rlp::rlp::encode_sequence, ethereum_types::{bytes::Bytes, numeric::Uint}};

use super::Extended;

#[test]
fn basic_rlp() {
    {
        let mut buffer = Bytes::default();
        Bytes::from("dog".as_bytes()).encode(&mut buffer).unwrap();
        assert_eq!(buffer.deref(), &[0x83, b'd', b'o', b'g']);
    }
    {
        let mut buffer = Bytes::default();
        let val = ["cat", "dog"].map(|f| Bytes::from(f.as_bytes())).to_vec();
        val.encode(&mut buffer).unwrap();
        assert_eq!(buffer.deref(), [ 0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g' ]);
    }
    {
        let mut buffer = Bytes::default();
        Bytes::from("".as_bytes()).encode(&mut buffer).unwrap();
        assert_eq!(buffer.deref(), &[0x80]);
    }
    {
        let mut buffer = Bytes::default();
        Vec::<Uint>::new().encode(&mut buffer).unwrap();
        assert_eq!(buffer.deref(), &[0xc0]);
    }
    {
        let mut buffer = Bytes::default();
        Uint::from(0_u32).encode(&mut buffer).unwrap();
        assert_eq!(buffer.deref(), &[0x80]);
    }
    {
        let mut buffer = Bytes::default();
        Uint::from(15_u32).encode(&mut buffer).unwrap();
        assert_eq!(buffer.deref(), &[0x0f]);
    }
    {
        let mut buffer = Bytes::default();
        Uint::from(1024_u32).encode(&mut buffer).unwrap();
        assert_eq!(buffer.deref(), &[0x82, 0x04, 0x00]);
    }
    {
        let mut buffer = Bytes::default();
        let a = Vec::<Uint>::new();
        let b = vec![a.clone()];
        let c = (a.clone(), b.clone());
        (a, b, c).encode(&mut buffer).unwrap();
        assert_eq!(buffer.deref(), &[ 0xc7, 0xc0, 0xc1, 0xc0, 0xc3, 0xc0, 0xc1, 0xc0 ]);
    }
    {
        let mut buffer = Bytes::default();
        Bytes::from("Lorem ipsum dolor sit amet, consectetur adipisicing elit".as_bytes()).encode(&mut buffer).unwrap();
        assert_eq!(&buffer.deref()[0..2], &[0xb8, 0x38]);
    }
}
