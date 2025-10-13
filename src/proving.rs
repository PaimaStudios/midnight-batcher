use base_crypto::{
    data_provider::{FetchMode, MidnightDataProvider, OutputMode},
    signatures::Signature,
};
use ledger_storage::db::InMemoryDB;
use mn_ledger::{
    dust::{DustResolver, DUST_EXPECTED_FILES},
    prove::Resolver,
    structure::{ProofMarker, ProofPreimageMarker, Transaction, INITIAL_TRANSACTION_COST_MODEL},
};
use rand::rngs::OsRng;
use tokio::{runtime::Handle, sync::OnceCell};
use transient_crypto::commitment::PedersenRandomness;
use zswap::prove::ZswapResolver;

static MIDNIGHT_DATA_PROVIDER: OnceCell<MidnightDataProvider> = OnceCell::const_new();

async fn fetch_proof_params() -> &'static MidnightDataProvider {
    // just in case, make sure we fetch all the files we need inside a lock.
    MIDNIGHT_DATA_PROVIDER
        .get_or_init(|| async {
            let midnight_data_provider = MidnightDataProvider::new(
                FetchMode::Synchronous,
                OutputMode::Log,
                DUST_EXPECTED_FILES.to_owned(),
            )
            .unwrap();

            midnight_data_provider
                .fetch("bls_filecoin_2p13")
                .await
                .unwrap();

            midnight_data_provider
                .fetch("dust/6/spend.prover")
                .await
                .unwrap();

            midnight_data_provider
                .fetch("dust/6/spend.verifier")
                .await
                .unwrap();

            midnight_data_provider
                .fetch("dust/6/spend.bzkir")
                .await
                .unwrap();

            midnight_data_provider
        })
        .await
}

pub async fn prove_tx_in_rayon_pool(
    tx: Transaction<Signature, ProofPreimageMarker, PedersenRandomness, InMemoryDB>,
) -> Transaction<Signature, ProofMarker, PedersenRandomness, InMemoryDB> {
    let (oneshot_tx, oneshot_rx) = tokio::sync::oneshot::channel();

    let midnight_data_provider = fetch_proof_params().await.clone();

    {
        tokio::task::spawn_blocking(move || {
            let resolver = Resolver {
                zswap_resolver: ZswapResolver(
                    MidnightDataProvider::new(FetchMode::Synchronous, OutputMode::Log, vec![])
                        .unwrap(),
                ),
                external_resolver: Box::new(|_| unreachable!()),
                dust_resolver: DustResolver(midnight_data_provider),
            };

            let tokio_handle = Handle::current();

            let cost_model = INITIAL_TRANSACTION_COST_MODEL.runtime_cost_model;

            let provider = zkir::LocalProvingProvider {
                rng: OsRng,
                params: &resolver,
                resolver: &resolver,
            };

            let now = std::time::Instant::now();
            let proof = tokio_handle.block_on(tx.prove(provider, &cost_model));
            tracing::info!("dust proof generated in {} ms", now.elapsed().as_millis());

            oneshot_tx.send(proof).unwrap();
        });
    }

    oneshot_rx.await.unwrap().unwrap()
}
