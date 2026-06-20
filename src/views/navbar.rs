use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Navbar() -> Element {
    rsx! {
        nav { class: "flex gap-4 p-4 bg-surface-light dark:bg-surface-dark border-b border-gray-200 dark:border-gray-800",
            Link { to: Route::Home {}, class: "text-text-light dark:text-text-dark font-medium", "Home" }
            Link { to: Route::LogExercise {}, class: "text-text-light dark:text-text-dark font-medium", "Log Exercise" }
            Link { to: Route::LogCardio {}, class: "text-text-light dark:text-text-dark font-medium", "Log Cardio" }
            Link { to: Route::History {}, class: "text-text-light dark:text-text-dark font-medium", "History" }
        }
        Outlet::<Route> {}
    }
}