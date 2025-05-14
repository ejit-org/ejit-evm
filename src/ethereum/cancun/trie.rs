//!
//! This is just a reference implementation. A more performance
//! orientated one would be necessary for a real chain!
//! 
//! 
//! 
//! https://github.com/ethereum/execution-specs/blob/master/src/ethereum/cancun/trie.py
//! State Trie
//! ^^^^^^^^^^
//! 
//! .. contents:: Table of Contents
//!     :backlinks: none
//!     :local:
//! 
//! Introduction
//! ------------
//! 
//! The state trie is the structure responsible for storing
//! `.fork_types.Account` objects.

// note: an empty trie (regardless of whether it is secured) has root:
//
//   keccak256(RLP(b''))
//       ==
//   56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421 # noqa: E501,SC10
//
// also:
//
//   keccak256(RLP(()))
//       ==
//   1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347 # noqa: E501,SC10
//
// which is the sha3Uncles hash in block header with no uncles

use std::collections::BTreeMap;

use crate::ethereum::{cancun::fork_types::{Account, Root}, crypto::hash::{keccak256, Hash32}, ethereum_rlp::{exceptions::RLPException, rlp::{self, encode, encode_joined_encodings, encode_sequence, Extended}}, ethereum_types::{bytes::{Bytes, Bytes32, Verbatim}, numeric::{Uint, U256}}};

use super::fork_types::Address;

const EMPTY_TRIE_ROOT : Root = Root([0x56,0xe8,0x1f,0x17,0x1b,0xcc,0x55,0xa6,0xff,0x83,0x45,0xe6,0x92,0xc0,0xf8,0x6e,0x5b,0x48,0xe0,0x1b,0x99,0x6c,0xad,0xc0,0x01,0x62,0x2f,0xb5,0xe3,0x63,0xb4,0x21]);

#[derive(Debug)]
/// Leaf node in the Merkle Trie
struct LeafNode {
    rest_of_key: Bytes,
    value: Verbatim,
}

#[derive(Debug)]
/// Extension node in the Merkle Trie
struct ExtensionNode {
    key_segment: Bytes,
    subnode: Verbatim,
}

#[derive(Debug)]
/// Branch node in the Merkle Trie
struct BranchNode {
    subnodes: Vec<Verbatim>,
    value: Verbatim,
}

#[derive(Debug)]
enum InternalNode {
    LeafNode(LeafNode),
    ExtensionNode(ExtensionNode),
    BranchNode(BranchNode),
    None,
}

pub trait Key {
    fn get_bytes(&self) -> Bytes;
}

impl Key for String {
    fn get_bytes(&self) -> Bytes {
        Bytes::from(self.as_bytes())
    }
}

impl Key for Bytes {
    fn get_bytes(&self) -> Bytes {
        self.clone()
    }
}

pub trait Value {
    fn encode_node(&self) -> Verbatim;
}

impl Value for String {
    fn encode_node(&self) -> Verbatim {
        let mut buffer = Bytes::default();
        self.encode(&mut buffer);
        buffer.into_verbatim()
    }
}

impl Value for Bytes {
    fn encode_node(&self) -> Verbatim {
        let mut buffer = Bytes::default();
        self.encode(&mut buffer);
        buffer.into_verbatim()
    }
}

impl InternalNode {
    /// Encodes a Merkle Trie node into its RLP form. The RLP will then be
    /// serialized into a `Bytes` and hashed unless it is less that 32 bytes
    /// when serialized.
    /// 
    /// This function also accepts `None`, representing the absence of a node,
    /// which is encoded to `b""`.
    fn encode_internal_node(self, rlp_the_hash: bool) -> Verbatim {
        use InternalNode::*;
        let mut encoded = Bytes::default();
        if !matches!(&self, None) { println!("unencoded={self:?}"); }
        match self {
            LeafNode(node) => {
                (
                    nibble_list_to_compact(&node.rest_of_key, true),
                    node.value
                ).encode(&mut encoded).unwrap()
            }
            ExtensionNode(node) => {
                (
                    nibble_list_to_compact(&node.key_segment, false),
                    node.subnode
                ).encode(&mut encoded).unwrap()
            }
            BranchNode(node) => {
                let mut joined_encodings = Bytes::default();
                for s in node.subnodes {
                    joined_encodings.extend(s.0);
                }
                node.value.encode(&mut joined_encodings);
                encode_joined_encodings(&mut encoded, joined_encodings);
            }
            None => {
                encoded.push(0x80);
            }
        };

        if encoded.len() > 1 { println!("encoded={encoded:?}"); }
        if encoded.len() < 32 {
            Verbatim(encoded.0)
        } else if rlp_the_hash {
            let mut rlp = Bytes::default();
            keccak256(&encoded).encode(&mut rlp);
            Verbatim(rlp.0)
        } else {
            Verbatim(keccak256(&encoded).to_vec())
        }

    }
}


// def encode_internal_node(node: Optional[InternalNode]) -> rlp.Extended:
//     """
//     Encodes a Merkle Trie node into its RLP form. The RLP will then be
//     serialized into a `Bytes` and hashed unless it is less that 32 bytes
//     when serialized.

//     This function also accepts `None`, representing the absence of a node,
//     which is encoded to `b""`.

//     Parameters
//     ----------
//     node : Optional[InternalNode]
//         The node to encode.

//     Returns
//     -------
//     encoded : `rlp.Extended`
//         The node encoded as RLP.
//     """
//     unencoded: rlp.Extended
//     if node is None:
//         unencoded = b""
//     elif isinstance(node, LeafNode):
//         unencoded = (
//             nibble_list_to_compact(node.rest_of_key, True),
//             node.value,
//         )
//     elif isinstance(node, ExtensionNode):
//         unencoded = (
//             nibble_list_to_compact(node.key_segment, False),
//             node.subnode,
//         )
//     elif isinstance(node, BranchNode):
//         unencoded = list(node.subnodes) + [node.value]
//     else:
//         raise AssertionError(f"Invalid internal node type {type(node)}!")

//     encoded = rlp.encode(unencoded)
//     if len(encoded) < 32:
//         return unencoded
//     else:
//         return keccak256(encoded)


// def encode_node(node: Node, storage_root: Optional[Bytes] = None) -> Bytes:
//     """
//     Encode a Node for storage in the Merkle Trie.

//     Currently mostly an unimplemented stub.
//     """
//     if isinstance(node, Account):
//         assert storage_root is not None
//         return encode_account(node, storage_root)
//     elif isinstance(node, (LegacyTransaction, Receipt, Withdrawal, U256)):
//         return rlp.encode(node)
//     elif isinstance(node, Bytes):
//         return node
//     else:
//         return previous_trie.encode_node(node, storage_root)

/// The Merkle Trie.
#[derive(Debug, Clone, Default)]
pub struct Trie<K : Ord, V : PartialEq + Clone> {
    secured: bool,
    default_value: V,
    data: BTreeMap<K, V>,
}

impl<K : Ord, V : PartialEq + Clone> Trie<K, V> {
    pub fn new(secured: bool, default_value: V) -> Self {
        Self { secured, default_value, data: Default::default() }
    }
    
    ///  Stores an item in a Merkle Trie.
    ///  This method deletes the key if `value == trie.default`, because the Merkle
    ///  Trie represents the default value by omitting it from the trie.
    pub fn set(&mut self, k: K, v: V) {
        if v == self.default_value {
            self.data.remove(&k);
        } else {
            self.data.insert(k, v);
        }
    }

    /// Gets an item from the Merkle Trie.
    ///
    /// This method returns `trie.default` if the key is missing.
    pub fn get(&self, k: &K) -> V {
        let g = self.data.get(k).unwrap_or(&self.default_value);
        g.clone()
    }
    
    pub fn secured(mut self, secured: bool) -> Self {
        self.secured = secured;
        self
    }
}

/// Find the longest common prefix of two sequences.
fn common_prefix_length(a: &[u8], b:&[u8]) -> usize {
    a.iter().zip(b.iter())
        .position(|(a, b)| a != b)
        .unwrap_or(a.len().min(b.len()))
}

/// Compresses nibble-list into a standard byte array with a flag.
/// 
/// A nibble-list is a list of byte values no greater than `15`. The flag is
/// encoded in high nibble of the highest byte. The flag nibble can be broken
/// down into two two-bit flags.
/// 
/// Highest nibble::
/// 
///     +---+---+----------+--------+
///     | _ | _ | is_leaf | parity |
///     +---+---+----------+--------+
///         3   2      1         0
/// 
/// 
/// The lowest bit of the nibble encodes the parity of the length of the
/// remaining nibbles -- `0` when even and `1` when odd. The second lowest bit
/// is used to distinguish leaf and extension nodes. The other two bits are not
/// used.
fn nibble_list_to_compact(x: &[u8], is_leaf: bool) -> Bytes {
    let mut compact = Bytes::default();

    if x.len() % 2 == 0 {
        compact.push(16 * (2 * is_leaf as u8 ));
        for i in (0..x.len()).step_by(2) {
            compact.push(16 * x[i] + x[i + 1])
        }
    } else {
        compact.push(16 * ((2 * is_leaf as u8) + 1) + x[0]);
        for i in (1..x.len()).step_by(2) {
            compact.push(16 * x[i] + x[i + 1])
        }
    }
    compact
}


/// Converts a `Bytes` into to a sequence of nibbles (bytes with value < 16).
fn bytes_to_nibble_list(bytes_: &[u8]) -> Bytes {
    let mut nibble_list = vec![0; 2 * bytes_.len()];
    for (byte_index, &byte) in bytes_.iter().enumerate() {
        nibble_list[byte_index * 2] = (byte & 0xF0) >> 4;
        nibble_list[byte_index * 2 + 1] = byte & 0x0F;
    }
    return Bytes(nibble_list)
}

impl<K : Ord + Key, V : PartialEq + Clone + Value> Trie<K, V> {
    /// Prepares the trie for root calculation. Removes values that are empty,
    /// hashes the keys (if `secured == True`) and encodes all the nodes.
    fn prepare_trie(&self) -> BTreeMap<Bytes, Verbatim> {
        let mut mapped = BTreeMap::new();
        for (key, value) in &self.data {
            let preimage = key.get_bytes();
            let encoded_value = value.encode_node();
            let key = if self.secured {
                keccak256(preimage.as_ref()).to_vec()
            } else {
                preimage.as_ref().to_vec()
            };
            let nibbles = bytes_to_nibble_list(&key);
            mapped.insert(nibbles, encoded_value);
        }
        mapped
    }

    /// Computes the root of a modified merkle patricia trie (MPT).
    /// returns MPT root of the underlying key-value pairs.
    pub fn root(&self) -> Result<Root, RLPException> {
        let obj = self.prepare_trie();
        println!("obj={obj:?}");
        let pat = Self::patricialize(obj, 0);
        println!("pat={pat:?}");
        let root_node = pat.encode_internal_node(false);
        // println!("root_node={root_node:?}");
        let root_node = Bytes(root_node.0);
        let encoded = rlp::encode(&root_node)?;
        if encoded.len() < 32 {
            Ok(Root(keccak256(&encoded).0))
        } else {
            Ok(Root(root_node.0.try_into().unwrap()))
        }
    }

    /// Structural composition function.
    /// 
    /// Used to recursively patricialize and merkleize a dictionary. Includes
    /// memoization of the tree structure and hashes.
    fn patricialize(obj: BTreeMap<Bytes, Verbatim>, level: usize) -> InternalNode {
        if obj.is_empty() {
            return InternalNode::None;
        }

        let objlen = obj.len();
        let (arbitrary_key, value) = obj.iter().next().unwrap();
        let arbitrary_key = arbitrary_key.clone();
        let value = value.clone();

        // if leaf node
        if objlen == 1 {
            let rest_of_key = Bytes::from(&arbitrary_key[level..]);
            return InternalNode::LeafNode(LeafNode { rest_of_key, value });
        }

        // prepare for extension node check by finding max j such that all keys in
        // obj have the same key[i:j]
        let substring = &arbitrary_key[level..];
        let mut prefix_length = substring.len();
        for (key, value) in &obj {
            prefix_length = prefix_length.min(
                common_prefix_length(substring, &key[level..])
            );

            // finished searching, found another key at the current level
            if prefix_length == 0 {
                break;
            }
        }

        // if extension node
        if prefix_length > 0 {
            let key_segment = Bytes::from(&arbitrary_key[level..level + prefix_length]);
            let pat = Self::patricialize(obj, level + prefix_length);
            println!("pat2={pat:?}");
            let subnode = pat.encode_internal_node(true);

            return InternalNode::ExtensionNode(ExtensionNode{key_segment, subnode});
        }

        let mut branches = Vec::new();
        for _ in 0..16 {
            branches.push(BTreeMap::new());
        }

        let mut value = Bytes::default();
        value.push(0x80);
        for (key, v) in obj {
            if key.len() == level {
                value.0.clear();
                v.encode(&mut value).unwrap();
            } else {
                branches[key[level] as usize].insert(key, v);
            }
        }

        println!("branches={branches:?}");

        let subnodes : Vec<Verbatim> = branches.into_iter().map(|b| {
            let pat = Self::patricialize(b, level + 1);
            pat.encode_internal_node(true)
        }).collect();

        return InternalNode::BranchNode(BranchNode{
            subnodes,
            value: value.into_verbatim(),
        });

    }
}

#[cfg(test)]
mod tests {
    use crate::{ethereum::{cancun::fork_types::Root, ethereum_types::bytes::Bytes, utils::hexadecimal::hex_to_bytes}, json::{Decoder, JsonDecode, JsonError, ObjectParser, Value}};

    use super::Trie;


    #[test]
    fn trie_any_order() -> Result<(), JsonError> {
        test_trie("trieanyorder.json", false)?;
        Ok(())
    }

    #[test]
    fn trie_any_order_secure() -> Result<(), JsonError> {
        test_trie("trieanyorder_secureTrie.json", true)?;
        Ok(())
    }

    #[test]
    fn hex_encoded_secure() -> Result<(), JsonError> {
        test_trie("hex_encoded_securetrie_test.json", true)?;
        Ok(())
    }

    fn test_trie(file: &str, secured: bool) -> Result<(), JsonError> {
        let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let text = std::fs::read_to_string(
            format!("{dir}/assets/TrieTests/{file}")
        ).unwrap();
        let mut decoder = Decoder::new(text.as_bytes());
        let mut p = ObjectParser::new(&mut decoder);
        Ok(while let Some(name) = p.next_key()? {
            let mut p = ObjectParser::new(p.decoder);
            let mut trie = Trie::default().secured(secured);
            let mut root = Root::default();
            while let Some(k) = p.next_key()? {
                match k {
                    "in" => {
                        let mut p = ObjectParser::new(p.decoder);
                        while let Some(k) = p.next_key()? {
                            let mut v = "";
                            v.decode_json(p.decoder)?;
                            trie.set(convert(k), convert(v));
                        }
                    }
                    "root" => {
                        root.decode_json(p.decoder)?;
                    }
                    
                    _ => {
                        let mut v = Value::Null;
                        v.decode_json(p.decoder)?;
                    }
                }
            }
            let r = trie.root().unwrap();
            println!("{trie:?} {root:?} {r:?}");
            assert_eq!(root, r, "{name}");
        })
    }
    
    fn convert(s: &str) -> Bytes {
        if s.starts_with("0x") {
            hex_to_bytes(s).unwrap()
        } else {
            Bytes::from(s.as_bytes())
        }
    }
}

// p encoded=0xf84080808080a094a9f95bd89698e4da1812e0518053813b4d5b87caaf6b3c6fa57e9e50c0ff68808080cf85206f727365887374616c6c696f6e8080808080808080
// r encoded=0xf84080808080a0898225ff043fefb3a91b2f95bd64ed0b82d844b4d69bf07885586df7bc434814808080cf85206f727365887374616c6c696f6e8080808080808080
