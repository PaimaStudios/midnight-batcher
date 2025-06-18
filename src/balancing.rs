use crate::{
    db::Db,
    endpoints::Error,
    midnight::{self},
    preproofing::{prove_tx_in_rayon_pool, PreProvingServiceChannelTx},
    utils::OnDrop,
    whitelisting::{self, check_call, check_deploy},
};
use midnight_ledger::structure::{Proof, ProofPreimage, Transaction, DUMMY_PARAMETERS};
use midnight_zswap::{
    coin_structure::{self, coin::NATIVE_TOKEN},
    keys::SecretKeys,
    local::State,
    serialize::{deserialize, serialize, NetworkId},
    storage::db::InMemoryDB,
    Offer, Output,
};
use rand::{rngs::OsRng, Rng as _};
use std::{cmp::Reverse, sync::Arc, time::Duration};
use subxt::{OnlineClient, SubstrateConfig};
use tokio::sync::Mutex;

#[allow(clippy::too_many_arguments)]
pub async fn balance_and_submit_tx(
    api: &OnlineClient<SubstrateConfig>,
    base_state: Arc<Mutex<State<InMemoryDB>>>,
    tx: &str,
    network_id: NetworkId,
    inputs_service: PreProvingServiceChannelTx,
    whitelisting: &Option<whitelisting::Constraints>,
    db: &Db,
    secret_keys: &SecretKeys,
) -> Result<(String, Vec<String>), Error> {
    // TODO: we should fetch this from the ledger state, but this works right now anyway.
    let parameters = DUMMY_PARAMETERS;

    let unbalanced_tx: Transaction<Proof, InMemoryDB> =
        deserialize(
            std::io::Cursor::new(hex::decode(tx).map_err(|_| {
                Error::BadRequest("Transaction payload is not valid hex".to_string())
            })?),
            network_id,
        )
        .map_err(|e| Error::BadRequest(format!("Invalid transaction. Error: {}", e)))?;

    tracing::trace!(?unbalanced_tx, "unbalanced transaction received");

    if let Some(constraints) = whitelisting {
        let is_call_to_known_contract = check_call(db, &unbalanced_tx, network_id).await?.is_some();

        if !is_call_to_known_contract {
            let deploy = check_deploy(constraints, &unbalanced_tx, network_id)?;

            if let Some(address) = &deploy {
                tracing::info!(address, "received new contract deploy");
            } else {
                return Err(Error::BadRequest("Transaction not allowed".to_string()));
            }
        }
    }

    let mut state_guard = base_state.lock().await;

    // TODO: this probably only works for a single input and a single output.
    //
    // figure out how to generalize
    let zswap_cost_estimation = 40500;
    let cost = unbalanced_tx
        .cost(&parameters)
        .map_err(|e| Error::InternalError(e.to_string()))?;

    let fees = cost + zswap_cost_estimation;

    let mut to_spend = vec![];
    let mut curr_balance = 0;

    let mut sorted_coins = state_guard
        .coins
        .iter()
        // we only need to pay fees, so we don't care about utxos for other assets
        .filter(|(_, coin)| coin.type_ == NATIVE_TOKEN)
        .filter(|(null, _)| !state_guard.pending_spends.contains_key(null))
        .collect::<Vec<_>>();

    // always pick the biggest unused utxo first, to spend evenly from the pool.
    sorted_coins.sort_by_key(|(_, coin)| Reverse(coin.value));

    for coin in sorted_coins {
        curr_balance += coin.1.value;
        to_spend.push(coin);

        if curr_balance >= fees {
            break;
        }
    }

    if curr_balance < fees {
        tracing::error!(
            curr_balance,
            fees,
            "not enough funds to balance transaction"
        );
        return Err(Error::NotAvailable("No funds available".to_string()));
    }

    let mut inputs = vec![];
    for coin in to_spend {
        // TODO: what does this mean?
        let segment = 0;
        let (new_state, input) = state_guard
            .spend(&mut OsRng, secret_keys, &coin.1, segment)
            .map_err(|e| Error::InternalError(e.to_string()))?;

        *state_guard = new_state;
        inputs.push(input);
    }

    let coin_public_key = secret_keys.coin_public_key();
    let enc_public_key = secret_keys.enc_public_key();

    std::mem::drop(state_guard);

    let mut on_drop_remove_inputs_from_pending = {
        let inputs = inputs.clone();
        OnDrop::new(move || {
            tokio::task::spawn(async move {
                let offer = Offer {
                    inputs,
                    outputs: vec![],
                    transient: vec![],
                    deltas: vec![],
                };
                let mut state = base_state.lock().await;
                *state = state.apply_failed(&offer);
            });
        })
    };

    let (inputs_tx, inputs_rx) = tokio::sync::oneshot::channel();
    inputs_service
        .send((
            inputs.iter().map(|input| input.nullifier).collect(),
            inputs_tx,
        ))
        .await
        .map_err(|e| Error::InternalError(e.to_string()))?;

    let proven_inputs = tokio::time::timeout(Duration::from_secs(30), inputs_rx)
        .await
        .map_err(|_e| {
            Error::InternalError(
                "Prover task never produced a proof for the selected input".to_string(),
            )
        })?
        // this one shouldn't really happen, but it's better to be panic free.
        .map_err(|_e| Error::InternalError("Failed to get prove for selected input".to_string()))?;

    let inputs_tx = proven_inputs
        .into_iter()
        .map(Ok)
        .reduce(|tx1, tx2| tx1?.merge(&tx2?))
        .ok_or_else(|| Error::InternalError("pre-computed proofs are empty".to_string()))?
        .map_err(|e| Error::InternalError(e.to_string()))?;

    let tx_ids = prove_and_submit(
        inputs_tx.clone(),
        curr_balance,
        fees,
        PublicKeys {
            coin_public_key,
            enc_public_key,
        },
        unbalanced_tx,
        network_id,
        api,
    )
    .await?;

    on_drop_remove_inputs_from_pending.cancel();

    Ok(tx_ids)
}

struct PublicKeys {
    coin_public_key: coin_structure::coin::PublicKey,
    enc_public_key: midnight_transient_crypto::encryption::PublicKey,
}

#[allow(clippy::too_many_arguments)]
async fn prove_and_submit(
    inputs_tx: Transaction<Proof, InMemoryDB>,
    curr_balance: u128,
    fees: u128,
    public_keys: PublicKeys,
    unbalanced_tx: Transaction<Proof, InMemoryDB>,
    network_id: NetworkId,
    api: &OnlineClient<SubstrateConfig>,
) -> Result<(String, Vec<String>), Error> {
    let value = curr_balance - fees;

    // TODO: what's this for?
    let segment = 0;
    let outputs_offer_tx = Offer {
        inputs: vec![],
        outputs: vec![Output::new(
            &mut OsRng,
            &coin_structure::coin::Info {
                nonce: OsRng.gen(),
                type_: NATIVE_TOKEN,
                value,
            },
            segment,
            &public_keys.coin_public_key,
            Some(public_keys.enc_public_key),
        )
        .map_err(|e| Error::InternalError(e.to_string()))?],
        transient: vec![],
        deltas: vec![(NATIVE_TOKEN, fees as i128)],
    };

    let outputs_tx: Transaction<ProofPreimage, InMemoryDB> =
        Transaction::new(outputs_offer_tx, None, None);

    let instant = std::time::Instant::now();

    let outputs_tx = prove_tx_in_rayon_pool(outputs_tx).await;

    tracing::info!(
        "proved outputs zswap in {} ms",
        instant.elapsed().as_millis()
    );

    let final_tx = inputs_tx
        .merge(&outputs_tx)
        .map_err(|e| Error::InternalError(e.to_string()))?
        .merge(&unbalanced_tx)
        .map_err(|e| Error::InternalError(e.to_string()))?;

    let mut serialized_final_tx = vec![];

    serialize(
        &final_tx,
        std::io::Cursor::new(&mut serialized_final_tx),
        network_id,
    )
    .map_err(|e| Error::InternalError(e.to_string()))?;

    let tx_hash = hex::encode(final_tx.transaction_hash().0 .0);

    let identifiers = final_tx
        .identifiers()
        .map(|id| {
            let mut buf = vec![];
            serialize(&id, std::io::Cursor::new(&mut buf), network_id).map_err(|error| {
                anyhow::anyhow!(
                    "Failed to serialize transaction identifier, reason: {}",
                    error
                )
            })?;
            Ok(hex::encode(buf))
        })
        .collect::<anyhow::Result<Vec<_>>>()
        .map_err(|e| Error::InternalError(e.to_string()))?;

    let hex_tx = hex::encode(serialized_final_tx);

    let extrinsic = midnight::tx()
        .midnight()
        .send_mn_transaction(hex_tx.as_bytes().to_vec());

    let client = api.tx();

    let submittable = client
        .create_unsigned(&extrinsic)
        .map_err(|e| Error::InternalError(e.to_string()))?;

    let watch = submittable
        .submit_and_watch()
        .await
        .map_err(|e| Error::InternalError(e.to_string()))?;

    let now = std::time::Instant::now();

    let in_tx_block = watch
        .wait_for_finalized()
        .await
        .map_err(|e| Error::InternalError(e.to_string()))?;

    tracing::info!(
        tx_hash,
        "transaction submitted and finalized, took {} ms",
        now.elapsed().as_millis()
    );

    let _result = in_tx_block
        .wait_for_success()
        .await
        .map_err(|e| Error::InternalError(e.to_string()))?;

    Ok((tx_hash, identifiers))
}
