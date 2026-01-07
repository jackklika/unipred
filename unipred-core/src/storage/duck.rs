use anyhow::Result;
use duckdb::{params, Connection};
use crate::proto::FetchedMarket;

pub struct DuckStore {
    conn: Connection,
}

impl DuckStore {
    /// Open a connection to a DuckDB database at the given path.
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    /// Initialize the database schema if it doesn't exist.
    fn init_schema(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS markets (
                ticker VARCHAR,
                source VARCHAR,
                title VARCHAR,
                status VARCHAR,
                description VARCHAR,
                outcomes VARCHAR,
                start_date VARCHAR,
                end_date VARCHAR,
                volume VARCHAR,
                liquidity VARCHAR,
                url VARCHAR,
                ingested_at TIMESTAMP DEFAULT current_timestamp,
                PRIMARY KEY (ticker, source)
            )",
            [],
        )?;
        Ok(())
    }

    /// Batch insert or replace markets.
    pub fn insert_batch(&mut self, markets: &[FetchedMarket]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO markets (
                    ticker, source, title, status, description, outcomes, 
                    start_date, end_date, volume, liquidity, url
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )?;

            for m in markets {
                // Join outcomes into a single string for storage
                let outcomes_str = m.outcomes.join(", ");
                
                stmt.execute(params![
                    m.ticker,
                    m.source,
                    m.title,
                    m.status,
                    m.description,
                    outcomes_str,
                    m.start_date,
                    m.end_date,
                    m.volume,
                    m.liquidity,
                    m.url
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }
}