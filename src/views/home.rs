use dioxus::prelude::*;
use crate::Route;

#[component]
pub fn Home() -> Element {
    rsx! {
        div { class: "min-h-screen bg-background-light dark:bg-background-dark p-6",
            h1 { class: "text-3xl font-bold text-text-light dark:text-text-dark mb-8",
                "Gym Tracker"
            }
            div { class: "flex flex-col gap-4 max-w-md",
                Link {
                    to: Route::LogExercise {},
                    class: "bg-primary hover:bg-primary-dark text-white text-lg font-semibold rounded-lg px-6 py-4 text-center",
                    "Log Exercise"
                }
                Link {
                    to: Route::LogCardio {},
                    class: "bg-primary hover:bg-primary-dark text-white text-lg font-semibold rounded-lg px-6 py-4 text-center",
                    "Log Cardio"
                }
                Link {
                    to: Route::History {},
                    class: "bg-accent hover:opacity-90 text-white text-lg font-semibold rounded-lg px-6 py-4 text-center",
                    "Workout History"
                }
            }
        }
    }
}