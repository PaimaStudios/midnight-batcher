#[macro_use]
extern crate rocket;

mod balancing;
mod db;
mod endpoints;

use anyhow::Context as _;
use balancing::ProvingParams;
use clap::{arg, Command};
use db::Db;
use futures::{SinkExt, StreamExt};
use midnight_ledger::structure::Transaction;
use midnight_transient_crypto::proofs::Proof;
use midnight_zswap::local::State;
use midnight_zswap::serialize::{deserialize, serialize, NetworkId};
use rand::SeedableRng as _;
use rand_chacha::ChaCha20Rng;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use subxt::{OnlineClient, SubstrateConfig};
use tokio::sync::Mutex;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{self, client::IntoClientRequest, http::HeaderValue},
};
use url::Url;

const STABLE_STATE_ID: &str = "committed";
const INDEXER_LOCALHOST: &str = "ws://127.0.0.1:8088/api/v1/graphql/ws";
const NODE_LOCALHOST: &str = "ws://127.0.0.1:9944";

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod midnight {}

pub enum SyncStatus {
    Syncing { progress: f64 },
    UpToDate,
}

fn address(zswap_state: &State, network_id: NetworkId) -> String {
    let pk = zswap_state.coin_public_key();
    let epk = zswap_state.enc_public_key();

    let pk_hex = hex::encode(pk.0 .0);
    let mut buf = vec![];
    serialize(&epk, &mut buf, network_id).unwrap();
    let ec_hex = hex::encode(&buf[1..]);

    format!("{}|{}", pk_hex, ec_hex)
}

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let matches = Command::new("Midnight Batcher")
        .version("1.0")
        .author("Enzo Cioppettini <enzo@dcspark.com>")
        .about("Midnight paymaster for Paima")
        .arg(arg!(--indexer <WEBSOCKET_URL>).default_value(INDEXER_LOCALHOST))
        .arg(arg!(--node <URL>).default_value(NODE_LOCALHOST))
        .arg(
            arg!(--secret <FILEPATH>)
                .default_value("./seed")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            arg!(--network <NETWORK>)
                .value_parser(["testnet", "undeployed"])
                .default_value("undeployed"),
        )
        .get_matches();

    let indexer = matches.get_one::<String>("indexer").expect("default");
    let node = matches.get_one::<String>("node").expect("default");
    let credentials = matches.get_one::<PathBuf>("secret").expect("default");
    let network = matches.get_one::<String>("network").expect("default");

    info!("URLs: {:?}", indexer);
    info!("File path: {:?}", node);
    info!("Wallet: {:?}", credentials);
    info!("Network: {}", network);

    let api = OnlineClient::<SubstrateConfig>::from_url(node)
        .await
        .context("Couldn't establish connection with the node")?;

    let network_id = match network.as_ref() {
        "testnet" => NetworkId::TestNet,
        "undeployed" => NetworkId::Undeployed,
        _ => anyhow::bail!("invalid network"),
    };

    let url = Url::parse(INDEXER_LOCALHOST).expect("Invalid indexer URL");

    let proving_params = ProvingParams::new()?;

    let db = Db::open_db("db.sqlite", network_id)?;

    let seed = std::fs::read_to_string(credentials).context("Failed to read credentials")?;

    let mut rng = ChaCha20Rng::from_seed(
        <[u8; 32]>::try_from(hex::decode(seed.trim()).context("seed should be a valid hex")?)
            .map_err(|_| anyhow::anyhow!("expected seed to contain 32 bytes"))?,
    );

    let maybe_latest_state = db.get_state(STABLE_STATE_ID)?;

    let initial_state = maybe_latest_state
        .as_ref()
        .map(|(_, state)| state.clone())
        .unwrap_or_else(|| State::new(&mut rng));

    let address = address(&initial_state, network_id);

    info!("Batcher address {}", address);

    let current_tx = maybe_latest_state.map(|(hash, _)| hash);

    let sync_status = Arc::new(Mutex::new(SyncStatus::Syncing { progress: 0.0 }));

    let initial_state = Arc::new(Mutex::new(initial_state));

    tokio::task::spawn(wallet_indexer(
        db,
        url,
        Arc::clone(&initial_state),
        current_tx,
        network_id,
        Arc::clone(&sync_status),
    ));

    endpoints::rocket(proving_params, api, initial_state, network_id, sync_status)
        .launch()
        .await
        .unwrap();

    Ok(())
}

async fn wallet_indexer(
    db: Db,
    url: Url,
    latest_state: Arc<Mutex<State>>,
    current_tx: Option<String>,
    network_id: NetworkId,
    sync_status: Arc<Mutex<SyncStatus>>,
) -> anyhow::Result<()> {
    // There is no pending state initially
    //
    // For this to be true then no transaction has to be built before the SyncStatus changes.
    let mut confirmed_state = latest_state.lock().await.clone();

    let mut req = url.into_client_request().unwrap();
    req.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        HeaderValue::from_static("graphql-ws"),
    );

    let (ws_stream, _) = connect_async(req).await.expect("Failed to connect");

    println!("WebSocket connection established!");

    let (mut write, mut read) = ws_stream.split();

    let init_query = json!(
        {
            "id":"1",
            "type": "connection_init"
        }
    );

    write
        .send(tungstenite::Message::Text(init_query.to_string()))
        .await
        .expect("Failed to send init message");

    let message = read.next().await.unwrap().unwrap();

    if let tungstenite::Message::Text(text) = message {
        println!("Received?: {}", text);
    }

    let _message = read.next().await.unwrap().unwrap();

    let subscription_query = |start: Option<String>| {
        if let Some(start) = start {
            json!({
                "id": "2",
                "type": "start",
                "payload": {
                    "query": format!(r#"subscription{{ transactions(offset: {{hash: "{}" }}) {{__typename ... on TransactionAdded {{ transaction {{ hash raw applyStage block {{ hash height }} }} }} ... on ProgressUpdate {{synced total}} }} }}"#, start)
                }
            })
        } else {
            json!({
                "id": "2",
                "type": "start",
                "payload": {
                    "query":"subscription{transactions {__typename ... on TransactionAdded { transaction { hash raw block { hash height } applyStage }} ... on ProgressUpdate {synced total} } }"
                }
            })
        }
    };

    let (subscription_query, mut skip_first) = if let Some(txhash) = current_tx {
        (subscription_query(Some(txhash)), true)
    } else {
        (subscription_query(None), false)
    };

    write
        .send(tungstenite::Message::Text(subscription_query.to_string()))
        .await
        .expect("Failed to send message");

    while let Some(message) = read.next().await {
        match message {
            Ok(tungstenite::Message::Text(text)) => {
                mod gql {
                    use serde::Deserialize;

                    #[derive(Debug, Deserialize)]
                    #[serde(tag = "__typename")]
                    pub enum TransactionOrUpdate {
                        TransactionAdded(Transaction),
                        ProgressUpdate(ProgressUpdate),
                    }

                    #[derive(Debug, Deserialize)]
                    pub struct Transaction {
                        pub transaction: TransactionAdded,
                    }

                    #[derive(Debug, Deserialize)]
                    pub struct TransactionAdded {
                        pub hash: String,
                        #[serde(rename = "applyStage")]
                        pub apply_stage: String,
                        pub raw: String,
                    }

                    #[derive(Debug, Deserialize)]
                    pub struct ProgressUpdate {
                        pub synced: f64,
                        pub total: f64,
                    }

                    #[derive(Debug, Deserialize)]
                    pub struct Transactions {
                        pub transactions: TransactionOrUpdate,
                    }

                    #[derive(Debug, Deserialize)]
                    pub struct Data {
                        pub data: Transactions,
                    }

                    #[derive(Debug, Deserialize)]
                    pub struct Val {
                        pub payload: Data,
                    }
                }

                let val: gql::Val = serde_json::from_str(&text).unwrap();

                let transaction = match val.payload.data.transactions {
                    gql::TransactionOrUpdate::TransactionAdded(tx_added) => tx_added.transaction,
                    gql::TransactionOrUpdate::ProgressUpdate(pu) => {
                        let mut sync_status = sync_status.lock().await;
                        if pu.synced == pu.total {
                            tracing::info!("wallet state up to date");
                            *sync_status = SyncStatus::UpToDate;
                        } else {
                            tracing::info!("progress update: {}/{}", pu.synced, pu.total);
                            *sync_status = SyncStatus::Syncing {
                                progress: (pu.synced / pu.total) * 100.0,
                            };
                        }

                        continue;
                    }
                };

                if skip_first {
                    skip_first = false;
                    continue;
                }

                let tx_hash = transaction.hash;

                let apply_stage = transaction.apply_stage;

                if apply_stage == "FailEntirely" {
                    continue;
                }

                let raw_tx = transaction.raw;

                let tx: Transaction<Proof> = deserialize::<Transaction<Proof>, _>(
                    std::io::Cursor::new(hex::decode(raw_tx).unwrap()),
                    network_id,
                )
                .unwrap();

                let current_coins = confirmed_state.coins.clone();

                let mut unconfirmed_state_guard = latest_state.lock().await;
                let mut unconfirmed_state = unconfirmed_state_guard.clone();

                tracing::info!("processing tx: {:#?}", &tx);

                match tx {
                    Transaction::Standard(stx) => {
                        confirmed_state = confirmed_state.apply(&stx.guaranteed_coins);
                        unconfirmed_state = unconfirmed_state.apply(&stx.guaranteed_coins);

                        if let Some(fallible_coins) = &stx.fallible_coins {
                            confirmed_state = confirmed_state.apply(fallible_coins);
                            unconfirmed_state = unconfirmed_state.apply(fallible_coins);
                        }
                    }
                    Transaction::ClaimMint(cmtx) => {
                        confirmed_state = confirmed_state.apply_mint(&cmtx.mint);
                        unconfirmed_state = unconfirmed_state.apply_mint(&cmtx.mint);
                    }
                }

                *unconfirmed_state_guard = unconfirmed_state;

                if current_coins == confirmed_state.coins {
                    continue;
                }

                db.persist_state(STABLE_STATE_ID, &tx_hash, &confirmed_state)?;

                dbg!(&confirmed_state.coins);
                // dbg!(&confirmed_state.merkle_tree);
                dbg!(&confirmed_state.merkle_tree.root());
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error: {}", e);
                panic!("{}", e);
            }
        }
    }

    Ok(())
}
