use crate::sync_status::SharedSyncStatus;
use midnight_ledger::{
    prove::Resolver,
    structure::{Proof, ProofPreimage, Transaction},
};
use midnight_zswap::{
    base_crypto::data_provider::{FetchMode, MidnightDataProvider, OutputMode},
    coin_structure::coin::{Nullifier, NATIVE_TOKEN},
    keys::SecretKeys,
    local::State,
    storage::db::InMemoryDB,
    Offer, ZSWAP_EXPECTED_FILES,
};
use rand::rngs::OsRng;
use std::{cmp::Reverse, collections::HashMap, sync::Arc};
use tokio::{runtime::Handle, sync::Mutex};
use tracing::{info_span, Instrument as _};

#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
enum ProofOrNotifier {
    Waiting(Arc<tokio::sync::Notify>),
    Ready(Transaction<Proof, InMemoryDB>),
}

pub type PreProvingServiceChannelRx = tokio::sync::mpsc::Receiver<(
    Vec<Nullifier>,
    tokio::sync::oneshot::Sender<Vec<Transaction<Proof, InMemoryDB>>>,
)>;

pub type PreProvingServiceChannelTx = tokio::sync::mpsc::Sender<(
    Vec<Nullifier>,
    tokio::sync::oneshot::Sender<Vec<Transaction<Proof, InMemoryDB>>>,
)>;

pub async fn pre_proving_service(
    state: Arc<Mutex<State<InMemoryDB>>>,
    signal: Arc<tokio::sync::Notify>,
    mut comm: PreProvingServiceChannelRx,
    sync_status: SharedSyncStatus,
    secret_keys: SecretKeys,
) {
    let proven: Arc<Mutex<HashMap<Nullifier, ProofOrNotifier>>> =
        Arc::new(Mutex::new(HashMap::new()));

    {
        let proven = Arc::clone(&proven);
        tokio::spawn(async move {
            while let Some((nullifiers, tx)) = comm.recv().await {
                let proven = Arc::clone(&proven);
                tokio::spawn(async move {
                    let mut proofs = vec![];

                    for null in nullifiers {
                        loop {
                            let mut coins = proven.lock().await;

                            let proof = coins
                                .entry(null)
                                .or_insert(ProofOrNotifier::Waiting(Arc::new(
                                    tokio::sync::Notify::new(),
                                )))
                                .clone();

                            std::mem::drop(coins);

                            match proof {
                                ProofOrNotifier::Waiting(notify) => notify.notified().await,
                                ProofOrNotifier::Ready(transaction) => {
                                    proofs.push(transaction);
                                    break;
                                }
                            }
                        }
                    }

                    if tx.send(proofs).is_err() {
                        tracing::warn!("Can't send proof to the handler that requested it");
                    }
                });
            }
        });
    }

    loop {
        sync_status.wait_for_ready().await;

        let mut state = state.lock().await.clone();

        let mut proven_guard = proven.lock().await;

        // remove old proofs
        proven_guard.retain(|nul, _| state.coins.contains_key(nul));

        let mut unspent_coins = state
            .coins
            .iter()
            .filter(|(_, coin)| coin.type_ == NATIVE_TOKEN)
            .filter(|coin| {
                !state.pending_spends.contains_key(&coin.0) && !proven_guard.contains_key(&coin.0)
            })
            .collect::<Vec<_>>();

        std::mem::drop(proven_guard);

        unspent_coins.sort_by_key(|(_, coin)| Reverse(coin.value));

        for coin in unspent_coins {
            // TODO: what does this do?
            let segment = 0;
            let (new_state, input) = state
                .spend(&mut OsRng, &secret_keys, &coin.1, segment)
                .unwrap();

            state = new_state;

            let offer = Offer {
                inputs: vec![input],
                outputs: vec![],
                transient: vec![],
                // we can adjust this later
                deltas: vec![],
            };

            let tx = Transaction::new(offer, None, None);

            let proven_tx = prove_tx_in_rayon_pool(tx)
                .instrument(info_span!("proving input", nullifer = ?coin.0))
                .await;

            if let Some(ProofOrNotifier::Waiting(tx)) = proven
                .lock()
                .await
                .insert(coin.0, ProofOrNotifier::Ready(proven_tx))
            {
                tx.notify_waiters();
            }
        }

        // dbg!(&proven);
        signal.notified().await;
    }
}

pub async fn prove_tx_in_rayon_pool(
    tx: Transaction<ProofPreimage, InMemoryDB>,
) -> Transaction<Proof, InMemoryDB> {
    let (oneshot_tx, oneshot_rx) = tokio::sync::oneshot::channel();

    // TODO: maybe don't want to initialize this every time?
    let midnight_data_provider = MidnightDataProvider::new(
        // and also having this be OnDemand may lead to races the first time
        FetchMode::OnDemand,
        OutputMode::Log,
        ZSWAP_EXPECTED_FILES.to_vec(),
    );

    {
        tokio::task::spawn_blocking(move || {
            let resolver = Resolver {
                zswap_resolver: midnight_zswap::prove::ZswapResolver(
                    midnight_data_provider.clone(),
                ),
                external_resolver: Box::new(|_| unreachable!()),
            };

            let tokio_handle = Handle::current();

            let now = std::time::Instant::now();
            tracing::info!("blocking on {:?}", std::thread::current().id());
            let proof = tokio_handle.block_on(tx.prove(OsRng, &midnight_data_provider, &resolver));
            tracing::info!("input proven in {} ms", now.elapsed().as_millis());

            oneshot_tx.send(proof).unwrap();
        });
    }

    oneshot_rx.await.unwrap().unwrap()
}
