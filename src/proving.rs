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
use zswap::{prove::ZswapResolver, ZSWAP_EXPECTED_FILES};

static MIDNIGHT_DATA_PROVIDER: OnceCell<MidnightDataProvider> = OnceCell::const_new();

async fn fetch_proof_params() -> &'static MidnightDataProvider {
    // just in case, make sure we fetch all the files we need inside a lock.
    MIDNIGHT_DATA_PROVIDER
        .get_or_init(|| async {
            let midnight_data_provider = MidnightDataProvider::new(
                FetchMode::Synchronous,
                OutputMode::Log,
                ZSWAP_EXPECTED_FILES.to_vec(),
            )
            .unwrap();

            for zswap_file in ZSWAP_EXPECTED_FILES {
                midnight_data_provider.fetch(zswap_file.0).await.unwrap();
            }

            midnight_data_provider
                .fetch("bls_filecoin_2p15")
                .await
                .unwrap();

            midnight_data_provider
                .fetch("bls_filecoin_2p14")
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
                zswap_resolver: ZswapResolver(midnight_data_provider.clone()),
                external_resolver: Box::new(|_| unreachable!()),
                dust_resolver: DustResolver(
                    MidnightDataProvider::new(
                        FetchMode::OnDemand,
                        OutputMode::Log,
                        DUST_EXPECTED_FILES.to_owned(),
                    )
                    .expect("data provider initialization failed"),
                ),
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
