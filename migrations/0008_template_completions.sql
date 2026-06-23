CREATE TABLE template_completions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    template_id INTEGER NOT NULL REFERENCES workout_templates(id),
    logged_at TEXT NOT NULL DEFAULT (datetime('now'))
);