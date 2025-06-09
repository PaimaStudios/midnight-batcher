use super::graphql_deser;
use std::sync::Arc;
use tokio::sync::RwLock;

pub enum SyncStatus {
    Syncing {
        progress: f64,
        notify: Option<Arc<tokio::sync::Notify>>,
    },
    UpToDate,
}

#[derive(Clone)]
pub struct SharedSyncStatus {
    pub inner: Arc<RwLock<SyncStatus>>,
}

impl SharedSyncStatus {
    pub fn new(inner: SyncStatus) -> Self {
        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    pub async fn update(
        &self,
        progress_update: graphql_deser::ProgressUpdate,
        highest_known_index: u64,
    ) {
        let mut sync_status = self.inner.write().await;

        let mut sync_percent =
            highest_known_index as f64 / progress_update.highest_relevant_wallet_index as f64;

        if progress_update.highest_relevant_wallet_index == 0 {
            sync_percent = 1.0;
        }

        if sync_percent > 0.95 {
            if let SyncStatus::Syncing {
                progress: _,
                notify: Some(notify),
            } = &*sync_status
            {
                notify.notify_waiters();
            }

            *sync_status = SyncStatus::UpToDate;
        } else {
            tracing::info!(
                "progress update: {}/{}",
                highest_known_index,
                progress_update.highest_relevant_wallet_index
            );
            let notify = if let SyncStatus::Syncing {
                progress: _,
                notify,
            } = &*sync_status
            {
                // we need to be careful to not forget about any notifiers awaiting for ready.
                notify.clone()
            } else {
                None
            };

            *sync_status = SyncStatus::Syncing {
                progress: sync_percent * 100.0,
                notify,
            };
        }
    }

    pub async fn wait_for_ready(&self) {
        let mut sync_status_guard = self.inner.write().await;
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
    }

    pub async fn on_indexer_fail(&self) {
        let mut sync_status = self.inner.write().await;

        let (notify, progress) = if let SyncStatus::Syncing { progress, notify } = &*sync_status {
            (notify.clone(), Some(*progress))
        } else {
            (None, None)
        };

        *sync_status = SyncStatus::Syncing {
            progress: progress
                // this happens when the SyncStatus was UpToDate but
                // something fails, we don't really have a more useful
                // number to put here, so just make it a 100.0 so it's
                // clearer that we already got to the tip.
                .unwrap_or(100.0),
            notify,
        };
    }

    pub async fn current_progress(&self) -> f64 {
        let sync_status = self.inner.read().await;

        match *sync_status {
            SyncStatus::Syncing {
                progress,
                notify: _,
            } => progress,
            SyncStatus::UpToDate => 100.0,
        }
    }
}
