use std::{io::{BufRead, BufReader, Write}, net::TcpStream, path::PathBuf, sync::Arc, time::Duration};

use crate::ethereum::{cancun::blocks::Block, ethereum_rlp::rlp, ethereum_types::bytes::Bytes, genesis::{self, Genesis}};

use super::BlockChain;

#[test]
fn test_against_alchemy() {
    let url = std::env::var("ALCHEMY_URL").unwrap();
    let client = reqwest::blocking::Client::new();

    let latest_block = 22445332;
    let genesis = Genesis::mainnet().unwrap();
    let chain = BlockChain::from_genesis(genesis);

    // for block in (0..latest_block) /* .step_by(1000000)*/ {
    //     let res = loop {
    //         println!("{block}");
    //         // std::io::stdout().flush();
    //         let body = format!(
    //             r#"{{"id": 1,"jsonrpc": "2.0","method": "debug_getRawBlock","params": ["0x{block:x}"]}}"#
    //         );
        
    //         let resp = client
    //             .post(&url)
    //             .header("accept", "application/json")
    //             .header("content-type", "application/json")
    //             .body(body).send().unwrap();
        
    //         if resp.status() == 200 {
    //             break resp.text().unwrap();
    //         }
    //         println!("{}", resp.text().unwrap());
    //         std::thread::sleep(Duration::from_millis(500));
    //     };

    //     let (_, rest) = res.split_once(r#"result":"0x"#).unwrap();
    //     let (hex, _) = rest.split_once('"').unwrap();
    
    //     let bytes : Vec<u8> = hex
    //         .as_bytes()
    //         .chunks_exact(2)
    //         .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
    //         .collect();
    
    //     // std::fs::write("/tmp/1", format!("{bytes:02x?}"));
    //     let block : Block = rlp::decode_to(&bytes).unwrap();
    // }

    // println!("block: {block:?}");

}
