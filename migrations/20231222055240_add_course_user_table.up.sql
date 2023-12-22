-- Add up migration script here
CREATE TABLE IF NOT EXISTS course_user (
  id        SERIAL PRIMARY KEY,
  owned     BOOLEAN NOT NULL,
  course_id INT REFERENCES courses(id) ON DELETE CASCADE,
  user_id   INT REFERENCES users(id)   ON DELETE CASCADE
);
