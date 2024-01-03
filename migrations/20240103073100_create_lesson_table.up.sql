-- Add up migration script here
CREATE TABLE IF NOT EXISTS lessons (
  id SERIAL PRIMARY KEY,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  title TEXT NOT NULL,
  cover_path TEXT,
  subject TEXT,
  content_path TEXT NOT NULL,
  course_id INT REFERENCES courses(id) ON DELETE CASCADE
);
