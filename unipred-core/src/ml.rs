use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

pub struct Embedder {
    model: TextEmbedding,
}

impl Embedder {
    /// Initialize the embedding model. This may download the model files if not present.
    pub fn new() -> Result<Self> {
        let mut options = InitOptions::new(EmbeddingModel::AllMiniLML6V2);
        options.show_download_progress = true;
        let model = TextEmbedding::try_new(options)?;
        
        Ok(Self { model })
    }

    /// Generate embeddings for a batch of texts.
    pub fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let embeddings = self.model.embed(texts, None)?;
        Ok(embeddings)
    }
}