use crate::db::exercises::{insert_exercise_log, NewExerciseLog, SetEntry};
use crate::db::templates::{
    last_logged_sets, list_template_exercises, list_templates, move_template_exercise,
    record_template_completion, suggested_next_template, Template, TemplateExercise,
};
use dioxus::prelude::*;
use sqlx::SqlitePool;

#[derive(Clone, Default)]
struct SetInput {
    weight: String,
    reps: String,
}

#[derive(Clone)]
struct ExerciseCard {
    id: i64,
    name: String,
    rep_range: Option<String>,
    rest_time: Option<String>,
    suggested_weight: Option<String>,
    notes: Option<String>,
    sets: Vec<SetInput>,
}

pub fn today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let days = secs / 86400;
    civil_date_from_days(days as i64)
}

fn civil_date_from_days(z: i64) -> String {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", y, m, d)
}

async fn build_cards(pool: &SqlitePool, exercises: Vec<TemplateExercise>) -> Vec<ExerciseCard> {
    let mut new_cards = Vec::new();

    for te in exercises {
        let last_sets = last_logged_sets(pool, &te.exercise_name).await.unwrap_or_default();

        let sets = if !last_sets.is_empty() {
            last_sets
                .into_iter()
                .map(|s| SetInput { weight: s.weight.to_string(), reps: s.reps.to_string() })
                .collect()
        } else {
            (0..te.target_sets).map(|_| SetInput::default()).collect()
        };

        new_cards.push(ExerciseCard {
            id: te.id,
            name: te.exercise_name,
            rep_range: te.rep_range,
            rest_time: te.rest_time,
            suggested_weight: te.suggested_weight,
            notes: te.notes,
            sets,
        });
    }

    new_cards
}

#[component]
pub fn StartWorkout() -> Element {
    let pool = use_context::<SqlitePool>();

    let mut suggestion_reload = use_signal(|| 0);

    let templates = use_resource({
        let pool = pool.clone();
        move || {
            let pool = pool.clone();
            async move { list_templates(&pool).await.unwrap_or_default() }
        }
    });

    let suggested = use_resource({
        let pool = pool.clone();
        move || {
            let _ = suggestion_reload();
            let pool = pool.clone();
            async move { suggested_next_template(&pool).await.ok().flatten() }
        }
    });

    let mut selected_template = use_signal(|| None::<Template>);
    let mut cards = use_signal(Vec::<ExerciseCard>::new);
    let mut status = use_signal(String::new);
    let mut logged_at = use_signal(today);
    let mut just_moved_id = use_signal(|| None::<i64>);

    use_effect(move || {
        if just_moved_id().is_some() {
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(700)).await;
                just_moved_id.set(None);
            });
        }
    });

    rsx! {
        div { class: "min-h-screen bg-background-light dark:bg-background-dark p-6",
            h1 { class: "text-2xl font-bold text-text-light dark:text-text-dark mb-6", "Start Workout" }

            if selected_template().is_none() {
                div { class: "flex flex-col gap-3 max-w-md",
                    if let Some(Some(next)) = &*suggested.read() {
                        p { class: "text-text-muted text-sm mb-2", "Suggested next: {next.name}" }
                    }
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
                                                let exercises =
                                                    list_template_exercises(&pool, template.id).await.unwrap_or_default();
                                                let new_cards = build_cards(&pool, exercises).await;
                                                cards.set(new_cards);
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
                div { class: "flex flex-col gap-5 max-w-md",
                    label { class: "text-text-light dark:text-text-dark font-medium",
                        "Date"
                        input {
                            class: "mt-1 w-full rounded-md border border-gray-300 dark:border-gray-700 bg-surface-light dark:bg-surface-dark text-text-light dark:text-text-dark px-3 py-2",
                            r#type: "date",
                            value: "{logged_at}",
                            oninput: move |e| logged_at.set(e.value()),
                        }
                        span { class: "text-xs text-text-muted", "{crate::views::format::to_british_date(&logged_at())}" }
                    }

                    for (card_index , card) in cards().iter().enumerate() {
                        {
                        let id = card.id;
                        rsx! {
                        div {
                            key: "{card.id}",
                            class: if just_moved_id() == Some(card.id) {
                                "bg-accent/20 dark:bg-accent/20 rounded-lg p-4 flex flex-col gap-3 transition-colors duration-700 ease-out"
                            } else {
                                "bg-surface-light dark:bg-surface-dark rounded-lg p-4 flex flex-col gap-3 transition-colors duration-700 ease-out"
                            },
                            div { class: "flex items-center gap-2",
                                input {
                                    class: "flex-1 min-w-0 font-semibold text-text-light dark:text-text-dark bg-transparent border-b border-gray-300 dark:border-gray-700 px-1 py-1",
                                    value: "{card.name}",
                                    oninput: move |e| cards.write()[card_index].name = e.value(),
                                }
                                button {
                                    class: "text-text-muted px-2 shrink-0",
                                    onclick: {
                                        let pool = pool.clone();
                                        move |_| {
                                            if card_index == 0 {
                                                return;
                                            }
                                            cards.write().swap(card_index, card_index - 1);
                                            just_moved_id.set(Some(id));

                                            let pool = pool.clone();
                                            let template_id = selected_template().as_ref().map(|t| t.id).unwrap_or_default();
                                            spawn(async move {
                                                let _ = move_template_exercise(&pool, template_id, id, -1).await;
                                            });
                                        }
                                    },
                                    "^"
                                }
                                button {
                                    class: "text-text-muted px-2 shrink-0",
                                    onclick: {
                                        let pool = pool.clone();
                                        move |_| {
                                            if card_index + 1 >= cards().len() {
                                                return;
                                            }
                                            cards.write().swap(card_index, card_index + 1);
                                            just_moved_id.set(Some(id));

                                            let pool = pool.clone();
                                            let template_id = selected_template().as_ref().map(|t| t.id).unwrap_or_default();
                                            spawn(async move {
                                                let _ = move_template_exercise(&pool, template_id, id, 1).await;
                                            });
                                        }
                                    },
                                    "v"
                                }
                            }

                            div { class: "text-xs text-text-muted",
                                if let Some(reps) = &card.rep_range {
                                    span { "{reps} reps" }
                                }
                                if let Some(rest) = &card.rest_time {
                                    span { "  -  rest {rest}" }
                                }
                            }
                            if let Some(notes) = &card.notes {
                                p { class: "text-xs text-text-muted italic", "{notes}" }
                            }
                            if card.sets.is_empty() {
                                if let Some(sw) = &card.suggested_weight {
                                    p { class: "text-xs text-accent", "Suggested start: {sw}" }
                                }
                            }

                            div { class: "flex flex-col gap-2",
                                for (set_index , set) in card.sets.iter().enumerate() {
                                    div { class: "flex gap-2 items-center",
                                        span { class: "text-xs text-text-muted w-10 shrink-0", "Set {set_index + 1}" }
                                        input {
                                            class: "flex-1 min-w-0 rounded-md border border-gray-300 dark:border-gray-700 bg-background-light dark:bg-background-dark text-text-light dark:text-text-dark px-2 py-1",
                                            r#type: "number",
                                            placeholder: "kg",
                                            value: "{set.weight}",
                                            oninput: move |e| cards.write()[card_index].sets[set_index].weight = e.value(),
                                        }
                                        span { class: "text-text-muted shrink-0", "x" }
                                        input {
                                            class: "flex-1 min-w-0 rounded-md border border-gray-300 dark:border-gray-700 bg-background-light dark:bg-background-dark text-text-light dark:text-text-dark px-2 py-1",
                                            r#type: "number",
                                            placeholder: "reps",
                                            value: "{set.reps}",
                                            oninput: move |e| cards.write()[card_index].sets[set_index].reps = e.value(),
                                        }
                                        button {
                                            class: "text-text-muted px-1 shrink-0",
                                            onclick: move |_| { cards.write()[card_index].sets.remove(set_index); },
                                            "x"
                                        }
                                    }
                                }
                            }

                            button {
                                class: "text-accent text-sm text-left",
                                onclick: move |_| cards.write()[card_index].sets.push(SetInput::default()),
                                "+ Add set"
                            }
                        }
                        }
                        }
                    }

                    div { class: "flex gap-3",
                        button {
                            class: "bg-primary hover:bg-primary-dark text-white font-semibold rounded-lg px-6 py-3",
                            onclick: {
                                let pool = pool.clone();
                                move |_| {
                                    let pool = pool.clone();
                                    let date_value = logged_at();
                                    let all_cards = cards();
                                    let template_id = selected_template().as_ref().map(|t| t.id);

                                    spawn(async move {
                                        status.set("Saving...".to_string());

                                        for card in &all_cards {
                                            let mut parsed_sets = Vec::new();
                                            for set in &card.sets {
                                                let weight = match set.weight.parse::<f64>() {
                                                    Ok(w) => w,
                                                    Err(_) => continue,
                                                };
                                                let reps = match set.reps.parse::<i64>() {
                                                    Ok(r) => r,
                                                    Err(_) => continue,
                                                };
                                                parsed_sets.push(SetEntry { weight, reps });
                                            }

                                            if parsed_sets.is_empty() {
                                                continue;
                                            }

                                            let log = NewExerciseLog {
                                                exercise_name: card.name.clone(),
                                                logged_at: date_value.clone(),
                                                sets: parsed_sets,
                                            };

                                            if let Err(e) = insert_exercise_log(&pool, log).await {
                                                status.set(format!("Failed to save {}: {e}", card.name));
                                                return;
                                            }
                                        }

                                        if let Some(id) = template_id {
                                            let _ = record_template_completion(&pool, id).await;
                                        }

                                        status.set("Workout saved!".to_string());
                                        cards.set(Vec::new());
                                        selected_template.set(None);
                                        suggestion_reload.set(suggestion_reload() + 1);
                                    });
                                }
                            },
                            "Save Workout"
                        }
                        button {
                            class: "text-text-muted",
                            onclick: move |_| { selected_template.set(None); cards.set(Vec::new()); },
                            "Cancel"
                        }
                    }
                    if !status().is_empty() {
                        p { class: "text-text-muted text-sm", "{status}" }
                    }
                }
            }
        }
    }
}