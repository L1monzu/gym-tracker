use crate::db::exercises::{insert_exercise_log, list_exercises, NewExerciseLog, SetEntry};
use dioxus::prelude::*;
use sqlx::SqlitePool;

#[derive(Clone, Default)]
struct SetInput {
    weight: String,
    reps: String,
}

#[component]
pub fn LogExercise() -> Element {
    let pool = use_context::<SqlitePool>();

    let known_exercises = use_resource({
        let pool = pool.clone();
        move || {
            let pool = pool.clone();
            async move { list_exercises(&pool).await.unwrap_or_default() }
        }
    });

    let mut exercise_name = use_signal(String::new);
    let mut logged_at = use_signal(crate::views::start_workout::today);
    let mut set_inputs = use_signal(|| vec![SetInput::default()]);
    let mut status = use_signal(String::new);

    let add_set = move |_| {
        set_inputs.write().push(SetInput::default());
    };

    let mut remove_set = move |index: usize| {
        set_inputs.write().remove(index);
    };

    let save = move |_| {
        let pool = pool.clone();
        let name = exercise_name();
        let date_value = logged_at();
        let raw_sets = set_inputs();

        spawn(async move {
            if name.trim().is_empty() {
                status.set("Enter an exercise name".to_string());
                return;
            }
            if date_value.trim().is_empty() {
                status.set("Pick a date".to_string());
                return;
            }

            let mut parsed_sets = Vec::new();
            for (i, set) in raw_sets.iter().enumerate() {
                let weight = match set.weight.parse::<f64>() {
                    Ok(w) => w,
                    Err(_) => {
                        status.set(format!("Set {}: weight must be a number", i + 1));
                        return;
                    }
                };
                let reps = match set.reps.parse::<i64>() {
                    Ok(r) => r,
                    Err(_) => {
                        status.set(format!("Set {}: reps must be a whole number", i + 1));
                        return;
                    }
                };
                parsed_sets.push(SetEntry { weight, reps });
            }

            let log = NewExerciseLog {
                exercise_name: name,
                logged_at: date_value,
                sets: parsed_sets,
            };

            match insert_exercise_log(&pool, log).await {
                Ok(()) => {
                    status.set("Saved!".to_string());
                    exercise_name.set(String::new());
                    set_inputs.set(vec![SetInput::default()]);
                }
                Err(e) => status.set(format!("Failed to save: {e}")),
            }
        });
    };

    rsx! {
        div { class: "min-h-screen bg-background-light dark:bg-background-dark p-6",
            h1 { class: "text-2xl font-bold text-text-light dark:text-text-dark mb-6", "Log Exercise" }
            div { class: "flex flex-col gap-4 max-w-md",
                label { class: "text-text-light dark:text-text-dark font-medium",
                    "Exercise name"
                    input {
                        class: "mt-1 w-full rounded-md border border-gray-300 dark:border-gray-700 bg-surface-light dark:bg-surface-dark text-text-light dark:text-text-dark px-3 py-2",
                        r#type: "text",
                        list: "known-exercises",
                        value: "{exercise_name}",
                        oninput: move |e| exercise_name.set(e.value()),
                    }
                    datalist { id: "known-exercises",
                        if let Some(exercises) = &*known_exercises.read() {
                            for exercise in exercises {
                                option { value: "{exercise.name}" }
                            }
                        }
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

                div { class: "flex flex-col gap-3",
                    for (i , _set) in set_inputs().iter().enumerate() {
                        div { class: "flex gap-2 items-end",
                            label { class: "text-text-light dark:text-text-dark text-sm flex-1 min-w-0",
                                "Weight (kg)"
                                input {
                                    class: "mt-1 w-full rounded-md border border-gray-300 dark:border-gray-700 bg-surface-light dark:bg-surface-dark text-text-light dark:text-text-dark px-3 py-2",
                                    r#type: "number",
                                    value: "{set_inputs()[i].weight}",
                                    oninput: move |e| set_inputs.write()[i].weight = e.value(),
                                }
                            }
                            label { class: "text-text-light dark:text-text-dark text-sm flex-1 min-w-0",
                                "Reps"
                                input {
                                    class: "mt-1 w-full rounded-md border border-gray-300 dark:border-gray-700 bg-surface-light dark:bg-surface-dark text-text-light dark:text-text-dark px-3 py-2",
                                    r#type: "number",
                                    value: "{set_inputs()[i].reps}",
                                    oninput: move |e| set_inputs.write()[i].reps = e.value(),
                                }
                            }
                            button {
                                class: "text-text-muted px-2 py-2 shrink-0",
                                onclick: move |_| remove_set(i),
                                "x"
                            }
                        }
                    }
                }

                button {
                    class: "bg-accent hover:opacity-90 text-white font-medium rounded-lg px-4 py-2",
                    onclick: add_set,
                    "+ Add set"
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