use pgvector::Vector;
use tokio_postgres::Row;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Summary {
    pub title: String,
    pub author: String,
    pub genres: Vec<String>,
    pub summary: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub embedding: Option<Vector>
}

impl From<Row> for Summary {
    fn from(r: Row) -> Summary {
        Summary {
            summary: r.get("summary"),
            title: r.get("title"),
            author: r.get("author"),
            genres: r.get("genres"),
            embedding: Some(r.get("embedding"))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String
}