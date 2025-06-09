// deserialization of the graphql output
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "__typename")]
pub enum TransactionOrUpdate {
    ViewingUpdate(ViewingUpdate),
    ProgressUpdate(ProgressUpdate),
}

#[derive(Debug, Deserialize)]
pub struct ViewingUpdate {
    // pub index: u64,
    pub update: Vec<ZswapChainStateUpdate>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "__typename")]
pub enum ZswapChainStateUpdate {
    RelevantTransaction(RelevantTransaction),
    MerkleTreeCollapsedUpdate(MerkleTreeCollapsedUpdate),
}

#[derive(Debug, Deserialize)]
pub struct TransactionBlock {
    pub height: u64,
}

#[derive(Debug, Deserialize)]
pub struct RelevantTransaction {
    pub start: u64,
    // pub end: u64,
    pub transaction: Transaction,
}

#[derive(Debug, Deserialize)]
pub struct MerkleTreeCollapsedUpdate {
    // #[serde(rename = "protocolVersion")]
    // pub protocol_version: u64,
    // pub start: u64,
    // pub end: u64,
    pub update: String,
}

#[derive(Debug, Deserialize)]
pub struct Transaction {
    // pub hash: String,
    #[serde(rename = "applyStage")]
    pub apply_stage: String,
    pub raw: String,
    pub block: TransactionBlock,
}

#[derive(Debug, Deserialize)]
pub struct ProgressUpdate {
    #[allow(unused)]
    #[serde(rename = "highestIndex")]
    pub highest_index: u64,
    #[allow(unused)]
    #[serde(rename = "highestRelevantIndex")]
    pub highest_relevant_index: u64,
    #[serde(rename = "highestRelevantWalletIndex")]
    pub highest_relevant_wallet_index: u64,
}

#[derive(Debug, Deserialize)]
pub struct Wallet {
    pub wallet: TransactionOrUpdate,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    pub data: Wallet,
}

#[derive(Debug, Deserialize)]
pub struct Val {
    pub payload: Data,
}
