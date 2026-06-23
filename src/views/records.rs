use crate::db::exercises::list_personal_records;
use crate::views::format::to_british_date;
use dioxus::prelude::*;
use sqlx::SqlitePool;

#[component]
pub fn Records() -> Element {
    let pool = use_context::<SqlitePool>();

    let records = use_resource({
        let pool = pool.clone();
        move || {
            let pool = pool.clone();
            async move { list_personal_records(&pool).await.unwrap_or_default() }
        }
    });

    rsx! {
        div { class: "min-h-screen bg-background-light dark:bg-background-dark p-6",
            h1 { class: "text-2xl font-bold text-text-light dark:text-text-dark mb-6", "Personal Records" }

            match &*records.read() {
                None => rsx! { p { class: "text-text-muted", "Loading..." } },
                Some(list) if list.is_empty() => rsx! {
                    p { class: "text-text-muted", "Log some exercises to see your records here." }
                },
                Some(list) => rsx! {
                    div { class: "flex flex-col gap-3 max-w-md",
                        for record in list {
                            div { class: "bg-surface-light dark:bg-surface-dark rounded-lg p-4",
                                h2 { class: "font-semibold text-text-light dark:text-text-dark mb-1", "{record.exercise_name}" }
                                p { class: "text-sm text-text-muted",
                                    "Heaviest: {record.best_weight}kg x {record.best_weight_reps} ({to_british_date(&record.best_weight_date)})"
                                }
                                p { class: "text-sm text-text-muted",
                                    "Most reps: {record.best_reps} reps at {record.best_reps_weight}kg ({to_british_date(&record.best_reps_date)})"
                                }
                            }
                        }
                    }
                },
            }
        }
    }
}