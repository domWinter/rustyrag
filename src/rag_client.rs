use super::{model::Summary, CONFIG};
use anyhow::{anyhow, Result};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use ollama_rs::error::OllamaError;
use ollama_rs::generation::completion::GenerationResponse;
use ollama_rs::{generation::completion::request::GenerationRequest, Ollama};
use ort::{
    CPUExecutionProvider, CUDAExecutionProvider, CoreMLExecutionProvider,
    DirectMLExecutionProvider, TensorRTExecutionProvider,
};
use pgvector::Vector;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_postgres::NoTls;
use tokio_stream::Stream;

pub type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

#[derive(Clone)]
pub struct RagClient {
    ollama_client: Ollama,
    embedding_model: Arc<Mutex<TextEmbedding>>,
    db_pool: ConnectionPool,
}

impl RagClient {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            ollama_client: Self::create_ollama_client(),
            embedding_model: Self::create_embedding_model().await?,
            db_pool: Self::create_pg_pool().await?,
        })
    }

    async fn create_embedding_model() -> Result<Arc<Mutex<TextEmbedding>>> {
        let model = Arc::new(Mutex::new(TextEmbedding::try_new(InitOptions {
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
        })?));
        Ok(model)
    }

    async fn create_pg_pool() -> Result<ConnectionPool> {
        let manager = PostgresConnectionManager::new_from_stringlike(&CONFIG.db_connection, NoTls)?;
        let db_pool = Pool::builder().build(manager).await?;
        Ok(db_pool)
    }

    fn create_ollama_client() -> Ollama {
        Ollama::new(
            format!("http://{}", CONFIG.ollama_host).to_string(),
            CONFIG.ollama_port,
        )
    }

    pub async fn embed(&self, text: &str) -> Result<Vector> {
        let val = { self.embedding_model.lock().await.embed(vec![&text], None) };
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

    pub async fn ollama_generate_stream(
        &self,
        request: &str,
        summary_text: &str,
    ) -> Result<
        Pin<
            Box<(dyn Stream<Item = Result<Vec<GenerationResponse>, OllamaError>> + Send + 'static)>,
        >,
        OllamaError,
    > {
        let prompt = format!("Given this book search request: {}, explain why the book with the following summary should be recommended, summary: {}", request, summary_text);
        let stream = self
            .ollama_client
            .generate_stream(
                GenerationRequest::new("phi3".to_owned(), prompt).system(
                    "Answer the book search request using the summary provided. Be succinct."
                        .to_owned(),
                ),
            )
            .await;
        stream
    }
}
