use crate::db::exercises::{exercise_progress, list_exercises};
use dioxus::prelude::*;
use sqlx::SqlitePool;

#[component]
pub fn Progress() -> Element {
    let pool = use_context::<SqlitePool>();
    let is_dark = use_context::<Signal<bool>>();

    let known_exercises = use_resource({
        let pool = pool.clone();
        move || {
            let pool = pool.clone();
            async move { list_exercises(&pool).await.unwrap_or_default() }
        }
    });

    let mut selected_exercise = use_signal(String::new);

    let points = use_resource({
        let pool = pool.clone();
        move || {
            let pool = pool.clone();
            let name = selected_exercise();
            async move {
                if name.trim().is_empty() {
                    Vec::new()
                } else {
                    exercise_progress(&pool, &name).await.unwrap_or_default()
                }
            }
        }
    });

    rsx! {
        div { class: "min-h-screen bg-background-light dark:bg-background-dark p-6",
            h1 { class: "text-2xl font-bold text-text-light dark:text-text-dark mb-6", "Progress" }

            div { class: "max-w-md mb-6",
                label { class: "text-text-light dark:text-text-dark font-medium",
                    "Exercise"
                    input {
                        class: "mt-1 w-full rounded-md border border-gray-300 dark:border-gray-700 bg-surface-light dark:bg-surface-dark text-text-light dark:text-text-dark px-3 py-2",
                        r#type: "text",
                        list: "progress-known-exercises",
                        value: "{selected_exercise}",
                        oninput: move |e| selected_exercise.set(e.value()),
                    }
                    datalist { id: "progress-known-exercises",
                        if let Some(exercises) = &*known_exercises.read() {
                            for exercise in exercises {
                                option { value: "{exercise.name}" }
                            }
                        }
                    }
                }
            }

            {
                let points_state = points.read();
                match &*points_state {
                    None => rsx! { p { class: "text-text-muted", "Loading..." } },
                    Some(list) if selected_exercise().trim().is_empty() => rsx! {
                        p { class: "text-text-muted", "Pick an exercise to see its progress." }
                    },
                    Some(list) if list.len() < 2 => rsx! {
                        p { class: "text-text-muted", "Log this exercise on at least two different days to see a graph." }
                    },
                    Some(list) => {
                        let width = 600.0;
                        let height = 300.0;
                        let padding = 40.0;

                        let min_weight = list.iter().map(|p| p.weight).fold(f64::INFINITY, f64::min);
                        let max_weight = list.iter().map(|p| p.weight).fold(f64::NEG_INFINITY, f64::max);
                        let weight_range = (max_weight - min_weight).max(1.0);

                        let plot_width = width - padding * 2.0;
                        let plot_height = height - padding * 2.0;
                        let step = if list.len() > 1 { plot_width / (list.len() - 1) as f64 } else { 0.0 };

                        let coords: Vec<(f64, f64)> = list
                            .iter()
                            .enumerate()
                            .map(|(i, p)| {
                                let x = padding + step * i as f64;
                                let y = padding + plot_height - ((p.weight - min_weight) / weight_range * plot_height);
                                (x, y)
                            })
                            .collect();

                        let line_points = coords
                            .iter()
                            .map(|(x, y)| format!("{x},{y}"))
                            .collect::<Vec<_>>()
                            .join(" ");

                        let label_colour = if is_dark() { "#e5e7eb" } else { "#2b2d42" };

                        rsx! {
                            svg {
                                view_box: "0 0 {width} {height}",
                                width: "100%",
                                height: "300",
                                polyline {
                                    points: "{line_points}",
                                    fill: "none",
                                    stroke: "#c41e35",
                                    stroke_width: "2",
                                }
                                for (i , (x , y)) in coords.iter().enumerate() {
                                    circle { cx: "{x}", cy: "{y}", r: "4", fill: "#c41e35" }
                                    text {
                                        x: "{x}",
                                        y: "{y - 10.0}",
                                        font_size: "11",
                                        text_anchor: "middle",
                                        fill: "{label_colour}",
                                        "{list[i].weight}"
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