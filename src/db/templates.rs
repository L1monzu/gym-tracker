use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Template {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TemplateExercise {
    pub id: i64,
    pub exercise_name: String,
    pub target_sets: i64,
    pub rep_range: Option<String>,
    pub rest_time: Option<String>,
    pub suggested_weight: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LastSet {
    pub weight: f64,
    pub reps: i64,
}

pub async fn list_templates(pool: &SqlitePool) -> Result<Vec<Template>, sqlx::Error> {
    sqlx::query_as::<_, Template>("SELECT id, name FROM workout_templates ORDER BY position")
        .fetch_all(pool)
        .await
}

pub async fn list_template_exercises(
    pool: &SqlitePool,
    template_id: i64,
) -> Result<Vec<TemplateExercise>, sqlx::Error> {
    sqlx::query_as::<_, TemplateExercise>(
        "SELECT id, exercise_name, target_sets, rep_range, rest_time, suggested_weight, notes
         FROM template_exercises WHERE template_id = ?1 ORDER BY position",
    )
    .bind(template_id)
    .fetch_all(pool)
    .await
}

/// Swaps which exercise this slot points to, used when the gym is busy
/// and an alternate exercise is needed for the same slot.
pub async fn rename_template_exercise(
    pool: &SqlitePool,
    id: i64,
    new_name: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE template_exercises SET exercise_name = ?1 WHERE id = ?2")
        .bind(new_name)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Swaps the position of this slot with the one directly above or below
/// it. `direction` is -1 to move up, 1 to move down.
pub async fn move_template_exercise(
    pool: &SqlitePool,
    template_id: i64,
    id: i64,
    direction: i64,
) -> Result<(), sqlx::Error> {
    let current_position: i64 =
        sqlx::query_scalar("SELECT position FROM template_exercises WHERE id = ?1")
            .bind(id)
            .fetch_one(pool)
            .await?;

    let target_position = current_position + direction;

    let neighbour_id: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM template_exercises WHERE template_id = ?1 AND position = ?2",
    )
    .bind(template_id)
    .bind(target_position)
    .fetch_optional(pool)
    .await?;

    if let Some(neighbour_id) = neighbour_id {
        sqlx::query("UPDATE template_exercises SET position = ?1 WHERE id = ?2")
            .bind(target_position)
            .bind(id)
            .execute(pool)
            .await?;
        sqlx::query("UPDATE template_exercises SET position = ?1 WHERE id = ?2")
            .bind(current_position)
            .bind(neighbour_id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

/// Returns the sets from the most recent time this exercise was logged,
/// so a new entry can start pre-filled with what was lifted last time.
/// Returns an empty list if the exercise has never been logged before.
pub async fn last_logged_sets(
    pool: &SqlitePool,
    exercise_name: &str,
) -> Result<Vec<LastSet>, sqlx::Error> {
    let last_date: Option<String> = sqlx::query_scalar(
        "SELECT exercise_logs.logged_at
         FROM exercise_logs
         JOIN exercises ON exercises.id = exercise_logs.exercise_id
         WHERE exercises.name = ?1 COLLATE NOCASE AND exercise_logs.deleted_at IS NULL
         ORDER BY exercise_logs.logged_at DESC, exercise_logs.id DESC
         LIMIT 1",
    )
    .bind(exercise_name)
    .fetch_optional(pool)
    .await?;

    let Some(date) = last_date else {
        return Ok(Vec::new());
    };

    #[derive(sqlx::FromRow)]
    struct Row {
        weight: f64,
        reps: i64,
    }

    let rows = sqlx::query_as::<_, Row>(
        "SELECT weight, reps
         FROM exercise_logs
         JOIN exercises ON exercises.id = exercise_logs.exercise_id
         WHERE exercises.name = ?1 COLLATE NOCASE
           AND exercise_logs.logged_at = ?2
           AND exercise_logs.deleted_at IS NULL
         ORDER BY exercise_logs.id",
    )
    .bind(exercise_name)
    .bind(date)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| LastSet { weight: r.weight, reps: r.reps }).collect())
}

/// Records that this template was just completed, used to figure out
/// which day comes next in the rotation.
pub async fn record_template_completion(pool: &SqlitePool, template_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO template_completions (template_id) VALUES (?1)")
        .bind(template_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Looks at the most recently completed template and returns the next
/// one in the rotation (by position, wrapping back to the first after
/// the last). Returns the first template if nothing has been completed
/// yet.
pub async fn suggested_next_template(pool: &SqlitePool) -> Result<Option<Template>, sqlx::Error> {
    let last_template_id: Option<i64> = sqlx::query_scalar(
        "SELECT template_id FROM template_completions ORDER BY logged_at DESC, id DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    let templates = list_templates(pool).await?;
    if templates.is_empty() {
        return Ok(None);
    }

    let Some(last_id) = last_template_id else {
        return Ok(Some(templates[0].clone()));
    };

    let current_index = templates.iter().position(|t| t.id == last_id);
    let next_index = match current_index {
        Some(i) => (i + 1) % templates.len(),
        None => 0,
    };

    Ok(Some(templates[next_index].clone()))
}