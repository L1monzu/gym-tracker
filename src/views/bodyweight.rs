use crate::db::bodyweight::{delete_body_weight, insert_body_weight, list_body_weight};
use crate::views::format::to_british_date;
use crate::views::start_workout::today;
use dioxus::prelude::*;
use sqlx::SqlitePool;

#[component]
pub fn BodyWeight() -> Element {
    let pool = use_context::<SqlitePool>();
    let mut reload_trigger = use_signal(|| 0);
    let mut confirming_id = use_signal(|| None::<i64>);

    let entries = use_resource({
        let pool = pool.clone();
        move || {
            let _ = reload_trigger();
            let pool = pool.clone();
            async move { list_body_weight(&pool).await.unwrap_or_default() }
        }
    });

    let mut weight = use_signal(String::new);
    let mut logged_at = use_signal(today);
    let mut status = use_signal(String::new);

    let save = {
        let pool = pool.clone();
        move |_| {
        let pool = pool.clone();
        let weight_value = weight();
        let date_value = logged_at();

        spawn(async move {
            let parsed = match weight_value.parse::<f64>() {
                Ok(w) => w,
                Err(_) => {
                    status.set("Weight must be a number".to_string());
                    return;
                }
            };

            match insert_body_weight(&pool, parsed, &date_value).await {
                Ok(()) => {
                    status.set("Saved!".to_string());
                    weight.set(String::new());
                    reload_trigger.set(reload_trigger() + 1);
                }
                Err(e) => status.set(format!("Failed to save: {e}")),
            }
        });
        }
    };

    rsx! {
        div { class: "min-h-screen bg-background-light dark:bg-background-dark p-6",
            h1 { class: "text-2xl font-bold text-text-light dark:text-text-dark mb-6", "Bodyweight" }

            div { class: "flex flex-col gap-4 max-w-md mb-6",
                label { class: "text-text-light dark:text-text-dark font-medium",
                    "Weight (kg)"
                    input {
                        class: "mt-1 w-full rounded-md border border-gray-300 dark:border-gray-700 bg-surface-light dark:bg-surface-dark text-text-light dark:text-text-dark px-3 py-2",
                        r#type: "number",
                        value: "{weight}",
                        oninput: move |e| weight.set(e.value()),
                    }
                }
                label { class: "text-text-light dark:text-text-dark font-medium",
                    "Date"
                    input {
                        class: "mt-1 w-full rounded-md border border-gray-300 dark:border-gray-700 bg-surface-light dark:bg-surface-dark text-text-light dark:text-text-dark px-3 py-2",
                        r#type: "date",
                        value: "{logged_at}",
                        oninput: move |e| logged_at.set(e.value()),
                    }
                }
                button {
                    class: "bg-primary hover:bg-primary-dark text-white font-semibold rounded-lg px-6 py-3",
                    onclick: save,
                    "Save"
                }
                if !status().is_empty() {
                    p { class: "text-text-muted", "{status}" }
                }
            }
            {
            let entries_state = entries.read();
            match &*entries_state {
                None => rsx! { p { class: "text-text-muted", "Loading..." } },
                Some(list) if list.is_empty() => rsx! {
                    p { class: "text-text-muted", "No bodyweight entries yet." }
                },
                Some(list) => rsx! {
                    div { class: "flex flex-col gap-2 max-w-md",
                        for entry in list {
                            { let id = entry.id; rsx! {
                            div { class: "flex justify-between items-center bg-surface-light dark:bg-surface-dark rounded-md px-3 py-2",
                                span { class: "text-text-light dark:text-text-dark",
                                    "{entry.weight_kg}kg, {to_british_date(&entry.logged_at)}"
                                }
                                if confirming_id() == Some(id) {
                                    div { class: "flex gap-2",
                                        button {
                                            class: "text-primary font-medium text-sm",
                                            onclick: {
                                                let pool = pool.clone();
                                                move |_| {
                                                    let pool = pool.clone();
                                                    spawn(async move {
                                                        let _ = delete_body_weight(&pool, id).await;
                                                        confirming_id.set(None);
                                                        reload_trigger.set(reload_trigger() + 1);
                                                    });
                                                }
                                            },
                                            "Confirm"
                                        }
                                        button {
                                            class: "text-text-muted text-sm",
                                            onclick: move |_| confirming_id.set(None),
                                            "Cancel"
                                        }
                                    }
                                } else {
                                    button {
                                        class: "text-text-muted text-sm",
                                        onclick: move |_| confirming_id.set(Some(id)),
                                        "Delete"
                                    }
                                }
                            }
                            } }
                        }
                    }
                },
            }
            }
        }
    }
}