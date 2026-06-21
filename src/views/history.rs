use crate::db::export::build_workbook;
use crate::db::history::{delete_cardio_log, delete_exercise_logs, load_history, HistoryEntry};
use dioxus::prelude::*;
use sqlx::SqlitePool;

#[component]
pub fn History() -> Element {
    let pool = use_context::<SqlitePool>();
    let mut reload_trigger = use_signal(|| 0);
    let mut confirming_key = use_signal(|| None::<String>);

    let mut export_status = use_signal(String::new);

    let do_export = {
        let pool = pool.clone();
        move |_| {
            let pool = pool.clone();
            spawn(async move {
                export_status.set("Exporting...".to_string());
                match build_workbook(&pool).await {
                    Ok(bytes) => {
                        #[cfg(target_os = "android")]
                        {
                            match crate::db::save_to_downloads("gym-tracker-export.xlsx", &bytes) {
                                Ok(()) => export_status
                                    .set("Saved to Downloads. Open the Files app or Drive to share it.".to_string()),
                                Err(e) => export_status.set(format!("Failed to save file: {e}")),
                            }
                        }
                        #[cfg(not(target_os = "android"))]
                        {
                            let path = crate::db::shareable_files_dir().join("gym-tracker-export.xlsx");
                            match std::fs::write(&path, bytes) {
                                Ok(()) => export_status.set(format!("Saved to {}", path.display())),
                                Err(e) => export_status.set(format!("Failed to save file: {e}")),
                            }
                        }
                    }
                    Err(e) => export_status.set(format!("Export failed: {e}")),
                }
            });
        }
    };

    let history = use_resource({
        let pool = pool.clone();
        move || {
            let _ = reload_trigger();
            let pool = pool.clone();
            async move { load_history(&pool).await.unwrap_or_default() }
        }
    });

    rsx! {
        div { class: "min-h-screen bg-background-light dark:bg-background-dark p-6",
            h1 { class: "text-2xl font-bold text-text-light dark:text-text-dark mb-6", "History" }

            div { class: "mb-6",
                button {
                    class: "bg-accent hover:opacity-90 text-white font-medium rounded-lg px-4 py-2",
                    onclick: do_export,
                    "Export to Excel"
                }
                if !export_status().is_empty() {
                    p { class: "text-text-muted text-sm mt-2 break-words", "{export_status}" }
                }
            }

            match &*history.read() {
                None => rsx! { p { class: "text-text-muted", "Loading..." } },
                Some(groups) if groups.is_empty() => rsx! {
                    p { class: "text-text-muted", "No workouts logged yet." }
                },
                Some(groups) => rsx! {
                    div { class: "flex flex-col gap-6 max-w-md",
                        for group in groups {
                            div {
                                h2 { class: "text-lg font-semibold text-text-light dark:text-text-dark mb-2",
                                    "{group.date}"
                                }
                                div { class: "flex flex-col gap-2",
                                    for (entry_index , entry) in group.entries.iter().enumerate() {
                                        {
                                            let key = format!("{}-{}", group.date, entry_index);
                                            let is_confirming = confirming_key() == Some(key.clone());
                                            let pool_for_confirm = pool.clone();

                                            let (summary, delete_ids, is_exercise) = match entry {
                                                HistoryEntry::Exercise { exercise_name, sets } => {
                                                    let set_text = sets
                                                        .iter()
                                                        .map(|s| format!("{}kg x {}", s.weight, s.reps))
                                                        .collect::<Vec<_>>()
                                                        .join(", ");
                                                    let ids = sets.iter().map(|s| s.id).collect::<Vec<_>>();
                                                    (format!("{exercise_name} — {set_text}"), ids, true)
                                                }
                                                HistoryEntry::Cardio { id, activity, duration_minutes, distance_km, .. } => {
                                                    let distance_text = distance_km
                                                        .map(|d| format!(", {d}km"))
                                                        .unwrap_or_default();
                                                    (
                                                        format!("{activity} — {duration_minutes}min{distance_text}"),
                                                        vec![*id],
                                                        false,
                                                    )
                                                }
                                            };

                                            rsx! {
                                                div { class: "flex justify-between items-center bg-surface-light dark:bg-surface-dark rounded-md px-3 py-2",
                                                    span { class: "text-text-light dark:text-text-dark", "{summary}" }
                                                    if is_confirming {
                                                        div { class: "flex gap-2",
                                                            button {
                                                                class: "text-primary font-medium text-sm",
                                                                onclick: move |_| {
                                                                    let pool = pool_for_confirm.clone();
                                                                    let ids = delete_ids.clone();
                                                                    spawn(async move {
                                                                        let result = if is_exercise {
                                                                            delete_exercise_logs(&pool, &ids).await
                                                                        } else {
                                                                            delete_cardio_log(&pool, ids[0]).await
                                                                        };
                                                                        if result.is_ok() {
                                                                            confirming_key.set(None);
                                                                            reload_trigger.set(reload_trigger() + 1);
                                                                        }
                                                                    });
                                                                },
                                                                "Confirm"
                                                            }
                                                            button {
                                                                class: "text-text-muted text-sm",
                                                                onclick: move |_| confirming_key.set(None),
                                                                "Cancel"
                                                            }
                                                        }
                                                    } else {
                                                        button {
                                                            class: "text-text-muted text-sm",
                                                            onclick: move |_| confirming_key.set(Some(key.clone())),
                                                            "Delete"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
            }
        }
    }
}