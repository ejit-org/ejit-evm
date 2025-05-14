//! Types and functions for beginning a new chain.
//! 
//! _Genesis_ is the term for the beginning of a new chain, and so a genesis block
//! is a block with no parent (its [`parent_hash`] is all zeros.)
//! 
//! The genesis configuration for a chain is specified with a
//! [`GenesisConfiguration`], and genesis blocks are created with
//! [`add_genesis_block`].
//! 
//! [`parent_hash`]: ref:ethereum.frontier.blocks.Header.parent_hash
//! [`GenesisConfiguration`]: ref:ethereum.genesis.GenesisConfiguration
//! [`add_genesis_block`]: ref:ethereum.genesis.add_genesis_block

use std::collections::BTreeMap;

use crate::json::{Decoder, JsonDecode, JsonError, ObjectParser};

use super::{cancun::{self, blocks::Header, fork::BlockChain, fork_types::{Account, Address, Root}}, crypto::hash::Hash32, ethereum_rlp::rlp::Extended, ethereum_types::{bytes::{Bytes, Bytes32, Bytes8}, numeric::{Uint, U256, U64}}, exceptions::Exception, utils::hexadecimal::{hex_to_bytes, hex_to_bytes8, hex_to_u256, hex_to_uint}};

#[derive(Default, Debug)]
pub struct Genesis {
    pub header: Header,
    pub alloc: BTreeMap<Address, Account>,
    pub chain_id: U64,
}

impl<'de> JsonDecode<'de> for Genesis {
    fn decode_json(&mut self, decoder: &mut Decoder<'de>) -> Result<(), JsonError> {
        let mut p = ObjectParser::new(decoder);
        loop {
            match p.next_key()? {
                Some(k) if k == "nonce" => self.header.nonce.decode_json(p.decoder)?,
                Some(k) if k == "timestamp" => self.header.timestamp.decode_json(p.decoder)?,
                Some(k) if k == "extraData" => self.header.extra_data.decode_json(p.decoder)?,
                Some(k) if k == "gasLimit" => self.header.gas_limit.decode_json(p.decoder)?,
                Some(k) if k == "difficulty" => self.header.difficulty.decode_json(p.decoder)?,
                Some(k) if k == "mixHash" => self.header.prev_randao.decode_json(p.decoder)?,
                Some(k) if k == "coinbase" => self.header.coinbase.decode_json(p.decoder)?,
                Some(k) if k == "stateRoot" => self.header.state_root.decode_json(p.decoder)?,
                Some(k) if k == "alloc" => self.alloc.decode_json(p.decoder)?,
                Some(k) if k == "number" => self.header.number.decode_json(p.decoder)?,
                Some(k) if k == "gasUsed" => self.header.gas_used.decode_json(p.decoder)?,
                Some(k) if k == "parentHash" => self.header.parent_hash.decode_json(p.decoder)?,
                None => return Ok(()),
                _ => return Err(crate::json::JsonError::MissingKey),
            };
        }
    }
}

const MAINNET : &'static str = include_str!("../../assets/mainnet.json");

impl Genesis {
    pub fn mainnet() -> Result<Self, Exception> {
        let mut g = Genesis::default();
        let mut cursor = MAINNET.as_bytes();
        g.decode_json(&mut Decoder::new(cursor))
            .map_err(|e| Exception::JsonError(e))?;
        Ok(g)
    }
}

#[test]
fn test_mainnet() {
    let g = Genesis::mainnet().unwrap();
    let chain = BlockChain::from_genesis(g);
    println!("{chain:?}");
}

