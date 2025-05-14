//! Transactions are atomic units of work created externally to Ethereum and
//! submitted to be executed. If Ethereum is viewed as a state machine,
//! transactions are the events that move between states.

use crate::{ethereum::{cancun::{execptions::TransactionTypeError, fork_types::{Address, VersionedHash}}, crypto::{eliptic_curve::{secp256k1_recover, SECP256K1N}, hash::{keccak256, Hash32}}, ethereum_rlp::{exceptions::RLPException, rlp::{self, decode_to_sequence, encode_sequence, Extended}}, ethereum_types::{bytes::{Bytes, Bytes0, Bytes32}, numeric::{Uint, U256, U64}}, exceptions::Exception}, impl_extended};

use super::vm::{gas::init_code_cost, interpreter::MAX_CODE_SIZE};

// TODO: KILLME
#[derive(Debug, Clone, PartialEq)]
pub enum Either<A : std::fmt::Debug+Clone, B : std::fmt::Debug+Clone> {
    A(A),
    B(B),
}

const TX_BASE_COST : Uint = 21000;
const TX_DATA_COST_PER_NON_ZERO : Uint = 16;
const TX_DATA_COST_PER_ZERO : Uint = 4;
const TX_CREATE_COST : Uint = 32000;
const TX_ACCESS_LIST_ADDRESS_COST : Uint = 2400;
const TX_ACCESS_LIST_STORAGE_KEY_COST : Uint = 1900;

#[derive(Debug, Clone, Default)]
/// Atomic operation performed on the block chain.
pub struct LegacyTransaction {
    pub nonce: U256,
    pub gas_price: Uint,
    pub gas: Uint,
    pub to: Option<Address>,
    pub value: U256,
    pub data: Bytes,
    pub v: U256,
    pub r: U256,
    pub s: U256,
}

impl_extended!(LegacyTransaction : nonce, gas_price, gas, to, value, data, v, r, s);

/// The transaction type added in EIP-2930 to support access lists.
#[derive(Debug, Clone, Default)]
pub struct AccessListTransaction {
    pub chain_id: U64,
    pub nonce: U256,
    pub gas_price: Uint,
    pub gas: Uint,
    pub to: Option<Address>,
    pub value: U256,
    pub data: Bytes,
    pub access_list: Vec<(Address, Vec<Bytes32>)>,
    pub y_parity: U256,
    pub r: U256,
    pub s: U256,
}

impl_extended!(AccessListTransaction : chain_id, nonce, gas_price, gas, to, value, data, access_list, y_parity, r, s);



/// The transaction type added in EIP-1559.
#[derive(Debug, Clone, Default)]
pub struct FeeMarketTransaction {
    pub chain_id: U64,
    pub nonce: U256,
    pub max_priority_fee_per_gas: Uint,
    pub max_fee_per_gas: Uint,
    pub gas: Uint,
    pub to: Option<Address>,
    pub value: U256,
    pub data: Bytes,
    pub access_list: Vec<(Address, Vec<Bytes32>)>,
    pub y_parity: U256,
    pub r: U256,
    pub s: U256,
}

impl_extended!(FeeMarketTransaction : chain_id, nonce, max_priority_fee_per_gas, max_fee_per_gas, gas, to, value, data, access_list, y_parity, r, s);

/// The transaction type added in EIP-4844.
#[derive(Debug, Clone, Default)]
pub struct BlobTransaction {
    pub chain_id: U64,
    pub nonce: U256,
    pub max_priority_fee_per_gas: Uint,
    pub max_fee_per_gas: Uint,
    pub gas: Uint,
    pub to: Address,
    pub value: U256,
    pub data: Bytes,
    pub access_list: Vec<(Address, Vec<Bytes32>)>,
    pub max_fee_per_blob_gas: U256,
    pub blob_versioned_hashes: Vec<VersionedHash>,
    pub y_parity: U256,
    pub r: U256,
    pub s: U256,
}

impl_extended!(BlobTransaction : chain_id, nonce, max_priority_fee_per_gas, max_fee_per_gas, gas, to, value, data, access_list, max_fee_per_blob_gas, blob_versioned_hashes, y_parity, r, s);

#[derive(Debug, Clone)]
pub enum Transaction {
    LegacyTransaction(LegacyTransaction),
    AccessListTransaction(AccessListTransaction),
    FeeMarketTransaction(FeeMarketTransaction),
    BlobTransaction(BlobTransaction),
}

impl Default for Transaction {
    fn default() -> Self {
        Self::LegacyTransaction(Default::default())
    }
}

impl Extended for Transaction {
    fn encode<'a, 'b>(&self, buffer: &'a mut Bytes) -> Result<(), RLPException> {
        todo!()
    }

    fn decode<'a, 'b>(&mut self, buffer: &'a mut &'b [u8]) -> Result<(), RLPException> {
        if buffer.len() > 0 && buffer[0] >= 0xc0 {
            let mut t = LegacyTransaction::default();
            t.decode(buffer)?;
            *self = Self::LegacyTransaction(t);
        } else {
            let mut bytes = Bytes::default();
            bytes.decode(buffer)?;
            if bytes.is_empty() {
                return Err(RLPException::DecodingError("empty transaction"));
            }
            match bytes[0] {
                0x01 => *self = Transaction::AccessListTransaction(rlp::decode_to::<AccessListTransaction>(&bytes[1..])?),
                0x02 => *self = Transaction::FeeMarketTransaction(rlp::decode_to::<FeeMarketTransaction>(&bytes[1..])?),
                0x03 => *self = Transaction::BlobTransaction(rlp::decode_to::<BlobTransaction>(&bytes[1..])?),
                _ => return Err(RLPException::DecodingError("Bad transaction type")),
            }
        }
        Ok(())
    }
}



macro_rules! extract {
    ($field: ident, $self : expr) => {
        {
            use Transaction::*;
            match $self {
                LegacyTransaction(tx) => &tx.$field,
                AccessListTransaction(tx) => &tx.$field,
                FeeMarketTransaction(tx) => &tx.$field,
                BlobTransaction(tx) => &tx.$field,
            }
        }
    }
}

impl Transaction {
    pub fn nonce(&self) -> &U256 {
        extract!(nonce, &self)
    }

    pub fn gas_price(&self) -> Option<Uint> {
        use Transaction::*;
        match self {
            LegacyTransaction(tx) => Some(tx.gas_price),
            AccessListTransaction(tx) => Some(tx.gas_price),
            FeeMarketTransaction(tx) => None,
            BlobTransaction(tx) => None,
        }
    }

    pub fn gas(&self) -> &Uint {
        extract!(gas, &self)
    }

    pub fn to(&self) -> Option<Address> {
        use Transaction::*;
        match self {
            LegacyTransaction(tx) => tx.to.clone(),
            AccessListTransaction(tx) => tx.to.clone(),
            FeeMarketTransaction(tx) => tx.to.clone(),
            BlobTransaction(tx) => Some(tx.to.clone()),
        }
    }

    pub fn value(&self) -> &U256 {
        extract!(value, &self)
    }

    pub fn data(&self) -> &[u8] {
        extract!(data, &self)
    }

    pub fn v(&self) -> Option<&U256> {
        use Transaction::*;
        match self {
            LegacyTransaction(tx) => Some(&tx.v),
            AccessListTransaction(tx) => None,
            FeeMarketTransaction(tx) => None,
            BlobTransaction(tx) => None,
        }
    }

    pub fn r(&self) -> &U256 {
        extract!(r, &self)
    }

    pub fn s(&self) -> &U256 {
        extract!(s, &self)
    }

    pub fn access_list(&self) -> Option<&[(Address, Vec<Bytes32>)]> {
        use Transaction::*;
        match self {
            LegacyTransaction(tx) => None,
            AccessListTransaction(tx) => Some(&tx.access_list),
            FeeMarketTransaction(tx) => Some(&tx.access_list),
            BlobTransaction(tx) => Some(&tx.access_list),
        }
    }
}



/// Encode a transaction. Needed because non-legacy transactions aren't RLP.
pub fn encode_transaction(tx: &Transaction) -> Result<Either<LegacyTransaction, Bytes>, Exception> {
    use Transaction::*;
    match tx {
        LegacyTransaction(tx) => Ok(Either::A(tx.clone())),
        AccessListTransaction(tx) => Ok(Either::B(Bytes([&b"\x01"[..], &rlp::encode(tx)?].concat()))),
        FeeMarketTransaction(tx) => Ok(Either::B(Bytes([&b"\x02"[..], &rlp::encode(tx)?].concat()))),
        BlobTransaction(tx) => Ok(Either::B(Bytes([&b"\x03"[..], &rlp::encode(tx)?].concat()))),
    }
}


/// Decode a transaction. Needed because non-legacy transactions aren't RLP.
pub fn decode_transaction(tx: Either<LegacyTransaction, Bytes>) -> Result<Transaction, Exception> {
    match tx {
        Either::A(tx) => Ok(Transaction::LegacyTransaction(tx)),
        Either::B(tx) => {
            let tx = &*tx;
            if tx[0] == 1 {
                Ok(Transaction::AccessListTransaction(rlp::decode_to::<AccessListTransaction>(&tx[1..])?))
            } else if tx[0] == 2 {
                Ok(Transaction::FeeMarketTransaction(rlp::decode_to::<FeeMarketTransaction>(&tx[1..])?))
            } else if tx[0] == 3 {
                Ok(Transaction::BlobTransaction(rlp::decode_to::<BlobTransaction>(&tx[1..])?))
            } else {
                Err(Exception::TransactionTypeError{ transaction_type: tx[0] })
            }
        }
    }
}


/// """
/// Verifies a transaction.
/// 
/// The gas in a transaction gets used to pay for the intrinsic cost of
/// operations, therefore if there is insufficient gas then it would not
/// be possible to execute a transaction and it will be declared invalid.
/// 
/// Additionally, the nonce of a transaction must not equal or exceed the
/// limit defined in `EIP-2681 <https://eips.ethereum.org/EIPS/eip-2681>`_.
/// In practice, defining the limit as ``2**64-1`` has no impact because
/// sending ``2**64-1`` transactions is improbable. It's not strictly
/// impossible though, ``2**64-1`` transactions is the entire capacity of the
/// Ethereum blockchain at 2022 gas limits for a little over 22 years.
/// 
/// Parameters
/// ----------
/// tx :
///     Transaction to validate.
/// 
/// Returns
/// -------
/// verified : `bool`
///     True if the transaction can be executed, or false otherwise.
/// """
pub fn validate_transaction(tx: &Transaction) -> bool {
    if calculate_intrinsic_cost(tx) > *tx.gas() {
        return false;
    }

    if tx.nonce() >= &U256::from(u64::MAX) {
        return false;
    }

    if tx.to().is_none() && tx.data().len() > 2 * MAX_CODE_SIZE {
        return false;
    }

    true
}


/// """
/// Calculates the gas that is charged before execution is started.
/// 
/// The intrinsic cost of the transaction is charged before execution has
/// begun. Functions/operations in the EVM cost money to execute so this
/// intrinsic cost is for the operations that need to be paid for as part of
/// the transaction. Data transfer, for example, is part of this intrinsic
/// cost. It costs ether to send data over the wire and that ether is
/// accounted for in the intrinsic cost calculated in this function. This
/// intrinsic cost must be calculated and paid for before execution in order
/// for all operations to be implemented.
/// 
/// Parameters
/// ----------
/// tx :
///     Transaction to compute the intrinsic cost of.
/// 
/// Returns
/// -------
/// verified : `ethereum.base_types.Uint`
///     The intrinsic cost of the transaction.
/// """
pub fn calculate_intrinsic_cost(tx: &Transaction) -> Uint {
    let mut data_cost = 0;

    for byte in tx.data() {
        if *byte == 0 {
            data_cost += TX_DATA_COST_PER_ZERO;
        } else {
            data_cost += TX_DATA_COST_PER_NON_ZERO;
        }
    }

    let create_cost = if tx.to().is_none() {
        TX_CREATE_COST + init_code_cost(tx.data().len() as Uint)
    } else {
        0
    };

    let mut access_list_cost = 0;
    if let Some(access_list) = tx.access_list() {
        for (_address, keys) in access_list {
            access_list_cost += TX_ACCESS_LIST_ADDRESS_COST;
            access_list_cost += keys.len() as Uint * TX_ACCESS_LIST_STORAGE_KEY_COST;
        }
    }

    return Uint::from(TX_BASE_COST + data_cost + create_cost + access_list_cost)
}


/// Extracts the sender address from a transaction.
/// 
/// The v, r, and s values are the three parts that make up the signature
/// of a transaction. In order to recover the sender of a transaction the two
/// components needed are the signature (``v``, ``r``, and ``s``) and the
/// signing hash of the transaction. The sender's public key can be obtained
/// with these two values and therefore the sender address can be retrieved.
/// 
/// Parameters
/// ----------
/// tx :
///     Transaction of interest.
/// chain_id :
///     ID of the executing chain.
/// 
/// Returns
/// -------
/// sender : `ethereum.fork_types.Address`
///     The address of the account that signed the transaction.
pub fn recover_sender(chain_id: U64, tx: &Transaction) -> Result<Address, Exception> {
    let (&r, &s) = (tx.r(), tx.s());
    if U256::from(0_u32) >= r || r >= SECP256K1N {
        return Err(Exception::InvalidSignatureError("bad r"));
    }
    if U256::from(0_u32) >= s || s > SECP256K1N.shr(1) {
        return Err(Exception::InvalidSignatureError("bad s"));
    }

    use Transaction::*;
    let public_key = match tx {
        LegacyTransaction(tx) => {
            let v = tx.v;
            if v == U256::from(27_u32) || v == U256::from(28_u32) {
                secp256k1_recover(
                    r, s, v - U256::from(27_u32), signing_hash_pre155(tx)?
                )
            } else {
                let chain_id_x2 = U256::from(chain_id * 2);
                if v != U256::from(35_u32) + chain_id_x2 && v != U256::from(36_u32) + chain_id_x2 {
                    return Err(Exception::InvalidSignatureError("bad v"));
                }
                secp256k1_recover(
                    r,
                    s,
                    v - U256::from(35) - chain_id_x2,
                    signing_hash_155(tx, chain_id)?,
                )
            }
        }
        AccessListTransaction(tx) => {
            if tx.y_parity != U256::from(0_u32) && tx.y_parity != U256::from(1_u32) {
                return Err(Exception::InvalidSignatureError("bad y_parity"));
            }
            secp256k1_recover(
                r, s, tx.y_parity, signing_hash_2930(tx)?
            )
        }
        FeeMarketTransaction(tx) => {
            if tx.y_parity != U256::from(0_u32) && tx.y_parity != U256::from(1_u32) {
                return Err(Exception::InvalidSignatureError("bad y_parity"));
            }
            secp256k1_recover(
                r, s, tx.y_parity, signing_hash_1559(tx)?
            )
        }
        BlobTransaction(tx) => {
            if tx.y_parity != U256::from(0_u32) && tx.y_parity != U256::from(1_u32) {
                return Err(Exception::InvalidSignatureError("bad y_parity"));
            }
            secp256k1_recover(
                r, s, tx.y_parity, signing_hash_4844(tx)?
            )
        }
    };

    Ok(Address::from_be_bytes(keccak256(&public_key)[12..32].try_into().unwrap()))
}



/// """
/// Compute the hash of a transaction used in a legacy (pre EIP 155) signature.
/// 
/// Parameters
/// ----------
/// tx :
///     Transaction of interest.
/// 
/// Returns
/// -------
/// hash : `ethereum.crypto.hash.Hash32`
///     Hash of the transaction.
/// """
pub fn signing_hash_pre155(tx: &LegacyTransaction) -> Result<Hash32, Exception> {
    let mut dest = Bytes::default();
    rlp::encode_sequence(&mut dest, &[
        &tx.nonce,
        &tx.gas_price,
        &tx.gas,
        &tx.to,
        &tx.value,
        &tx.data,
    ])?;
    Ok(keccak256(&dest))
}


/// """
/// Compute the hash of a transaction used in a EIP 155 signature.
/// 
/// Parameters
/// ----------
/// tx :
///     Transaction of interest.
/// chain_id :
///     The id of the current chain.
/// 
/// Returns
/// -------
/// hash : `ethereum.crypto.hash.Hash32`
///     Hash of the transaction.
/// """
pub fn signing_hash_155(tx: &LegacyTransaction, chain_id: U64) -> Result<Hash32, Exception> {
    let mut dest = Bytes::default();
    rlp::encode_sequence(&mut dest, &[
        &tx.nonce,
        &tx.gas_price,
        &tx.gas,
        &tx.to,
        &tx.value,
        &tx.data,
        &chain_id,
        &Uint::from(0_u32),
        &Uint::from(0_u32),
    ])?;
    Ok(keccak256(&dest))
}


/// """
/// Compute the hash of a transaction used in a EIP 2930 signature.
/// 
/// Parameters
/// ----------
/// tx :
///     Transaction of interest.
/// 
/// Returns
/// -------
/// hash : `ethereum.crypto.hash.Hash32`
///     Hash of the transaction.
/// """
pub fn signing_hash_2930(tx: &AccessListTransaction) -> Result<Hash32, Exception> {
    let mut dest = Bytes::default();
    dest.push(0x01);
    rlp::encode_sequence(&mut dest, &[
        &tx.chain_id,
        &tx.nonce,
        &tx.gas_price,
        &tx.gas,
        &tx.to,
        &tx.value,
        &tx.data,
        &tx.access_list,
    ])?;
    Ok(keccak256(&dest))
}


/// """
/// Compute the hash of a transaction used in a EIP 1559 signature.
/// 
/// Parameters
/// ----------
/// tx :
///     Transaction of interest.
/// 
/// Returns
/// -------
/// hash : `ethereum.crypto.hash.Hash32`
///     Hash of the transaction.
/// """
pub fn signing_hash_1559(tx: &FeeMarketTransaction) -> Result<Hash32, Exception> {
    let mut res = Bytes::default();
    res.push(0x02);
    rlp::encode_sequence(&mut res, &[
        &tx.chain_id,
        &tx.nonce,
        &tx.max_priority_fee_per_gas,
        &tx.max_fee_per_gas,
        &tx.gas,
        &tx.to,
        &tx.value,
        &tx.data,
        &tx.access_list,
    ]);
    Ok(keccak256(&res))
}


/// Compute the hash of a transaction used in a EIP-4844 signature.
/// 
/// Parameters
/// ----------
/// tx :
///     Transaction of interest.
/// 
/// Returns
/// -------
/// hash : `ethereum.crypto.hash.Hash32`
///     Hash of the transaction.
pub fn signing_hash_4844(tx: &BlobTransaction) -> Result<Hash32, Exception> {
    let mut res = Bytes::default();
    res.push(3);
    rlp::encode_sequence(&mut res, &[
        &tx.chain_id,
        &tx.nonce,
        &tx.max_priority_fee_per_gas,
        &tx.max_fee_per_gas,
        &tx.gas,
        &tx.to,
        &tx.value,
        &tx.data,
        &tx.access_list,
        &tx.max_fee_per_blob_gas,
        &tx.blob_versioned_hashes,
    ]);
    Ok(keccak256(&res))
}
