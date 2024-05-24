use super::{model::Summary, CONFIG};
use anyhow::{anyhow, Result};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use ort::{
    CPUExecutionProvider, CUDAExecutionProvider, CoreMLExecutionProvider,
    DirectMLExecutionProvider, TensorRTExecutionProvider,
};
use pgvector::Vector;
use tokio_stream::wrappers::ReceiverStream;
use std::sync::Arc;
use tokio_postgres::NoTls;
use super::chat_completion::CompletionModel;

pub type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

#[derive(Clone)]
pub struct RagClient {
    chat_model: CompletionModel,
    embedding_model: Arc<TextEmbedding>,
    db_pool: ConnectionPool,
}

impl RagClient {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            chat_model: CompletionModel::new()?,
            embedding_model: Self::create_embedding_model().await?,
            db_pool: Self::create_pg_pool().await?,
        })
    }

    async fn create_embedding_model() -> Result<Arc<TextEmbedding>> {
        let model = Arc::new(TextEmbedding::try_new(InitOptions {
            model_name: EmbeddingModel::BGEBaseENV15Q,
            execution_providers: vec![
                TensorRTExecutionProvider::default().build(),
                CUDAExecutionProvider::default().build(),
                // Use DirectML on Windows if NVIDIA EPs are not available
                DirectMLExecutionProvider::default().build(),
                // Or use ANE on Apple platforms
                CoreMLExecutionProvider::default().build(),
                CPUExecutionProvider::default().build(),
            ],
            ..Default::default()
        })?);
        Ok(model)
    }

    async fn create_pg_pool() -> Result<ConnectionPool> {
        let manager = PostgresConnectionManager::new_from_stringlike(&CONFIG.db_connection, NoTls)?;
        let db_pool = Pool::builder().build(manager).await?;
        Ok(db_pool)
    }

    pub async fn embed(&self, text: &str) -> Result<Vector> {
        let val = { self.embedding_model.embed(vec![&text], None) };
        let embedding_vec = val?.pop().ok_or(anyhow!("out of bounds"))?;
        Ok(Vector::from(embedding_vec))
    }

    pub async fn semantic_search(&self, query: &str) -> Result<Summary> {
        let embedding = self.embed(query).await?;
        let query = "SELECT *, (embedding <=> $1) AS similarity_score FROM book_summaries ORDER BY similarity_score LIMIT 1";
        let conn = self.db_pool.get_owned().await?;
        let statement = conn.prepare(query).await?;
        let row = conn.query_one(&statement, &[&embedding]).await?;
        Ok(Summary::from(row))
    }

    pub async fn chat_model_generate_stream(
        &self,
        request: &str,
        summary_text: &str,
    ) -> Result<ReceiverStream<mistralrs::Response>, anyhow::Error> {
        self.chat_model.complete(&format!("Given this book search request: {}, explain why the book with the following summary should be recommended, summary: {}", request, summary_text)).await
    }
}
