use crate::balancing::{balance_and_submit_tx, ProvingParams};
use midnight_zswap::serialize::NetworkId;
use rocket::{response::status::BadRequest, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

struct AppState {
    proving_params: Arc<ProvingParams>,
    zswap_state: Arc<Mutex<midnight_zswap::local::State>>,
    network_id: NetworkId,
}

#[derive(Deserialize)]
struct Transaction {
    tx: String,
}

#[derive(Serialize)]
struct Response {
    tx_hash: String,
}

// Create the endpoint
#[post("/submitTx", format = "json", data = "<transaction>")]
async fn submit_tx(
    transaction: Json<Transaction>,
    state: &State<AppState>,
) -> Result<Json<Response>, BadRequest<()>> {
    let mut zswap_state = state.zswap_state.lock().await;

    let result = balance_and_submit_tx(
        &state.proving_params,
        &mut zswap_state,
        &transaction.tx,
        state.network_id,
    )
    .await;

    match result {
        Ok((new_state, tx_hash)) => {
            *zswap_state = new_state;
            Ok(Json(Response { tx_hash }))
        }
        Err(_) => Err(BadRequest(())),
    }
}

pub fn rocket(
    prover_params: ProvingParams,
    zswap_state: Arc<Mutex<midnight_zswap::local::State>>,
    network_id: NetworkId,
) -> rocket::Rocket<rocket::Build> {
    let state = AppState {
        proving_params: Arc::new(prover_params),
        zswap_state,
        network_id,
    };

    rocket::build().manage(state).mount("/", routes![submit_tx])
}
