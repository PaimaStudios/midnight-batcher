use std::path::Path;

use anyhow::Context as _;
use midnight_zswap::{
    local::State,
    serialize::{deserialize, serialize, NetworkId},
};
use rusqlite::{Connection, OptionalExtension as _};

pub struct Db {
    conn: Connection,
    network_id: NetworkId,
}

impl Db {
    pub fn temporary(network_id: NetworkId) -> anyhow::Result<Self> {
        let conn = Connection::open_in_memory()?;

        let res = Self { conn, network_id };

        res.create_tables()?;

        Ok(res)
    }

    pub fn open_db(path: impl AsRef<Path>, network_id: NetworkId) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;

        let res = Self { conn, network_id };

        res.create_tables()?;

        Ok(res)
    }

    pub fn persist_state(&self, id: &str, hash: &str, state: &State) -> anyhow::Result<()> {
        let mut buf = vec![];
        serialize(&state, &mut buf, self.network_id)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO state (id, hash, state) VALUES (?1, ?2, ?3)",
            (&id, &hash, &buf),
        )?;

        Ok(())
    }

    pub fn get_state(&self, id: &str) -> anyhow::Result<Option<(String, State)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT hash, state FROM state WHERE id = ?1 ORDER BY rowid DESC LIMIT 1")?;

        let row = stmt
            .query_row([id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
            })
            .optional()
            .context("Database access error")?;

        if let Some((hash, unserialized_state)) = row {
            let state =
                deserialize(std::io::Cursor::new(unserialized_state), self.network_id).unwrap();

            Ok(Some((hash, state)))
        } else {
            Ok(None)
        }
    }

    fn create_tables(&self) -> anyhow::Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS state (
            id TEXT PRIMARY KEY,
            hash TEXT NOT NULL,
            state BLOB NOT NULL
        )",
            (),
        )?;

        Ok(())
    }
}
