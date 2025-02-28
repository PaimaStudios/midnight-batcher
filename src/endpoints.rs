use crate::{
    balancing::{balance_and_submit_tx, ProvingParams},
    db::Db,
    preproofing::PreProvingServiceChannelTx,
    whitelisting, SyncStatus,
};
use midnight_zswap::serialize::{self, NetworkId};
use rand::{rngs::OsRng, Rng};
use rocket::{
    figment::{
        providers::{Env, Format as _, Toml},
        Figment,
    },
    http::Method,
    serde::json::Json,
    State,
};
use rocket_cors::{AllowedOrigins, CorsOptions};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use subxt::{OnlineClient, SubstrateConfig};
use tokio::sync::Mutex;
use tracing::Instrument as _;

struct AppState {
    proving_params: Arc<ProvingParams>,
    zswap_state: Arc<Mutex<midnight_zswap::local::State>>,
    network_id: NetworkId,
    sync_status: Arc<Mutex<SyncStatus>>,
    api: OnlineClient<SubstrateConfig>,
    inputs_service: PreProvingServiceChannelTx,
    whitelisting: Arc<Option<whitelisting::Constraints>>,
    db: Db,
    address: String,
}

#[derive(Deserialize)]
struct Transaction {
    tx: String,
}

#[derive(Serialize)]
struct SubmitTxResponse {
    tx_hash: String,
    identifiers: Vec<String>,
}

#[derive(Serialize)]
struct GetFundsResponse {
    coins: Vec<(String, String)>,
    pending: Vec<String>,
}

#[derive(Serialize)]
struct OpenLobby {
    address: String,
    block_height: u64,
    p1_public_key: String,
}

#[derive(Serialize)]
#[serde(transparent)]
struct GetOpenLobbiesResponse(Vec<OpenLobby>);

#[derive(Serialize)]
struct PlayerLobby {
    address: String,
    state: String,
    block_height: u64,
    p1_public_key: String,
    p2_public_key: Option<String>,
}

#[derive(Serialize)]
#[serde(transparent)]
struct GetPlayerLobbiesResponse(Vec<PlayerLobby>);

#[derive(Responder)]
pub enum Error {
    #[response(status = 400)]
    BadRequest(String),
    #[response(status = 500)]
    #[allow(clippy::enum_variant_names)]
    InternalError(String),
    #[response(status = 503)]
    NotAvailable(String),
}

impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        Self::InternalError(value.to_string())
    }
}

#[post("/submitTx", format = "json", data = "<transaction>")]
async fn submit_tx(
    transaction: Json<Transaction>,
    state: &State<AppState>,
) -> Result<Json<SubmitTxResponse>, Error> {
    let span_id: u128 = OsRng.gen();
    let span = tracing::info_span!("submit_tx handler", span_id);

    let sync_status = state.sync_status.lock().await;

    match *sync_status {
        SyncStatus::Syncing {
            progress: _,
            notify: _,
        } => return Err(Error::NotAvailable("Wallet not in sync".to_string())),
        SyncStatus::UpToDate => {}
    }

    let now = std::time::Instant::now();

    let (tx_hash, identifiers) = balance_and_submit_tx(
        &state.proving_params,
        &state.api,
        Arc::clone(&state.zswap_state),
        &transaction.tx,
        state.network_id,
        state.inputs_service.clone(),
        &state.whitelisting,
        &state.db,
    )
    .instrument(span.clone())
    .await?;

    span.in_scope(|| {
        tracing::info!(
            "submit_tx handler took in: {} ms",
            now.elapsed().as_millis()
        );
    });

    Ok(Json(SubmitTxResponse {
        tx_hash,
        identifiers,
    }))
}

#[get("/funds")]
async fn funds(state: &State<AppState>) -> Result<Json<GetFundsResponse>, Error> {
    let sync_status = state.sync_status.lock().await;

    match *sync_status {
        SyncStatus::Syncing {
            progress: _,
            notify: _,
        } => return Err(Error::NotAvailable("Wallet not in sync".to_string())),
        SyncStatus::UpToDate => {}
    };

    let lock = state.zswap_state.lock().await;

    let coins = lock
        .coins
        .iter()
        .map(|(nul, coin)| {
            let mut buf = vec![];
            serialize::serialize(&nul, &mut buf, state.network_id).unwrap();
            (hex::encode(buf), coin.value.to_string())
        })
        .collect();

    let pending = lock
        .pending_spends
        .iter()
        .map(|(nul, _)| {
            let mut buf = vec![];
            serialize::serialize(&nul, &mut buf, state.network_id).unwrap();
            hex::encode(buf)
        })
        .collect();

    Ok(Json(GetFundsResponse { coins, pending }))
}

#[get("/address")]
async fn address(state: &State<AppState>) -> String {
    state.address.clone()
}

#[get("/lobbies/open?<after>&<count>")]
async fn get_open_lobbies(
    state: &State<AppState>,
    after: Option<String>,
    count: Option<u8>,
) -> Result<Json<GetOpenLobbiesResponse>, Error> {
    let sync_status = state.sync_status.lock().await;

    match *sync_status {
        SyncStatus::Syncing {
            progress: _,
            notify: _,
        } => return Err(Error::NotAvailable("Wallet not in sync".to_string())),
        SyncStatus::UpToDate => {}
    };

    let lobbies = state.db.get_lobbies_waiting_for_p2(after, count).await;

    match lobbies {
        Ok(lobbies) => Ok(Json(GetOpenLobbiesResponse(
            lobbies
                .into_iter()
                .map(|(address, block_height, p1_public_key)| OpenLobby {
                    // for some reason the game expects the address without the network prefix
                    address: address[2..].to_string(),
                    block_height,
                    p1_public_key,
                })
                .collect::<Vec<_>>(),
        ))),
        Err(error) => Err(Error::InternalError(error.to_string())),
    }
}

#[get("/lobbies/player/<player_id>?<after>&<count>")]
async fn get_player_lobbies(
    state: &State<AppState>,
    player_id: String,
    after: Option<String>,
    count: Option<u8>,
) -> Result<Json<GetPlayerLobbiesResponse>, Error> {
    let sync_status = state.sync_status.lock().await;

    match *sync_status {
        SyncStatus::Syncing {
            progress: _,
            notify: _,
        } => return Err(Error::NotAvailable("Wallet not in sync".to_string())),
        SyncStatus::UpToDate => {}
    };

    let lobbies = state.db.get_player_lobbies(player_id, count, after).await;

    match lobbies {
        Ok(lobbies) => Ok(Json(GetPlayerLobbiesResponse(
            lobbies
                .into_iter()
                .map(
                    |(address, state, block_height, p1_public_key, p2_public_key)| PlayerLobby {
                        // for some reason the game expects the address without the network prefix
                        address: address[2..].to_string(),
                        state,
                        block_height,
                        p1_public_key,
                        p2_public_key: p2_public_key
                            .split(";")
                            .nth(1)
                            .map(|s| s.to_string())
                            .filter(|s| !s.is_empty()),
                    },
                )
                .collect::<Vec<_>>(),
        ))),
        Err(error) => Err(Error::InternalError(error.to_string())),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn rocket(
    prover_params: Arc<ProvingParams>,
    api: OnlineClient<SubstrateConfig>,
    zswap_state: Arc<Mutex<midnight_zswap::local::State>>,
    network_id: NetworkId,
    sync_status: Arc<Mutex<SyncStatus>>,
    inputs_service: PreProvingServiceChannelTx,
    whitelisting: Option<whitelisting::Constraints>,
    db: Db,
    address: String,
) -> rocket::Rocket<rocket::Build> {
    let state = AppState {
        proving_params: prover_params,
        api,
        zswap_state,
        network_id,
        sync_status,
        inputs_service,
        whitelisting: Arc::new(whitelisting),
        db,
        address,
    };

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Patch]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true);

    let figment = Figment::from(rocket::Config::default())
        .merge(Toml::file("batcher-config.toml").nested())
        .merge(Env::prefixed("PAIMA_MIDNIGHT_BATCHER_").global());

    rocket::custom(figment)
        .manage(state)
        .mount(
            "/",
            routes![
                submit_tx,
                funds,
                address,
                get_open_lobbies,
                get_player_lobbies
            ],
        )
        .attach(cors.to_cors().unwrap())
}
