-- Stores one logged cardio session: activity type, duration, optional
-- distance, and when it happened. Activity stays as plain text (rather
-- than its own table like exercises) since cardio activity names are
-- simpler and less prone to duplicate-tracking issues right now.
CREATE TABLE cardio_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER REFERENCES workout_sessions(id),
    activity TEXT NOT NULL,
    duration_minutes INTEGER NOT NULL,
    distance_km REAL,
    notes TEXT,
    logged_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);

CREATE INDEX idx_cardio_logs_logged_at ON cardio_logs(logged_at);