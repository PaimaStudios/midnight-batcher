use crate::{balancing::ProvingParams, SyncStatus};
use midnight_ledger::structure::Transaction;
use midnight_transient_crypto::proofs::Proof;
use midnight_zswap::{
    coin_structure::coin::{Nullifier, NATIVE_TOKEN},
    local::State,
    Offer,
};
use rand::rngs::OsRng;
use std::{cmp::Reverse, collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

#[derive(Clone)]
enum ProofOrNotifier {
    Waiting(Arc<tokio::sync::Notify>),
    Ready(Transaction<Proof>),
}

pub type PreProvingServiceChannelRx = tokio::sync::mpsc::Receiver<(
    Vec<Nullifier>,
    tokio::sync::oneshot::Sender<Vec<Transaction<Proof>>>,
)>;

pub type PreProvingServiceChannelTx = tokio::sync::mpsc::Sender<(
    Vec<Nullifier>,
    tokio::sync::oneshot::Sender<Vec<Transaction<Proof>>>,
)>;

pub async fn pre_proving_service(
    state: Arc<Mutex<State>>,
    prover_params: Arc<ProvingParams>,
    signal: Arc<tokio::sync::Notify>,
    mut comm: PreProvingServiceChannelRx,
    sync_status: Arc<Mutex<SyncStatus>>,
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
        let mut sync_status_guard = sync_status.lock().await;
        if let SyncStatus::Syncing {
            progress: _,
            notify,
        } = &mut *sync_status_guard
        {
            let waiter = Arc::new(tokio::sync::Notify::new());
            notify.replace(Arc::clone(&waiter));

            std::mem::drop(sync_status_guard);
            tracing::info!("waiting for wallet to sync before starting to pre-compute proofs");
            waiter.notified().await;
        } else {
            std::mem::drop(sync_status_guard);
        }

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
            let (new_state, input) = state.spend(&mut OsRng, &coin.1).unwrap();

            state = new_state;

            let offer = Offer {
                inputs: vec![input],
                outputs: vec![],
                transient: vec![],
                // we can adjust this later
                deltas: vec![],
            };

            let tx = Transaction::new(offer, None, None);

            let now = std::time::Instant::now();

            let proven_tx = tx
                .prove(OsRng, &prover_params.pp, |loc| match &*loc.0 {
                    "midnight/zswap/spend" => Some(prover_params.spend.clone()),
                    "midnight/zswap/output" => Some(prover_params.output.clone()),
                    "midnight/zswap/sign" => Some(prover_params.sign.clone()),
                    _ => unreachable!("this transaction does not have contract calls"),
                })
                .await
                .unwrap();

            tracing::info!(
                nullifier = ?coin.0,
                "input proven in {} ms",
                now.elapsed().as_millis()
            );

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
