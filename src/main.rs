mod config;
mod model;
mod rag_client;
use anyhow::Result;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use config::CONFIG;
use model::SearchQuery;
use model::Summary;
use rag_client::RagClient;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let rag_client = RagClient::new().await?;
    let app = Router::new()
        .route("/", get(root_get))
        .route("/ws/v1/search", get(search))
        .route("/api/v1/semantic_search", post(semantic_search))
        .with_state(rag_client.clone());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", CONFIG.server_port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn root_get() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("src/index.html").await.unwrap();
    Html(markup)
}

async fn semantic_search(
    State(client): State<RagClient>,
    axum::extract::Json(request): axum::extract::Json<SearchQuery>,
) -> Json<Summary> {
    let book = client.semantic_search(&request.query).await.unwrap();
    Json(book)
}

async fn search(ws: WebSocketUpgrade, State(client): State<RagClient>) -> impl IntoResponse {
    ws.on_upgrade(|ws: WebSocket| async { search_stream(client, ws).await.unwrap() })
}

async fn search_stream(client: RagClient, mut ws: WebSocket) -> Result<()> {
    while let Some(msg) = ws.recv().await {
        if let Ok(msg) = msg {
            process_ws_message(msg, &mut ws, &client).await?;
        } else {
            return Ok(());
        }
    }
    Ok(())
}

async fn process_ws_message(msg: Message, ws: &mut WebSocket, client: &RagClient) -> Result<()> {
    match msg {
        Message::Text(t) => {
            let summary = client.semantic_search(&t).await?;
            let mut answer_stream = client.ollama_generate_stream(&t, &summary.summary).await?;

            while let Some(res) = answer_stream.next().await {
                let responses = res?;
                for resp in responses {
                    ws.send(Message::Text(resp.response)).await?;
                }
            }
            ws.send(Message::Text("CLOSE".to_owned())).await?;
        }
        Message::Ping(_) => ws.send(Message::Pong("Pong".into())).await?,
        Message::Close(_) => return Ok(()),
        _ => (),
    }
    Ok(())
}
