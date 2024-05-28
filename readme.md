# Rusty RAG - LLM-powered Book Recommendation
This project implements a simple semantic search and RAG use-case with rust (mistral.rs/fastembed-rs) + pgvector and serves a frontend and rest-api to recommend books based on the [CMU Book Summary Dataset](https://www.kaggle.com/datasets/ymaricar/cmu-book-summary-dataset/data).

<img src="img/setup.excalidraw.png" width="500">

# Setup

## Start database and ollama instance
First run:
```bash
docker-compose up
```
to start the postgres database.

## Generate book summary embeddings

Then download the dataset from [Kaggle](https://www.kaggle.com/datasets/ymaricar/cmu-book-summary-dataset?resource=download&select=booksummaries.txt) and start the embedding generation:

```
cargo run --release --bin embedding-generator -- --num-summaries 16384 --num-chunks 2048 --summaries-file ./booksummaries.txt
```

## Set Hugging Face Token
This project uses hugging face to remotely load models from the internet.
To authenticate your account, save the hugging face api token from [token-settings](https://huggingface.co/settings/tokens) to <i>~/.cache/huggingface/token</i>.
## Run the web application
```
cargo run --release --bin rustyrag
```

# Troubleshooting

## Request Error 401

If you are facing this error:

```bash
thread 'main' panicked at /Users/z0011bjk/.cargo/git/checkouts/mistral.rs-0a2607fe9768eac5/5c3e9f1/mistralrs-core/src/pipeline/gguf.rs:282:58:
RequestError(Status(401, Response[status: 401, status_text: Unauthorized, url:
https://huggingface.co/mistralai/Mistral-7B-Instruct-v0.2/resolve/main/tokenizer.json]))
```

You need to save your hugging face token first to: <i>~/.cache/huggingface/token</i>

## Request Error 403

If you are facing the following error:

```bash
thread 'main' panicked at /Users/z0011bjk/.cargo/git/checkouts/mistral.rs-0a2607fe9768eac5/5c3e9f1/mistralrs-core/src/pipeline/gguf.rs:282:58:
RequestError(Status(403, Response[status: 403, status_text: Forbidden, url:
https://huggingface.co/mistralai/Mistral-7B-Instruct-v0.2/resolve/main/tokenizer.json]))
```

If you are using the default mistralai/Mistral-7B-Instruct-v0.2 model, you have to be accepted the license at [huggingface](https://huggingface.co/mistralai/Mistral-7B-Instruct-v0.2) first.
