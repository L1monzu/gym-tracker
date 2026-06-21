use rust_xlsxwriter::{ExcelDateTime, Format, Workbook};
use sqlx::SqlitePool;

use super::history::{load_history, HistoryEntry};

struct OverviewRow {
    date: String,
    kind: &'static str,
    details: String,
    avg_weight: Option<f64>,
    avg_reps: Option<f64>,
}

struct ExerciseRow {
    date: String,
    exercise_name: String,
    set_number: i64,
    weight: f64,
    reps: i64,
}

struct CardioRow {
    date: String,
    activity: String,
    duration_minutes: i64,
    distance_km: Option<f64>,
    incline_percent: Option<f64>,
    avg_speed: Option<f64>,
    calories: Option<i64>,
    floors_climbed: Option<i64>,
}

/// Builds a workbook with three sheets, a combined overview sorted by
/// date, then separate detail sheets for exercises and cardio. Returns
/// the finished file as raw bytes, ready to write to disk.
pub async fn build_workbook(pool: &SqlitePool) -> Result<Vec<u8>, String> {
    let groups = load_history(pool).await.map_err(|e| e.to_string())?;

    let mut overview_rows = Vec::new();
    let mut exercise_rows = Vec::new();
    let mut cardio_rows = Vec::new();

    for group in &groups {
        for entry in &group.entries {
            match entry {
                HistoryEntry::Exercise { exercise_name, sets } => {
                    let set_text = sets
                        .iter()
                        .map(|s| format!("{}kg x {}", s.weight, s.reps))
                        .collect::<Vec<_>>()
                        .join(", ");

                    let set_count = sets.len() as f64;
                    let avg_weight = sets.iter().map(|s| s.weight).sum::<f64>() / set_count;
                    let avg_reps = sets.iter().map(|s| s.reps as f64).sum::<f64>() / set_count;

                    overview_rows.push(OverviewRow {
                        date: group.date.clone(),
                        kind: "Exercise",
                        details: format!("{exercise_name} - {set_text}"),
                        avg_weight: Some(avg_weight),
                        avg_reps: Some(avg_reps),
                    });

                    for (set_index, set) in sets.iter().enumerate() {
                        exercise_rows.push(ExerciseRow {
                            date: group.date.clone(),
                            exercise_name: exercise_name.clone(),
                            set_number: (set_index + 1) as i64,
                            weight: set.weight,
                            reps: set.reps,
                        });
                    }
                }
                HistoryEntry::Cardio {
                    activity,
                    duration_minutes,
                    distance_km,
                    incline_percent,
                    avg_speed,
                    calories,
                    floors_climbed,
                    ..
                } => {
                    let distance_text = distance_km.map(|d| format!(", {d}km")).unwrap_or_default();

                    overview_rows.push(OverviewRow {
                        date: group.date.clone(),
                        kind: "Cardio",
                        details: format!("{activity} - {duration_minutes}min{distance_text}"),
                        avg_weight: None,
                        avg_reps: None,
                    });

                    cardio_rows.push(CardioRow {
                        date: group.date.clone(),
                        activity: activity.clone(),
                        duration_minutes: *duration_minutes,
                        distance_km: *distance_km,
                        incline_percent: *incline_percent,
                        avg_speed: *avg_speed,
                        calories: *calories,
                        floors_climbed: *floors_climbed,
                    });
                }
            }
        }
    }

    let mut workbook = Workbook::new();
    let bold = Format::new().set_bold();
    let date_format = Format::new().set_num_format("yyyy-mm-dd");

    {
        let sheet = workbook.add_worksheet().set_name("Overview").map_err(|e| e.to_string())?;
        sheet.write_with_format(0, 0, "Date", &bold).map_err(|e| e.to_string())?;
        sheet.write_with_format(0, 1, "Type", &bold).map_err(|e| e.to_string())?;
        sheet.write_with_format(0, 2, "Details", &bold).map_err(|e| e.to_string())?;
        sheet.write_with_format(0, 3, "Avg weight (kg)", &bold).map_err(|e| e.to_string())?;
        sheet.write_with_format(0, 4, "Avg reps", &bold).map_err(|e| e.to_string())?;

        for (i, row) in overview_rows.iter().enumerate() {
            let r = (i + 1) as u32;
            if let Ok(date) = ExcelDateTime::parse_from_str(&row.date) {
                sheet.write_with_format(r, 0, &date, &date_format).map_err(|e| e.to_string())?;
            } else {
                sheet.write(r, 0, &row.date).map_err(|e| e.to_string())?;
            }
            sheet.write(r, 1, row.kind).map_err(|e| e.to_string())?;
            sheet.write(r, 2, &row.details).map_err(|e| e.to_string())?;
            if let Some(w) = row.avg_weight {
                sheet.write(r, 3, w).map_err(|e| e.to_string())?;
            }
            if let Some(reps) = row.avg_reps {
                sheet.write(r, 4, reps).map_err(|e| e.to_string())?;
            }
        }

        sheet.set_column_width(0, 12).map_err(|e| e.to_string())?;
        sheet.set_column_width(1, 10).map_err(|e| e.to_string())?;
        sheet.set_column_width(2, 40).map_err(|e| e.to_string())?;
        sheet.set_column_width(3, 14).map_err(|e| e.to_string())?;
        sheet.set_column_width(4, 10).map_err(|e| e.to_string())?;
        sheet.set_freeze_panes(1, 0).map_err(|e| e.to_string())?;
    }

    {
        let sheet = workbook.add_worksheet().set_name("Exercises").map_err(|e| e.to_string())?;
        for (col, header) in ["Date", "Exercise", "Set", "Weight (kg)", "Reps"].iter().enumerate() {
            sheet.write_with_format(0, col as u16, *header, &bold).map_err(|e| e.to_string())?;
        }

        for (i, row) in exercise_rows.iter().enumerate() {
            let r = (i + 1) as u32;
            if let Ok(date) = ExcelDateTime::parse_from_str(&row.date) {
                sheet.write_with_format(r, 0, &date, &date_format).map_err(|e| e.to_string())?;
            } else {
                sheet.write(r, 0, &row.date).map_err(|e| e.to_string())?;
            }
            sheet.write(r, 1, &row.exercise_name).map_err(|e| e.to_string())?;
            sheet.write(r, 2, row.set_number as f64).map_err(|e| e.to_string())?;
            sheet.write(r, 3, row.weight).map_err(|e| e.to_string())?;
            sheet.write(r, 4, row.reps as f64).map_err(|e| e.to_string())?;
        }

        sheet.set_column_width(0, 12).map_err(|e| e.to_string())?;
        sheet.set_column_width(1, 20).map_err(|e| e.to_string())?;
        sheet.set_column_width(3, 12).map_err(|e| e.to_string())?;
        sheet.set_freeze_panes(1, 0).map_err(|e| e.to_string())?;
    }

    {
        let sheet = workbook.add_worksheet().set_name("Cardio").map_err(|e| e.to_string())?;
        for (col, header) in [
            "Date", "Activity", "Duration (min)", "Distance (km)",
            "Incline (%)", "Avg speed", "Calories", "Floors climbed",
        ]
        .iter()
        .enumerate()
        {
            sheet.write_with_format(0, col as u16, *header, &bold).map_err(|e| e.to_string())?;
        }

        for (i, row) in cardio_rows.iter().enumerate() {
            let r = (i + 1) as u32;
            if let Ok(date) = ExcelDateTime::parse_from_str(&row.date) {
                sheet.write_with_format(r, 0, &date, &date_format).map_err(|e| e.to_string())?;
            } else {
                sheet.write(r, 0, &row.date).map_err(|e| e.to_string())?;
            }
            sheet.write(r, 1, &row.activity).map_err(|e| e.to_string())?;
            sheet.write(r, 2, row.duration_minutes as f64).map_err(|e| e.to_string())?;
            if let Some(d) = row.distance_km {
                sheet.write(r, 3, d).map_err(|e| e.to_string())?;
            }
            if let Some(i) = row.incline_percent {
                sheet.write(r, 4, i).map_err(|e| e.to_string())?;
            }
            if let Some(s) = row.avg_speed {
                sheet.write(r, 5, s).map_err(|e| e.to_string())?;
            }
            if let Some(c) = row.calories {
                sheet.write(r, 6, c as f64).map_err(|e| e.to_string())?;
            }
            if let Some(f) = row.floors_climbed {
                sheet.write(r, 7, f as f64).map_err(|e| e.to_string())?;
            }
        }

        sheet.set_column_width(0, 12).map_err(|e| e.to_string())?;
        sheet.set_column_width(1, 16).map_err(|e| e.to_string())?;
        sheet.set_freeze_panes(1, 0).map_err(|e| e.to_string())?;
    }

    workbook.save_to_buffer().map_err(|e| e.to_string())
}