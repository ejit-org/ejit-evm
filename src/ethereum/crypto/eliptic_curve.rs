//! Elliptic Curves
//! ^^^^^^^^^^^^^^^

use std::{fmt::Debug, ops::{Add, Div, Mul, Sub}};

use crate::ethereum::ethereum_types::{bytes::Bytes, numeric::U256};

use super::hash::Hash32;


pub const SECP256K1N : U256 = U256::from_limbs([0xFFFFFFFFFFFFFFFF,0xFFFFFFFFFFFFFFFE,0xBAAEDCE6AF48A03B,0xBFD25E8CD0364141]);

pub trait Field : Copy + Debug + PartialEq + Add<Self, Output=Self> + Sub<Self, Output=Self> + Mul<Self, Output=Self> + Div<Self, Output=Self> + Sized {
    fn zero() -> Self;
    fn is_zero(&self) -> bool;
    fn from_int(i: u32) -> Self;
}

pub trait EllipticCurveAB<F> {
    fn a() -> F;
    fn b() -> F;
}

/// Recovers the public key from a given signature.
/// 
/// Parameters
/// ----------
/// r :
///     TODO
/// s :
///     TODO
/// v :
///     TODO
/// msg_hash :
///     Hash of the message being recovered.
/// 
/// Returns
/// -------
/// public_key : `ethereum.base_types.Bytes`
///     Recovered public key.
pub fn secp256k1_recover(r: U256, s: U256, v: U256, msg_hash: Hash32) -> Bytes {
    let r_bytes = r.to_be_bytes();
    let s_bytes = s.to_be_bytes();

    let mut signature = [0; 65];
    signature[32 - r_bytes.len() .. 32].copy_from_slice(&r_bytes[..]);
    signature[64 - s_bytes.len() .. 64].copy_from_slice(&s_bytes[..]);
    signature[64] = v.to_be_bytes()[31];
    todo!("this is a horrible dependency");
    // let public_key = coincurve::PublicKey::from_signature_and_message(
    //     &signature, msg_hash, None
    // );
    //public_key = public_key.format(compressed=False)[1:];
    // return public_key;
    Bytes::default()
}


// /// Superclass for integers modulo a prime. Not intended to be used
// /// directly, but rather to be subclassed.
// #[derive(Debug, PartialEq)]
// pub struct EllipticCurve<F : Field> {
//     x: F,
//     y: F,
// }

// pub struct ValueError(&'static str);

// impl<F : Field> EllipticCurve<F> where Self : EllipticCurveAB<F> {

//     /// Make new point on the curve. The point is not checked to see if it is
//     /// on the curve.
//     pub fn new(x: F, y: F) -> Self {
//         Self { x, y }
//     }

//     /// Checks if the point is on the curve. To skip this check call
//     /// [new] directly.
//     pub fn init(x: F, y: F) -> Result<Self, ValueError> {
//         if (!x.is_zero() || !y.is_zero()) && y * y - x * x * x - Self::a() * x - Self::b() != F::zero() {
//             return Err(ValueError("Point not on curve"));
//         }
//         Ok(Self::new(x, y))
//     }

//     /// Return the point at infinity. This is the identity element of the group
//     /// operation.
//     /// 
//     /// The point at infinity doesn't actually have coordinates so we use
//     /// `(0, 0)` (which isn't on the curve) to represent it.
//     pub fn point_at_infinity() -> Self {
//         Self::new(F::zero(), F::zero())
//     }

//     /// Add a point to itself.
//     pub fn double(self) -> Self {
//         let (x, y) = (self.x, self.y);
//         if x.is_zero() && y.is_zero() {
//             return Self::new(x, y);
//         }
//         let lam = (F::from_int(3) * x * x + Self::a()) / (F::from_int(2) * y);
//         let new_x = lam + lam - x - x;
//         let new_y = lam * (x - new_x) - y;
//         Self::new(new_x, new_y)
//     }

//     /// Multiply `self` by `n` using the double and add algorithm.
//     pub fn mul_by(self, mut n: u64) -> Self {
//         let mut res = Self::new(F::zero(), F::zero());
//         let mut s = self;
//         while n != 0 {
//             if n % 2 == 1 {
//                 res = res + s;
//             }
//             s = s.double();
//             n /= 2;
//         }
//         return res
//     }

// }

// impl<F : Field> std::ops::Add for EllipticCurve<F> {
//     type Output = Self;

//     /// Add two points together.
//     fn add(self, other: Self) -> Self::Output {
//         let (self_x, self_y, other_x, other_y) = (self.x, self.y, other.x, other.y);
//         if self_x.is_zero() && self_y.is_zero() {
//             return other;
//         }
//         if other_x.is_zero() && other_y.is_zero() {
//             return self
//         }
//         if self_x == other_x {
//             if self_y == other_y {
//                 return self.double()
//             } else {
//                 return self.point_at_infinity()
//             }
//         }
//         lam = (other_y - self_y) / (other_x - self_x)
//         x = lam**2 - self_x - other_x
//         y = lam * (self_x - x) - self_y
//         return self.__new__(type(self), x, y)
//     }
// }
