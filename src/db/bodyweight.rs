use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BodyWeightEntry {
    pub id: i64,
    pub weight_kg: f64,
    pub logged_at: String,
}

pub async fn insert_body_weight(pool: &SqlitePool, weight_kg: f64, logged_at: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO body_weight_logs (weight_kg, logged_at) VALUES (?1, ?2)")
        .bind(weight_kg)
        .bind(logged_at)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_body_weight(pool: &SqlitePool) -> Result<Vec<BodyWeightEntry>, sqlx::Error> {
    sqlx::query_as::<_, BodyWeightEntry>(
        "SELECT id, weight_kg, logged_at FROM body_weight_logs
         WHERE deleted_at IS NULL ORDER BY logged_at DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn delete_body_weight(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE body_weight_logs SET deleted_at = datetime('now') WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}