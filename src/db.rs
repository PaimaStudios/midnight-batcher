use anyhow::Context as _;
use deadpool_sqlite::{Config, Pool, Runtime};
use midnight_zswap::{
    local::State,
    serialize::{deserialize, serialize, NetworkId},
};
use rusqlite::OptionalExtension as _;
use std::path::Path;

#[derive(Clone)]
pub struct Db {
    pool: Pool,
    network_id: NetworkId,
}

impl Db {
    pub async fn open_db(path: impl AsRef<Path>, network_id: NetworkId) -> anyhow::Result<Self> {
        let cfg = Config::new(path.as_ref());
        let pool = cfg
            .create_pool(Runtime::Tokio1)
            .context("Failed to initialize sqlite pool")?;

        let res = Self { pool, network_id };

        res.create_tables().await?;

        Ok(res)
    }

    pub async fn persist_state(&self, id: &str, hash: &str, state: &State) -> anyhow::Result<()> {
        let mut buf = vec![];
        serialize(&state, &mut buf, self.network_id)?;

        let conn = self.pool.get().await.unwrap();

        let id = id.to_string();
        let hash = hash.to_string();

        conn.interact(move |conn| {
            conn.execute(
                "INSERT OR REPLACE INTO state (id, hash, state) VALUES (?1, ?2, ?3)",
                (&id, &hash, &buf),
            )
        })
        .await
        .unwrap()
        .context("Db error persisting state")?;

        Ok(())
    }

    pub async fn get_state(&self, id: &str) -> anyhow::Result<Option<(String, State)>> {
        let conn = self.pool.get().await.unwrap();

        let id = id.to_string();

        let row = conn
            .interact(move |conn| -> anyhow::Result<Option<(String, Vec<u8>)>> {
                let mut stmt = conn.prepare(
                    "SELECT hash, state FROM state WHERE id = ?1 ORDER BY rowid DESC LIMIT 1",
                )?;

                let row = stmt
                    .query_row([id], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
                    })
                    .optional()
                    .context("Database access error")?;

                Ok(row)
            })
            .await
            .unwrap()?;

        if let Some((hash, unserialized_state)) = row {
            let state = deserialize(std::io::Cursor::new(unserialized_state), self.network_id)
                .context("Can't deserialize state object")?;

            Ok(Some((hash, state)))
        } else {
            Ok(None)
        }
    }

    pub async fn check_address(&self, address: impl AsRef<str>) -> anyhow::Result<bool> {
        let conn = self.pool.get().await.unwrap();

        let address = address.as_ref().to_string();

        conn.interact(move |conn| {
            let mut stmt = conn.prepare("SELECT 1 FROM contract_address WHERE id = ?1")?;

            let mut rows = stmt.query([address]).context("Database access error")?;

            Ok(rows.next()?.is_some())
        })
        .await
        .unwrap()
    }

    pub async fn insert_contract_address(&self, id: &str) -> anyhow::Result<()> {
        let conn = self.pool.get().await.unwrap();

        let id = id.to_string();

        conn.interact(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO contract_address (id) VALUES (?1)",
                [id],
            )
        })
        .await
        .unwrap()?;

        Ok(())
    }

    pub async fn update_contract_state(
        &self,
        contract_address: &str,
        game_state: &str,
        p1_public_key: &str,
        p2_public_key: &str,
        block_number: u64,
    ) -> anyhow::Result<()> {
        let conn = self.pool.get().await.unwrap();

        let contract_address = contract_address.to_string();
        let game_state = game_state.to_string();
        let p1_public_key = p1_public_key.to_string();
        let p2_public_key = p2_public_key.to_string();

        conn.interact(move |conn| {
            conn.execute(
                "UPDATE contract_address SET game_state = ?1, p1_public_key = ?2, p2_public_key = ?3, block_number = ?5 WHERE id = ?4 ",
                (game_state, p1_public_key, p2_public_key, contract_address, block_number),
            )
        })
        .await
        .unwrap()?;

        Ok(())
    }

    pub async fn get_lobbies_waiting_for_p2(
        &self,
        after: Option<String>,
        count: Option<u8>,
    ) -> anyhow::Result<Vec<(String, u64, String)>> {
        let conn = self.pool.get().await.unwrap();

        conn.interact(move |conn| {
            let mut stmt = conn.prepare(
                "
                SELECT id, block_number, p1_public_key FROM contract_address
                WHERE p2_public_key = '00;' AND
                    (?1 IS NULL OR rowid < (SELECT max(rowid) FROM contract_address WHERE id = ?1))
                ORDER BY rowid DESC
                LIMIT ?2",
            )?;

            let rows = stmt
                .query_map((after, count.unwrap_or(10).to_string()), |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })
                .context("Database error")?;

            let mut res = vec![];
            for row in rows {
                res.push(row?);
            }

            Ok(res)
        })
        .await
        .unwrap()
    }

    pub async fn get_player_lobbies(
        &self,
        public_key: String,
        limit: Option<u8>,
        after: Option<String>,
    ) -> anyhow::Result<Vec<(String, String, u64, String, String)>> {
        let conn = self.pool.get().await.unwrap();

        let public_key = public_key.to_string();

        conn.interact(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT id, game_state, block_number, p1_public_key, p2_public_key FROM contract_address
                WHERE
                    (p1_public_key = ?1 OR p2_public_key = ?1) AND
                    (?3 IS NULL OR (rowid < (SELECT max(rowid) FROM contract_address WHERE id = ?3)))
                ORDER BY rowid DESC
                LIMIT ?2",
            )?;

            let rows = stmt
                .query_map((public_key, limit.unwrap_or(10), after), |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
                })
                .context("Database error")?;

            let mut res = vec![];
            for row in rows {
                res.push(row?);
            }

            Ok(res)
        })
        .await
        .unwrap()
    }

    async fn create_tables(&self) -> anyhow::Result<()> {
        let conn = self.pool.get().await.unwrap();

        conn.interact(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS state (
                id TEXT PRIMARY KEY,
                hash TEXT NOT NULL,
                state BLOB NOT NULL
            )",
                (),
            )?;

            conn.execute(
                "CREATE TABLE IF NOT EXISTS contract_address (
                id TEXT PRIMARY KEY,
                game_state TEXT,
                p1_public_key TEXT,
                p2_public_key TEXT,
                block_number INTEGER
            )",
                (),
            )?;

            Ok(())
        })
        .await
        .unwrap()
    }
}
