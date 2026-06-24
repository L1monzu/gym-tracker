use crate::db::templates::{
    add_template_exercise, list_template_exercises, list_templates, remove_template_exercise,
    update_template_exercise, Template, TemplateExercise,
};
use dioxus::prelude::*;
use sqlx::SqlitePool;

#[derive(Clone)]
struct EditableExercise {
    id: i64,
    exercise_name: String,
    target_sets: String,
    rep_range: String,
    rest_time: String,
    suggested_weight: String,
    notes: String,
}

impl From<TemplateExercise> for EditableExercise {
    fn from(te: TemplateExercise) -> Self {
        EditableExercise {
            id: te.id,
            exercise_name: te.exercise_name,
            target_sets: te.target_sets.to_string(),
            rep_range: te.rep_range.unwrap_or_default(),
            rest_time: te.rest_time.unwrap_or_default(),
            suggested_weight: te.suggested_weight.unwrap_or_default(),
            notes: te.notes.unwrap_or_default(),
        }
    }
}

#[component]
pub fn ManageTemplates() -> Element {
    let pool = use_context::<SqlitePool>();
    let mut reload_trigger = use_signal(|| 0);

    let templates = use_resource({
        let pool = pool.clone();
        move || {
            let pool = pool.clone();
            async move { list_templates(&pool).await.unwrap_or_default() }
        }
    });

    let mut selected_template = use_signal(|| None::<Template>);
    let mut exercises = use_signal(Vec::<EditableExercise>::new);
    let mut new_exercise_name = use_signal(String::new);
    let mut status = use_signal(String::new);

    rsx! {
        div { class: "min-h-screen bg-background-light dark:bg-background-dark p-6",
            h1 { class: "text-2xl font-bold text-text-light dark:text-text-dark mb-6", "Manage Templates" }

            if selected_template().is_none() {
                div { class: "flex flex-col gap-3 max-w-md",
                    match &*templates.read() {
                        None => rsx! { p { class: "text-text-muted", "Loading..." } },
                        Some(list) => rsx! {
                            for template in list.clone() {
                                button {
                                    class: "bg-primary hover:bg-primary-dark text-white text-lg font-semibold rounded-lg px-6 py-4 text-left",
                                    onclick: {
                                        let pool = pool.clone();
                                        let template = template.clone();
                                        move |_| {
                                            let pool = pool.clone();
                                            let template = template.clone();
                                            spawn(async move {
                                                let te = list_template_exercises(&pool, template.id).await.unwrap_or_default();
                                                exercises.set(te.into_iter().map(EditableExercise::from).collect());
                                                selected_template.set(Some(template));
                                            });
                                        }
                                    },
                                    "{template.name}"
                                }
                            }
                        },
                    }
                }
            } else {
                div { class: "flex flex-col gap-4 max-w-md",
                    button {
                        class: "text-text-muted text-sm text-left",
                        onclick: move |_| selected_template.set(None),
                        "< Back to templates"
                    }

                    for (i , ex) in exercises().iter().enumerate() {
                        div { class: "bg-surface-light dark:bg-surface-dark rounded-lg p-4 flex flex-col gap-2",
                            input {
                                class: "font-semibold bg-transparent border-b border-gray-300 dark:border-gray-700 text-text-light dark:text-text-dark px-1 py-1",
                                value: "{ex.exercise_name}",
                                oninput: move |e| exercises.write()[i].exercise_name = e.value(),
                            }
                            div { class: "flex gap-2",
                                input {
                                    class: "flex-1 min-w-0 text-sm rounded-md border border-gray-300 dark:border-gray-700 bg-background-light dark:bg-background-dark text-text-light dark:text-text-dark px-2 py-1",
                                    placeholder: "Sets",
                                    value: "{ex.target_sets}",
                                    oninput: move |e| exercises.write()[i].target_sets = e.value(),
                                }
                                input {
                                    class: "flex-1 min-w-0 text-sm rounded-md border border-gray-300 dark:border-gray-700 bg-background-light dark:bg-background-dark text-text-light dark:text-text-dark px-2 py-1",
                                    placeholder: "Reps e.g. 8-10",
                                    value: "{ex.rep_range}",
                                    oninput: move |e| exercises.write()[i].rep_range = e.value(),
                                }
                                input {
                                    class: "flex-1 min-w-0 text-sm rounded-md border border-gray-300 dark:border-gray-700 bg-background-light dark:bg-background-dark text-text-light dark:text-text-dark px-2 py-1",
                                    placeholder: "Rest",
                                    value: "{ex.rest_time}",
                                    oninput: move |e| exercises.write()[i].rest_time = e.value(),
                                }
                            }
                            input {
                                class: "text-sm rounded-md border border-gray-300 dark:border-gray-700 bg-background-light dark:bg-background-dark text-text-light dark:text-text-dark px-2 py-1",
                                placeholder: "Suggested weight",
                                value: "{ex.suggested_weight}",
                                oninput: move |e| exercises.write()[i].suggested_weight = e.value(),
                            }
                            input {
                                class: "text-sm rounded-md border border-gray-300 dark:border-gray-700 bg-background-light dark:bg-background-dark text-text-light dark:text-text-dark px-2 py-1",
                                placeholder: "Notes",
                                value: "{ex.notes}",
                                oninput: move |e| exercises.write()[i].notes = e.value(),
                            }
                            div { class: "flex justify-end",
                                button {
                                    class: "text-text-muted text-sm",
                                    onclick: {
                                        let pool = pool.clone();
                                        let id = ex.id;
                                        move |_| {
                                            let pool = pool.clone();
                                            let template_id = selected_template().as_ref().map(|t| t.id).unwrap_or_default();
                                            spawn(async move {
                                                let _ = remove_template_exercise(&pool, id).await;
                                                let te = list_template_exercises(&pool, template_id).await.unwrap_or_default();
                                                exercises.set(te.into_iter().map(EditableExercise::from).collect());
                                            });
                                        }
                                    },
                                    "Remove"
                                }
                            }
                        }
                    }

                    div { class: "flex gap-2",
                        input {
                            class: "flex-1 min-w-0 rounded-md border border-gray-300 dark:border-gray-700 bg-surface-light dark:bg-surface-dark text-text-light dark:text-text-dark px-3 py-2",
                            placeholder: "New exercise name",
                            value: "{new_exercise_name}",
                            oninput: move |e| new_exercise_name.set(e.value()),
                        }
                        button {
                            class: "bg-accent hover:opacity-90 text-white font-medium rounded-lg px-4 py-2 shrink-0",
                            onclick: {
                                let pool = pool.clone();
                                move |_| {
                                    let pool = pool.clone();
                                    let name = new_exercise_name();
                                    let template_id = selected_template().as_ref().map(|t| t.id).unwrap_or_default();
                                    if name.trim().is_empty() {
                                        return;
                                    }
                                    spawn(async move {
                                        let _ = add_template_exercise(&pool, template_id, &name).await;
                                        let te = list_template_exercises(&pool, template_id).await.unwrap_or_default();
                                        exercises.set(te.into_iter().map(EditableExercise::from).collect());
                                        new_exercise_name.set(String::new());
                                    });
                                }
                            },
                            "+ Add"
                        }
                    }

                    button {
                        class: "bg-primary hover:bg-primary-dark text-white font-semibold rounded-lg px-6 py-3",
                        onclick: {
                            let pool = pool.clone();
                            move |_| {
                                let pool = pool.clone();
                                let all_exercises = exercises();
                                spawn(async move {
                                    status.set("Saving...".to_string());
                                    for ex in &all_exercises {
                                        let sets = ex.target_sets.parse::<i64>().unwrap_or(3);
                                        let _ = update_template_exercise(
                                            &pool,
                                            ex.id,
                                            &ex.exercise_name,
                                            sets,
                                            &ex.rep_range,
                                            &ex.rest_time,
                                            &ex.suggested_weight,
                                            &ex.notes,
                                        )
                                        .await;
                                    }
                                    status.set("Saved!".to_string());
                                    reload_trigger.set(reload_trigger() + 1);
                                });
                            }
                        },
                        "Save Changes"
                    }
                    if !status().is_empty() {
                        p { class: "text-text-muted text-sm", "{status}" }
                    }
                }
            }
        }
    }
}