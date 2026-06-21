use crate::db::cardio::{insert_cardio_log, list_cardio_activities, NewCardioLog};
use dioxus::prelude::*;
use sqlx::SqlitePool;

#[component]
pub fn LogCardio() -> Element {
    let pool = use_context::<SqlitePool>();

    let known_activities = use_resource({
        let pool = pool.clone();
        move || {
            let pool = pool.clone();
            async move { list_cardio_activities(&pool).await.unwrap_or_default() }
        }
    });

    let mut activity = use_signal(String::new);
    let mut duration = use_signal(String::new);
    let mut distance = use_signal(String::new);
    let mut incline = use_signal(String::new);
    let mut avg_speed = use_signal(String::new);
    let mut calories = use_signal(String::new);
    let mut floors = use_signal(String::new);
    let mut logged_at = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut show_details = use_signal(|| false);

    // Parses a text field into an optional number. Blank text means "not
    // tracked for this activity" rather than an error.
    fn parse_optional<T: std::str::FromStr>(text: &str) -> Result<Option<T>, ()> {
        if text.trim().is_empty() {
            Ok(None)
        } else {
            text.parse::<T>().map(Some).map_err(|_| ())
        }
    }

    let save = move |_| {
        let pool = pool.clone();
        let activity_value = activity();
        let duration_value = duration();
        let distance_value = distance();
        let incline_value = incline();
        let avg_speed_value = avg_speed();
        let calories_value = calories();
        let floors_value = floors();
        let date_value = logged_at();

        spawn(async move {
            if activity_value.trim().is_empty() {
                status.set("Enter an activity".to_string());
                return;
            }
            if date_value.trim().is_empty() {
                status.set("Pick a date".to_string());
                return;
            }

            let duration_parsed = match duration_value.parse::<i64>() {
                Ok(d) => d,
                Err(_) => {
                    status.set("Duration must be a whole number of minutes".to_string());
                    return;
                }
            };

            let distance_parsed = match parse_optional::<f64>(&distance_value) {
                Ok(v) => v,
                Err(_) => {
                    status.set("Distance must be a number, e.g. 5.2".to_string());
                    return;
                }
            };
            let incline_parsed = match parse_optional::<f64>(&incline_value) {
                Ok(v) => v,
                Err(_) => {
                    status.set("Incline must be a number".to_string());
                    return;
                }
            };
            let avg_speed_parsed = match parse_optional::<f64>(&avg_speed_value) {
                Ok(v) => v,
                Err(_) => {
                    status.set("Speed must be a number".to_string());
                    return;
                }
            };
            let calories_parsed = match parse_optional::<i64>(&calories_value) {
                Ok(v) => v,
                Err(_) => {
                    status.set("Calories must be a whole number".to_string());
                    return;
                }
            };
            let floors_parsed = match parse_optional::<i64>(&floors_value) {
                Ok(v) => v,
                Err(_) => {
                    status.set("Floors must be a whole number".to_string());
                    return;
                }
            };

            let log = NewCardioLog {
                activity: activity_value,
                duration_minutes: duration_parsed,
                distance_km: distance_parsed,
                incline_percent: incline_parsed,
                avg_speed: avg_speed_parsed,
                calories: calories_parsed,
                floors_climbed: floors_parsed,
                logged_at: date_value,
            };

            match insert_cardio_log(&pool, log).await {
                Ok(()) => {
                    status.set("Saved!".to_string());
                    activity.set(String::new());
                    duration.set(String::new());
                    distance.set(String::new());
                    incline.set(String::new());
                    avg_speed.set(String::new());
                    calories.set(String::new());
                    floors.set(String::new());
                    logged_at.set(String::new());
                }
                Err(e) => status.set(format!("Failed to save: {e}")),
            }
        });
    };

    rsx! {
        div { class: "min-h-screen bg-background-light dark:bg-background-dark p-6",
            h1 { class: "text-2xl font-bold text-text-light dark:text-text-dark mb-6", "Log Cardio" }
            div { class: "flex flex-col gap-4 max-w-md",
                label { class: "text-text-light dark:text-text-dark font-medium",
                    "Activity"
                    input {
                        class: "mt-1 w-full rounded-md border border-gray-300 dark:border-gray-700 bg-surface-light dark:bg-surface-dark text-text-light dark:text-text-dark px-3 py-2",
                        r#type: "text",
                        list: "known-activities",
                        value: "{activity}",
                        oninput: move |e| activity.set(e.value()),
                    }
                    datalist { id: "known-activities",
                        if let Some(activities) = &*known_activities.read() {
                            for a in activities {
                                option { value: "{a.activity}" }
                            }
                        }
                    }
                }
                label { class: "text-text-light dark:text-text-dark font-medium",
                    "Duration (minutes)"
                    input {
                        class: "mt-1 w-full rounded-md border border-gray-300 dark:border-gray-700 bg-surface-light dark:bg-surface-dark text-text-light dark:text-text-dark px-3 py-2",
                        r#type: "number",
                        value: "{duration}",
                        oninput: move |e| duration.set(e.value()),
                    }
                }
                label { class: "text-text-light dark:text-text-dark font-medium",
                    "Distance (km, optional)"
                    input {
                        class: "mt-1 w-full rounded-md border border-gray-300 dark:border-gray-700 bg-surface-light dark:bg-surface-dark text-text-light dark:text-text-dark px-3 py-2",
                        r#type: "number",
                        value: "{distance}",
                        oninput: move |e| distance.set(e.value()),
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
                    class: "text-accent text-left font-medium",
                    onclick: move |_| show_details.set(!show_details()),
                    if show_details() { "− Hide details" } else { "+ Add more details" }
                }

                if show_details() {
                    div { class: "flex flex-col gap-4 border-l-2 border-accent pl-4",
                        label { class: "text-text-light dark:text-text-dark font-medium",
                            "Incline (%)"
                            input {
                                class: "mt-1 w-full rounded-md border border-gray-300 dark:border-gray-700 bg-surface-light dark:bg-surface-dark text-text-light dark:text-text-dark px-3 py-2",
                                r#type: "number",
                                value: "{incline}",
                                oninput: move |e| incline.set(e.value()),
                            }
                        }
                        label { class: "text-text-light dark:text-text-dark font-medium",
                            "Avg speed"
                            input {
                                class: "mt-1 w-full rounded-md border border-gray-300 dark:border-gray-700 bg-surface-light dark:bg-surface-dark text-text-light dark:text-text-dark px-3 py-2",
                                r#type: "number",
                                value: "{avg_speed}",
                                oninput: move |e| avg_speed.set(e.value()),
                            }
                        }
                        label { class: "text-text-light dark:text-text-dark font-medium",
                            "Calories"
                            input {
                                class: "mt-1 w-full rounded-md border border-gray-300 dark:border-gray-700 bg-surface-light dark:bg-surface-dark text-text-light dark:text-text-dark px-3 py-2",
                                r#type: "number",
                                value: "{calories}",
                                oninput: move |e| calories.set(e.value()),
                            }
                        }
                        label { class: "text-text-light dark:text-text-dark font-medium",
                            "Floors climbed"
                            input {
                                class: "mt-1 w-full rounded-md border border-gray-300 dark:border-gray-700 bg-surface-light dark:bg-surface-dark text-text-light dark:text-text-dark px-3 py-2",
                                r#type: "number",
                                value: "{floors}",
                                oninput: move |e| floors.set(e.value()),
                            }
                        }
                    }
                }

                button {
                    class: "bg-primary hover:bg-primary-dark text-white text-lg font-semibold rounded-lg px-6 py-3",
                    onclick: save,
                    "Save"
                }
                if !status().is_empty() {
                    p { class: "text-text-muted", "{status}" }
                }
            }
        }
    }
}