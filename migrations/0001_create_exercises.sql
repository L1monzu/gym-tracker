-- Stores exercise definitions (e.g. "Bench Press"), created once and reused
-- across many logged sets. Keeping this separate from exercise_logs avoids
-- retyping exercise names and enables future features like images,
-- descriptions, and personal records without changing this table again.
CREATE TABLE exercises (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    image_path TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);