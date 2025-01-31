#[macro_use]
extern crate rocket;

mod balancing;
mod db;
mod endpoints;

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

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let matches = Command::new("Midnight Batcher")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("A CLI application that accepts multiple URLs, a file path, and a user wallet.")
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
    let file = matches.get_one::<String>("node").expect("default");
    let credentials = matches.get_one::<PathBuf>("secret").expect("default");
    let network = matches.get_one::<String>("network").expect("default");

    // Print out the arguments (for demonstration purposes)
    println!("URLs: {:?}", indexer);
    println!("File path: {:?}", file);
    println!("Wallet: {:?}", credentials);
    println!("Network: {}", network);

    let network_id = match network.as_ref() {
        "testnet" => NetworkId::TestNet,
        "undeployed" => NetworkId::Undeployed,
        _ => anyhow::bail!("invalid network"),
    };

    let url = Url::parse(INDEXER_LOCALHOST).expect("Invalid indexer URL");

    let db = Db::open_db("db.sqlite", network_id)?;

    let mut rng = ChaCha20Rng::from_seed(
        <[u8; 32]>::try_from(
            hex::decode("22ec1dd24c6c52218632bf178df6ab5ed124bfb31b68d64b51572f46999e5e9c")
                .unwrap(),
        )
        .unwrap(),
    );

    let maybe_latest_state = db.get_state(STABLE_STATE_ID)?;

    let initial_state = maybe_latest_state
        .as_ref()
        .map(|(_, state)| state.clone())
        .unwrap_or_else(|| State::new(&mut rng));

    let current_tx = maybe_latest_state.map(|(hash, _)| hash);

    tokio::task::spawn(wallet_indexer(
        db,
        url,
        initial_state.clone(),
        current_tx,
        network_id,
    ));

    let proving_params = ProvingParams::new();

    endpoints::rocket(
        proving_params,
        Arc::new(Mutex::new(initial_state)),
        network_id,
    )
    .launch()
    .await
    .unwrap();

    Ok(())
}

async fn wallet_indexer(
    db: Db,
    url: Url,
    latest_state: State,
    current_tx: Option<String>,
    network_id: NetworkId,
) -> anyhow::Result<()> {
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

    // Listen for messages from the server.
    let message = read.next().await.unwrap().unwrap();

    if let tungstenite::Message::Text(text) = message {
        println!("Received?: {}", text);
    }

    let message = read.next().await.unwrap().unwrap();
    dbg!(message);

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

    let (subscription_query, mut zswap_state, mut skip_first) = if let Some(txhash) = current_tx {
        (subscription_query(Some(txhash)), latest_state, true)
    } else {
        (subscription_query(None), latest_state, false)
    };

    write
        .send(tungstenite::Message::Text(subscription_query.to_string()))
        .await
        .expect("Failed to send message");

    let pk = zswap_state.coin_public_key();
    let epk = zswap_state.enc_public_key();

    let pk_hex = hex::encode(pk.0 .0);

    let mut buf = vec![];
    serialize(&epk, &mut buf, network_id).unwrap();
    let ec_hex = hex::encode(&buf[1..]);

    // println!("Address: {}|{}", pk_hex, ec_hex);

    info!("Batcher address {}|{}", pk_hex, ec_hex);

    // panic!();

    // let mut tx_counter = 0;

    // Listen for messages from the server.
    while let Some(message) = read.next().await {
        match message {
            Ok(tungstenite::Message::Text(text)) => {
                // println!("Received: {}", text);
                //
                // tx_counter += 1;

                // dbg!(tx_counter);

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
                    gql::TransactionOrUpdate::ProgressUpdate(_) => continue,
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

                let current_coins = zswap_state.coins.clone();

                match dbg!(tx) {
                    Transaction::Standard(stx) => {
                        zswap_state = zswap_state.apply(&stx.guaranteed_coins);
                        if let Some(fallible_coins) = &stx.fallible_coins {
                            zswap_state = zswap_state.apply(fallible_coins);
                        }
                    }
                    Transaction::ClaimMint(_) => todo!(),
                }

                if current_coins == zswap_state.coins {
                    continue;
                }

                db.persist_state(STABLE_STATE_ID, &tx_hash, &zswap_state)?;

                dbg!(&zswap_state.coins);
                // dbg!(&zswap_state.merkle_tree);
                dbg!(&zswap_state.merkle_tree.root());
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
