CREATE EXTENSION IF NOT EXISTS vector;
DROP TABLE IF EXISTS book_summaries;
CREATE TABLE IF NOT EXISTS book_summaries (
  id SERIAL PRIMARY KEY,
  summary text not null,
  author VARCHAR(128) not null,
  title VARCHAR(128) not null,
  genres varchar[],
  embedding vector(768)
);