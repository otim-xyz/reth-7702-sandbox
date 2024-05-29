//! EIP-7702 transaction
#![allow(missing_docs, unreachable_pub)]

use alloy_primitives::{U160, U256};
use alloy_rlp::{Encodable, RlpEncodable};
use k256::ecdsa::SigningKey;
use sha3::{Digest, Sha3_256};

pub const TX_TYPE: u8 = 0x04;

#[derive(Debug, RlpEncodable)]
pub struct AccessListItem {
    address: U160,
    storage_keys: Vec<Vec<u8>>,
}

#[derive(Debug, RlpEncodable)]
pub struct CodeBundleItem {
    address: U160,
    v: u8,
    r: U256,
    s: U256,
}

#[derive(Debug, RlpEncodable)]
pub struct TxEip7702 {
    pub chain_id: u64,
    pub nonce: u64,
    pub max_priority_fee_per_gas: u64,
    pub max_fee_per_gas: u64,
    pub gas_limit: u64,
    pub to: U160,
    pub amount: U256,
    pub data: u64,
    pub access_list: Vec<AccessListItem>,
    pub code_bundles: Vec<CodeBundleItem>,
}

impl TxEip7702 {
    pub fn rlp_encode(&self) -> Vec<u8> {
        let mut buffer = Vec::<u8>::new();
        buffer.push(TX_TYPE);
        self.encode(&mut buffer);
        buffer
    }

    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = Sha3_256::new();
        hasher.update(&self.rlp_encode());
        let hash = hasher.finalize();
        hash.to_vec()
    }

    pub fn sign(self, pk_hex: &str) -> TxEip7702Signed {
        let signing_key =
            SigningKey::from_slice(&hex::decode(pk_hex).unwrap()).unwrap();
        let (sig, recid) =
            signing_key.sign_prehash_recoverable(&self.hash()).unwrap();
        let TxEip7702 {
            chain_id,
            nonce,
            max_priority_fee_per_gas,
            max_fee_per_gas,
            gas_limit,
            to,
            amount,
            data,
            access_list,
            code_bundles,
        } = self;
        TxEip7702Signed {
            chain_id,
            nonce,
            max_priority_fee_per_gas,
            max_fee_per_gas,
            gas_limit,
            to,
            amount,
            data,
            access_list,
            code_bundles,
            v: recid.to_byte(),
            r: U256::from_be_bytes::<32>(
                sig.r().to_bytes().to_vec().try_into().unwrap(),
            ),
            s: U256::from_be_bytes::<32>(
                sig.s().to_bytes().to_vec().try_into().unwrap(),
            ),
        }
    }
}

#[derive(Debug, RlpEncodable)]
pub struct TxEip7702Signed {
    pub chain_id: u64,
    pub nonce: u64,
    pub max_priority_fee_per_gas: u64,
    pub max_fee_per_gas: u64,
    pub gas_limit: u64,
    pub to: U160,
    pub amount: U256,
    pub data: u64,
    pub access_list: Vec<AccessListItem>,
    pub code_bundles: Vec<CodeBundleItem>,
    pub v: u8,
    pub r: U256,
    pub s: U256,
}

impl TxEip7702Signed {
    pub fn rlp_encode(&self) -> Vec<u8> {
        let mut buffer = Vec::<u8>::new();
        buffer.push(TX_TYPE);
        self.encode(&mut buffer);
        buffer
    }
}
