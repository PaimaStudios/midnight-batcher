// deserialization of the graphql output
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DustLedgerEvent {
    #[allow(unused)]
    #[serde(rename = "__typename")]
    pub typename: String,
    pub id: u64,
    pub raw: String,
    #[serde(rename = "maxId")]
    pub max_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct DustLedgerEvents {
    #[serde(rename = "dustLedgerEvents")]
    pub dust_ledger_events: DustLedgerEvent,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    pub data: DustLedgerEvents,
}

#[derive(Debug, Deserialize)]
pub struct Val {
    pub payload: Data,
}
