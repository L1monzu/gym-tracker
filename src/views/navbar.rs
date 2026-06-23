use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Navbar() -> Element {
    let mut is_dark = use_signal(|| true);

    use_effect(move || {
        let dark = is_dark();
        let js = if dark {
            "document.documentElement.classList.add('dark')"
        } else {
            "document.documentElement.classList.remove('dark')"
        };
        document::eval(js);
    });

    rsx! {
        nav { class: "flex gap-4 p-4 items-center bg-surface-light dark:bg-surface-dark border-b border-gray-200 dark:border-gray-800",
            Link { to: Route::Home {}, class: "text-text-light dark:text-text-dark font-medium", "Home" }
            Link { to: Route::StartWorkout {}, class: "text-text-light dark:text-text-dark font-medium", "Start Workout" }
            Link { to: Route::LogExercise {}, class: "text-text-light dark:text-text-dark font-medium", "Log Exercise" }
            Link { to: Route::LogCardio {}, class: "text-text-light dark:text-text-dark font-medium", "Log Cardio" }
            Link { to: Route::History {}, class: "text-text-light dark:text-text-dark font-medium", "History" }
            button {
                class: "ml-auto text-text-light dark:text-text-dark text-sm border border-gray-300 dark:border-gray-700 rounded-md px-3 py-1",
                onclick: move |_| is_dark.set(!is_dark()),
                if is_dark() { "Light mode" } else { "Dark mode" }
            }
        }
        Outlet::<Route> {}
    }
}