use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CardioActivity {
    pub activity: String,
}

/// Returns every distinct activity name previously logged, used for
/// autocomplete suggestions. Unlike exercises, cardio activities aren't
/// a separate table — we just pull the distinct values already in use.
pub async fn list_cardio_activities(
    pool: &SqlitePool,
) -> Result<Vec<CardioActivity>, sqlx::Error> {
    sqlx::query_as::<_, CardioActivity>(
        "SELECT DISTINCT activity FROM cardio_logs ORDER BY activity",
    )
    .fetch_all(pool)
    .await
}

pub struct NewCardioLog {
    pub activity: String,
    pub duration_minutes: i64,
    pub distance_km: Option<f64>,
    pub incline_percent: Option<f64>,
    pub avg_speed: Option<f64>,
    pub calories: Option<i64>,
    pub floors_climbed: Option<i64>,
    pub logged_at: String,
}

pub async fn insert_cardio_log(pool: &SqlitePool, log: NewCardioLog) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO cardio_logs
         (activity, duration_minutes, distance_km, incline_percent, avg_speed, calories, floors_climbed, logged_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )
    .bind(log.activity)
    .bind(log.duration_minutes)
    .bind(log.distance_km)
    .bind(log.incline_percent)
    .bind(log.avg_speed)
    .bind(log.calories)
    .bind(log.floors_climbed)
    .bind(log.logged_at)
    .execute(pool)
    .await?;

    Ok(())
}