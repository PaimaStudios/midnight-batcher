use crate::{
    balancing::{balance_and_submit_tx, ProvingParams},
    SyncStatus,
};
use midnight_zswap::serialize::NetworkId;
use rocket::{
    http::{Method, Status},
    serde::json::Json,
    State,
};
use rocket_cors::{AllowedOrigins, CorsOptions};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use subxt::{OnlineClient, SubstrateConfig};
use tokio::sync::Mutex;

struct AppState {
    proving_params: Arc<ProvingParams>,
    zswap_state: Arc<Mutex<midnight_zswap::local::State>>,
    network_id: NetworkId,
    sync_status: Arc<Mutex<SyncStatus>>,
    api: OnlineClient<SubstrateConfig>,
}

#[derive(Deserialize)]
struct Transaction {
    tx: String,
}

#[derive(Serialize)]
struct Response {
    tx_hash: String,
}

#[derive(Responder)]
pub enum Error {
    #[response(status = 400)]
    BadRequest(String),
    #[response(status = 500)]
    ServerError(String),
    #[response(status = 503)]
    NotAvailable(String),
}

#[post("/submitTx", format = "json", data = "<transaction>")]
async fn submit_tx(
    transaction: Json<Transaction>,
    state: &State<AppState>,
) -> Result<Json<Response>, Error> {
    let sync_status = state.sync_status.lock().await;

    match *sync_status {
        SyncStatus::Syncing { progress: _ } => {
            return Err(Error::NotAvailable("Wallet not in sync".to_string()))
        }
        SyncStatus::UpToDate => {}
    }

    let mut zswap_state = state.zswap_state.lock().await;

    let (new_state, tx_hash) = balance_and_submit_tx(
        &state.proving_params,
        &state.api,
        &zswap_state,
        &transaction.tx,
        state.network_id,
    )
    .await?;

    *zswap_state = new_state;
    Ok(Json(Response { tx_hash }))
}

pub fn rocket(
    prover_params: ProvingParams,
    api: OnlineClient<SubstrateConfig>,
    zswap_state: Arc<Mutex<midnight_zswap::local::State>>,
    network_id: NetworkId,
    sync_status: Arc<Mutex<SyncStatus>>,
) -> rocket::Rocket<rocket::Build> {
    let state = AppState {
        proving_params: Arc::new(prover_params),
        api,
        zswap_state,
        network_id,
        sync_status,
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

    rocket::build()
        .manage(state)
        .mount("/", routes![submit_tx])
        .attach(cors.to_cors().unwrap())
}
