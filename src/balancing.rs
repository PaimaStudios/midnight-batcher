use crate::{endpoints::Error, midnight};
use anyhow::Context as _;
use midnight_ledger::structure::{Transaction, DUMMY_PARAMETERS};
use midnight_transient_crypto::proofs::{
    IrSource, ParamsProver, Proof, ProofPreimage, ProverKey, VerifierKey,
};
use midnight_zswap::{
    coin_structure::{self, coin::NATIVE_TOKEN},
    local::State,
    serialize::{deserialize, serialize, NetworkId},
    Offer, Output,
};
use rand::{rngs::OsRng, Rng as _};
use std::{
    cmp::Reverse,
    fs::File,
    io::{BufReader, Cursor},
    sync::Arc,
};
use subxt::{OnlineClient, SubstrateConfig};
use tokio::sync::Mutex;

const OUTPUT_VK_RAW: &str = concat!(
    env!("MIDNIGHT_LEDGER_STATIC_DIR"),
    "/zswap/keys/output.verifier"
);

const OUTPUT_PK_RAW: &str = concat!(
    env!("MIDNIGHT_LEDGER_STATIC_DIR"),
    "/zswap/keys/output.prover"
);

const OUTPUT_IR_RAW: &str = concat!(
    env!("MIDNIGHT_LEDGER_STATIC_DIR"),
    "/zswap/zkir/output.zkir"
);

const SPEND_VK_RAW: &str = concat!(
    env!("MIDNIGHT_LEDGER_STATIC_DIR"),
    "/zswap/keys/spend.verifier"
);

const SPEND_PK_RAW: &str = concat!(
    env!("MIDNIGHT_LEDGER_STATIC_DIR"),
    "/zswap/keys/spend.prover"
);

const SPEND_IR_RAW: &str = concat!(env!("MIDNIGHT_LEDGER_STATIC_DIR"), "/zswap/zkir/spend.zkir");

const SIGN_VK_RAW: &str = concat!(
    env!("MIDNIGHT_LEDGER_STATIC_DIR"),
    "/zswap/keys/sign.verifier"
);

const SIGN_PK_RAW: &str = concat!(
    env!("MIDNIGHT_LEDGER_STATIC_DIR"),
    "/zswap/keys/sign.prover"
);

const SIGN_IR_RAW: &str = concat!(env!("MIDNIGHT_LEDGER_STATIC_DIR"), "/zswap/zkir/sign.zkir");

pub fn decode_zswap_proof_params(
    pk: Vec<u8>,
    vk: Vec<u8>,
    ir: Vec<u8>,
) -> anyhow::Result<(ProverKey, VerifierKey, IrSource)> {
    let pk = deserialize::<ProverKey, _>(Cursor::new(pk), NetworkId::TestNet)
        .context("Failed to read proving key")?;
    let vk = deserialize::<VerifierKey, _>(Cursor::new(vk), NetworkId::TestNet)
        .context("Failed to read verifying key")?;
    let ir = IrSource::load(Cursor::new(ir)).context("Failed to read ZKIR source")?;

    Ok((pk, vk, ir))
}

pub fn read_kzg_params() -> anyhow::Result<ParamsProver> {
    let pp = concat!(env!("MIDNIGHT_LEDGER_STATIC_DIR"), "/kzg");

    ParamsProver::read(BufReader::new(
        File::open(pp).expect("failed to read kzg params"),
    ))
    .context("Failed to read KZG params")
}

pub struct ProvingParams {
    pp: ParamsProver,
    spend: (ProverKey, VerifierKey, IrSource),
    output: (ProverKey, VerifierKey, IrSource),
    sign: (ProverKey, VerifierKey, IrSource),
}

impl ProvingParams {
    pub fn new() -> anyhow::Result<Self> {
        let pp = read_kzg_params()?;

        fn read_proof_params(path: &str) -> anyhow::Result<Vec<u8>> {
            std::fs::read(std::path::Path::new(path))
                .context(format!("Failed to read from {} into memory", path))
        }

        let spend = decode_zswap_proof_params(
            read_proof_params(SPEND_PK_RAW)?,
            read_proof_params(SPEND_VK_RAW)?,
            read_proof_params(SPEND_IR_RAW)?,
        )?;
        let output = decode_zswap_proof_params(
            read_proof_params(OUTPUT_PK_RAW)?,
            read_proof_params(OUTPUT_VK_RAW)?,
            read_proof_params(OUTPUT_IR_RAW)?,
        )?;
        let sign = decode_zswap_proof_params(
            read_proof_params(SIGN_PK_RAW)?,
            read_proof_params(SIGN_VK_RAW)?,
            read_proof_params(SIGN_IR_RAW)?,
        )?;

        Ok(ProvingParams {
            pp,
            spend,
            output,
            sign,
        })
    }
}

pub async fn balance_and_submit_tx(
    prover_params: &ProvingParams,
    api: &OnlineClient<SubstrateConfig>,
    base_state: Arc<Mutex<State>>,
    tx: &str,
    network_id: NetworkId,
) -> Result<String, Error> {
    // let mut state = base_state.clone();
    let mut state_guard = base_state.lock().await;

    // let ledger_state = midnight::apis().midnight_runtime_api().get_ledger_state();

    // let ledger_state = api
    //     .runtime_api()
    //     .at_latest()
    //     .await
    //     .unwrap()
    //     .call(ledger_state)
    //     .await
    //     .unwrap()
    //     .unwrap();

    // let ledger_state: LedgerState = deserialize(Cursor::new(ledger_state), network_id).unwrap();

    // dbg!(&ledger_state);

    // assert_eq!(&ledger_state.parameters, &DUMMY_PARAMETERS);
    //
    let parameters = DUMMY_PARAMETERS;

    let unbalanced_tx: Transaction<Proof> =
        deserialize(std::io::Cursor::new(hex::decode(tx).unwrap()), network_id).unwrap();

    tracing::trace!(?unbalanced_tx, "unbalanced transaction received");

    // TODO: this probably only works for a single input and a single output.
    //
    // figure out how to generalize
    let zswap_cost_estimation = 40000;
    let fees = unbalanced_tx.cost(&parameters).unwrap() + zswap_cost_estimation;

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

    dbg!(&sorted_coins);

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
        let (new_state, input) = state_guard
            .spend(&mut OsRng, &coin.1)
            .map_err(|e| Error::ServerError(e.to_string()))?;

        *state_guard = new_state;
        inputs.push(input);
    }

    let coin_public_key = state_guard.coin_public_key();
    let enc_public_key = state_guard.enc_public_key();

    std::mem::drop(state_guard);

    let inputs_offer = Offer {
        inputs,
        outputs: vec![],
        transient: vec![],
        deltas: vec![(NATIVE_TOKEN, curr_balance as i128)],
    };

    let tx_hash = prove_and_submit(
        inputs_offer.clone(),
        curr_balance,
        fees,
        prover_params,
        PublicKeys {
            coin_public_key,
            enc_public_key,
        },
        unbalanced_tx,
        network_id,
        api,
    )
    .await;

    if tx_hash.is_err() {
        base_state.lock().await.apply_failed(&inputs_offer);
    }

    // dbg!(result);

    tx_hash
}

struct PublicKeys {
    coin_public_key: coin_structure::coin::PublicKey,
    enc_public_key: midnight_transient_crypto::encryption::PublicKey,
}

async fn prove_and_submit(
    inputs_offer: Offer<ProofPreimage>,
    curr_balance: u128,
    fees: u128,
    prover_params: &ProvingParams,
    public_keys: PublicKeys,
    unbalanced_tx: Transaction<Proof>,
    network_id: NetworkId,
    api: &OnlineClient<SubstrateConfig>,
) -> Result<String, Error> {
    let inputs_tx = Transaction::new(inputs_offer, None, None);

    let instant = std::time::Instant::now();
    let inputs_tx = inputs_tx
        .prove(OsRng, &prover_params.pp, |loc| match &*loc.0 {
            "midnight/zswap/spend" => Some(prover_params.spend.clone()),
            "midnight/zswap/output" => Some(prover_params.output.clone()),
            "midnight/zswap/sign" => Some(prover_params.sign.clone()),
            _ => unreachable!("this transaction does not have contract calls"),
        })
        .await
        .map_err(|e| Error::BadRequest(format!("Invalid transaction {}", e)))?;

    tracing::info!(
        "proved inputs zswap in {} ms",
        instant.elapsed().as_millis()
    );

    let value = curr_balance - fees;

    let outputs_offer_tx = Offer {
        inputs: vec![],
        outputs: vec![Output::new(
            &mut OsRng,
            &coin_structure::coin::Info {
                nonce: OsRng.gen(),
                type_: NATIVE_TOKEN,
                value,
            },
            &public_keys.coin_public_key,
            Some(public_keys.enc_public_key),
        )
        .map_err(|e| Error::ServerError(e.to_string()))?],
        transient: vec![],
        deltas: vec![(NATIVE_TOKEN, -(value as i128))],
    };
    let outputs_tx = Transaction::new(outputs_offer_tx, None, None);

    let instant = std::time::Instant::now();

    let outputs_tx = outputs_tx
        .prove(OsRng, &prover_params.pp, |loc| match &*loc.0 {
            "midnight/zswap/spend" => Some(prover_params.spend.clone()),
            "midnight/zswap/output" => Some(prover_params.output.clone()),
            "midnight/zswap/sign" => Some(prover_params.sign.clone()),
            _ => unreachable!("this transaction does not have contract calls"),
        })
        .await
        .map_err(|e| Error::BadRequest(format!("Invalid transaction {}", e)))?;

    tracing::info!(
        "proved outputs zswap in {} ms",
        instant.elapsed().as_millis()
    );

    let final_tx = inputs_tx
        .merge(&outputs_tx)
        .map_err(|e| Error::ServerError(e.to_string()))?
        .merge(&unbalanced_tx)
        .map_err(|e| Error::ServerError(e.to_string()))?;

    tracing::debug!(?final_tx, "merged transactions");

    let mut serialized_final_tx = vec![];

    serialize(
        &final_tx,
        std::io::Cursor::new(&mut serialized_final_tx),
        network_id,
    )
    .map_err(|e| Error::ServerError(e.to_string()))?;

    let tx_hash = hex::encode(final_tx.transaction_hash().0 .0);
    let hex_tx = hex::encode(serialized_final_tx);

    let extrinsic = midnight::tx()
        .midnight()
        .send_mn_transaction(hex_tx.as_bytes().to_vec());
    let client = api.tx();

    let submittable = client
        .create_unsigned(&extrinsic)
        .map_err(|e| Error::ServerError(e.to_string()))?;

    let watch = submittable
        .submit_and_watch()
        .await
        .map_err(|e| Error::ServerError(e.to_string()))?;

    tracing::info!(tx_hash, "submitting transaction");

    let result = watch
        .wait_for_finalized_success()
        .await
        .map_err(|e| Error::ServerError(e.to_string()))?;
    tracing::info!(tx_hash, "transaction submitted");

    Ok(tx_hash)
}
