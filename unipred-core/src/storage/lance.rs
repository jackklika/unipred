use anyhow::Result;
use arrow_array::{
    FixedSizeListArray, Float32Array, RecordBatch, RecordBatchIterator, StringArray,
};
use arrow_schema::{DataType, Field, Schema};
use futures::TryStreamExt;
use lancedb::{Connection, index::Index, query::{ExecutableQuery, QueryBase}};
use std::sync::Arc;

pub const VECTOR_DIM: i32 = 384; // Using all-MiniLM-L6-v2 dimension
pub const TABLE_NAME: &str = "markets";

pub struct LanceStore {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct MarketEmbedding {
    pub id: String,
    pub vector: Vec<f32>,
    pub ticker: String,
    pub source: String,
    pub title: String,
    pub description: String,
    pub outcomes: String,
}

impl LanceStore {
    /// Connect to a LanceDB instance at the given URI.
    /// URI can be a local path (e.g. "./data/lancedb") or S3 (e.g. "s3://bucket/path")
    pub async fn connect(uri: &str) -> Result<Self> {
        let conn = lancedb::connect(uri).execute().await?;
        Ok(Self { conn })
    }

    fn get_schema() -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    VECTOR_DIM,
                ),
                false,
            ),
            Field::new("ticker", DataType::Utf8, false),
            Field::new("source", DataType::Utf8, false),
            Field::new("title", DataType::Utf8, false),
            Field::new("description", DataType::Utf8, true),
            Field::new("outcomes", DataType::Utf8, true),
        ]))
    }

    /// Add markets to the store. If the table doesn't exist, it creates it.
    /// If it exists, it appends.
    pub async fn add_markets(&self, markets: Vec<MarketEmbedding>) -> Result<()> {
        if markets.is_empty() {
            return Ok(());
        }

        let schema = Self::get_schema();
        let batch = Self::create_record_batch(markets, schema.clone())?;

        let batches = RecordBatchIterator::new(
            vec![Ok(batch)],
            schema.clone(),
        );

        let table_exists = self.conn.table_names().execute().await?.contains(&TABLE_NAME.to_string());

        if table_exists {
            let table = self.conn.open_table(TABLE_NAME).execute().await?;
            table.add(Box::new(batches)).execute().await?;
        } else {
            self.conn
                .create_table(TABLE_NAME, Box::new(batches))
                .execute()
                .await?;
        }

        Ok(())
    }

    /// Create an IVF-PQ index on the vector column for fast search.
    pub async fn create_index(&self) -> Result<()> {
        let table = self.conn.open_table(TABLE_NAME).execute().await?;
        table
            .create_index(&["vector"], Index::Auto)
            .execute()
            .await?;
        Ok(())
    }

    /// Search for similar markets using a query vector.
    pub async fn search(&self, query_vector: Vec<f32>, limit: usize) -> Result<Vec<MarketEmbedding>> {
        let table = self.conn.open_table(TABLE_NAME).execute().await?;
        
        // Ensure query vector size matches dimension
        if query_vector.len() != VECTOR_DIM as usize {
            anyhow::bail!("Query vector dimension mismatch. Expected {}, got {}", VECTOR_DIM, query_vector.len());
        }

        let results = table
            .query()
            .nearest_to(query_vector)?
            .limit(limit)
            .execute()
            .await?
            .try_collect::<Vec<_>>()
            .await?;

        let mut markets = Vec::new();

        for batch in results {
            let ids = batch.column(0).as_any().downcast_ref::<StringArray>().unwrap();
            // Vector column retrieval is complex due to nesting, skipping strictly for returning search results
            // if we don't need the vector back. If we do, we need to handle FixedSizeListArray.
            
            // For now, let's just grab metadata columns.
            let tickers = batch.column(2).as_any().downcast_ref::<StringArray>().unwrap();
            let sources = batch.column(3).as_any().downcast_ref::<StringArray>().unwrap();
            let titles = batch.column(4).as_any().downcast_ref::<StringArray>().unwrap();
            let descriptions = batch.column(5).as_any().downcast_ref::<StringArray>().unwrap();
            let outcomes = batch.column(6).as_any().downcast_ref::<StringArray>().unwrap();

            for i in 0..batch.num_rows() {
                markets.push(MarketEmbedding {
                    id: ids.value(i).to_string(),
                    vector: vec![], // Omitted for efficiency/simplicity in read-path
                    ticker: tickers.value(i).to_string(),
                    source: sources.value(i).to_string(),
                    title: titles.value(i).to_string(),
                    description: descriptions.value(i).to_string(),
                    outcomes: outcomes.value(i).to_string(),
                });
            }
        }

        Ok(markets)
    }

    fn create_record_batch(markets: Vec<MarketEmbedding>, schema: Arc<Schema>) -> Result<RecordBatch> {
        let num_rows = markets.len();

        let mut id_builder = Vec::with_capacity(num_rows);
        let mut vector_values = Vec::with_capacity(num_rows * VECTOR_DIM as usize);
        let mut ticker_builder = Vec::with_capacity(num_rows);
        let mut source_builder = Vec::with_capacity(num_rows);
        let mut title_builder = Vec::with_capacity(num_rows);
        let mut description_builder = Vec::with_capacity(num_rows);
        let mut outcomes_builder = Vec::with_capacity(num_rows);

        for m in markets {
            if m.vector.len() != VECTOR_DIM as usize {
                anyhow::bail!("Vector dimension mismatch for market {}", m.ticker);
            }
            id_builder.push(m.id);
            vector_values.extend(m.vector);
            ticker_builder.push(m.ticker);
            source_builder.push(m.source);
            title_builder.push(m.title);
            description_builder.push(m.description);
            outcomes_builder.push(m.outcomes);
        }

        let id_array = StringArray::from(id_builder);
        
        let vector_data = Float32Array::from(vector_values);
        let vector_array = FixedSizeListArray::try_new(
            Arc::new(Field::new("item", DataType::Float32, true)),
            VECTOR_DIM,
            Arc::new(vector_data),
            None,
        )?;

        let ticker_array = StringArray::from(ticker_builder);
        let source_array = StringArray::from(source_builder);
        let title_array = StringArray::from(title_builder);
        let description_array = StringArray::from(description_builder);
        let outcomes_array = StringArray::from(outcomes_builder);

        Ok(RecordBatch::try_new(
            schema,
            vec![
                Arc::new(id_array),
                Arc::new(vector_array),
                Arc::new(ticker_array),
                Arc::new(source_array),
                Arc::new(title_array),
                Arc::new(description_array),
                Arc::new(outcomes_array),
            ],
        )?)
    }
}