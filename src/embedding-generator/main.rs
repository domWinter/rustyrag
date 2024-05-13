#[path = "../model.rs"]
mod model;
mod config;
use anyhow::Result;
use csv::ReaderBuilder;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use model::Summary;
use ort::{CPUExecutionProvider, CUDAExecutionProvider, CoreMLExecutionProvider, TensorRTExecutionProvider, DirectMLExecutionProvider};
use pgvector::Vector;
use postgres::{Client, NoTls};
use std::collections::HashMap;
use std::time::Instant;
use config::CONFIG;

fn main() -> Result<()> {
    let execution_providers = vec![
        TensorRTExecutionProvider::default().build(),
        CUDAExecutionProvider::default().build(),
        // Use DirectML on Windows if NVIDIA EPs are not available
        DirectMLExecutionProvider::default().build(),
        // Or use ANE on Apple platforms
        CoreMLExecutionProvider::default().build(),
        CPUExecutionProvider::default().build()
    ];

    let model = TextEmbedding::try_new(InitOptions {
        model_name: EmbeddingModel::BGEBaseENV15Q,
        execution_providers,
        ..Default::default()
    })?;
    let mut summaries = read_summaries(&CONFIG.summaries_file, CONFIG.num_summaries)?;

    println!(
        "Creating {} embeddings with {} chunks",
        CONFIG.num_summaries, CONFIG.num_chunks
    );
    let mut client = Client::connect(&CONFIG.db_connection, NoTls)?;

    for (i, chunk) in summaries
        .chunks_mut((CONFIG.num_summaries / CONFIG.num_chunks) as usize)
        .enumerate()
    {
        println!("Inserting chunk {} from {}", i, CONFIG.num_chunks);
        let start = Instant::now();
        generate_embeddings(chunk, &model)?;
        insert_summary(&mut client, chunk);
        let duration = start.elapsed();
        println!("Generation took {:?} seconds", duration.as_secs());
        println!(
            "Estimated remaining time: {} minutes",
            (duration.as_secs() * ((CONFIG.num_chunks as usize - i) as u64)) / 60
        );
    }

    Ok(())
}

fn read_summaries(path: &str, num_summaries: i32) -> Result<Vec<Summary>> {
    let mut summary_reader = ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(&path)?;
    let summary_iter = summary_reader.records();
    let summaries: Vec<Summary> = summary_iter
        .take(num_summaries as usize)
        .map(|s| {
            let entry = s.unwrap();
            let genres: HashMap<String, String> =
                serde_json::from_str(&entry.get(6).unwrap().to_owned()).unwrap_or(HashMap::new());
            Summary {
                title: entry.get(2).unwrap().to_owned(),
                author: entry.get(3).unwrap().to_owned(),
                genres: genres.values().cloned().collect(),
                summary: entry.get(6).unwrap().to_owned(),
                embedding: None,
            }
        })
        .collect();
    Ok(summaries)
}

fn generate_embeddings(chunk: &mut [Summary], embedding_model: &TextEmbedding) -> Result<()> {
    let texts = chunk.into_iter().map(|s| &s.summary).collect();
    let embeddings = embedding_model.embed(texts, None)?;
    for (i, embedding) in embeddings.iter().enumerate() {
        chunk[i].embedding = Some(Vector::from(embedding.to_vec()));
    }
    Ok(())
}

fn insert_summary(client: &mut Client, summaries: &mut [model::Summary]) {
    for summary in summaries {
        client.execute(
            "INSERT INTO book_summaries (summary, author, title, genres, embedding) VALUES ($1, $2, $3, $4, $5)",
            &[&summary.summary, &summary.author, &summary.title, &summary.genres, &summary.embedding.clone().unwrap()],
        ).unwrap();
    }
}
