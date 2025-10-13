use crate::{
    db::Db,
    endpoints::Error,
    midnight::{self},
    proving::prove_tx_in_rayon_pool,
    utils::{get_current_time, OnDrop},
    whitelisting::{self, check_call, check_deploy},
    NetworkId, WalletState,
};
use base_crypto::signatures::Signature;
use coin_structure::coin::TokenType;
use ledger_storage::{
    arena::Sp,
    db::InMemoryDB,
    storage::{Array, HashMap},
};
use midnight_serialize::{tagged_deserialize, tagged_serialize};
use mn_ledger::{
    dust::{DustActions, DustOutput, DustSecretKey},
    structure::{Intent, LedgerParameters, ProofKind, ProofMarker, Transaction},
};
use rand::rngs::OsRng;
use subxt::{OnlineClient, SubstrateConfig};
use transient_crypto::commitment::PedersenRandomness;

// Intent TTL for Dust spend
// (chosen arbitrarily)
const DUST_INTENT_TTL: i128 = 60;

#[allow(clippy::too_many_arguments)]
pub async fn balance_and_submit_tx(
    api: &OnlineClient<SubstrateConfig>,
    wallet_state: WalletState,
    tx: &str,
    whitelisting: &Option<whitelisting::Constraints>,
    db: &Db,
    dust_secret_key: &DustSecretKey,
    network_id: &NetworkId,
) -> Result<(String, Vec<String>), Error> {
    let ledger_parameters = db
        .get_ledger_parameters(crate::LEDGER_PARAMETERS_DB_ID)
        .await?;

    let unbalanced_tx: Transaction<Signature, ProofMarker, PedersenRandomness, InMemoryDB> =
        tagged_deserialize(std::io::Cursor::new(hex::decode(tx).map_err(|_| {
            Error::BadRequest("Transaction payload is not valid hex".to_string())
        })?))
        .map_err(|e| {
            tracing::debug!(?e, "Invalid transaction");
            Error::BadRequest(format!("Invalid transaction. Error: {e}"))
        })?;

    if let Some(constraints) = whitelisting {
        let is_call_to_known_contract = check_call(db, &unbalanced_tx).await?.is_some();

        if !is_call_to_known_contract {
            let deploy = check_deploy(constraints, &unbalanced_tx)?;

            if let Some(address) = &deploy {
                db.insert_contract_address(address).await?;

                tracing::info!(address, "received new contract deploy");
            } else {
                return Err(Error::BadRequest("Transaction not allowed".to_string()));
            }
        }
    }

    // we don't keep the guard here
    //
    // concurrency is controlled through the wallet_state.locked mutex instead.
    //
    // similarly, we don't update the dust_state in this function
    //
    // instead, only the subscription loop in main can update the state
    let mut old_dust = wallet_state.dust_state.lock().await.clone();

    let mut merged_tx = unbalanced_tx;

    let current_time = get_current_time();

    old_dust = old_dust.process_ttls(current_time);

    let mut drop_guards = vec![];

    let segment = 10;

    let mut locked_in_this_tx = std::collections::HashSet::new();
    let mut balanced_unproven_tx = None;

    let mut prev_dust = 0;

    while let Some(mut dust_missing) = get_missing_dust(&ledger_parameters, &merged_tx)? {
        dust_missing += prev_dust;

        tracing::info!(
            "Need to cover {dust_missing} specks of Dust. Adding {prev_dust} for the balancing utxo fees. Wallet balance: {}",
            old_dust.wallet_balance(current_time)
        );

        let mut spends = Array::new();
        let mut updated_dust = old_dust.clone();

        for utxo in old_dust.utxos() {
            let mut pending = wallet_state.locked.lock().await;

            if pending.contains_key(&utxo.backing_night.0)
                && !locked_in_this_tx.contains(&utxo.backing_night.0)
            {
                continue;
            }

            let gen_info = old_dust.generation_info(&utxo).ok_or_else(|| {
                Error::InternalError("dust generation information not found".to_string())
            })?;

            let value = DustOutput::from(utxo).updated_value(
                &gen_info,
                current_time,
                &ledger_parameters.dust,
            );

            let v_fee = u128::min(value, dust_missing);

            dust_missing = dust_missing.saturating_sub(value);

            let (new_dust, spend) = updated_dust
                .spend(dust_secret_key, &utxo, v_fee, current_time)
                .map_err(|e| Error::InternalError(e.to_string()))?;

            updated_dust = new_dust;

            pending.insert(utxo.backing_night.0, spend.old_nullifier);
            locked_in_this_tx.insert(utxo.backing_night.0);

            spends = spends.push(spend);

            {
                let wallet_state = wallet_state.clone();
                drop_guards.push(OnDrop::new(move || {
                    tokio::task::spawn(async move {
                        wallet_state
                            .locked
                            .lock()
                            .await
                            .remove(&utxo.backing_night.0);
                    });
                }));
            }

            if dust_missing == 0 {
                break;
            }
        }

        // old_dust = updated_dust;

        if dust_missing > 0 {
            return Err(Error::NotAvailable(
                "not enough funds to balance transaction".to_string(),
            ));
        }

        let mut intent = Intent::empty(
            &mut OsRng,
            // the parameter is called ttl, but it's a timestamp
            current_time + base_crypto::time::Duration::from_secs(DUST_INTENT_TTL),
        );
        intent.dust_actions = Some(Sp::new(DustActions {
            spends,
            registrations: vec![].into(),
            ctime: current_time,
        }));

        let intents = HashMap::new().insert(segment, intent);
        let tx2_unproven = Transaction::from_intents(network_id, intents);

        if let Some(missing_dust) = get_missing_dust(&ledger_parameters, &tx2_unproven)? {
            prev_dust += missing_dust;
            continue;
        } else {
            balanced_unproven_tx.replace(tx2_unproven);
            break;
        }
    }

    let new_tx = prove_tx_in_rayon_pool(balanced_unproven_tx.unwrap()).await;

    merged_tx = merged_tx
        .merge(&new_tx)
        .map_err(|e| Error::InternalError(e.to_string()))?;

    let final_tx = merged_tx.seal(OsRng);

    submit_tx(api, merged_tx, drop_guards, final_tx).await
}

fn get_missing_dust<P: ProofKind<InMemoryDB>>(
    ledger_parameters: &LedgerParameters,
    merged_tx: &Transaction<Signature, P, transient_crypto::curve::EmbeddedFr, InMemoryDB>,
) -> Result<Option<u128>, Error> {
    Ok(merged_tx
        .balance(Some(
            merged_tx
                .fees(ledger_parameters, true)
                .map_err(|e| Error::InternalError(e.to_string()))?,
        ))
        .map_err(|e| Error::InternalError(e.to_string()))?
        .get(&(TokenType::Dust, 0))
        .and_then(|bal| (*bal < 0).then_some((-*bal) as u128)))
}

async fn submit_tx(
    api: &OnlineClient<SubstrateConfig>,
    merged_tx: Transaction<Signature, ProofMarker, transient_crypto::curve::EmbeddedFr, InMemoryDB>,
    drop_guards: Vec<OnDrop<impl FnOnce()>>,
    final_tx: Transaction<
        Signature,
        ProofMarker,
        transient_crypto::commitment::PureGeneratorPedersen,
        InMemoryDB,
    >,
) -> Result<(String, Vec<String>), Error> {
    let mut serialized_final_tx = vec![];

    tagged_serialize(&final_tx, std::io::Cursor::new(&mut serialized_final_tx))
        .map_err(|e| Error::InternalError(e.to_string()))?;

    let tx_hash = hex::encode(merged_tx.transaction_hash().0 .0);

    let identifiers = merged_tx
        .identifiers()
        .map(|id| {
            let mut buf = vec![];
            tagged_serialize(&id, std::io::Cursor::new(&mut buf)).map_err(|error| {
                anyhow::anyhow!("Failed to serialize transaction identifier, reason: {error}",)
            })?;
            Ok(hex::encode(buf))
        })
        .collect::<anyhow::Result<Vec<_>>>()
        .map_err(|e| Error::InternalError(e.to_string()))?;

    tracing::info!("identifiers: {:?}", &identifiers);

    let extrinsic = midnight::tx()
        .midnight()
        .send_mn_transaction(serialized_final_tx);

    let client = api.tx();

    let submittable = client
        .create_unsigned(&extrinsic)
        .map_err(|e| Error::InternalError(e.to_string()))?;

    tracing::trace!("submitting tx");

    let watch = submittable
        .submit_and_watch()
        .await
        .map_err(|e| Error::InternalError(e.to_string()))?;

    let now = std::time::Instant::now();

    tracing::trace!("waiting for finalization");

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

    for mut drop_guard in drop_guards {
        drop_guard.cancel();
    }

    Ok((tx_hash, identifiers))
}
