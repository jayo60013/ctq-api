CREATE DATABASE "crack-the-quote";

\connect "crack-the-quote"

CREATE TABLE quotes (
  id SERIAL PRIMARY KEY,
  author TEXT NOT NULL,
  quote TEXT NOT NULL
);
