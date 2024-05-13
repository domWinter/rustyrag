use clap::Parser;
use lazy_static::lazy_static;

#[derive(Parser)]
#[clap(version="1.0")]
#[derive(Debug, Default)]
pub struct Config {
    #[clap(short, long, env = "DB_CONNECTION", default_value = "postgresql://testuser:testpwd@localhost:5432/vectordb")]
    pub db_connection: String,

    #[clap(short, long, env="OLLAMA_HOST", default_value = "127.0.0.1")]
    pub ollama_host: String,

    #[clap(short, long, env="OLLAMA_PORT", default_value = "11434")]
    pub ollama_port: u16,

    #[clap(short, long, env="SERVER_PORT", default_value = "8082")]
    pub server_port: u16,
}

lazy_static! {
    pub static ref CONFIG: Config = Config::parse();
}