-- Represents a single gym visit / workout session, grouping multiple
-- exercise and cardio logs together. session_id on those tables is
-- nullable, so quick standalone logs work fine without a session too.
CREATE TABLE workout_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    notes TEXT,
    deleted_at TEXT
);

CREATE INDEX idx_workout_sessions_started_at ON workout_sessions(started_at);