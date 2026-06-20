-- Stores one logged set of an exercise: which exercise, how much weight,
-- how many sets/reps, and when. Linked to exercises by exercise_id rather
-- than storing the exercise name directly, to keep data consistent and
-- support future per-exercise features (personal records, progress graphs).
CREATE TABLE exercise_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    exercise_id INTEGER NOT NULL REFERENCES exercises(id),
    session_id INTEGER REFERENCES workout_sessions(id),
    weight REAL NOT NULL,
    sets INTEGER NOT NULL,
    reps INTEGER NOT NULL,
    notes TEXT,
    logged_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);

CREATE INDEX idx_exercise_logs_exercise_id ON exercise_logs(exercise_id);
CREATE INDEX idx_exercise_logs_logged_at ON exercise_logs(logged_at);