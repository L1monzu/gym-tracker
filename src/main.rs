// The dioxus prelude contains a ton of common items used in dioxus apps. It's a good idea to import wherever you
// need dioxus
use dioxus::prelude::*;

use views::{History, Home, LogCardio, LogExercise, Navbar, Records, StartWorkout};

/// Define a components module that contains all shared components for our app.
mod components;
/// Define a views module that contains the UI for all Layouts and Routes for our app.
mod views;
/// Handles opening the SQLite database and keeping its schema up to date.
mod db;
/// The Route enum is used to define the structure of internal routes in our app. All route enums need to derive
/// the [`Routable`] trait, which provides the necessary methods for the router to work.
/// 
/// Each variant represents a different URL pattern that can be matched by the router. If that pattern is matched,
/// the components for that route will be rendered.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
        #[route("/")]
        Home {},
        #[route("/records")]
        Records {},
        #[route("/start-workout")]
        StartWorkout {},
        #[route("/log-exercise")]
        LogExercise {},
        #[route("/log-cardio")]
        LogCardio {},
        #[route("/history")]
        History {},
}

// We can import assets in dioxus with the `asset!` macro. This macro takes a path to an asset relative to the crate root.
// The macro returns an `Asset` type that will display as the path to the asset in the browser or a local path in desktop bundles.
const FAVICON: Asset = asset!("/assets/favicon.ico");
// The asset macro also minifies some assets like CSS and JS to make bundled smaller
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    // The `launch` function is the main entry point for a dioxus app. It takes a component and renders it with the platform feature
    // you have enabled
    dioxus::launch(App);
}

/// App is the main component of our app. Components are the building blocks of dioxus apps. Each component is a function
/// that takes some props and returns an Element. In this case, App takes no props because it is the root of our app.
///
/// Components should be annotated with `#[component]` to support props, better error messages, and autocomplete
/// The main component of our app. It's responsible for opening the database
/// once when the app starts, then making it available to every other screen
/// via Dioxus's context system.
#[component]
fn App() -> Element {
    // Works out the right place to store the database file for whichever
    // platform we're running on. `data_dir()` gives us the app's own
    // private storage area — on Android this is sandboxed storage only
    // this app can access; on desktop it's a sensible per-user data folder.
    let db_path = db::app_data_dir()
        .join("gym-tracker.db")
        .to_string_lossy()
        .to_string();

    // `use_resource` runs this async block once when the component first
    // renders, and hands us back a way to check whether it's still loading,
    // finished successfully, or failed. We need this because Dioxus
    // components themselves aren't async functions.
    let db_pool = use_resource(move || {
        let db_path = db_path.clone();
        async move { db::init_db(&db_path).await }
    });

    let db_pool_state = db_pool.read();
    match &*db_pool_state {
        // Still opening the database and running migrations.
        None => rsx! { p { "Setting up your database..." } },
        // Something went wrong opening the database or running migrations.
        Some(Err(e)) => rsx! { p { "Failed to set up the database: {e}" } },
        // Success — make the pool available to the rest of the app, then
        // carry on rendering as normal.
        Some(Ok(pool)) => {
            use_context_provider(|| pool.clone());

            rsx! {
                document::Link { rel: "icon", href: FAVICON }
                document::Link { rel: "stylesheet", href: MAIN_CSS }
                document::Link { rel: "stylesheet", href: TAILWIND_CSS }
                Router::<Route> {}
            }
        }
    }
}
