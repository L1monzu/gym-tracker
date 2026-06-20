use dioxus::prelude::*;

#[component]
pub fn LogExercise() -> Element {
    rsx! {
        div { class: "p-4",
            h1 { class: "text-2xl font-bold text-text-light dark:text-text-dark", "Log Exercise" }
        }
    }
}