use clap::Parser;
use lazy_static::lazy_static;

#[derive(Parser)]
#[clap(version="1.0")]
#[derive(Debug, Default)]
pub struct Config {
    #[clap(short, long, env = "DB_CONNECTION", default_value = "postgresql://testuser:testpwd@localhost:5432/vectordb")]
    pub db_connection: String,

    #[arg(short, long, env = "NUM_SUMMARIES", default_value = "4096")]
    pub num_summaries: i32,

    #[arg(short, long, env = "NUM_CHUNKS", default_value = "256")]
    pub num_chunks: i32,

    #[arg(short, long, env = "SUMMARIES_FILE", default_value = "booksummaries.txt")]
    pub summaries_file: String,
}

lazy_static! {
    pub static ref CONFIG: Config = Config::parse();
}