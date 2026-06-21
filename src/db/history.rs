use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct SetDetail {
    pub id: i64,
    pub weight: f64,
    pub reps: i64,
}

#[derive(Debug, Clone)]
pub enum HistoryEntry {
    Exercise {
        exercise_name: String,
        sets: Vec<SetDetail>,
    },
    Cardio {
        id: i64,
        activity: String,
        duration_minutes: i64,
        distance_km: Option<f64>,
        incline_percent: Option<f64>,
        avg_speed: Option<f64>,
        calories: Option<i64>,
        floors_climbed: Option<i64>,
    },
}

#[derive(Debug, Clone)]
pub struct DayGroup {
    pub date: String,
    pub entries: Vec<HistoryEntry>,
}

#[derive(sqlx::FromRow)]
struct ExerciseRow {
    id: i64,
    exercise_name: String,
    weight: f64,
    reps: i64,
    logged_at: String,
}

#[derive(sqlx::FromRow)]
struct CardioRow {
    id: i64,
    activity: String,
    duration_minutes: i64,
    distance_km: Option<f64>,
    incline_percent: Option<f64>,
    avg_speed: Option<f64>,
    calories: Option<i64>,
    floors_climbed: Option<i64>,
    logged_at: String,
}

/// Loads every non-deleted exercise and cardio log, merged together and
/// grouped by date (most recent day first). This is what powers the
/// History screen.
pub async fn load_history(pool: &SqlitePool) -> Result<Vec<DayGroup>, sqlx::Error> {
    let exercise_rows = sqlx::query_as::<_, ExerciseRow>(
        "SELECT exercise_logs.id, exercises.name AS exercise_name, weight, reps, logged_at
         FROM exercise_logs
         JOIN exercises ON exercises.id = exercise_logs.exercise_id
         WHERE exercise_logs.deleted_at IS NULL",
    )
    .fetch_all(pool)
    .await?;

    let cardio_rows = sqlx::query_as::<_, CardioRow>(
        "SELECT id, activity, duration_minutes, distance_km, incline_percent,
                avg_speed, calories, floors_climbed, logged_at
         FROM cardio_logs
         WHERE deleted_at IS NULL",
    )
    .fetch_all(pool)
    .await?;

    // Merge both kinds of rows into a single map, keyed by date, so a day
    // with both a lift and a run after it ends up as one group.
    let mut by_date: std::collections::BTreeMap<String, Vec<HistoryEntry>> =
        std::collections::BTreeMap::new();

    // Group exercise rows by (date, exercise name) first, so multiple
    // sets of the same exercise on the same day end up as one entry.
    let mut exercise_groups: std::collections::BTreeMap<(String, String), Vec<SetDetail>> =
        std::collections::BTreeMap::new();

    for row in exercise_rows {
        exercise_groups
            .entry((row.logged_at, row.exercise_name))
            .or_default()
            .push(SetDetail { id: row.id, weight: row.weight, reps: row.reps });
    }

    for ((date, exercise_name), sets) in exercise_groups {
        by_date.entry(date).or_default().push(HistoryEntry::Exercise { exercise_name, sets });
    }

    for row in cardio_rows {
        by_date.entry(row.logged_at).or_default().push(HistoryEntry::Cardio {
            id: row.id,
            activity: row.activity,
            duration_minutes: row.duration_minutes,
            distance_km: row.distance_km,
            incline_percent: row.incline_percent,
            avg_speed: row.avg_speed,
            calories: row.calories,
            floors_climbed: row.floors_climbed,
        });
    }

    // BTreeMap keeps dates sorted oldest-first; we want newest-first, so
    // we reverse once we've turned it into a plain list.
    let mut groups: Vec<DayGroup> = by_date
        .into_iter()
        .map(|(date, entries)| DayGroup { date, entries })
        .collect();
    groups.reverse();

    Ok(groups)
}

/// Soft-deletes every set passed in, used when the user deletes a whole
/// exercise entry, which may represent several sets logged together.
pub async fn delete_exercise_logs(pool: &SqlitePool, ids: &[i64]) -> Result<(), sqlx::Error> {
    for id in ids {
        sqlx::query("UPDATE exercise_logs SET deleted_at = datetime('now') WHERE id = ?1")
            .bind(id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

/// Soft-deletes a cardio log, same approach as `delete_exercise_log`.
pub async fn delete_cardio_log(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE cardio_logs SET deleted_at = datetime('now') WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}