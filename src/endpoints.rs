use crate::{
    balancing::{balance_and_submit_tx, ProvingParams},
    db::Db,
    preproofing::PreProvingServiceChannelTx,
    whitelisting, SyncStatus,
};
use midnight_zswap::serialize::NetworkId;
use rand::{rngs::OsRng, Rng};
use rocket::{http::Method, serde::json::Json, State};
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
}

#[derive(Deserialize)]
struct Transaction {
    tx: String,
}

#[derive(Serialize)]
struct Response {
    tx_hash: String,
    identifiers: Vec<String>,
}

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
) -> Result<Json<Response>, Error> {
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

    Ok(Json(Response {
        tx_hash,
        identifiers,
    }))
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
