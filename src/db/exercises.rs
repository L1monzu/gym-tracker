use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Exercise {
    pub id: i64,
    pub name: String,
}

/// Returns every known exercise name, used for autocomplete suggestions.
pub async fn list_exercises(pool: &SqlitePool) -> Result<Vec<Exercise>, sqlx::Error> {
    sqlx::query_as::<_, Exercise>("SELECT id, name FROM exercises ORDER BY name")
        .fetch_all(pool)
        .await
}

/// Finds an exercise by name, or creates it if it doesn't exist yet.
/// Matching is case-insensitive so "bench press" and "Bench Press" reuse
/// the same row instead of creating duplicates.
pub async fn find_or_create_exercise(pool: &SqlitePool, name: &str) -> Result<i64, sqlx::Error> {
    let existing: Option<Exercise> = sqlx::query_as::<_, Exercise>(
        "SELECT id, name FROM exercises WHERE name = ?1 COLLATE NOCASE",
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;

    if let Some(exercise) = existing {
        return Ok(exercise.id);
    }

    let result = sqlx::query("INSERT INTO exercises (name) VALUES (?1)")
        .bind(name)
        .execute(pool)
        .await?;

    Ok(result.last_insert_rowid())
}

pub struct NewExerciseLog {
    pub exercise_name: String,
    pub logged_at: String,
    pub sets: Vec<SetEntry>,
}

pub struct SetEntry {
    pub weight: f64,
    pub reps: i64,
}

/// Saves every set from one logging session as its own row, all sharing
/// the same exercise and date so they read back as one entry.
pub async fn insert_exercise_log(
    pool: &SqlitePool,
    log: NewExerciseLog,
) -> Result<(), sqlx::Error> {
    let exercise_id = find_or_create_exercise(pool, &log.exercise_name).await?;

    for set in &log.sets {
        sqlx::query(
            "INSERT INTO exercise_logs (exercise_id, weight, reps, logged_at)
             VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(exercise_id)
        .bind(set.weight)
        .bind(set.reps)
        .bind(&log.logged_at)
        .execute(pool)
        .await?;
    }

    Ok(())
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PersonalRecord {
    pub exercise_name: String,
    pub best_weight: f64,
    pub best_weight_reps: i64,
    pub best_weight_date: String,
    pub best_reps: i64,
    pub best_reps_weight: f64,
    pub best_reps_date: String,
}

/// For each exercise, finds the heaviest set ever logged, and separately
/// the highest rep count ever logged, these are often different sets,
/// a max-weight single differs from a max-rep set at a lighter load.
pub async fn list_personal_records(pool: &SqlitePool) -> Result<Vec<PersonalRecord>, sqlx::Error> {
    sqlx::query_as::<_, PersonalRecord>(
        "SELECT
            exercises.name AS exercise_name,
            best_weight_row.weight AS best_weight,
            best_weight_row.reps AS best_weight_reps,
            best_weight_row.logged_at AS best_weight_date,
            best_reps_row.reps AS best_reps,
            best_reps_row.weight AS best_reps_weight,
            best_reps_row.logged_at AS best_reps_date
         FROM exercises
         JOIN (
             SELECT exercise_id, weight, reps, logged_at
             FROM exercise_logs el
             WHERE deleted_at IS NULL
               AND weight = (
                   SELECT MAX(weight) FROM exercise_logs
                   WHERE exercise_id = el.exercise_id AND deleted_at IS NULL
               )
             GROUP BY exercise_id
         ) AS best_weight_row ON best_weight_row.exercise_id = exercises.id
         JOIN (
             SELECT exercise_id, weight, reps, logged_at
             FROM exercise_logs el
             WHERE deleted_at IS NULL
               AND reps = (
                   SELECT MAX(reps) FROM exercise_logs
                   WHERE exercise_id = el.exercise_id AND deleted_at IS NULL
               )
             GROUP BY exercise_id
         ) AS best_reps_row ON best_reps_row.exercise_id = exercises.id
         ORDER BY exercises.name",
    )
    .fetch_all(pool)
    .await
}