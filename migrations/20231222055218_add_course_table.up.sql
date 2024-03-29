-- Add up migration script here
CREATE TABLE IF NOT EXISTS courses (
  id SERIAL PRIMARY KEY,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  title VARCHAR(255) NOT NULL,
  invite_link TEXT,
  language language NOT NULL
);