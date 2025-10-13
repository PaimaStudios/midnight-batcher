use crate::db::Db;
use anyhow::Context as _;
use base_crypto::signatures::Signature;
use ledger_storage::db::InMemoryDB;
use midnight_serialize::{tagged_deserialize, tagged_serialize};
use mn_ledger::structure::{ContractAction, ProofMarker, Transaction};
use onchain_runtime::state::EntryPointBuf;
use std::{collections::HashMap, path::Path, sync::Arc};
use transient_crypto::{commitment::PedersenRandomness, proofs::VerifierKey};

pub type Constraints = Arc<HashMap<EntryPointBuf, VerifierKey>>;

pub fn read_constraints(dir: impl AsRef<Path>) -> anyhow::Result<Constraints> {
    let mut res = HashMap::default();

    let dir = std::fs::read_dir(dir.as_ref()).context("Failed to read keys directory")?;

    for entry in dir {
        let entry = entry.context("Failed to read dir entry")?;

        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        let parts = file_name.split(".").collect::<Vec<_>>();

        if parts[1] == "verifier" {
            let raw = std::fs::read(entry.path()).context("Failed to read vk file")?;

            let checksum = sha256::digest(&raw);

            let vk = tagged_deserialize::<VerifierKey>(std::io::Cursor::new(raw))?;

            tracing::info!(
                sha256 = hex::encode(checksum),
                "loaded vk for {}",
                String::from_utf8_lossy(parts[0].as_bytes())
            );

            res.insert(EntryPointBuf(parts[0].as_bytes().to_vec()), vk);
        }
    }

    Ok(Arc::new(res))
}

pub async fn check_call(
    db: &Db,
    tx: &Transaction<Signature, ProofMarker, PedersenRandomness, InMemoryDB>,
) -> anyhow::Result<Option<String>> {
    let tx = match tx {
        Transaction::Standard(standard_transaction) => standard_transaction,
        Transaction::ClaimRewards(_) => return Ok(None),
    };

    let mut cc = None;

    for (_segment, intent) in tx.intents() {
        for action in intent.actions.iter_deref() {
            match action {
                ContractAction::Call(call) => {
                    if cc.replace(call.clone()).is_some() {
                        tracing::debug!("transaction does have more than one contract call");

                        return Ok(None);
                    }
                }
                ContractAction::Deploy(_) => return Ok(None),
                ContractAction::Maintain(_) => return Ok(None),
            }
        }
    }

    let Some(call) = cc else {
        return Ok(None);
    };

    let mut buf = vec![];

    tagged_serialize(&call.address, &mut buf)?;

    let hex_address = hex::encode(buf);

    if db.check_address(&hex_address).await? {
        Ok(Some(hex_address))
    } else {
        Ok(None)
    }
}

pub fn check_deploy(
    constraints: &Constraints,
    tx: &Transaction<Signature, ProofMarker, PedersenRandomness, InMemoryDB>,
) -> anyhow::Result<Option<String>> {
    let tx = match tx {
        Transaction::Standard(standard_transaction) => standard_transaction,
        Transaction::ClaimRewards(_) => return Ok(None),
    };

    let mut deploy = None;

    for (_segment, intent) in tx.intents() {
        for action in intent.actions.iter_deref() {
            match action {
                ContractAction::Deploy(contract_deploy) => {
                    if deploy.replace(contract_deploy.clone()).is_some() {
                        tracing::debug!("transaction does have more than one contract deploy");

                        return Ok(None);
                    }
                }
                ContractAction::Call(_) => return Ok(None),
                ContractAction::Maintain(_) => return Ok(None),
            }
        }
    }

    let Some(deploy) = deploy else {
        return Ok(None);
    };

    let mut len = 0;
    for op in deploy.initial_state.operations.iter() {
        let op_s = String::from_utf8_lossy(&op.0 .0);

        let Some(vk_c) = constraints.get(&*op.0) else {
            tracing::debug!(op = %op_s, "vk not found");
            return Ok(None);
        };

        let Some(vk_d) = &op.1.v2 else {
            return Ok(None);
        };

        if vk_c != vk_d {
            tracing::debug!(op = %op_s, "vk mismatch");
            return Ok(None);
        }

        len += 1;
    }

    if constraints.len() != len {
        tracing::debug!(
            expected = constraints.len(),
            found = len,
            "operations size mismatch"
        );
        return Ok(None);
    }

    let mut buf = vec![];
    tagged_serialize(&deploy.address(), &mut buf)?;
    let hex_address = hex::encode(buf);

    Ok(Some(hex_address))
}
