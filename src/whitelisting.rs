use crate::db::Db;
use anyhow::Context as _;
use midnight_ledger::{
    onchain_runtime::state::EntryPointBuf,
    structure::{ContractAction, Transaction},
};
use midnight_transient_crypto::proofs::{Proof, VerifierKey};
use midnight_zswap::serialize::{deserialize, serialize, NetworkId};
use std::{collections::HashMap, path::Path, sync::Arc};

pub type Constraints = Arc<HashMap<EntryPointBuf, VerifierKey>>;

pub fn read_constraints(
    dir: impl AsRef<Path>,
    network_id: NetworkId,
) -> anyhow::Result<Constraints> {
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

            let vk = deserialize::<VerifierKey, _>(std::io::Cursor::new(raw), network_id)?;

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
    tx: &Transaction<Proof>,
    network_id: NetworkId,
) -> anyhow::Result<Option<String>> {
    let tx = match tx {
        Transaction::Standard(standard_transaction) => standard_transaction,
        Transaction::ClaimMint(_) => return Ok(None),
    };

    let Some(contract_calls) = &tx.contract_calls else {
        return Ok(None);
    };

    if contract_calls.calls.len() > 1 {
        return Ok(None);
    }

    let ContractAction::Call(call) = &contract_calls.calls[0] else {
        return Ok(None);
    };

    let mut buf = vec![];

    serialize(&call.address, &mut buf, network_id)?;

    let hex_address = hex::encode(buf);

    if db.check_address(&hex_address).await? {
        Ok(Some(hex_address))
    } else {
        Ok(None)
    }
}

pub fn check_deploy(
    constraints: &Constraints,
    tx: &Transaction<Proof>,
    network_id: NetworkId,
) -> anyhow::Result<Option<String>> {
    let tx = match tx {
        Transaction::Standard(standard_transaction) => standard_transaction,
        Transaction::ClaimMint(_) => return Ok(None),
    };

    let Some(contract_calls) = &tx.contract_calls else {
        tracing::debug!("transaction does not have contract calls");
        return Ok(None);
    };

    if contract_calls.calls.len() > 1 {
        tracing::debug!(
            "transaction does have more than one ({}) contract call, only a single call per tx allowed", contract_calls.calls.len()
        );

        return Ok(None);
    }

    let ContractAction::Deploy(deploy) = &contract_calls.calls[0] else {
        return Ok(None);
    };

    let mut len = 0;
    for op in deploy.initial_state.operations.iter() {
        let op_s = String::from_utf8_lossy(&op.0 .0);

        let Some(vk_c) = constraints.get(&op.0) else {
            tracing::debug!(op = %op_s, "vk not found");
            return Ok(None);
        };

        let Some(vk_d) = &op.1.v1 else {
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
    serialize(&deploy.address(), &mut buf, network_id)?;
    let hex_address = hex::encode(buf);

    Ok(Some(hex_address))
}
