use crate::midnight;
use midnight_ledger::{
    structure::{LedgerState, Transaction, DUMMY_PARAMETERS},
    verify::WellFormedStrictness,
};
use midnight_transient_crypto::proofs::{IrSource, ParamsProver, Proof, ProverKey, VerifierKey};
use midnight_zswap::{
    coin_structure::{self, coin::NATIVE_TOKEN},
    local::State,
    serialize::{deserialize, serialize, NetworkId},
    Offer, Output,
};
use rand::{rngs::OsRng, Rng as _};
use std::{
    fs::File,
    io::{BufReader, Cursor},
};
use subxt::{OnlineClient, SubstrateConfig};

const OUTPUT_VK_RAW: &[u8] = include_bytes!(concat!(
    env!("MIDNIGHT_LEDGER_STATIC_DIR"),
    "/zswap/keys/output.verifier"
));

const OUTPUT_PK_RAW: &[u8] = include_bytes!(concat!(
    env!("MIDNIGHT_LEDGER_STATIC_DIR"),
    "/zswap/keys/output.prover"
));

const OUTPUT_IR_RAW: &[u8] = include_bytes!(concat!(
    env!("MIDNIGHT_LEDGER_STATIC_DIR"),
    "/zswap/zkir/output.zkir"
));

const SPEND_VK_RAW: &[u8] = include_bytes!(concat!(
    env!("MIDNIGHT_LEDGER_STATIC_DIR"),
    "/zswap/keys/spend.verifier"
));

const SPEND_PK_RAW: &[u8] = include_bytes!(concat!(
    env!("MIDNIGHT_LEDGER_STATIC_DIR"),
    "/zswap/keys/spend.prover"
));

const SPEND_IR_RAW: &[u8] = include_bytes!(concat!(
    env!("MIDNIGHT_LEDGER_STATIC_DIR"),
    "/zswap/zkir/spend.zkir"
));

const SIGN_VK_RAW: &[u8] = include_bytes!(concat!(
    env!("MIDNIGHT_LEDGER_STATIC_DIR"),
    "/zswap/keys/sign.verifier"
));

const SIGN_PK_RAW: &[u8] = include_bytes!(concat!(
    env!("MIDNIGHT_LEDGER_STATIC_DIR"),
    "/zswap/keys/sign.prover"
));

const SIGN_IR_RAW: &[u8] = include_bytes!(concat!(
    env!("MIDNIGHT_LEDGER_STATIC_DIR"),
    "/zswap/zkir/sign.zkir"
));

pub fn decode_zswap_proof_params(
    pk: &[u8],
    vk: &[u8],
    ir: &[u8],
) -> (ProverKey, VerifierKey, IrSource) {
    let pk = deserialize::<ProverKey, _>(Cursor::new(pk), NetworkId::TestNet).unwrap();
    let vk = deserialize::<VerifierKey, _>(Cursor::new(vk), NetworkId::TestNet).unwrap();
    let ir = IrSource::load(Cursor::new(ir)).unwrap();

    (pk, vk, ir)
}

pub fn read_kzg_params() -> ParamsProver {
    let pp = concat!(env!("MIDNIGHT_LEDGER_STATIC_DIR"), "/kzg");

    ParamsProver::read(BufReader::new(File::open(pp).expect(
        "kzg params not found, run: cargo run --bin make_params to generate new ones",
    )))
    .unwrap()
}

pub struct ProvingParams {
    pp: ParamsProver,
    spend: (ProverKey, VerifierKey, IrSource),
    output: (ProverKey, VerifierKey, IrSource),
    sign: (ProverKey, VerifierKey, IrSource),
}

impl ProvingParams {
    pub fn new() -> Self {
        let pp = read_kzg_params();

        let spend = decode_zswap_proof_params(SPEND_PK_RAW, SPEND_VK_RAW, SPEND_IR_RAW);
        let output = decode_zswap_proof_params(OUTPUT_PK_RAW, OUTPUT_VK_RAW, OUTPUT_IR_RAW);
        let sign = decode_zswap_proof_params(SIGN_PK_RAW, SIGN_VK_RAW, SIGN_IR_RAW);

        ProvingParams {
            pp,
            spend,
            output,
            sign,
        }
    }
}

pub async fn balance_and_submit_tx(
    prover_params: &ProvingParams,
    base_state: &State,
    tx: &str,
    network_id: NetworkId,
) -> anyhow::Result<(State, String)> {
    let mut state = base_state.clone();

    let api = OnlineClient::<SubstrateConfig>::from_url("ws://127.0.0.1:9944")
        .await
        .unwrap();

    let ledger_state = midnight::apis().midnight_runtime_api().get_ledger_state();

    let ledger_state = api
        .runtime_api()
        .at_latest()
        .await
        .unwrap()
        .call(ledger_state)
        .await
        .unwrap()
        .unwrap();

    let ledger_state: LedgerState = deserialize(Cursor::new(ledger_state), network_id).unwrap();

    dbg!(&ledger_state);

    assert_eq!(&ledger_state.parameters, &DUMMY_PARAMETERS);

    let unbalanced_tx: Transaction<Proof> =
        deserialize(std::io::Cursor::new(hex::decode(tx).unwrap()), network_id).unwrap();

    dbg!(&unbalanced_tx);

    let fees = unbalanced_tx.cost(&ledger_state.parameters).unwrap() + 45000;

    let mut to_spend = vec![];
    let mut curr_balance = 0;
    for coin in state.coins.iter() {
        if state.pending_spends.contains_key(&coin.0) {
            continue;
        }

        curr_balance += coin.1.value;
        to_spend.push(coin);

        if curr_balance >= fees {
            break;
        }
    }

    let mut inputs = vec![];
    for coin in to_spend {
        // dbg!(&coin);
        let (new_state, input) = state.spend(&mut OsRng, &coin.1).unwrap();

        state = new_state;
        inputs.push(input);
    }

    let guaranted_coins = Offer {
        inputs,
        outputs: vec![Output::new(
            &mut OsRng,
            &coin_structure::coin::Info {
                nonce: OsRng.gen(),
                type_: NATIVE_TOKEN,
                value: curr_balance - fees,
            },
            &state.coin_public_key(),
            Some(state.enc_public_key()),
        )
        .unwrap()],
        transient: vec![],
        deltas: vec![(NATIVE_TOKEN, fees as i128)],
    };

    let balanced_tx = Transaction::new(guaranted_coins, None, None);

    let balanced_tx = balanced_tx
        .prove(OsRng, &prover_params.pp, |loc| match &*loc.0 {
            "midnight/zswap/spend" => Some(prover_params.spend.clone()),
            "midnight/zswap/output" => Some(prover_params.output.clone()),
            "midnight/zswap/sign" => Some(prover_params.sign.clone()),
            _ => unreachable!("this transaction does not have contract calls"),
        })
        .await
        .unwrap();

    let final_tx = balanced_tx.merge(&unbalanced_tx).unwrap();

    dbg!(final_tx.imbalances(true, Some(fees)));

    dbg!(&final_tx);
    dbg!(&final_tx.fees(&ledger_state.parameters));

    final_tx
        .well_formed(&ledger_state, WellFormedStrictness::default())
        .unwrap();

    // panic!();

    let mut serialized_final_tx = vec![];

    dbg!("serializing final tx");
    serialize(
        &final_tx,
        std::io::Cursor::new(&mut serialized_final_tx),
        network_id,
    )
    .unwrap();

    let tx_hash = hex::encode(final_tx.transaction_hash().0 .0);

    println!("tx hash: {}", tx_hash);

    // let api = OnlineClient::<SubstrateConfig>::from_url("wss://rpc.testnet.midnight.network")
    //     .await
    //     .unwrap();
    //
    let hex_tx = hex::encode(serialized_final_tx);

    let extrinsic = midnight::tx()
        .midnight()
        .send_mn_transaction(hex_tx.as_bytes().to_vec());

    let client = api.tx();

    let submittable = client.create_unsigned(&extrinsic).unwrap();

    let watch = submittable.submit_and_watch().await.unwrap();

    let result = watch.wait_for_finalized_success().await.unwrap();

    dbg!(result);

    Ok((state, tx_hash))
}
